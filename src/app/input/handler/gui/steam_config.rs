use egui::{Id, RichText, Vec2};
use tracing::warn;

use crate::app::input::handler::State;
use crate::app::steam_utils;

pub fn draw(state: &mut State, ctx: &egui::Context, open: &mut bool) {
    if !*open {
        return;
    }

    egui::Window::new("🚂 Steam Input Config")
        .id(Id::new("steam_config"))
        .default_pos(ctx.available_rect().center() - Vec2::new(210.0, 200.0))
        .default_size(Vec2::new(360.0, 260.0))
        .collapsible(false)
        .resizable(true)
        .open(open)
        .show(ctx, |ui| {
            let enforcer = &mut state.binding_enforcer;

            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Game ID:").strong());
                ui.label(
                    RichText::new(
                        enforcer
                            .game_id()
                            .map(|id| id.to_string())
                            .unwrap_or("N/A".to_string())
                            .to_string(),
                    )
                    .weak(),
                );
            });

            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("App ID:").strong());
                ui.label(
                    RichText::new(
                        enforcer
                            .app_id()
                            .map(|id| id.to_string())
                            .unwrap_or("N/A".to_string())
                            .to_string(),
                    )
                    .weak(),
                );
            });

            ui.separator();

            let has_app_id = enforcer.app_id().is_some();
            let mut active = enforcer.is_active();

            ui.add_enabled_ui(has_app_id, |ui| {
                if ui.checkbox(&mut active, "Force Config").changed() {
                    if active {
                        enforcer.activate();
                    } else {
                        enforcer.deactivate();
                    }
                }
                if ui.button("⚙ Open Configurator").clicked() {
                    let str = &format!("steam://controllerconfig/{}", enforcer.app_id().unwrap());
                    _ = steam_utils::open_steam_url(str)
                        .inspect_err(|e| warn!("Failed to open Steam Input Configurator: {}", e));
                }
            });
        });
}
