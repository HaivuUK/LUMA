//! Top level egui
//! Renders --

use super::*;

/// Main egui ui system
/// Renders ---

pub(super) fn render_egui_ui(
    mut contexts: EguiContexts,
    mut app: ResMut<LumaGuiApp>,
    window_q: Query<&Window, With<PrimaryWindow>>
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    // Apply dark styling
    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = egui::Color32::from_rgba_unmultiplied(20, 20, 24, 230);
    visuals.panel_fill = egui::Color32::from_rgb(28, 28, 34);
    ctx.set_visuals(visuals);
}