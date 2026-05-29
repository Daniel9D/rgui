use crate::core::{
    Color, DisplayList, FontStyle, FontWeight, PaintCommand, Rect, SizeU32, effective_clip,
};

use super::{RendererError, RendererResult};

pub struct GlyphonTextBridge {
    font_system: glyphon::FontSystem,
    swash_cache: glyphon::SwashCache,
    cache: glyphon::Cache,
    atlas: glyphon::TextAtlas,
    renderer: glyphon::TextRenderer,
    viewport: glyphon::Viewport,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GlyphonTextStats {
    pub glyphon_enabled: bool,
    pub text_area_count: usize,
    pub clipped_text_area_count: usize,
    pub skipped_text_area_count: usize,
    pub glyph_count: usize,
    pub fallback_used: bool,
}

struct PreparedTextArea {
    buffer: glyphon::Buffer,
    left: f32,
    top: f32,
    bounds: glyphon::TextBounds,
    color: glyphon::Color,
    clipped: bool,
}

impl GlyphonTextBridge {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let cache = glyphon::Cache::new(device);
        let mut atlas = glyphon::TextAtlas::new(device, queue, &cache, format);
        let renderer =
            glyphon::TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);
        let viewport = glyphon::Viewport::new(device, &cache);

        Self {
            font_system: glyphon::FontSystem::new(),
            swash_cache: glyphon::SwashCache::new(),
            cache,
            atlas,
            renderer,
            viewport,
        }
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        display_list: &DisplayList,
        viewport_size: SizeU32,
    ) -> RendererResult<GlyphonTextStats> {
        self.viewport.update(
            queue,
            glyphon::Resolution {
                width: viewport_size.width,
                height: viewport_size.height,
            },
        );

        let (prepared, skipped_text_area_count) =
            self.prepare_text_areas(display_list, viewport_size);
        let stats = GlyphonTextStats {
            glyphon_enabled: true,
            text_area_count: prepared.len(),
            clipped_text_area_count: prepared.iter().filter(|area| area.clipped).count(),
            skipped_text_area_count,
            glyph_count: prepared
                .iter()
                .map(|area| {
                    area.buffer
                        .layout_runs()
                        .map(|run| run.glyphs.len())
                        .sum::<usize>()
                })
                .sum(),
            fallback_used: false,
        };

        let areas = prepared.iter().map(|area| glyphon::TextArea {
            buffer: &area.buffer,
            left: area.left,
            top: area.top,
            scale: 1.0,
            bounds: area.bounds,
            default_color: area.color,
            custom_glyphs: &[],
        });

        self.renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                areas,
                &mut self.swash_cache,
            )
            .map_err(|err| {
                RendererError::InvalidDisplayList(format!("glyphon prepare failed: {err:?}"))
            })?;

        Ok(stats)
    }

    pub fn render(&self, pass: &mut wgpu::RenderPass<'_>) -> RendererResult<()> {
        self.renderer
            .render(&self.atlas, &self.viewport, pass)
            .map_err(|err| {
                RendererError::InvalidDisplayList(format!("glyphon render failed: {err:?}"))
            })
    }

    fn prepare_text_areas(
        &mut self,
        display_list: &DisplayList,
        viewport_size: SizeU32,
    ) -> (Vec<PreparedTextArea>, usize) {
        let mut clip_stack = Vec::new();
        let mut areas = Vec::new();
        let mut skipped_text_area_count = 0usize;

        for command in display_list.commands() {
            match command {
                PaintCommand::PushClip(spec) => clip_stack.push(spec.rect),
                PaintCommand::PopClip => {
                    clip_stack.pop();
                }
                PaintCommand::DrawText(cmd) => {
                    if cmd.text.is_empty() {
                        continue;
                    }
                    let rect = text_rect(cmd);
                    if rect.size.width <= 0.0 || rect.size.height <= 0.0 {
                        skipped_text_area_count += 1;
                        continue;
                    }

                    let viewport = viewport_rect(viewport_size);
                    let Some(clip) = (if clip_stack.is_empty() {
                        Some(viewport)
                    } else {
                        effective_clip(&clip_stack, viewport)
                    }) else {
                        skipped_text_area_count += 1;
                        continue;
                    };
                    if clip.size.width <= 0.0 || clip.size.height <= 0.0 {
                        skipped_text_area_count += 1;
                        continue;
                    }
                    let Some(_visible_text_rect) = rect.intersect(clip) else {
                        skipped_text_area_count += 1;
                        continue;
                    };

                    let mut buffer = glyphon::Buffer::new(
                        &mut self.font_system,
                        glyphon::Metrics::new(
                            cmd.size.max(1.0),
                            cmd.line_height.unwrap_or(cmd.size * 1.2).max(1.0),
                        ),
                    );
                    buffer.set_size(
                        &mut self.font_system,
                        Some(rect.size.width.max(1.0)),
                        Some(rect.size.height.max(1.0)),
                    );
                    buffer.set_text(
                        &mut self.font_system,
                        &cmd.text,
                        &attrs_for(cmd.font_weight, cmd.font_style),
                        glyphon::Shaping::Advanced,
                        None,
                    );
                    buffer.set_wrap(&mut self.font_system, glyphon::Wrap::Word);
                    buffer.shape_until_scroll(&mut self.font_system, false);
                    if std::env::var_os("RGUI_DEBUG_TEXT").is_some() {
                        eprintln!(
                            "{}",
                            debug_text_area_line(&cmd.text, rect, clip, text_bounds(clip))
                        );
                    }
                    areas.push(PreparedTextArea {
                        buffer,
                        left: rect.origin.x,
                        top: rect.origin.y,
                        bounds: text_bounds(clip),
                        color: to_glyphon_color(cmd.color),
                        clipped: !clip_stack.is_empty(),
                    });
                }
                _ => {}
            }
        }

        (areas, skipped_text_area_count)
    }
}

impl Drop for GlyphonTextBridge {
    fn drop(&mut self) {
        let _ = &self.cache;
    }
}

fn text_rect(cmd: &crate::core::TextCmd) -> Rect {
    if cmd.rect.size.width > 0.0 && cmd.rect.size.height > 0.0 {
        return cmd.rect;
    }
    let line_height = cmd.line_height.unwrap_or(cmd.size * 1.2).max(1.0);
    let glyph_count = cmd.text.chars().count().max(1) as f32;
    Rect::new(
        cmd.rect.origin,
        crate::core::Size::new((glyph_count * cmd.size).max(1.0), line_height),
    )
}

fn attrs_for(weight: FontWeight, style: FontStyle) -> glyphon::Attrs<'static> {
    glyphon::Attrs::new()
        .family(glyphon::Family::SansSerif)
        .weight(match weight {
            FontWeight::Thin => glyphon::Weight::THIN,
            FontWeight::ExtraLight => glyphon::Weight::EXTRA_LIGHT,
            FontWeight::Light => glyphon::Weight::LIGHT,
            FontWeight::Normal => glyphon::Weight::NORMAL,
            FontWeight::Medium => glyphon::Weight::MEDIUM,
            FontWeight::Semibold => glyphon::Weight::SEMIBOLD,
            FontWeight::Bold => glyphon::Weight::BOLD,
            FontWeight::ExtraBold => glyphon::Weight::EXTRA_BOLD,
            FontWeight::Black => glyphon::Weight::BLACK,
            FontWeight::Number(n) => glyphon::Weight(n as u16),
        })
        .style(match style {
            FontStyle::Normal => glyphon::Style::Normal,
            FontStyle::Italic => glyphon::Style::Italic,
            FontStyle::Oblique => glyphon::Style::Oblique,
        })
}

fn to_glyphon_color(color: Color) -> glyphon::Color {
    glyphon::Color::rgba(color.r, color.g, color.b, color.a)
}

fn text_bounds(rect: Rect) -> glyphon::TextBounds {
    glyphon::TextBounds {
        left: rect.origin.x.floor() as i32,
        top: rect.origin.y.floor() as i32,
        right: rect.max_x().ceil() as i32,
        bottom: rect.max_y().ceil() as i32,
    }
}

fn viewport_rect(size: SizeU32) -> Rect {
    Rect::new(
        crate::core::Point::new(0.0, 0.0),
        crate::core::Size::new(size.width as f32, size.height as f32),
    )
}

fn debug_text_area_line(text: &str, rect: Rect, clip: Rect, bounds: glyphon::TextBounds) -> String {
    format!(
        "TEXT text={text:?} rect=({:.1},{:.1},{:.1},{:.1}) clip=({:.1},{:.1},{:.1},{:.1}) bounds=({}, {}, {}, {})",
        rect.origin.x,
        rect.origin.y,
        rect.size.width,
        rect.size.height,
        clip.origin.x,
        clip.origin.y,
        clip.size.width,
        clip.size.height,
        bounds.left,
        bounds.top,
        bounds.right,
        bounds.bottom
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Point, Size};

    #[test]
    fn debug_text_area_line_includes_text_rect_clip_and_bounds() {
        let rect = Rect::new(Point::new(8.0, 4.0), Size::new(64.0, 18.0));
        let clip = Rect::new(Point::new(0.0, 0.0), Size::new(80.0, 24.0));
        let line = debug_text_area_line("Hello", rect, clip, text_bounds(clip));

        assert!(line.contains("TEXT text=\"Hello\""));
        assert!(line.contains("rect=(8.0,4.0,64.0,18.0)"));
        assert!(line.contains("clip=(0.0,0.0,80.0,24.0)"));
        assert!(line.contains("bounds=(0, 0, 80, 24)"));
    }
}
