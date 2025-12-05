use egui::{Id, RichText, Vec2};

use crate::app::input::handler::State;
use crate::app::input::sdl_device_info::SdlValue;

pub fn draw(state: &mut State, ctx: &egui::Context, open: &mut bool) {
    egui::Window::new("🎮 Gamepads")
        .id(Id::new("controller_info"))
        .default_pos(ctx.available_rect().center() - Vec2::new(210.0, 200.0))
        .default_height(400.0)
        .collapsible(false)
        .default_size(Vec2::new(420.0, 320.0))
        .resizable(true)
        .open(open)
        .show(ctx, |ui| {
            egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                ui.label("Connected Gamepads:");
                for (_, device) in state.devices.iter() {
                    ui.group(|ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new("Device ID:").strong());
                            ui.label(RichText::new(format!("{}", device.id)).weak());
                        });
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new("SDL IDs:").strong());
                            ui.label(
                                RichText::new(
                                    device
                                        .sdl_ids
                                        .iter()
                                        .map(|id| id.to_string())
                                        .collect::<Vec<_>>()
                                        .join(", "),
                                )
                                .weak(),
                            );
                        });
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new("Steam Handle:").strong());
                            ui.label(RichText::new(format!("{}", device.steam_handle)).weak());
                        });
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new("SDL Device Count:").strong());
                            ui.label(
                                RichText::new(format!("{}", device.sdl_device_infos.len())).weak(),
                            );
                        });

                        for (idx, info) in device.sdl_device_infos.iter().enumerate() {
                            ui.collapsing(
                                format!(
                                    "SDL {} #{}-{}",
                                    if info.is_gamepad {
                                        "Gamepad"
                                    } else {
                                        "Joystick"
                                    },
                                    device.id,
                                    idx
                                ),
                                |ui| {
                                    render_properties(ui, &info.properties);
                                },
                            );
                        }

                        ui.collapsing(format!("🐍 VIIPER Device #{}", device.id), |ui| {
                            match &device.viiper_device {
                                Some(viiper_dev) => {
                                    ui.horizontal_wrapped(|ui| {
                                        ui.label(RichText::new("Connected:").strong());
                                        ui.label(
                                            RichText::new(format!("{}", device.viiper_connected))
                                                .weak(),
                                        );
                                    });
                                    ui.horizontal_wrapped(|ui| {
                                        ui.label(RichText::new("Bus ID:").strong());
                                        ui.label(
                                            RichText::new(format!("{}", viiper_dev.bus_id)).weak(),
                                        );
                                    });
                                    ui.horizontal_wrapped(|ui| {
                                        ui.label(RichText::new("Device ID:").strong());
                                        ui.label(
                                            RichText::new(format!("{}", viiper_dev.dev_id)).weak(),
                                        );
                                    });
                                    ui.horizontal_wrapped(|ui| {
                                        ui.label(RichText::new("Type:").strong());
                                        ui.label(
                                            RichText::new(format!("{}", viiper_dev.r#type)).weak(),
                                        );
                                    });
                                    ui.horizontal_wrapped(|ui| {
                                        ui.label(RichText::new("Vendor ID:").strong());
                                        ui.label(
                                            RichText::new(format!("{:?}", viiper_dev.vid)).weak(),
                                        );
                                    });
                                    ui.horizontal_wrapped(|ui| {
                                        ui.label(RichText::new("Product ID:").strong());
                                        ui.label(
                                            RichText::new(format!("{:?}", viiper_dev.pid)).weak(),
                                        );
                                    });
                                }
                                None => {
                                    ui.label("Not connected");
                                }
                            }
                        });
                    });
                }
            });
        });
}

fn render_properties(ui: &mut egui::Ui, properties: &std::collections::HashMap<String, SdlValue>) {
    let mut keys: Vec<_> = properties.keys().collect();
    keys.sort();

    for key in keys {
        let value = &properties[key];
        match value {
            SdlValue::Nested(nested) => {
                ui.collapsing(format!("📁 {}", key), |ui| {
                    render_properties(ui, nested);
                });
            }
            _ => {
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(format!("{}:", key)).strong());
                    ui.label(RichText::new(format!("{}", value)).weak());
                });
            }
        }
    }
}
