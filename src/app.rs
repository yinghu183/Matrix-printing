use chrono::Local;
use eframe::Frame;
use egui::Context;
use fontdue::Font;
use image::{DynamicImage, GenericImage, GenericImageView, Rgba, RgbaImage};
use rfd::FileDialog;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use crate::logic::{calculate_grid_metrics, split_text_paragraphs};

const OUTPUT_SIZES: &[(&str, Option<(u32, u32)>)] = &[
    ("原始尺寸", None),
    ("A4 (2480×3508)", Some((2480, 3508))),
    ("A3 (3508×4961)", Some((3508, 4961))),
    ("4K (3840×2160)", Some((3840, 2160))),
    ("高清 (1920×1080)", Some((1920, 1080))),
];

pub struct MatrixPrintingApp {
    // Image state
    original_image: Option<DynamicImage>,
    display_image: Option<DynamicImage>,
    image_path: Option<PathBuf>,

    // Font state
    fonts: Vec<FontData>,
    selected_font_idx: usize,
    fonts_dir: PathBuf,

    // Grid params (stored as strings for the text fields)
    start_x: String,
    start_y: String,
    cell_width: String,
    cell_height: String,
    font_size: String,
    offset_x: String,
    offset_y: String,
    grid_columns: String,
    grid_rows: String,
    line_thickness: String,

    // Text
    text_input: String,
    first_line_indent: bool,
    first_line_newline: bool,

    // Output
    output_size_idx: usize,
    status_message: Option<String>,

    // Preview
    preview_dirty: bool,
    preview_texture: Option<egui::TextureHandle>,

    // Directories
    uploads_dir: PathBuf,
    output_dir: PathBuf,
    config_dir: PathBuf,

    // Dirty tracking for preview
    last_params_hash: u64,

    // First frame flag for visual setup
    first_frame: bool,
}

#[derive(Clone)]
struct FontData {
    name: String,
    #[allow(dead_code)]
    path: PathBuf,
    font: Font,
}

impl Default for MatrixPrintingApp {
    fn default() -> Self {
        let base = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        let fonts_dir = base.join("fonts");
        let uploads_dir = base.join("uploads");
        let output_dir = base.join("output");
        let config_dir = base.join("config");

        // Ensure directories exist
        let _ = std::fs::create_dir_all(&fonts_dir);
        let _ = std::fs::create_dir_all(&uploads_dir);
        let _ = std::fs::create_dir_all(&output_dir);
        let _ = std::fs::create_dir_all(&config_dir);

        let fonts = Self::scan_fonts(&fonts_dir);

        let mut app = Self {
            original_image: None,
            display_image: None,
            image_path: None,
            fonts,
            selected_font_idx: 0,
            fonts_dir,
            start_x: String::new(),
            start_y: String::new(),
            cell_width: String::new(),
            cell_height: String::new(),
            font_size: String::new(),
            offset_x: "1".into(),
            offset_y: "-1".into(),
            grid_columns: String::new(),
            grid_rows: String::new(),
            line_thickness: "2".into(),
            text_input: String::new(),
            first_line_indent: true,
            first_line_newline: false,
            output_size_idx: 0,
            status_message: None,
            preview_dirty: false,
            preview_texture: None,
            uploads_dir,
            output_dir,
            config_dir,
            last_params_hash: 0,
            first_frame: true,
        };
        app.load_default_config();
        app
    }
}

impl MatrixPrintingApp {
    fn scan_fonts(dir: &PathBuf) -> Vec<FontData> {
        let mut fonts = Vec::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    let ext = ext.to_string_lossy().to_lowercase();
                    if ext == "ttf" || ext == "otf" {
                        let name = path.file_name().unwrap().to_string_lossy().to_string();
                        if let Ok(data) = std::fs::read(&path) {
                            if let Ok(font) =
                                Font::from_bytes(data, fontdue::FontSettings::default())
                            {
                                fonts.push(FontData { name, path, font });
                            }
                        }
                    }
                }
            }
        }
        fonts
    }

    fn params_hash(&self) -> u64 {
        let mut h = DefaultHasher::new();
        self.start_x.hash(&mut h);
        self.start_y.hash(&mut h);
        self.cell_width.hash(&mut h);
        self.cell_height.hash(&mut h);
        self.font_size.hash(&mut h);
        self.offset_x.hash(&mut h);
        self.offset_y.hash(&mut h);
        self.grid_columns.hash(&mut h);
        self.grid_rows.hash(&mut h);
        self.line_thickness.hash(&mut h);
        self.text_input.hash(&mut h);
        self.first_line_indent.hash(&mut h);
        self.first_line_newline.hash(&mut h);
        self.output_size_idx.hash(&mut h);
        self.selected_font_idx.hash(&mut h);
        self.display_image.is_some().hash(&mut h);
        h.finish()
    }

    fn mark_dirty(&mut self) {
        self.preview_dirty = true;
    }

    fn load_image_dialog(&mut self) {
        if let Some(path) = FileDialog::new()
            .add_filter("Image", &["png", "jpg", "jpeg", "gif", "bmp"])
            .pick_file()
        {
            if let Ok(img) = image::open(&path) {
                // Copy to uploads
                let ts = Local::now().format("%Y%m%d_%H%M%S");
                let ext = path.extension().unwrap_or_default().to_string_lossy();
                let upload_name = format!("upload_{}.{}", ts, ext);
                let upload_path = self.uploads_dir.join(&upload_name);
                let _ = img.save(&upload_path);

                self.image_path = Some(upload_path);
                self.original_image = Some(img.clone());
                self.apply_output_size();
                self.auto_calculate_params();
                self.mark_dirty();
            }
        }
    }

    fn load_font_dialog(&mut self) {
        if let Some(path) = FileDialog::new()
            .add_filter("Font", &["ttf", "otf"])
            .pick_file()
        {
            if let Some(name) = path.file_name() {
                let dest = self.fonts_dir.join(name);
                let _ = std::fs::copy(&path, &dest);
                self.fonts = Self::scan_fonts(&self.fonts_dir);
                // Select the new font
                let name_str = name.to_string_lossy().to_string();
                if let Some(pos) = self.fonts.iter().position(|f| f.name == name_str) {
                    self.selected_font_idx = pos;
                }
                self.mark_dirty();
            }
        }
    }

    fn apply_output_size(&mut self) {
        let orig = match &self.original_image {
            Some(img) => img.clone(),
            None => return,
        };

        let (_, target) = OUTPUT_SIZES[self.output_size_idx];
        let target = match target {
            Some(t) => t,
            None => {
                self.display_image = Some(orig);
                return;
            }
        };

        let (orig_w, orig_h) = orig.dimensions();
        let (target_w, target_h) = target;

        let scale = (target_w as f32 / orig_w as f32).min(target_h as f32 / orig_h as f32);
        let new_w = (orig_w as f32 * scale) as u32;
        let new_h = (orig_h as f32 * scale) as u32;

        let mut canvas = DynamicImage::new_rgba8(target_w, target_h);
        // Fill white
        for y in 0..target_h {
            for x in 0..target_w {
                canvas.put_pixel(x, y, Rgba([255, 255, 255, 255]));
            }
        }

        let resized = orig.resize_exact(new_w, new_h, image::imageops::FilterType::Lanczos3);
        let offset_x = (target_w - new_w) / 2;
        let offset_y = (target_h - new_h) / 2;

        for y in 0..new_h {
            for x in 0..new_w {
                let px = resized.get_pixel(x, y);
                canvas.put_pixel(offset_x + x, offset_y + y, px);
            }
        }

        self.display_image = Some(canvas);
    }

    fn auto_calculate_params(&mut self) {
        let img = match &self.display_image {
            Some(img) => img,
            None => return,
        };

        let cols: usize = match self.grid_columns.parse() {
            Ok(v) if v > 0 => v,
            _ => return,
        };
        let rows: usize = match self.grid_rows.parse() {
            Ok(v) if v > 0 => v,
            _ => return,
        };

        if let Ok(metrics) = calculate_grid_metrics(img.dimensions(), cols, rows, 4) {
            self.start_x = format!("{:.1}", metrics.start_x);
            self.start_y = format!("{:.1}", metrics.start_y);
            self.cell_width = format!("{:.1}", metrics.cell_width);
            self.cell_height = format!("{:.1}", metrics.cell_height);
            self.font_size = format!("{:.1}", metrics.font_size);
        }
    }

    fn render_preview(&mut self, ctx: &Context) {
        let img = match &self.display_image {
            Some(img) => img,
            None => {
                self.preview_dirty = false;
                return;
            }
        };

        let (img_w, img_h) = img.dimensions();
        let mut preview = RgbaImage::from_pixel(img_w, img_h, Rgba([255, 255, 255, 255]));

        // Copy the display image
        for y in 0..img_h {
            for x in 0..img_w {
                preview.put_pixel(x, y, img.get_pixel(x, y));
            }
        }

        // Parse params
        let start_x: f32 = self.start_x.parse().unwrap_or(0.0);
        let start_y: f32 = self.start_y.parse().unwrap_or(0.0);
        let cell_width: f32 = self.cell_width.parse().unwrap_or(50.0);
        let cell_height: f32 = self.cell_height.parse().unwrap_or(50.0);
        let columns: usize = self.grid_columns.parse().unwrap_or(10);
        let rows: usize = self.grid_rows.parse().unwrap_or(10);
        let font_size: f32 = self.font_size.parse().unwrap_or(30.0);
        let offset_x: f32 = self.offset_x.parse().unwrap_or(0.0);
        let offset_y: f32 = self.offset_y.parse().unwrap_or(0.0);
        let line_thickness: f32 = self.line_thickness.parse().unwrap_or(1.0_f32).max(1.0_f32);

        // Draw grid (bounded to grid area, matching Python behavior)
        let actual_cw = cell_width + line_thickness;
        let actual_ch = cell_height + line_thickness;

        let grid_x1 = start_x.round() as i32;
        let grid_y1 = start_y.round() as i32;
        let grid_x2 = (start_x + columns as f32 * actual_cw).round() as i32;
        let grid_y2 = (start_y + rows as f32 * actual_ch).round() as i32;

        let lt = line_thickness.round() as i32;
        for i in 0..=columns {
            let x = (start_x + i as f32 * actual_cw).round() as i32;
            for dx in 0..lt.max(1) {
                let px = x + dx;
                if px >= 0 && (px as u32) < img_w {
                    for py in grid_y1.max(0)..(grid_y2 + 1).min(img_h as i32) {
                        if py >= 0 {
                            preview.put_pixel(px as u32, py as u32, Rgba([255, 0, 0, 255]));
                        }
                    }
                }
            }
        }

        for i in 0..=rows {
            let y = (start_y + i as f32 * actual_ch).round() as i32;
            for dy in 0..lt.max(1) {
                let py = y + dy;
                if py >= 0 && (py as u32) < img_h {
                    for px in grid_x1.max(0)..(grid_x2 + 1).min(img_w as i32) {
                        if px >= 0 {
                            preview.put_pixel(px as u32, py as u32, Rgba([255, 0, 0, 255]));
                        }
                    }
                }
            }
        }

        // Draw text
        if !self.text_input.is_empty() && self.selected_font_idx < self.fonts.len() {
            let ref font = self.fonts[self.selected_font_idx].font;
            let paragraphs = split_text_paragraphs(&self.text_input);

            let mut cur_x = start_x;
            let mut cur_y = start_y;
            let mut line_count: usize = 0;

            for (pi, para) in paragraphs.iter().enumerate() {
                if pi == 0 && self.first_line_newline {
                    cur_x = start_x;
                    cur_y += actual_ch;
                }

                if pi == 0 && self.first_line_indent {
                    cur_x += actual_cw * 2.0;
                    line_count = 2;
                } else if pi > 0 {
                    if line_count > 0 {
                        cur_x = start_x;
                        cur_y += actual_ch;
                    }
                    cur_x += actual_cw * 2.0;
                    line_count = 2;
                } else {
                    line_count = 0;
                }

                for ch in para.chars() {
                    if line_count >= columns {
                        cur_x = start_x;
                        cur_y += actual_ch;
                        line_count = 0;
                    }

                    let (metrics, bitmap) = font.rasterize(ch, font_size);
                    if metrics.width > 0 && metrics.height > 0 {
                        let bx = cur_x + (actual_cw - metrics.width as f32) / 2.0 + offset_x;
                        let by = cur_y + (actual_ch - metrics.height as f32) / 2.0 + offset_y;

                        for gy in 0..metrics.height {
                            for gx in 0..metrics.width {
                                let cov = bitmap[gy * metrics.width + gx] as f32 / 255.0;
                                if cov > 0.0 {
                                    let px = (bx + gx as f32) as i32;
                                    let py = (by + gy as f32) as i32;
                                    if px >= 0
                                        && py >= 0
                                        && (px as u32) < img_w
                                        && (py as u32) < img_h
                                    {
                                        let existing = preview.get_pixel(px as u32, py as u32);
                                        let r = (0.0_f32
                                            .mul_add(cov, existing.0[0] as f32 * (1.0 - cov)))
                                            as u8;
                                        let g = (0.0_f32
                                            .mul_add(cov, existing.0[1] as f32 * (1.0 - cov)))
                                            as u8;
                                        let b = (255.0_f32
                                            .mul_add(cov, existing.0[2] as f32 * (1.0 - cov)))
                                            as u8;
                                        preview.put_pixel(
                                            px as u32,
                                            py as u32,
                                            Rgba([r, g, b, 255]),
                                        );
                                    }
                                }
                            }
                        }
                    }

                    cur_x += actual_cw;
                    line_count += 1;
                }
            }
        }

        // Upload to GPU
        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            [img_w as usize, img_h as usize],
            preview.as_raw(),
        );
        self.preview_texture = Some(ctx.load_texture("preview", color_image, Default::default()));
        self.preview_dirty = false;
    }

    fn generate_and_save(&mut self) {
        let img = match &self.display_image {
            Some(img) => img,
            None => {
                self.status_message = Some("请先上传格子纸图片".into());
                return;
            }
        };

        if self.selected_font_idx >= self.fonts.len() {
            self.status_message = Some("请先选择字体".into());
            return;
        }

        let (img_w, img_h) = img.dimensions();
        let mut result = RgbaImage::from_pixel(img_w, img_h, Rgba([255, 255, 255, 255]));
        for y in 0..img_h {
            for x in 0..img_w {
                result.put_pixel(x, y, img.get_pixel(x, y));
            }
        }

        // Parse params
        let start_x: f32 = self.start_x.parse().unwrap_or(0.0);
        let start_y: f32 = self.start_y.parse().unwrap_or(0.0);
        let cell_width: f32 = self.cell_width.parse().unwrap_or(50.0);
        let cell_height: f32 = self.cell_height.parse().unwrap_or(50.0);
        let columns: usize = self.grid_columns.parse().unwrap_or(10);
        let rows: usize = self.grid_rows.parse().unwrap_or(10);
        let font_size: f32 = self.font_size.parse().unwrap_or(30.0);
        let offset_x: f32 = self.offset_x.parse().unwrap_or(0.0);
        let offset_y: f32 = self.offset_y.parse().unwrap_or(0.0);
        let line_thickness: f32 = self.line_thickness.parse().unwrap_or(1.0_f32).max(1.0_f32);

        let actual_cw = cell_width + line_thickness;
        let actual_ch = cell_height + line_thickness;

        let grid_x1 = start_x.round() as i32;
        let grid_y1 = start_y.round() as i32;
        let grid_x2 = (start_x + columns as f32 * actual_cw).round() as i32;
        let grid_y2 = (start_y + rows as f32 * actual_ch).round() as i32;

        // Draw grid lines (black for final output, bounded to grid area)
        let lt = line_thickness.round() as i32;
        for i in 0..=columns {
            let x = (start_x + i as f32 * actual_cw).round() as i32;
            for dx in 0..lt.max(1) {
                let px = x + dx;
                if px >= 0 && (px as u32) < img_w {
                    for py in grid_y1.max(0)..(grid_y2 + 1).min(img_h as i32) {
                        if py >= 0 {
                            result.put_pixel(px as u32, py as u32, Rgba([0, 0, 0, 255]));
                        }
                    }
                }
            }
        }
        for i in 0..=rows {
            let y = (start_y + i as f32 * actual_ch).round() as i32;
            for dy in 0..lt.max(1) {
                let py = y + dy;
                if py >= 0 && (py as u32) < img_h {
                    for px in grid_x1.max(0)..(grid_x2 + 1).min(img_w as i32) {
                        if px >= 0 {
                            result.put_pixel(px as u32, py as u32, Rgba([0, 0, 0, 255]));
                        }
                    }
                }
            }
        }

        // Draw text (black)
        if !self.text_input.is_empty() {
            let ref font = self.fonts[self.selected_font_idx].font;
            let paragraphs = split_text_paragraphs(&self.text_input);

            let mut cur_x = start_x;
            let mut cur_y = start_y;
            let mut line_count: usize = 0;

            for (pi, para) in paragraphs.iter().enumerate() {
                if pi == 0 && self.first_line_newline {
                    cur_x = start_x;
                    cur_y += actual_ch;
                }
                if pi == 0 && self.first_line_indent {
                    cur_x += actual_cw * 2.0;
                    line_count = 2;
                } else if pi > 0 {
                    if line_count > 0 {
                        cur_x = start_x;
                        cur_y += actual_ch;
                    }
                    cur_x += actual_cw * 2.0;
                    line_count = 2;
                } else {
                    line_count = 0;
                }
                for ch in para.chars() {
                    if line_count >= columns {
                        cur_x = start_x;
                        cur_y += actual_ch;
                        line_count = 0;
                    }
                    let (metrics, bitmap) = font.rasterize(ch, font_size);
                    if metrics.width > 0 && metrics.height > 0 {
                        let bx = cur_x + (actual_cw - metrics.width as f32) / 2.0 + offset_x;
                        let by = cur_y + (actual_ch - metrics.height as f32) / 2.0 + offset_y;
                        for gy in 0..metrics.height {
                            for gx in 0..metrics.width {
                                let cov = bitmap[gy * metrics.width + gx] as f32 / 255.0;
                                if cov > 0.0 {
                                    let px = (bx + gx as f32) as i32;
                                    let py = (by + gy as f32) as i32;
                                    if px >= 0
                                        && py >= 0
                                        && (px as u32) < img_w
                                        && (py as u32) < img_h
                                    {
                                        result.put_pixel(
                                            px as u32,
                                            py as u32,
                                            Rgba([0, 0, 0, 255]),
                                        );
                                    }
                                }
                            }
                        }
                    }
                    cur_x += actual_cw;
                    line_count += 1;
                }
            }
        }

        // Save dialog
        let ts = Local::now().format("%Y%m%d_%H%M%S");
        let default_name = format!("output_{}.png", ts);

        if let Some(save_path) = FileDialog::new()
            .add_filter("PNG", &["png"])
            .set_file_name(&default_name)
            .set_directory(&self.output_dir)
            .save_file()
        {
            match result.save(&save_path) {
                Ok(_) => {
                    self.status_message = Some(format!("图片已保存: {}", save_path.display()));
                }
                Err(e) => {
                    self.status_message = Some(format!("保存失败: {}", e));
                }
            }
        }
    }

    fn save_config(&self) {
        let settings = serde_json::json!({
            "start_x": format!("{:.1}", self.start_x.parse::<f32>().unwrap_or(0.0)),
            "start_y": format!("{:.1}", self.start_y.parse::<f32>().unwrap_or(0.0)),
            "cell_width": format!("{:.1}", self.cell_width.parse::<f32>().unwrap_or(0.0)),
            "cell_height": format!("{:.1}", self.cell_height.parse::<f32>().unwrap_or(0.0)),
            "font_size": self.font_size,
            "offset_x": self.offset_x,
            "offset_y": self.offset_y,
            "grid_columns": self.grid_columns,
            "grid_rows": self.grid_rows,
            "line_thickness": format!("{:.1}", self.line_thickness.parse::<f32>().unwrap_or(1.0)),
        });

        if let Some(path) = FileDialog::new()
            .add_filter("JSON", &["json"])
            .set_directory(&self.config_dir)
            .save_file()
        {
            let _ = std::fs::write(&path, serde_json::to_string_pretty(&settings).unwrap());
        }
    }

    fn load_config(&mut self) {
        if let Some(path) = FileDialog::new()
            .add_filter("JSON", &["json"])
            .set_directory(&self.config_dir)
            .pick_file()
        {
            if let Ok(data) = std::fs::read_to_string(&path) {
                if let Ok(settings) = serde_json::from_str::<serde_json::Value>(&data) {
                    if let Some(v) = settings.get("start_x") {
                        self.start_x = v.as_str().unwrap_or("0.0").into();
                    }
                    if let Some(v) = settings.get("start_y") {
                        self.start_y = v.as_str().unwrap_or("0.0").into();
                    }
                    if let Some(v) = settings.get("cell_width") {
                        self.cell_width = v.as_str().unwrap_or("0.0").into();
                    }
                    if let Some(v) = settings.get("cell_height") {
                        self.cell_height = v.as_str().unwrap_or("0.0").into();
                    }
                    if let Some(v) = settings.get("font_size") {
                        self.font_size = v.as_str().unwrap_or("").into();
                    }
                    if let Some(v) = settings.get("offset_x") {
                        self.offset_x = v.as_str().unwrap_or("1").into();
                    }
                    if let Some(v) = settings.get("offset_y") {
                        self.offset_y = v.as_str().unwrap_or("-1").into();
                    }
                    if let Some(v) = settings.get("grid_columns") {
                        self.grid_columns = v.as_str().unwrap_or("").into();
                    }
                    if let Some(v) = settings.get("grid_rows") {
                        self.grid_rows = v.as_str().unwrap_or("").into();
                    }
                    if let Some(v) = settings.get("line_thickness") {
                        self.line_thickness = v.as_str().unwrap_or("1.0").into();
                    }
                    self.mark_dirty();
                }
            }
        }
    }

    fn load_default_config(&mut self) {
        let default = self.config_dir.join("default.json");
        if default.exists() {
            if let Ok(data) = std::fs::read_to_string(&default) {
                if let Ok(settings) = serde_json::from_str::<serde_json::Value>(&data) {
                    if let Some(v) = settings.get("start_x") {
                        self.start_x = v.as_str().unwrap_or("").into();
                    }
                    if let Some(v) = settings.get("start_y") {
                        self.start_y = v.as_str().unwrap_or("").into();
                    }
                    if let Some(v) = settings.get("cell_width") {
                        self.cell_width = v.as_str().unwrap_or("").into();
                    }
                    if let Some(v) = settings.get("cell_height") {
                        self.cell_height = v.as_str().unwrap_or("").into();
                    }
                    if let Some(v) = settings.get("font_size") {
                        self.font_size = v.as_str().unwrap_or("").into();
                    }
                    if let Some(v) = settings.get("offset_x") {
                        self.offset_x = v.as_str().unwrap_or("1").into();
                    }
                    if let Some(v) = settings.get("offset_y") {
                        self.offset_y = v.as_str().unwrap_or("-1").into();
                    }
                    if let Some(v) = settings.get("grid_columns") {
                        self.grid_columns = v.as_str().unwrap_or("").into();
                    }
                    if let Some(v) = settings.get("grid_rows") {
                        self.grid_rows = v.as_str().unwrap_or("").into();
                    }
                    if let Some(v) = settings.get("line_thickness") {
                        self.line_thickness = v.as_str().unwrap_or("1.0").into();
                    }
                }
            }
        }
    }

    fn section_frame(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
        egui::Frame::default()
            .fill(egui::Color32::from_rgb(250, 251, 253))
            .stroke(egui::Stroke::new(
                1.0,
                egui::Color32::from_rgb(226, 231, 238),
            ))
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(egui::Margin::symmetric(12, 10))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new(title)
                        .size(14.0)
                        .strong()
                        .color(egui::Color32::from_rgb(36, 48, 64)),
                );
                ui.add_space(8.0);
                add_contents(ui);
            });
    }
}

impl eframe::App for MatrixPrintingApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        // Apply visual style on first frame
        if self.first_frame {
            self.first_frame = false;
            let mut style = (*ctx.style()).clone();
            style.spacing.item_spacing = egui::vec2(10.0, 8.0);
            style.spacing.button_padding = egui::vec2(14.0, 8.0);
            style.spacing.indent = 12.0;
            style.visuals.panel_fill = egui::Color32::from_rgb(243, 246, 250);
            style.visuals.window_fill = egui::Color32::from_rgb(243, 246, 250);
            style.visuals.override_text_color = Some(egui::Color32::from_rgb(34, 42, 54));
            style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(248, 250, 252);
            style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(238, 242, 247);
            style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(224, 233, 246);
            style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(208, 222, 242);
            style.visuals.selection.bg_fill = egui::Color32::from_rgb(45, 103, 185);
            ctx.set_style(style);
        }

        // Check if params changed
        let hash = self.params_hash();
        if hash != self.last_params_hash {
            self.last_params_hash = hash;
            self.mark_dirty();
        }

        // Render preview if dirty
        if self.preview_dirty {
            self.render_preview(ctx);
        }

        // ===================== Left Panel =====================
        egui::SidePanel::left("settings_panel")
            .resizable(true)
            .default_width(286.0)
            .min_width(240.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new("设置")
                            .size(20.0)
                            .strong()
                            .color(egui::Color32::from_rgb(28, 38, 52)),
                    );

                    ui.add_space(10.0);
                    Self::section_frame(ui, "图片设置", |ui| {
                        egui::Grid::new("image_settings_grid")
                            .num_columns(2)
                            .spacing([10.0, 8.0])
                            .show(ui, |ui| {
                                ui.label("输出尺寸");
                                egui::ComboBox::from_id_salt("output_size")
                                    .selected_text(OUTPUT_SIZES[self.output_size_idx].0)
                                    .width(ui.available_width())
                                    .show_ui(ui, |ui| {
                                        for (i, (name, _)) in OUTPUT_SIZES.iter().enumerate() {
                                            ui.selectable_value(
                                                &mut self.output_size_idx,
                                                i,
                                                *name,
                                            );
                                        }
                                    });
                                ui.end_row();
                            });
                        ui.add_space(8.0);
                        if ui
                            .add(
                                egui::Button::new("上传格子图片")
                                    .min_size(egui::vec2(ui.available_width(), 32.0)),
                            )
                            .clicked()
                        {
                            self.load_image_dialog();
                        }
                        if ui
                            .add(
                                egui::Button::new("上传字体文件")
                                    .min_size(egui::vec2(ui.available_width(), 32.0)),
                            )
                            .clicked()
                        {
                            self.load_font_dialog();
                        }
                    });

                    ui.add_space(8.0);

                    Self::section_frame(ui, "网格参数", |ui| {
                        egui::Grid::new("grid_settings_grid")
                            .num_columns(3)
                            .spacing([10.0, 8.0])
                            .show(ui, |ui| {
                                ui.label("每行格子数");
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.grid_columns)
                                        .desired_width(76.0),
                                );
                                ui.label("列");
                                ui.end_row();

                                ui.label("格子行数");
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.grid_rows)
                                        .desired_width(76.0),
                                );
                                ui.label("行");
                                ui.end_row();
                            });

                        ui.add_space(8.0);
                        if !self.fonts.is_empty() {
                            ui.label("选择字体");
                            egui::ComboBox::from_id_salt("font_select")
                                .selected_text(
                                    self.fonts
                                        .get(self.selected_font_idx)
                                        .map(|f| f.name.as_str())
                                        .unwrap_or(""),
                                )
                                .width(ui.available_width())
                                .show_ui(ui, |ui| {
                                    for (i, fd) in self.fonts.iter().enumerate() {
                                        ui.selectable_value(
                                            &mut self.selected_font_idx,
                                            i,
                                            &fd.name,
                                        );
                                    }
                                });
                        } else {
                            ui.label(
                                egui::RichText::new("未找到字体，请上传 .ttf/.otf 文件")
                                    .color(egui::Color32::from_rgb(180, 92, 0))
                                    .size(12.0),
                            );
                        }
                    });

                    ui.add_space(8.0);

                    Self::section_frame(ui, "参数微调", |ui| {
                        egui::Grid::new("fine_tune_grid")
                            .num_columns(2)
                            .spacing([12.0, 7.0])
                            .show(ui, |ui| {
                                macro_rules! field {
                                    ($label:expr, $var:expr, $hint:expr) => {
                                        ui.label($label);
                                        ui.add(
                                            egui::TextEdit::singleline($var)
                                                .desired_width(86.0)
                                                .hint_text($hint),
                                        );
                                        ui.end_row();
                                    };
                                }
                                field!("起始 X", &mut self.start_x, "0");
                                field!("起始 Y", &mut self.start_y, "0");
                                field!("格子宽度", &mut self.cell_width, "50");
                                field!("格子高度", &mut self.cell_height, "50");
                                field!("线条粗细", &mut self.line_thickness, "2");
                                field!("字体大小", &mut self.font_size, "30");
                                field!("X 偏移", &mut self.offset_x, "0");
                                field!("Y 偏移", &mut self.offset_y, "0");
                            });
                    });

                    ui.add_space(8.0);

                    Self::section_frame(ui, "预设管理", |ui| {
                        if ui
                            .add(
                                egui::Button::new("保存当前参数")
                                    .min_size(egui::vec2(ui.available_width(), 30.0)),
                            )
                            .clicked()
                        {
                            self.save_config();
                        }
                        if ui
                            .add(
                                egui::Button::new("加载参数配置")
                                    .min_size(egui::vec2(ui.available_width(), 30.0)),
                            )
                            .clicked()
                        {
                            self.load_config();
                        }
                    });

                    ui.add_space(8.0);
                });
            });

        // ===================== Right Panel =====================
        egui::SidePanel::right("preview_panel")
            .resizable(true)
            .default_width(420.0)
            .min_width(280.0)
            .show(ctx, |ui| {
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new("预览")
                        .size(20.0)
                        .strong()
                        .color(egui::Color32::from_rgb(28, 38, 52)),
                );

                ui.add_space(10.0);
                if let Some(ref handle) = self.preview_texture {
                    let available = ui.available_size();
                    let img_size = handle.size_vec2();
                    if img_size.x > 0.0 && img_size.y > 0.0 {
                        let scale = (available.x / img_size.x)
                            .min(available.y / img_size.y)
                            .min(1.0);
                        let display = [img_size.x * scale, img_size.y * scale];

                        ui.centered_and_justified(|ui| {
                            ui.add(egui::Image::new(egui::ImageSource::Texture(
                                egui::load::SizedTexture::new(handle.id(), display),
                            )));
                        });
                    }
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label(
                            egui::RichText::new("请先上传格子纸图片\n并设置网格参数")
                                .color(egui::Color32::from_rgb(150, 150, 150))
                                .size(14.0),
                        );
                    });
                }
            });

        // ===================== Center Panel =====================
        // CentralPanel must be added after side panels, otherwise the preview panel
        // can overlap the editor instead of reducing the editor's available width.
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new("文本输入")
                    .size(20.0)
                    .strong()
                    .color(egui::Color32::from_rgb(28, 38, 52)),
            );

            ui.add_space(10.0);
            egui::Frame::default()
                .fill(egui::Color32::from_rgb(250, 251, 253))
                .stroke(egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgb(226, 231, 238),
                ))
                .corner_radius(egui::CornerRadius::same(8))
                .inner_margin(egui::Margin::symmetric(12, 10))
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.checkbox(&mut self.first_line_indent, "首行缩进（空两格）");
                        ui.add_space(16.0);
                        ui.checkbox(&mut self.first_line_newline, "首行换行");
                    });
                });

            ui.add_space(10.0);

            let mut cjk_layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
                let mut job = egui::text::LayoutJob::default();
                let font_size = ui.style().text_styles[&egui::TextStyle::Body].size;
                job.wrap.max_width = wrap_width.max(1.0);
                job.wrap.break_anywhere = true;
                job.break_on_newline = true;
                job.append(
                    text,
                    0.0,
                    egui::text::TextFormat::simple(
                        egui::FontId::proportional(font_size),
                        egui::Color32::BLACK,
                    ),
                );
                ui.fonts(|f| f.layout_job(job))
            };

            let editor_width = ui.available_width().max(120.0);
            ui.add_sized(
                [editor_width, ui.available_height().max(360.0) - 76.0],
                egui::TextEdit::multiline(&mut self.text_input)
                    .layouter(&mut cjk_layouter)
                    .desired_width(editor_width)
                    .desired_rows(24)
                    .hint_text("在此粘贴要排版到格子纸上的文字...")
                    .font(egui::TextStyle::Body),
            );

            ui.add_space(10.0);
            let gen_btn = egui::Button::new(
                egui::RichText::new("生成图片")
                    .size(17.0)
                    .strong()
                    .color(egui::Color32::WHITE),
            )
            .min_size(egui::vec2(ui.available_width(), 42.0))
            .fill(egui::Color32::from_rgb(45, 103, 185));
            ui.horizontal(|ui| {
                if ui.add(gen_btn).clicked() {
                    self.generate_and_save();
                }
            });

            if let Some(ref msg) = self.status_message {
                ui.add_space(8.0);
                egui::Frame::default()
                    .fill(egui::Color32::from_rgb(232, 246, 236))
                    .stroke(egui::Stroke::new(
                        1.0,
                        egui::Color32::from_rgb(178, 222, 190),
                    ))
                    .corner_radius(egui::CornerRadius::same(6))
                    .inner_margin(egui::Margin::symmetric(10, 8))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(msg)
                                .color(egui::Color32::from_rgb(34, 116, 58))
                                .size(13.0),
                        );
                    });
            }
        });

        // Request repaint when dirty to update texture
        if self.preview_dirty {
            ctx.request_repaint();
        }
    }
}
