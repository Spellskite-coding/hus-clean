#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod format;
mod theme;

const HUSKY_PNG: &[u8] = include_bytes!("../../../assets/husky.png");

fn main() -> eframe::Result<()> {
    let icon = eframe::icon_data::from_png_bytes(HUSKY_PNG).expect("icone husky.png invalide");

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_min_inner_size([640.0, 480.0])
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        "Hus-Clean - Nettoyage de fichiers",
        native_options,
        Box::new(|cc| Ok(Box::new(app::HusCleanApp::new(cc)))),
    )
}
