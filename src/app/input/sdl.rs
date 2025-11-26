use std::sync::{Arc, Mutex};

use sdl3::event::{Event, EventSender};
use tracing::{Level, Span, debug, error, info, span, trace, warn};
use winit::event_loop::EventLoopProxy;

use crate::app::{App, gui::dispatcher::GuiDispatcher, input::sdl_hints, window::RunnerEvent};

#[derive(Default)]
pub struct InputLoop {
    sdl_waker: Arc<Mutex<Option<EventSender>>>,
    winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
    gui_dispatcher: Arc<Mutex<Option<GuiDispatcher>>>,
    somedummy: Arc<Mutex<SomeTodoDummyDebugState>>,
}

#[derive(Default)]
struct SomeTodoDummyDebugState {
    some_names: Vec<String>,
    counter: u64,
}

struct EventHandleResult {
    continue_loop: bool,
    continue_processing: bool,
    request_redraw: bool,
}

impl InputLoop {
    pub fn new(
        sdl_waker: Arc<Mutex<Option<EventSender>>>,
        winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
        gui_dispatcher: Arc<Mutex<Option<GuiDispatcher>>>,
    ) -> Self {
        Self {
            sdl_waker,
            winit_waker,
            gui_dispatcher,
            somedummy: Arc::new(Mutex::new(SomeTodoDummyDebugState::default())),
        }
    }

    pub fn run(&mut self) {
        trace!("SDL_Init");

        let sdl = match sdl3::init() {
            Ok(sdl) => sdl,
            Err(e) => {
                error!("Failed to initialize SDL: {}", e);
                return;
            }
        };
        for (hint_name, hint_value) in sdl_hints::SDL_HINTS {
            match sdl3::hint::set(hint_name, hint_value) {
                true => trace!("Set SDL hint {hint_name}={hint_value}"),
                false => warn!("Failed to set SDL hint {hint_name}={hint_value}"),
            }
        }

        let sdl_joystick = match sdl.joystick() {
            Ok(sdl_joystick) => Some(sdl_joystick),
            Err(e) => {
                error!("Failed to initialize SDL joystick subsystem: {}", e);
                None
            }
        };
        let sdl_gamepad = match sdl.gamepad() {
            Ok(sdl_gamepad) => Some(sdl_gamepad),
            Err(e) => {
                error!("Failed to initialize SDL gamepad subsystem: {}", e);
                None
            }
        };

        match sdl.event() {
            Ok(event_subsystem) => match self.sdl_waker.lock() {
                Ok(mut guard) => {
                    *guard = Some(event_subsystem.event_sender());
                }
                Err(e) => {
                    error!("Failed to set SDL event sender: {}", e);
                }
            },
            Err(e) => {
                error!("Failed to get SDL event subsystem: {}", e);
            }
        }

        let mut event_pump = match sdl.event_pump() {
            Ok(pump) => pump,
            Err(e) => {
                error!("Failed to get SDL event pump: {}", e);
                return;
            }
        };

        if let Ok(dispatcher_guard) = self.gui_dispatcher.lock()
            && let Some(dispatcher) = &*dispatcher_guard
        {
            debug!("SDL loop GUI dispatcher initialized");
            let state = self.somedummy.clone();
            dispatcher.register_callback(move |ctx| {
                if let Ok(mut guard) = state.lock() {
                    let state = &mut *guard;
                    InputLoop::on_draw(state, ctx);
                }
            });
        }

        match self.run_loop(&mut event_pump) {
            Ok(_) => {}
            Err(_) => {
                error!("SDL loop exited with error");
            }
        }

        trace!("SDL loop exiting");
        App::shutdown(None, Some(&self.winit_waker));
    }

    fn run_loop(&mut self, event_pump: &mut sdl3::EventPump) -> Result<(), ()> {
        let span = span!(Level::INFO, "sdl_loop");

        trace!("SDL loop starting");
        loop {
            let meh = event_pump.wait_event();
            for event in std::iter::once(meh).chain(event_pump.poll_iter()) {
                match event {
                    Event::Quit { .. } => {
                        tracing::event!(parent: &span, Level::INFO, event = ?event, "Quit event received");
                        return Ok(());
                    }
                    _ => {
                        tracing::event!(parent: &span, Level::TRACE, event = ?event, "SDL event");
                        if let Ok(mut guard) = self.somedummy.lock() {
                            let state = &mut *guard;
                            state.counter += 1;
                        }
                    }
                }
            }
            self.request_redraw();
        }
    }

    fn request_redraw(&self) {
        if let Ok(guard) = self.winit_waker.lock()
            && let Some(proxy) = &*guard
        {
            match proxy.send_event(RunnerEvent::Redraw()) {
                Ok(_) => {}
                Err(e) => {
                    warn!("Failed to request GUI redraw: {}", e);
                }
            }
        }
    }

    fn on_draw(state: &mut SomeTodoDummyDebugState, ctx: &egui::Context) {
        egui::Window::new("SDL Input Loop").show(ctx, |ui| {
            ui.label("SDL Input Loop is running!");
            ui.label(format!("Event count: {}", state.counter));
        });
    }
}
