use std::sync::{Arc, Mutex, atomic::AtomicU64};

use tracing::{debug, error, warn};
use winit::event_loop::EventLoopProxy;

use crate::app::window::RunnerEvent;

pub type DrawNotifier = Box<dyn Fn(&egui::Context) + Send + Sync + 'static>;

#[derive(Default)]
pub struct GuiDispatcher {
    event_loop_proxy: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
    draw_listeners: Mutex<Vec<DrawNotifier>>,
}

impl GuiDispatcher {
    pub fn new(proxy: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>) -> Self {
        Self {
            event_loop_proxy: proxy,
            draw_listeners: Mutex::new(Vec::new()),
        }
    }

    pub fn register_callback<F>(&self, callback: F)
    where
        F: Fn(&egui::Context) + Send + Sync + 'static,
    {
        debug!(
            "Registering GUI draw callback, function at {:p}",
            &callback as *const F
        );
        match self.draw_listeners.lock() {
            Ok(mut cbs) => {
                cbs.push(Box::new(callback));
            }
            Err(e) => {
                error!("Failed to register GUI draw callback: {}", e);
            }
        }
    }

    pub fn request_redraw(&self) {
        match self.event_loop_proxy.lock() {
            Ok(guard) => {
                if let Some(proxy) = &*guard {
                    match proxy.send_event(RunnerEvent::Redraw()) {
                        Ok(_) => {}
                        Err(e) => {
                            warn!("Failed to request GUI redraw: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Failed to acquire event loop proxy lock: {}", e);
            }
        }
    }

    pub fn draw(&self, egui_ctx: &egui::Context) {
        if let Ok(pending) = self.draw_listeners.lock() {
            for cmd in pending.iter() {
                (cmd)(egui_ctx);
            }
        } else {
            warn!("Failed to acquire GUI draw listeners lock");
        }
    }
}
