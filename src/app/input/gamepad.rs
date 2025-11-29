use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, Mutex},
};

use sdl3::{event::Event, gamepad::Gamepad};
use tracing::{debug, error, info, trace, warn};
use winit::event_loop::EventLoopProxy;

use crate::app::{gui::dispatcher::GuiDispatcher, window::RunnerEvent};

pub struct EventHandler {
    winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
    gui_dispatcher: Arc<Mutex<Option<GuiDispatcher>>>,
    sdl_joystick: sdl3::JoystickSubsystem,
    sdl_gamepad: sdl3::GamepadSubsystem,
    sdl_devices: HashMap<u32, Vec<SDLDevice>>,
    state: Arc<Mutex<State>>,
}

struct State {
    devices: Vec<Device>,
}

enum SDLDevice {
    Joystick(sdl3::joystick::Joystick),
    Gamepad(sdl3::gamepad::Gamepad),
}

impl Debug for SDLDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SDLDevice::Joystick(joystick) => f
                .debug_struct("SDLDevice::Joystick")
                .field("name", &joystick.name())
                .field("id", &joystick.id())
                .finish(),
            SDLDevice::Gamepad(gamepad) => f
                .debug_struct("SDLDevice::Gamepad")
                .field("name", &gamepad.name())
                .field("id", &gamepad.id())
                .finish(),
        }
    }
}

#[derive(Debug, Default)]
struct Device {
    id: u32,
    steam_handle: u64,
    state: DeviceState,
    sdl_device_count: usize,
}

#[derive(Debug, Clone, Default)]
struct DeviceState {
    buttons: u32, // TODO
}

impl EventHandler {
    pub fn new(
        winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
        gui_dispatcher: Arc<Mutex<Option<GuiDispatcher>>>,
    ) -> Self {
        let sdl = sdl3::init()
            .inspect_err(|e| error!("failed to get handle on SDL: {}", e))
            .unwrap();
        let state = Arc::new(Mutex::new(State {
            devices: Vec::new(),
        }));
        let res = Self {
            winit_waker,
            gui_dispatcher,
            sdl_joystick: sdl.joystick().unwrap(),
            sdl_gamepad: sdl.gamepad().unwrap(),
            sdl_devices: HashMap::new(),
            state: state.clone(),
        };
        if let Ok(dispatcher_guard) = res.gui_dispatcher.lock()
            && let Some(dispatcher) = &*dispatcher_guard
        {
            debug!("SDL loop GUI dispatcher initialized");
            dispatcher.register_callback(move |ctx| {
                if let Ok(mut guard) = state.lock() {
                    let state = &mut *guard;
                    EventHandler::on_draw(state, ctx);
                }
            });
        }
        res
    }

    pub fn on_pad_added(&mut self, event: &Event) {
        match event {
            Event::JoyDeviceAdded { which, .. } | Event::ControllerDeviceAdded { which, .. } => {
                trace!(
                    "{} added with ID {}",
                    if matches!(event, Event::JoyDeviceAdded { .. }) {
                        "Joystick"
                    } else {
                        "Gamepad"
                    },
                    which
                );

                let sdl_dev = match event {
                    Event::JoyDeviceAdded { which, .. } => {
                        self.sdl_joystick.open(*which).ok().map(SDLDevice::Joystick)
                    }
                    Event::ControllerDeviceAdded { which, .. } => {
                        self.sdl_gamepad.open(*which).ok().map(SDLDevice::Gamepad)
                    }
                    _ => unreachable!(),
                };
                let sdl_device = match sdl_dev {
                    Some(device) => device,
                    None => {
                        warn!("Failed to open SDL device with ID {}", which);
                        return;
                    }
                };

                let steam_handle = match &sdl_device {
                    SDLDevice::Joystick(_) => 0,
                    SDLDevice::Gamepad(p) => get_gamepad_steam_handle(p),
                };

                self.sdl_devices.entry(*which).or_default().push(sdl_device);

                if let Ok(mut guard) = self
                    .state
                    .lock()
                    .map_err(|e| error!("Failed to lock state for adding device: {}", e))
                {
                    match guard.devices.iter_mut().find(|d| d.id == *which) {
                        Some(existing_device) => {
                            existing_device.sdl_device_count += 1;
                            debug!(
                                "Added extra SDL {} device count for {}; Number of SDL devices {}",
                                if matches!(event, Event::JoyDeviceAdded { .. }) {
                                    "Joystick"
                                } else {
                                    "Gamepad"
                                },
                                which,
                                existing_device.sdl_device_count
                            );
                            if existing_device.steam_handle == 0 && steam_handle != 0 {
                                existing_device.steam_handle = steam_handle;
                                info!(
                                    "Updated steam handle for device ID {} to {}",
                                    which, steam_handle
                                );
                            }
                        }
                        _ => {
                            let device = Device {
                                id: *which,
                                steam_handle,
                                state: DeviceState::default(),
                                sdl_device_count: 1,
                            };
                            guard.devices.push(device);
                            info!(
                                "Added {} device with ID {}",
                                if matches!(event, Event::JoyDeviceAdded { .. }) {
                                    "Joystick"
                                } else {
                                    "Gamepad"
                                },
                                which
                            );
                        }
                    }
                }
            }
            _ => {
                warn!("Unexpected event for pad addition: {:?}", event);
            }
        }
    }
    pub fn on_pad_removed(&mut self, event: &Event) {
        match event {
            Event::JoyDeviceRemoved { which, .. }
            | Event::ControllerDeviceRemoved { which, .. } => {
                trace!(
                    "{} removed with ID {}",
                    if matches!(event, Event::JoyDeviceRemoved { .. }) {
                        "Joystick"
                    } else {
                        "Gamepad"
                    },
                    which
                );

                if let Some(devices) = self.sdl_devices.get_mut(which) {
                    _ = devices.pop();
                    if devices.is_empty() {
                        self.sdl_devices.remove(which);
                    }
                }

                if let Ok(mut guard) = self
                    .state
                    .lock()
                    .map_err(|e| error!("Failed to lock state for removing device: {}", e))
                    && let Some(device) = guard.devices.iter_mut().find(|d| d.id == *which)
                {
                    if device.sdl_device_count > 0 {
                        device.sdl_device_count -= 1;
                        debug!(
                            "Removed SDL {} device count for {}; Remaining SDL devices {}",
                            if matches!(event, Event::JoyDeviceRemoved { .. }) {
                                "Joystick"
                            } else {
                                "Gamepad"
                            },
                            which,
                            device.sdl_device_count
                        );
                    }
                    if device.sdl_device_count == 0 {
                        guard.devices.retain(|d| d.id != *which);
                        info!(
                            "Removed {} device with ID {}",
                            if matches!(event, Event::JoyDeviceRemoved { .. }) {
                                "Joystick"
                            } else {
                                "Gamepad"
                            },
                            which
                        );
                    }
                }
            }
            _ => {
                warn!("Unexpected event for pad removal: {:?}", event);
            }
        }
    }

    // The high-level sdl3-rs `Event::Unknown` doesn't expose the `which` field from
    // `SDL_GamepadDeviceEvent`. We work around this by refreshing all tracked pads.
    //
    // See: https://github.com/libsdl-org/SDL/blob/main/include/SDL3/SDL_events.h#L672-L677
    pub fn on_steam_handle_updated(&self, _event: &Event) {
        let Ok(mut guard) = self.state.lock() else {
            warn!("Failed to lock state for steam handle update");
            return;
        };

        self.sdl_devices
            .values()
            .flat_map(|d| {
                d.iter().filter_map(|dev| match dev {
                    SDLDevice::Gamepad(pad) => Some(pad),
                    _ => None,
                })
            })
            .for_each(|pad| {
                let Ok(instance_id) = pad.id() else {
                    return;
                };
                let steam_handle = get_gamepad_steam_handle(pad);
                if let Some(device) = guard.devices.iter_mut().find(|d| d.id == instance_id) {
                    device.steam_handle = steam_handle;
                    info!(
                        "Updated steam handle for device ID {} to {}",
                        instance_id, steam_handle
                    );
                }
            });
        self.request_redraw();
    }

    pub fn on_pad_event(&self, event: &Event) {
        match event {
            Event::Unknown { .. } => {
                // if nothing "outside" changed is
                // GAMEPAD_STATE_UPDATE_COMPLETE or JOYPAD_STATE_UPDATE_COMPLETE
                // Silently Ignore for now
                // Would need "supertrace" log level lol
            }
            _ => {
                if event.is_joy() {
                    // Currently just drop lower level joystick events
                    return;
                }
                if !event.is_controller() {
                    warn!("Received non-gamepad event in on_pad_event: {:?}", event);
                    return;
                }
                // handle all other events and just "update gamepad"
                // instead of duplicating code for every shit"
                trace!("GamePadder: Pad event: {:?}", event);
            }
        }
    }

    fn request_redraw(&self) {
        _ = self
            // the legend of zelda: the
            .winit_waker
            .lock()
            .ok()
            .and_then(|g| g.as_ref().map(|p| p.send_event(RunnerEvent::Redraw())))
            .map(|r| r.inspect_err(|e| warn!("Failed to request GUI redraw: {}", e)));
    }

    fn on_draw(state: &mut State, ctx: &egui::Context) {
        egui::Window::new("GamePads").show(ctx, |ui| {
            ui.label("Connected GamePads:");
            for device in &state.devices {
                ui.group(|ui| {
                    ui.label(format!("Pad ID: {}", device.id));
                    ui.label(format!("Steam Handle: {}", device.steam_handle));
                    ui.label(format!("SDL Device Count: {}", device.sdl_device_count));
                });
            }
        });
    }
}

fn get_gamepad_steam_handle(pad: &Gamepad) -> u64 {
    use sdl3::sys::gamepad::SDL_GetGamepadSteamHandle;
    let instance_id = pad.id().unwrap_or(0);
    if instance_id == 0 {
        trace!("Cannot get steam handle for device with invalid instance ID 0");
        return 0;
    }

    unsafe {
        // Extract the raw SDL_Gamepad pointer from the opened gamepad
        // sdl3-0.16.2\src\sdl3\gamepad.rs:745
        #[repr(C)]
        struct GamepadRaw {
            _subsystem: [u8; std::mem::size_of::<sdl3::GamepadSubsystem>()],
            raw: *mut sdl3::sys::gamepad::SDL_Gamepad,
        }

        let gamepad_raw: &GamepadRaw = std::mem::transmute(pad);
        if gamepad_raw.raw.is_null() {
            warn!(
                "Gamepad raw pointer is null for instance ID {}",
                instance_id
            );
            return 0;
        }

        SDL_GetGamepadSteamHandle(gamepad_raw.raw)
    }
}

macro_rules! event_which {
    ($event:expr) => {
        match $event {
            Event::JoyAxisMotion { which, .. }
            | Event::JoyBallMotion { which, .. }
            | Event::JoyHatMotion { which, .. }
            | Event::JoyButtonDown { which, .. }
            | Event::JoyButtonUp { which, .. }
            | Event::JoyDeviceAdded { which, .. }
            | Event::JoyDeviceRemoved { which, .. }
            // FUCK RUSTFMT
            | Event::ControllerAxisMotion { which, .. }
            | Event::ControllerButtonDown { which, .. }
            | Event::ControllerButtonUp { which, .. }
            | Event::ControllerDeviceAdded { which, .. }
            | Event::ControllerDeviceRemoved { which, .. }
            | Event::ControllerDeviceRemapped { which, .. }
            | Event::ControllerTouchpadDown { which, .. }
            | Event::ControllerTouchpadMotion { which, .. }
            | Event::ControllerTouchpadUp { which, .. }
            | Event::ControllerSensorUpdated { which, .. } => Some(*which),
            _ => None,
        }
    };
}
