use super::{GpuAtlas, RenderItem, RendererResult};
#[cfg(feature = "bitmap-text-fallback")]
use super::{
    PipelineKind, RendererError,
    item::{MAX_RENDER_ITEMS_PER_FRAME, paint_order},
};
#[cfg(feature = "bitmap-text-fallback")]
use crate::core::{GlyphKey, Point, SizeU32};
use crate::core::{LayerKind, Rect, TextCmd};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextLoweringStrategy {
    Glyphon,
    BitmapFallback,
    GlyphAtlas,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextLoweringReport {
    pub strategy: TextLoweringStrategy,
    pub item_count: usize,
}

#[allow(dead_code)]
pub(crate) fn lower_text_bitmap(
    items: &mut Vec<RenderItem>,
    cmd: &TextCmd,
    command_order: usize,
    layer: LayerKind,
    clip_rect: Option<Rect>,
) -> RendererResult<TextLoweringReport> {
    #[cfg(not(feature = "bitmap-text-fallback"))]
    {
        let _ = (items, cmd, command_order, layer, clip_rect);
        return Ok(TextLoweringReport {
            strategy: TextLoweringStrategy::Glyphon,
            item_count: 0,
        });
    }

    #[cfg(feature = "bitmap-text-fallback")]
    {
        let before = items.len();
        let baseline = (cmd.size * 0.8).ceil();
        let origin = Point::new(cmd.rect.origin.x, cmd.rect.origin.y + baseline);
        super::bitmap_text::push_bitmap_text_runs(
            items,
            &cmd.text,
            origin,
            cmd.color,
            cmd.size,
            cmd.z_index,
            command_order,
            layer,
            clip_rect,
        )?;
        Ok(TextLoweringReport {
            strategy: TextLoweringStrategy::BitmapFallback,
            item_count: items.len().saturating_sub(before),
        })
    }
}

pub(crate) fn lower_text_glyph_atlas(
    items: &mut Vec<RenderItem>,
    cmd: &TextCmd,
    command_order: usize,
    layer: LayerKind,
    clip_rect: Option<Rect>,
    atlas: &mut GpuAtlas,
) -> RendererResult<TextLoweringReport> {
    #[cfg(not(feature = "bitmap-text-fallback"))]
    {
        let _ = (items, cmd, command_order, layer, clip_rect, atlas);
        return Ok(TextLoweringReport {
            strategy: TextLoweringStrategy::Glyphon,
            item_count: 0,
        });
    }

    #[cfg(feature = "bitmap-text-fallback")]
    {
        let before = items.len();
        let _ = atlas;
        if cmd.text.chars().filter(|ch| !ch.is_whitespace()).count() > MAX_RENDER_ITEMS_PER_FRAME {
            return Err(RendererError::InvalidDisplayList(format!(
                "render item limit exceeded: text visible glyphs > {MAX_RENDER_ITEMS_PER_FRAME}"
            )));
        }

        let baseline = (cmd.size * 0.8).ceil();
        let origin = Point::new(cmd.rect.origin.x, cmd.rect.origin.y + baseline);

        super::bitmap_text::push_bitmap_text_runs_with_pipeline(
            items,
            &cmd.text,
            origin,
            cmd.color,
            cmd.size,
            cmd.z_index,
            command_order,
            layer,
            clip_rect,
            PipelineKind::TextGlyph,
        )?;
        if items.len() > before {
            return Ok(TextLoweringReport {
                strategy: TextLoweringStrategy::GlyphAtlas,
                item_count: items.len().saturating_sub(before),
            });
        }

        if let Some(glyph_runs) = shape_text_for_atlas(&cmd.text, cmd.size) {
            let color = color_to_linear(cmd.color, 1.0);

            for (i, glyph) in glyph_runs.iter().enumerate() {
                let key = GlyphKey {
                    font_id: glyph.font_id,
                    glyph_id: glyph.glyph_id,
                    size_bits: cmd.size.to_bits(),
                };
                let Some(atlas_entry) = atlas.glyph_entry(key.clone()).or_else(|| {
                    atlas.reserve_glyph(
                        key,
                        SizeU32::new(glyph.width.ceil() as u32, glyph.height.ceil() as u32),
                    )
                }) else {
                    continue;
                };
                let rect = Rect::new(
                    Point::new(origin.x + glyph.x, origin.y + glyph.y),
                    crate::core::Size::new(glyph.width, glyph.height),
                );
                if rect.size.width > 0.0 && rect.size.height > 0.0 {
                    super::item::push_item(
                        items,
                        RenderItem {
                            layer,
                            clip_rect,
                            pipeline: PipelineKind::TextGlyph,
                            rect,
                            color,
                            uv_rect: atlas_entry.uv_rect,
                            radius: 0.0,
                            z_index: cmd.z_index,
                            order: paint_order(command_order, i * 64),
                        },
                    )?;
                }
            }

            if !items.is_empty() {
                return Ok(TextLoweringReport {
                    strategy: TextLoweringStrategy::GlyphAtlas,
                    item_count: items.len().saturating_sub(before),
                });
            }
        }

        lower_text_bitmap(items, cmd, command_order, layer, clip_rect)
    }
}

#[cfg(feature = "bitmap-text-fallback")]
fn shape_text_for_atlas(text: &str, font_size: f32) -> Option<Vec<GlyphPosition>> {
    use glyphon::cosmic_text::{Attrs, Buffer, Family, Metrics, Shaping, Wrap};

    let mut font_system = glyphon::cosmic_text::FontSystem::new();
    let font_px = font_size.max(1.0);
    let line_height = (font_px * 1.2).ceil();
    let metrics = Metrics::new(font_px, line_height);
    let mut buffer = Buffer::new(&mut font_system, metrics);

    let attrs = Attrs::new().family(Family::SansSerif);
    buffer.set_size(&mut font_system, Some(f32::MAX), Some(line_height * 50.0));
    buffer.set_text(&mut font_system, text, &attrs, Shaping::Advanced, None);
    buffer.set_wrap(&mut font_system, Wrap::Word);
    buffer.shape_until_scroll(&mut font_system, false);

    let layout_runs: Vec<_> = buffer.layout_runs().collect();
    if layout_runs.is_empty() {
        return None;
    }

    let mut glyphs = Vec::new();
    for run in &layout_runs {
        let font_id = 1u64;
        for glyph in run.glyphs.iter() {
            glyphs.push(GlyphPosition {
                font_id,
                glyph_id: glyph.glyph_id as u32,
                x: glyph.x,
                y: run.line_y - run.line_top,
                width: glyph.w,
                height: line_height,
            });
        }
    }

    Some(glyphs)
}

#[cfg(feature = "bitmap-text-fallback")]
#[derive(Clone, Debug)]
struct GlyphPosition {
    font_id: u64,
    glyph_id: u32,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[cfg(feature = "bitmap-text-fallback")]
fn color_to_linear(color: crate::core::Color, opacity: f32) -> [f32; 4] {
    [
        color.r as f32 / 255.0,
        color.g as f32 / 255.0,
        color.b as f32 / 255.0,
        color.a as f32 / 255.0 * opacity,
    ]
}
