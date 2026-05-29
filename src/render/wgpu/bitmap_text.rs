use crate::core::{Color, LayerKind, Point, Rect, Size};

use super::{PipelineKind, RenderItem, item::MAX_RENDER_ITEMS_PER_FRAME, item::paint_order};

const GLYPH_WIDTH: usize = 5;
const GLYPH_HEIGHT: usize = 7;

pub(crate) fn push_bitmap_text_runs(
    items: &mut Vec<RenderItem>,
    text: &str,
    origin: Point,
    color: Color,
    size: f32,
    z_index: i32,
    order: usize,
    layer: LayerKind,
    clip_rect: Option<Rect>,
) -> super::RendererResult<()> {
    push_bitmap_text_runs_with_pipeline(
        items,
        text,
        origin,
        color,
        size,
        z_index,
        order,
        layer,
        clip_rect,
        PipelineKind::SolidRect,
    )
}

pub(crate) fn push_bitmap_text_runs_with_pipeline(
    items: &mut Vec<RenderItem>,
    text: &str,
    origin: Point,
    color: Color,
    size: f32,
    z_index: i32,
    order: usize,
    layer: LayerKind,
    clip_rect: Option<Rect>,
    pipeline: PipelineKind,
) -> super::RendererResult<()> {
    let scale = (size / GLYPH_HEIGHT as f32).max(1.0);
    let advance = (GLYPH_WIDTH as f32 + 1.0) * scale;
    let run_height = scale;
    let linear_color = color_to_linear(color, 1.0);
    let mut x = origin.x;
    let mut visible_index = 0usize;

    for ch in text.chars() {
        if ch.is_whitespace() {
            x += advance;
            continue;
        }

        let glyph = bitmap_glyph(ch);
        for (row, bits) in glyph.iter().copied().enumerate() {
            for (run_start, run_len) in row_runs(bits) {
                let rect = Rect::new(
                    Point::new(x + run_start as f32 * scale, origin.y + row as f32 * scale),
                    Size::new(run_len as f32 * scale, run_height),
                );
                if rect.size.width == 0.0 || rect.size.height == 0.0 {
                    continue;
                }
                if items.len() >= MAX_RENDER_ITEMS_PER_FRAME {
                    return Ok(());
                }
                super::item::push_item(
                    items,
                    RenderItem {
                        layer,
                        clip_rect,
                        pipeline,
                        rect,
                        color: linear_color,
                        uv_rect: [0.0, 0.0, 1.0, 1.0],
                        radius: 0.0,
                        z_index,
                        order: paint_order(order, visible_index * 64 + row * 8 + run_start),
                    },
                )?;
            }
        }

        visible_index += 1;
        x += advance;
    }
    Ok(())
}

fn row_runs(bits: u8) -> Vec<(usize, usize)> {
    let mut runs = Vec::new();
    let mut col = 0usize;
    while col < GLYPH_WIDTH {
        if bits & (1 << (GLYPH_WIDTH - 1 - col)) == 0 {
            col += 1;
            continue;
        }
        let start = col;
        while col < GLYPH_WIDTH && bits & (1 << (GLYPH_WIDTH - 1 - col)) != 0 {
            col += 1;
        }
        runs.push((start, col - start));
    }
    runs
}

fn bitmap_glyph(ch: char) -> [u8; GLYPH_HEIGHT] {
    match ch.to_ascii_uppercase() {
        'A' => [
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'B' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
        ],
        'C' => [
            0b01111, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b01111,
        ],
        'D' => [
            0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
        ],
        'E' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
        'F' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'G' => [
            0b01111, 0b10000, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111,
        ],
        'H' => [
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'I' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111,
        ],
        'J' => [
            0b00111, 0b00010, 0b00010, 0b00010, 0b10010, 0b10010, 0b01100,
        ],
        'K' => [
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ],
        'L' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ],
        'M' => [
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ],
        'N' => [
            0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
        ],
        'O' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'P' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'Q' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101,
        ],
        'R' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ],
        'S' => [
            0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        'T' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'U' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'V' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100,
        ],
        'W' => [
            0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010,
        ],
        'X' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001,
        ],
        'Y' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'Z' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111,
        ],
        '0' => [
            0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
        ],
        '1' => [
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        '2' => [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
        ],
        '3' => [
            0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        '4' => [
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ],
        '5' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b00001, 0b00001, 0b11110,
        ],
        '6' => [
            0b01110, 0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ],
        '7' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ],
        '8' => [
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ],
        '9' => [
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001, 0b01110,
        ],
        ':' => [
            0b00000, 0b00100, 0b00100, 0b00000, 0b00100, 0b00100, 0b00000,
        ],
        '.' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b01100,
        ],
        ',' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00100, 0b00100, 0b01000,
        ],
        '-' => [
            0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000,
        ],
        '_' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b11111,
        ],
        '/' => [
            0b00001, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b10000,
        ],
        '?' => [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b00000, 0b00100,
        ],
        '!' => [
            0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100,
        ],
        _ => [
            0b11111, 0b10001, 0b00110, 0b00110, 0b00110, 0b10001, 0b11111,
        ],
    }
}

fn color_to_linear(color: Color, opacity: f32) -> [f32; 4] {
    [
        color.r as f32 / 255.0,
        color.g as f32 / 255.0,
        color.b as f32 / 255.0,
        color.a as f32 / 255.0 * opacity,
    ]
}
