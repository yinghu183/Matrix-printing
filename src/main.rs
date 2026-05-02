#![windows_subsystem = "windows"]

mod app;
mod logic;

use app::MatrixPrintingApp;

/// Try to find a font file in the `fonts/` directory to use for egui UI.
fn load_egui_font() -> Option<Vec<u8>> {
    let fonts_dir = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("fonts");
    if let Ok(entries) = std::fs::read_dir(&fonts_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                if ext == "ttf" || ext == "otf" {
                    if let Ok(data) = std::fs::read(&path) {
                        return Some(data);
                    }
                }
            }
        }
    }
    None
}

fn main() -> eframe::Result {
    let egui_font_data = load_egui_font();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([900.0, 600.0])
            .with_title("Matrix Printing Tool"),
        ..Default::default()
    };

    eframe::run_native(
        "Matrix Printing Tool",
        options,
        Box::new(move |cc| {
            // Configure egui with a Chinese-capable font for UI text
            if let Some(ref font_bytes) = egui_font_data {
                let mut fonts = egui::FontDefinitions::default();
                fonts.font_data.insert(
                    "chinese_font".to_owned(),
                    std::sync::Arc::new(egui::FontData::from_owned(font_bytes.clone())),
                );
                fonts
                    .families
                    .get_mut(&egui::FontFamily::Proportional)
                    .unwrap()
                    .insert(0, "chinese_font".to_owned());
                fonts
                    .families
                    .get_mut(&egui::FontFamily::Monospace)
                    .unwrap()
                    .insert(0, "chinese_font".to_owned());
                cc.egui_ctx.set_fonts(fonts);
            }
            Ok(Box::new(MatrixPrintingApp::default()))
        }),
    )
}
