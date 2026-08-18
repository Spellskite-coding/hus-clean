use egui::{Color32, Context, Visuals};

/// Reproduit la palette sombre "Fusion" de la version C++/Qt d'origine :
/// fond fenetre (53,53,53), fond des zones de texte/listes (30,30,30),
/// texte blanc, accent bleu (42,130,218).
pub fn apply(ctx: &Context) {
    let window_bg = Color32::from_rgb(53, 53, 53);
    let base_bg = Color32::from_rgb(30, 30, 30);
    let text = Color32::WHITE;
    let accent = Color32::from_rgb(42, 130, 218);

    let mut visuals = Visuals::dark();
    visuals.override_text_color = Some(text);
    visuals.window_fill = window_bg;
    visuals.panel_fill = window_bg;
    visuals.extreme_bg_color = base_bg; // fond des listes / champs de texte
    visuals.faint_bg_color = base_bg;

    visuals.widgets.noninteractive.bg_fill = window_bg;
    visuals.widgets.noninteractive.weak_bg_fill = window_bg;
    visuals.widgets.inactive.bg_fill = window_bg;
    visuals.widgets.inactive.weak_bg_fill = window_bg;
    visuals.widgets.hovered.bg_fill = accent.linear_multiply(0.5);
    visuals.widgets.active.bg_fill = accent;

    visuals.selection.bg_fill = accent;
    visuals.selection.stroke.color = text;
    visuals.hyperlink_color = accent;

    ctx.set_visuals(visuals);
}
