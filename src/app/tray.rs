use std::sync::{Arc, Mutex};

use sdl3::event::EventSender;
#[cfg(windows)]
use tracing::Span;
use tracing::{Level, event, info, span};
use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};
use winit::event_loop::EventLoopProxy;

use crate::app::window::RunnerEvent;

use super::core::App;

const ICON_BYTES: &[u8] = include_bytes!("../../assets/icon.png");

#[cfg(windows)]
static TRAY_THREAD_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

struct TrayContext {
    _tray_icon: TrayIcon,
    quit_id: MenuId,
    sdl_waker: Arc<Mutex<Option<EventSender>>>,
    winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
}

impl TrayContext {
    fn new(
        sdl_waker: Arc<Mutex<Option<EventSender>>>,
        winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
    ) -> Self {
        let icon = load_icon();
        let menu = Menu::new();
        let quit_item = MenuItem::new("Quit", true, None);
        let quit_id = quit_item.id().clone();
        menu.append(&quit_item).expect("Failed to add quit item");

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("SISR")
            .with_icon(icon)
            .build()
            .expect("Failed to create tray icon");

        Self {
            _tray_icon: tray_icon,
            quit_id,
            sdl_waker,
            winit_waker,
        }
    }

    fn handle_events(&self) -> bool {
        if let Ok(event) = MenuEvent::receiver().try_recv()
            && event.id == self.quit_id
        {
            info!("Quit requested from tray menu");
            App::shutdown(Some(&self.sdl_waker), Some(&self.winit_waker));
            return true;
        }
        false
    }
}

pub fn shutdown() {
    #[cfg(windows)]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{PostThreadMessageW, WM_QUIT};
        let thread_id = TRAY_THREAD_ID.load(std::sync::atomic::Ordering::SeqCst);
        if thread_id != 0 {
            use tracing::trace;

            unsafe {
                PostThreadMessageW(thread_id, WM_QUIT, 0, 0);
            }
            trace!("Posted WM_QUIT to tray thread");
        }
    }

    #[cfg(target_os = "linux")]
    {
        gtk::main_quit();
        trace!("Called gtk::main_quit()");
    }

    #[cfg(target_os = "macos")]
    {
        // TODO: macOS CFRunLoop stop
        tracing::warn!("macOS tray shutdown not yet implemented");
    }
}

pub fn run(
    sdl_waker: Arc<Mutex<Option<EventSender>>>,
    winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
) {
    let span = span!(Level::INFO, "tray");
    run_platform(span, sdl_waker, winit_waker);
}

#[cfg(windows)]
fn run_platform(
    span: Span,
    sdl_waker: Arc<Mutex<Option<EventSender>>>,
    winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
) {
    use windows_sys::Win32::System::Threading::GetCurrentThreadId;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, MSG, TranslateMessage, WM_QUIT,
    };

    let thread_id = unsafe { GetCurrentThreadId() };
    TRAY_THREAD_ID.store(thread_id, std::sync::atomic::Ordering::SeqCst);

    let ctx = TrayContext::new(sdl_waker, winit_waker);

    loop {
        if ctx.handle_events() {
            event!(parent: &span, Level::DEBUG, "Tray context requested quit, exiting tray loop");
            break;
        }

        unsafe {
            let mut msg: MSG = std::mem::zeroed();
            let ret = GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0);
            if ret == 0 || ret == -1 || msg.message == WM_QUIT {
                event!(parent: &span, Level::DEBUG, "Received WM_QUIT or error in GetMessageW, exiting tray loop");
                break;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

#[cfg(target_os = "linux")]
fn run_platform(
    span: Span,
    sdl_waker: Arc<Mutex<Option<EventSender>>>,
    winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
) {
    if gtk::init().is_err() {
        event!(parent: &span, Level::ERROR, "Failed to initialize GTK for tray icon");
        return;
    }

    let ctx = Arc::new(TrayContext::new(sdl_waker, winit_waker));

    glib::idle_add_local(move || {
        if ctx.handle_events() {
            gtk::main_quit();
            return glib::ControlFlow::Break;
        }
        glib::ControlFlow::Continue
    });

    gtk::main();
}

#[cfg(target_os = "macos")]
fn run_platform(
    span: Span,
    _sdl_waker: Arc<Mutex<Option<EventSender>>>,
    _winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
) {
    event!(parent: &span, Level::WARN, "macOS tray icon requires main thread NSApplication event loop - not yet implemented");
}

fn load_icon() -> Icon {
    let image = image::load_from_memory(ICON_BYTES)
        .expect("Failed to load icon")
        .into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();

    Icon::from_rgba(rgba, width, height).expect("Failed to create icon")
}
