/// Return cell size and a safe font size for the given grid settings.
pub fn calculate_grid_metrics(
    image_size: (u32, u32),
    columns: usize,
    rows: usize,
    font_padding: i32,
) -> Result<GridMetrics, String> {
    if columns == 0 || rows == 0 {
        return Err("Grid rows and columns must be positive integers".into());
    }

    let (img_w, img_h) = image_size;
    let cell_width = (img_w / columns as u32) as f32;
    let cell_height = (img_h / rows as u32) as f32;
    let font_size = (cell_width.min(cell_height) - font_padding as f32).max(1.0);

    Ok(GridMetrics {
        start_x: 0.0,
        start_y: 0.0,
        cell_width,
        cell_height,
        font_size,
    })
}

#[derive(Debug, Clone)]
pub struct GridMetrics {
    pub start_x: f32,
    pub start_y: f32,
    pub cell_width: f32,
    pub cell_height: f32,
    pub font_size: f32,
}

/// Normalize pasted text into stable paragraphs for grid rendering.
pub fn split_text_paragraphs(text: &str) -> Vec<String> {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    normalized
        .split('\n')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}
