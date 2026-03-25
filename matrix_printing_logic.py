"""Core calculation helpers for matrix printing."""

import re


def calculate_grid_metrics(image_size, columns, rows, font_padding=4):
    """Return cell size and a safe font size for the current grid settings."""
    if columns <= 0 or rows <= 0:
        raise ValueError("网格行数和列数必须为正整数")

    img_width, img_height = image_size
    cell_width = img_width // columns
    cell_height = img_height // rows
    font_size = max(1, min(cell_width, cell_height) - font_padding)

    return {
        "start_x": "0",
        "start_y": "0",
        "cell_width": str(cell_width),
        "cell_height": str(cell_height),
        "font_size": str(font_size),
    }


def split_text_paragraphs(text):
    """Normalize pasted text into stable paragraphs for grid rendering."""
    normalized = text.replace("\r\n", "\n").replace("\r", "\n")
    paragraphs = [
        segment.strip()
        for segment in re.split(r"\n+", normalized)
        if segment.strip()
    ]
    return paragraphs
