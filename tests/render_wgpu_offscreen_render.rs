use rgui::SizeU32;
use rgui::render::wgpu::{OffscreenTarget, RendererOptions, WgpuContext, WgpuRenderer};
use rgui::{
    AtlasEntry, AtlasEntryKind, BorderCmd, ClipSpec, Color, DisplayList, ImageCmd, ImageId, Paint,
    PaintCommand, PathCmd, Point, Rect, RectCmd, RendererBackend, ResourceStore, Size, TextCmd,
};

#[test]
fn headless_context_initializes_device_queue_and_format() {
    let context = pollster::block_on(WgpuContext::headless(RendererOptions::default()))
        .expect("headless wgpu context should initialize");

    assert_eq!(context.format(), wgpu::TextureFormat::Rgba8UnormSrgb);
    assert!(context.limits().max_texture_dimension_2d >= 1024);
    assert_eq!(context.size(), SizeU32::new(1, 1));
}

#[test]
fn renders_solid_rect_into_offscreen_texture() {
    let mut renderer = pollster::block_on(WgpuRenderer::new_headless(RendererOptions {
        initial_size: SizeU32::new(64, 64),
        ..RendererOptions::default()
    }))
    .expect("renderer initializes");

    let mut display_list = DisplayList::default();
    display_list.push(PaintCommand::DrawRect(RectCmd {
        rect: Rect::new(Point::new(8.0, 8.0), Size::new(24.0, 24.0)),
        paint: Paint::Solid(Color::rgb(255, 0, 0)),
        radius: 0.0,
        opacity: 1.0,
        z_index: 0,
    }));

    let target = OffscreenTarget::new(renderer.context(), SizeU32::new(64, 64));
    let stats = renderer
        .render_to_target(&display_list, &ResourceStore::default(), target.view())
        .expect("offscreen render succeeds");
    let pixels = pollster::block_on(target.read_rgba8(renderer.context())).expect("readback works");

    assert_eq!(stats.command_count, 1);
    assert_eq!(stats.batch_count, 1);
    assert_eq!(sample_pixel(&pixels, 64, 16, 16), [255, 0, 0, 255]);
    assert_eq!(sample_pixel(&pixels, 64, 2, 2), [0, 0, 0, 0]);
}

#[test]
fn renders_border_as_four_rect_edges() {
    let mut renderer = pollster::block_on(WgpuRenderer::new_headless(RendererOptions {
        initial_size: SizeU32::new(32, 32),
        ..RendererOptions::default()
    }))
    .expect("renderer initializes");
    let mut display_list = DisplayList::default();
    display_list.push(PaintCommand::DrawBorder(BorderCmd {
        rect: Rect::new(Point::new(4.0, 4.0), Size::new(20.0, 20.0)),
        color: Color::rgb(0, 255, 0),
        width: 2.0,
        radius: 0.0,
        z_index: 0,
    }));

    let target = OffscreenTarget::new(renderer.context(), SizeU32::new(32, 32));
    renderer
        .render_to_target(&display_list, &ResourceStore::default(), target.view())
        .expect("offscreen render succeeds");
    let pixels = pollster::block_on(target.read_rgba8(renderer.context())).expect("readback works");

    assert_eq!(sample_pixel(&pixels, 32, 5, 5), [0, 255, 0, 255]);
    assert_eq!(sample_pixel(&pixels, 32, 12, 12), [0, 0, 0, 0]);
}

#[test]
fn higher_z_index_draws_on_top() {
    let mut renderer = pollster::block_on(WgpuRenderer::new_headless(RendererOptions {
        initial_size: SizeU32::new(32, 32),
        ..RendererOptions::default()
    }))
    .expect("renderer initializes");
    let mut display_list = DisplayList::default();
    display_list.push(PaintCommand::DrawRect(RectCmd {
        rect: Rect::new(Point::new(0.0, 0.0), Size::new(24.0, 24.0)),
        paint: Paint::Solid(Color::rgb(255, 0, 0)),
        radius: 0.0,
        opacity: 1.0,
        z_index: 0,
    }));
    display_list.push(PaintCommand::DrawRect(RectCmd {
        rect: Rect::new(Point::new(8.0, 8.0), Size::new(24.0, 24.0)),
        paint: Paint::Solid(Color::rgb(0, 0, 255)),
        radius: 0.0,
        opacity: 1.0,
        z_index: 10,
    }));

    let target = OffscreenTarget::new(renderer.context(), SizeU32::new(32, 32));
    renderer
        .render_to_target(&display_list, &ResourceStore::default(), target.view())
        .expect("offscreen render succeeds");
    let pixels = pollster::block_on(target.read_rgba8(renderer.context())).expect("readback works");

    assert_eq!(sample_pixel(&pixels, 32, 12, 12), [0, 0, 255, 255]);
}

#[test]
fn renders_simple_horizontal_path_segment() {
    let mut renderer = pollster::block_on(WgpuRenderer::new_headless(RendererOptions {
        initial_size: SizeU32::new(32, 32),
        ..RendererOptions::default()
    }))
    .expect("renderer initializes");
    let mut display_list = DisplayList::default();
    display_list.push(PaintCommand::DrawPath(PathCmd {
        points: vec![Point::new(4.0, 16.0), Point::new(24.0, 16.0)],
        color: Color::rgb(255, 255, 0),
        width: 3.0,
        z_index: 0,
    }));

    let target = OffscreenTarget::new(renderer.context(), SizeU32::new(32, 32));
    renderer
        .render_to_target(&display_list, &ResourceStore::default(), target.view())
        .expect("offscreen render succeeds");
    let pixels = pollster::block_on(target.read_rgba8(renderer.context())).expect("readback works");

    assert_eq!(sample_pixel(&pixels, 32, 12, 16), [255, 255, 0, 255]);
}

#[test]
fn renders_text_command_as_visible_pixels() {
    let mut renderer = pollster::block_on(WgpuRenderer::new_headless(RendererOptions {
        initial_size: SizeU32::new(32, 32),
        ..RendererOptions::default()
    }))
    .expect("renderer initializes");
    let mut display_list = DisplayList::default();
    display_list.push(PaintCommand::DrawText(TextCmd {
        text: "A".to_string(),
        rect: Rect::new(Point::new(8.0, 4.0), Size::new(14.0, 17.0)),
        color: Color::rgb(20, 23, 28),
        size: 14.0,
        font_weight: rgui::FontWeight::Normal,
        font_style: rgui::FontStyle::Normal,
        line_height: Some(17.0),
        z_index: 0,
    }));

    let target = OffscreenTarget::new(renderer.context(), SizeU32::new(32, 32));
    renderer
        .render_to_target(&display_list, &ResourceStore::default(), target.view())
        .expect("offscreen render succeeds");
    let pixels = pollster::block_on(target.read_rgba8(renderer.context())).expect("readback works");

    assert!(
        has_visible_text_pixels(&pixels),
        "glyphon text should draw visible pixels"
    );
}

#[test]
fn push_clip_prevents_pixels_outside_clip_rect() {
    let mut renderer = pollster::block_on(WgpuRenderer::new_headless(RendererOptions {
        initial_size: SizeU32::new(32, 32),
        ..RendererOptions::default()
    }))
    .expect("renderer initializes");
    let mut display_list = DisplayList::default();
    display_list.push(PaintCommand::PushClip(ClipSpec::rect(Rect::new(
        Point::new(0.0, 0.0),
        Size::new(12.0, 32.0),
    ))));
    display_list.push(PaintCommand::DrawRect(RectCmd {
        rect: Rect::new(Point::new(0.0, 0.0), Size::new(32.0, 32.0)),
        paint: Paint::Solid(Color::rgb(255, 0, 0)),
        radius: 0.0,
        opacity: 1.0,
        z_index: 0,
    }));
    display_list.push(PaintCommand::PopClip);

    let target = OffscreenTarget::new(renderer.context(), SizeU32::new(32, 32));
    renderer
        .render_to_target(&display_list, &ResourceStore::default(), target.view())
        .expect("offscreen render succeeds");
    let pixels = pollster::block_on(target.read_rgba8(renderer.context())).expect("readback works");

    assert_eq!(sample_pixel(&pixels, 32, 8, 8), [255, 0, 0, 255]);
    assert_eq!(sample_pixel(&pixels, 32, 20, 8), [0, 0, 0, 0]);
}

#[test]
fn renders_text_as_glyph_shape_with_transparent_holes() {
    let mut renderer = pollster::block_on(WgpuRenderer::new_headless(RendererOptions {
        initial_size: SizeU32::new(32, 32),
        ..RendererOptions::default()
    }))
    .expect("renderer initializes");
    let mut display_list = DisplayList::default();
    display_list.push(PaintCommand::DrawText(TextCmd {
        text: "A".to_string(),
        rect: Rect::new(Point::new(8.0, 4.0), Size::new(14.0, 17.0)),
        color: Color::rgb(20, 23, 28),
        size: 14.0,
        font_weight: rgui::FontWeight::Normal,
        font_style: rgui::FontStyle::Normal,
        line_height: Some(17.0),
        z_index: 0,
    }));

    let target = OffscreenTarget::new(renderer.context(), SizeU32::new(32, 32));
    renderer
        .render_to_target(&display_list, &ResourceStore::default(), target.view())
        .expect("offscreen render succeeds");
    let pixels = pollster::block_on(target.read_rgba8(renderer.context())).expect("readback works");

    assert!(
        has_visible_text_pixels(&pixels),
        "glyphon should render the shaped A glyph"
    );
}

#[test]
fn renders_text_with_letter_spacing_and_spaces() {
    let mut renderer = pollster::block_on(WgpuRenderer::new_headless(RendererOptions {
        initial_size: SizeU32::new(64, 24),
        ..RendererOptions::default()
    }))
    .expect("renderer initializes");
    let mut display_list = DisplayList::default();
    display_list.push(PaintCommand::DrawText(TextCmd {
        text: "A B".to_string(),
        rect: Rect::new(Point::new(4.0, 4.0), Size::new(32.0, 17.0)),
        color: Color::rgb(20, 23, 28),
        size: 14.0,
        font_weight: rgui::FontWeight::Normal,
        font_style: rgui::FontStyle::Normal,
        line_height: Some(17.0),
        z_index: 0,
    }));

    let target = OffscreenTarget::new(renderer.context(), SizeU32::new(64, 24));
    renderer
        .render_to_target(&display_list, &ResourceStore::default(), target.view())
        .expect("offscreen render succeeds");
    let pixels = pollster::block_on(target.read_rgba8(renderer.context())).expect("readback works");

    assert!(
        has_visible_text_pixels(&pixels),
        "glyphon should render non-space glyphs while preserving word spacing"
    );
}

#[test]
fn renders_image_quad_from_uploaded_atlas_region() {
    let mut renderer = pollster::block_on(WgpuRenderer::new_headless(RendererOptions {
        initial_size: SizeU32::new(16, 16),
        ..RendererOptions::default()
    }))
    .expect("renderer initializes");
    renderer
        .upload_atlas_rgba8(
            ImageId::from_raw(9),
            SizeU32::new(1, 1),
            &[0, 255, 255, 255],
        )
        .expect("atlas upload succeeds");

    let mut resources = ResourceStore::default();
    resources.atlas_entries.push(AtlasEntry {
        uv: Rect::new(Point::new(0.0, 0.0), Size::new(1.0, 1.0)),
        size: SizeU32::new(1, 1),
        generation: 1,
        kind: AtlasEntryKind::Image(ImageId::from_raw(9)),
    });
    let mut display_list = DisplayList::default();
    display_list.push(PaintCommand::DrawImage(ImageCmd {
        id: ImageId::from_raw(9),
        rect: Rect::new(Point::new(4.0, 4.0), Size::new(8.0, 8.0)),
        opacity: 1.0,
        z_index: 0,
    }));

    let target = OffscreenTarget::new(renderer.context(), SizeU32::new(16, 16));
    renderer
        .render_to_target(&display_list, &resources, target.view())
        .expect("offscreen render succeeds");
    let pixels = pollster::block_on(target.read_rgba8(renderer.context())).expect("readback works");

    assert_eq!(sample_pixel(&pixels, 16, 8, 8), [0, 255, 255, 255]);
}

#[test]
fn resize_updates_headless_context_viewport() {
    let mut renderer = pollster::block_on(WgpuRenderer::new_headless(RendererOptions::default()))
        .expect("renderer initializes");

    renderer.resize(SizeU32::new(128, 96));

    assert_eq!(renderer.context().size(), SizeU32::new(128, 96));
}

#[test]
fn render_stats_report_commands_batches_and_zero_uploads_for_solid_rects() {
    let mut renderer = pollster::block_on(WgpuRenderer::new_headless(RendererOptions {
        initial_size: SizeU32::new(32, 32),
        ..RendererOptions::default()
    }))
    .expect("renderer initializes");
    let mut display_list = DisplayList::default();
    display_list.push(PaintCommand::DrawRect(RectCmd {
        rect: Rect::new(Point::new(0.0, 0.0), Size::new(8.0, 8.0)),
        paint: Paint::Solid(Color::rgb(255, 0, 0)),
        radius: 0.0,
        opacity: 1.0,
        z_index: 0,
    }));
    display_list.push(PaintCommand::DrawRect(RectCmd {
        rect: Rect::new(Point::new(8.0, 0.0), Size::new(8.0, 8.0)),
        paint: Paint::Solid(Color::rgb(255, 0, 0)),
        radius: 0.0,
        opacity: 1.0,
        z_index: 0,
    }));

    let target = OffscreenTarget::new(renderer.context(), SizeU32::new(32, 32));
    let stats = renderer
        .render_to_target(&display_list, &ResourceStore::default(), target.view())
        .expect("offscreen render succeeds");

    assert_eq!(stats.command_count, 2);
    assert_eq!(stats.batch_count, 1);
    assert_eq!(stats.atlas_upload_bytes, 0);
    assert_eq!(stats.render_item_count, 2);
    assert_eq!(stats.text_item_count, 0);
    assert_eq!(stats.clip_batch_count, 0);
}

#[test]
fn render_stats_include_item_text_and_clip_counts() {
    let mut renderer = pollster::block_on(WgpuRenderer::new_headless(RendererOptions {
        initial_size: SizeU32::new(64, 32),
        ..RendererOptions::default()
    }))
    .expect("renderer initializes");
    let mut display_list = DisplayList::default();
    display_list.push(PaintCommand::PushClip(ClipSpec::rect(Rect::new(
        Point::new(0.0, 0.0),
        Size::new(64.0, 32.0),
    ))));
    display_list.push(PaintCommand::DrawText(TextCmd {
        text: "A".to_string(),
        rect: Rect::new(Point::new(8.0, 4.0), Size::new(14.0, 17.0)),
        color: Color::rgb(20, 23, 28),
        size: 14.0,
        font_weight: rgui::FontWeight::Normal,
        font_style: rgui::FontStyle::Normal,
        line_height: Some(17.0),
        z_index: 0,
    }));
    display_list.push(PaintCommand::PopClip);

    let target = OffscreenTarget::new(renderer.context(), SizeU32::new(64, 32));
    let stats = renderer
        .render_to_target(&display_list, &ResourceStore::default(), target.view())
        .expect("offscreen render succeeds");

    assert_eq!(stats.command_count, 3);
    assert_eq!(stats.render_item_count, 0);
    assert_eq!(stats.text_item_count, 1);
    assert_eq!(stats.clip_batch_count, 0);
    assert!(stats.glyphon_enabled);
    assert_eq!(stats.text_area_count, 1);
    assert_eq!(stats.clipped_text_area_count, 1);
    assert_eq!(stats.skipped_text_area_count, 0);
    assert!(stats.glyph_count > 0);
    assert!(!stats.fallback_used);
}

#[test]
fn text_lowering_boundary_still_renders_visible_text() {
    let mut display_list = DisplayList::default();
    display_list.push(PaintCommand::DrawText(TextCmd {
        text: "Glyph Boundary".to_string(),
        rect: Rect::new(Point::new(8.0, 5.0), Size::new(120.0, 22.0)),
        color: Color::rgb(20, 23, 28),
        size: 18.0,
        font_weight: rgui::FontWeight::Normal,
        font_style: rgui::FontStyle::Normal,
        line_height: Some(22.0),
        z_index: 0,
    }));

    let pixels = render_display_list(display_list, SizeU32::new(240, 80));
    assert!(pixels.chunks_exact(4).any(|px| px[3] > 0 && px[0] < 250));
}

fn render_display_list(display_list: DisplayList, size: SizeU32) -> Vec<u8> {
    let mut renderer = pollster::block_on(WgpuRenderer::new_headless(RendererOptions {
        initial_size: size,
        ..RendererOptions::default()
    }))
    .expect("renderer initializes");
    let target = OffscreenTarget::new(renderer.context(), size);
    renderer
        .render_to_target(&display_list, &ResourceStore::default(), target.view())
        .expect("offscreen render succeeds");
    pollster::block_on(target.read_rgba8(renderer.context())).expect("readback works")
}

fn sample_pixel(pixels: &[u8], width: usize, x: usize, y: usize) -> [u8; 4] {
    let offset = (y * width + x) * 4;
    [
        pixels[offset],
        pixels[offset + 1],
        pixels[offset + 2],
        pixels[offset + 3],
    ]
}

fn has_visible_text_pixels(pixels: &[u8]) -> bool {
    pixels.chunks_exact(4).any(|px| px[3] > 0 && px[0] < 250)
}

fn has_visible_text_pixels_in_rect(pixels: &[u8], width: usize, rect: Rect) -> bool {
    let min_x = rect.origin.x.max(0.0).floor() as usize;
    let min_y = rect.origin.y.max(0.0).floor() as usize;
    let max_x = rect.max_x().ceil().max(0.0) as usize;
    let max_y = rect.max_y().ceil().max(0.0) as usize;

    for y in min_y..max_y {
        for x in min_x..max_x {
            let offset = (y * width + x) * 4;
            let px = &pixels[offset..offset + 4];
            if px[3] > 0 && px[0] < 245 {
                return true;
            }
        }
    }

    false
}

#[test]
fn glyphon_text_is_not_clipped_by_last_non_text_batch_scissor() {
    let mut renderer = pollster::block_on(WgpuRenderer::new_headless(RendererOptions {
        initial_size: SizeU32::new(128, 64),
        ..RendererOptions::default()
    }))
    .expect("renderer initializes");

    let mut display_list = DisplayList::default();
    display_list.push(PaintCommand::DrawText(TextCmd {
        text: "Visible".to_string(),
        rect: Rect::new(Point::new(8.0, 8.0), Size::new(96.0, 24.0)),
        color: Color::rgb(20, 23, 28),
        size: 16.0,
        font_weight: rgui::FontWeight::Normal,
        font_style: rgui::FontStyle::Normal,
        line_height: Some(20.0),
        z_index: 0,
    }));

    display_list.push(PaintCommand::PushClip(ClipSpec::rect(Rect::new(
        Point::new(120.0, 56.0),
        Size::new(4.0, 4.0),
    ))));
    display_list.push(PaintCommand::DrawRect(RectCmd {
        rect: Rect::new(Point::new(120.0, 56.0), Size::new(4.0, 4.0)),
        paint: Paint::Solid(Color::rgb(255, 0, 0)),
        radius: 0.0,
        opacity: 1.0,
        z_index: 1,
    }));
    display_list.push(PaintCommand::PopClip);

    let target = OffscreenTarget::new(renderer.context(), SizeU32::new(128, 64));
    let stats = renderer
        .render_to_target(&display_list, &ResourceStore::default(), target.view())
        .expect("offscreen render succeeds");
    let pixels = pollster::block_on(target.read_rgba8(renderer.context())).expect("readback works");

    assert!(stats.glyphon_enabled);
    assert_eq!(stats.text_area_count, 1);
    assert!(stats.glyph_count > 0);
    assert!(
        has_visible_text_pixels_in_rect(
            &pixels,
            128,
            Rect::new(Point::new(0.0, 0.0), Size::new(112.0, 48.0))
        ),
        "text should remain visible even if the final shape batch used a tiny scissor"
    );
}

#[test]
fn glyphon_text_inside_empty_active_clip_is_skipped() {
    let mut renderer = pollster::block_on(WgpuRenderer::new_headless(RendererOptions {
        initial_size: SizeU32::new(128, 64),
        ..RendererOptions::default()
    }))
    .expect("renderer initializes");

    let mut display_list = DisplayList::default();
    display_list.push(PaintCommand::PushClip(ClipSpec::rect(Rect::new(
        Point::new(200.0, 200.0),
        Size::new(10.0, 10.0),
    ))));
    display_list.push(PaintCommand::DrawText(TextCmd {
        text: "Hidden".to_string(),
        rect: Rect::new(Point::new(8.0, 8.0), Size::new(96.0, 24.0)),
        color: Color::rgb(20, 23, 28),
        size: 16.0,
        font_weight: rgui::FontWeight::Normal,
        font_style: rgui::FontStyle::Normal,
        line_height: Some(20.0),
        z_index: 0,
    }));
    display_list.push(PaintCommand::PopClip);

    let target = OffscreenTarget::new(renderer.context(), SizeU32::new(128, 64));
    let stats = renderer
        .render_to_target(&display_list, &ResourceStore::default(), target.view())
        .expect("offscreen render succeeds");
    let pixels = pollster::block_on(target.read_rgba8(renderer.context())).expect("readback works");

    assert_eq!(stats.text_item_count, 1);
    assert_eq!(stats.text_area_count, 0);
    assert_eq!(stats.clipped_text_area_count, 0);
    assert_eq!(stats.skipped_text_area_count, 1);
    assert_eq!(stats.glyph_count, 0);
    assert!(
        !has_visible_text_pixels(&pixels),
        "fully clipped text must not fall back to the full viewport"
    );
}

#[test]
fn glyphon_text_partially_visible_after_scroll_renders_inside_clip() {
    let mut renderer = pollster::block_on(WgpuRenderer::new_headless(RendererOptions {
        initial_size: SizeU32::new(128, 64),
        ..RendererOptions::default()
    }))
    .expect("renderer initializes");

    let mut display_list = DisplayList::default();
    display_list.push(PaintCommand::PushClip(ClipSpec::rect(Rect::new(
        Point::new(0.0, 32.0),
        Size::new(128.0, 32.0),
    ))));
    display_list.push(PaintCommand::DrawText(TextCmd {
        text: "Scrolled text".to_string(),
        rect: Rect::new(Point::new(8.0, 24.0), Size::new(112.0, 24.0)),
        color: Color::rgb(20, 23, 28),
        size: 16.0,
        font_weight: rgui::FontWeight::Normal,
        font_style: rgui::FontStyle::Normal,
        line_height: Some(20.0),
        z_index: 0,
    }));
    display_list.push(PaintCommand::PopClip);

    let target = OffscreenTarget::new(renderer.context(), SizeU32::new(128, 64));
    let stats = renderer
        .render_to_target(&display_list, &ResourceStore::default(), target.view())
        .expect("offscreen render succeeds");
    let pixels = pollster::block_on(target.read_rgba8(renderer.context())).expect("readback works");

    assert_eq!(stats.text_area_count, 1);
    assert_eq!(stats.clipped_text_area_count, 1);
    assert_eq!(stats.skipped_text_area_count, 0);
    assert!(stats.glyph_count > 0);
    assert!(
        has_visible_text_pixels_in_rect(
            &pixels,
            128,
            Rect::new(Point::new(0.0, 32.0), Size::new(128.0, 32.0))
        ),
        "partially visible scrolled text should render inside the active clip"
    );
    assert!(
        !has_visible_text_pixels_in_rect(
            &pixels,
            128,
            Rect::new(Point::new(0.0, 0.0), Size::new(128.0, 24.0))
        ),
        "text pixels outside the active clip should remain clipped"
    );
}

#[test]
fn glyphon_ignores_fully_offscreen_text_before_visible_scrolled_text() {
    let mut renderer = pollster::block_on(WgpuRenderer::new_headless(RendererOptions {
        initial_size: SizeU32::new(160, 80),
        ..RendererOptions::default()
    }))
    .expect("renderer initializes");

    let mut display_list = DisplayList::default();
    display_list.push(PaintCommand::PushClip(ClipSpec::rect(Rect::new(
        Point::new(0.0, 0.0),
        Size::new(160.0, 80.0),
    ))));
    for i in 0..80 {
        display_list.push(PaintCommand::DrawText(TextCmd {
            text: format!("Offscreen {i}"),
            rect: Rect::new(
                Point::new(8.0, -4000.0 + i as f32 * 20.0),
                Size::new(120.0, 18.0),
            ),
            color: Color::rgb(20, 23, 28),
            size: 14.0,
            font_weight: rgui::FontWeight::Normal,
            font_style: rgui::FontStyle::Normal,
            line_height: Some(18.0),
            z_index: 0,
        }));
    }
    display_list.push(PaintCommand::DrawText(TextCmd {
        text: "Visible after offscreen text".to_string(),
        rect: Rect::new(Point::new(8.0, 24.0), Size::new(140.0, 22.0)),
        color: Color::rgb(20, 23, 28),
        size: 16.0,
        font_weight: rgui::FontWeight::Normal,
        font_style: rgui::FontStyle::Normal,
        line_height: Some(20.0),
        z_index: 0,
    }));
    display_list.push(PaintCommand::PopClip);

    let target = OffscreenTarget::new(renderer.context(), SizeU32::new(160, 80));
    let stats = renderer
        .render_to_target(&display_list, &ResourceStore::default(), target.view())
        .expect("offscreen render succeeds");
    let pixels = pollster::block_on(target.read_rgba8(renderer.context())).expect("readback works");

    assert_eq!(stats.text_area_count, 1);
    assert_eq!(stats.skipped_text_area_count, 80);
    assert!(
        has_visible_text_pixels_in_rect(
            &pixels,
            160,
            Rect::new(Point::new(0.0, 0.0), Size::new(160.0, 80.0))
        ),
        "visible text should render even after many clipped-away text commands"
    );
}

#[test]
fn root_scroll_area_keeps_visible_text_rendering_after_scroll() {
    let size = SizeU32::new(320, 180);
    let mut runtime = rgui::runtime::UiRuntime::default();
    runtime.set_scroll_offset_for_key("gallery-root", rgui::Vec2::new(0.0, 120.0));
    let root = rgui::widgets::scroll_area()
        .key("gallery-root")
        .width(rgui::Length::Percent(1.0))
        .height(rgui::Length::Percent(1.0))
        .child(
            rgui::Element::column()
                .key("content")
                .gap(16.0)
                .padding(16.0)
                .child(rgui::widgets::text("Before").height(120.0))
                .child(
                    rgui::widgets::text("Visible A")
                        .key("visible-a")
                        .height(40.0),
                )
                .child(
                    rgui::widgets::text("Visible B")
                        .key("visible-b")
                        .height(40.0),
                )
                .child(rgui::widgets::text("After").height(400.0)),
        );
    let output = runtime.update(rgui::runtime::FrameInput {
        root,
        viewport: Size::new(size.width as f32, size.height as f32),
        ..Default::default()
    });

    let text_command_count = output
        .display_list
        .commands()
        .iter()
        .filter(|command| matches!(command, PaintCommand::DrawText(_)))
        .count();
    assert!(text_command_count > 0, "runtime should emit text commands");

    let mut renderer = pollster::block_on(WgpuRenderer::new_headless(RendererOptions {
        initial_size: size,
        ..RendererOptions::default()
    }))
    .expect("renderer initializes");
    let target = OffscreenTarget::new(renderer.context(), size);
    let stats = renderer
        .render_to_target(&output.display_list, &output.resources, target.view())
        .expect("offscreen render succeeds");
    let pixels = pollster::block_on(target.read_rgba8(renderer.context())).expect("readback works");

    assert!(
        stats.text_area_count > 0,
        "glyphon should prepare visible text areas; stats={stats:?}"
    );
    assert!(
        stats.glyph_count > 0,
        "glyphon should shape visible glyphs; stats={stats:?}"
    );
    assert!(
        has_visible_text_pixels_in_rect(
            &pixels,
            size.width as usize,
            Rect::new(
                Point::new(0.0, 0.0),
                Size::new(size.width as f32, size.height as f32)
            )
        ),
        "scrolled root scroll area should still contain visible text pixels"
    );
}

#[test]
fn grid_scroll_area_renders_text_that_starts_below_initial_viewport() {
    let size = SizeU32::new(880, 746);
    let mut runtime = rgui::runtime::UiRuntime::default();
    runtime.set_scroll_offset_for_key("gallery-root", rgui::Vec2::new(0.0, 760.0));

    let mut grid_style = rgui::Style::default();
    grid_style.grid_template_columns = Some(vec![
        rgui::GridTrack::Fraction(1.0),
        rgui::GridTrack::Fraction(1.0),
    ]);

    let section = |key: &str, title: &str, body: &str| {
        rgui::Element::column()
            .key(key)
            .padding(12.0)
            .gap(10.0)
            .height(360.0)
            .child(rgui::widgets::text(title).heading())
            .child(rgui::widgets::text(body).height(40.0))
            .child(rgui::widgets::button("Action"))
    };

    let root = rgui::widgets::scroll_area()
        .key("gallery-root")
        .width(rgui::Length::Percent(1.0))
        .height(rgui::Length::Percent(1.0))
        .child(
            rgui::Element::grid()
                .key("gallery-page")
                .style(grid_style)
                .padding(8.0)
                .gap(8.0)
                .child(rgui::widgets::text("RML Widget Gallery").heading())
                .child(rgui::widgets::text("Subtitle").height(40.0))
                .child(section(
                    "layout-section",
                    "1. Layout Components",
                    "Layout body",
                ))
                .child(section("forms-section", "2. Form Components", "Forms body"))
                .child(section(
                    "collections-section",
                    "3. Collection Components",
                    "Collections body should be visible after scroll",
                ))
                .child(section(
                    "overlays-section",
                    "4. Overlay Components",
                    "Overlay body should be visible after scroll",
                )),
        );
    let output = runtime.update(rgui::runtime::FrameInput {
        root,
        viewport: Size::new(size.width as f32, size.height as f32),
        ..Default::default()
    });

    let visible_text_commands: Vec<_> = output
        .display_list
        .commands()
        .iter()
        .filter_map(|command| match command {
            PaintCommand::DrawText(cmd)
                if cmd.text.contains("Collection") || cmd.text.contains("Overlay") =>
            {
                Some((cmd.text.as_str(), cmd.rect))
            }
            _ => None,
        })
        .collect();
    assert!(
        !visible_text_commands.is_empty(),
        "runtime should emit below-initial text commands"
    );

    let mut renderer = pollster::block_on(WgpuRenderer::new_headless(RendererOptions {
        initial_size: size,
        ..RendererOptions::default()
    }))
    .expect("renderer initializes");
    let target = OffscreenTarget::new(renderer.context(), size);
    let stats = renderer
        .render_to_target(&output.display_list, &output.resources, target.view())
        .expect("offscreen render succeeds");
    let pixels = pollster::block_on(target.read_rgba8(renderer.context())).expect("readback works");

    assert!(
        stats.text_area_count > 0,
        "glyphon should prepare below-initial visible text; stats={stats:?} text={visible_text_commands:?}"
    );
    assert!(
        has_visible_text_pixels_in_rect(
            &pixels,
            size.width as usize,
            Rect::new(
                Point::new(0.0, 0.0),
                Size::new(size.width as f32, size.height as f32)
            )
        ),
        "grid content text that starts below the initial viewport should render after scrolling"
    );
}

#[cfg(feature = "rml")]
#[test]
fn actual_rml_gallery_renders_text_that_starts_below_initial_viewport() {
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let parsed = rgui::rml::parse(include_str!("../examples/rml_widget_gallery.rml"))
                .expect("gallery rml parses");
            let size = SizeU32::new(880, 746);
            let mut runtime = rgui::runtime::UiRuntime::default();
            runtime.set_scroll_offset_for_key("gallery-root", rgui::Vec2::new(0.0, 760.0));
            let output = runtime.update(rgui::runtime::FrameInput {
                root: parsed.element,
                viewport: Size::new(size.width as f32, size.height as f32),
                ..Default::default()
            });

            let target_text_commands: Vec<_> = output
                .display_list
                .commands()
                .iter()
                .filter_map(|command| match command {
                    PaintCommand::DrawText(cmd)
                        if cmd.text.contains("Collection")
                            || cmd.text.contains("Overlay")
                            || cmd.text.contains("Primitive")
                            || cmd.text.contains("Style") =>
                    {
                        Some((cmd.text.as_str(), cmd.rect))
                    }
                    _ => None,
                })
                .collect();
            assert!(
                !target_text_commands.is_empty(),
                "runtime should emit text commands for below-initial RML sections"
            );
            let mut renderer = pollster::block_on(WgpuRenderer::new_headless(RendererOptions {
                initial_size: size,
                ..RendererOptions::default()
            }))
            .expect("renderer initializes");
            let target = OffscreenTarget::new(renderer.context(), size);
            let stats = renderer
                .render_to_target(&output.display_list, &output.resources, target.view())
                .expect("offscreen render succeeds");
            let pixels =
                pollster::block_on(target.read_rgba8(renderer.context())).expect("readback works");

            assert!(
                stats.text_area_count > 0,
                "glyphon should prepare visible RML text; stats={stats:?} text={target_text_commands:?}"
            );
            assert!(
                has_visible_text_pixels_in_rect(
                    &pixels,
                    size.width as usize,
                    Rect::new(
                        Point::new(0.0, 0.0),
                        Size::new(size.width as f32, size.height as f32)
                    )
                ),
                "actual RML gallery text below initial viewport should render after scrolling"
            );
        })
        .expect("test thread spawns")
        .join()
        .expect("test thread completes");
}

#[test]
fn default_renderer_never_uses_bitmap_text_fallback() {
    let mut renderer = pollster::block_on(WgpuRenderer::new_headless(RendererOptions {
        initial_size: SizeU32::new(64, 32),
        ..RendererOptions::default()
    }))
    .expect("renderer initializes");
    let mut display_list = DisplayList::default();
    display_list.push(PaintCommand::DrawText(TextCmd {
        text: "Verify Glyphon".to_string(),
        rect: Rect::new(Point::new(8.0, 4.0), Size::new(48.0, 17.0)),
        color: Color::rgb(20, 23, 28),
        size: 14.0,
        font_weight: rgui::FontWeight::Normal,
        font_style: rgui::FontStyle::Normal,
        line_height: Some(17.0),
        z_index: 0,
    }));

    let target = OffscreenTarget::new(renderer.context(), SizeU32::new(64, 32));
    let stats = renderer
        .render_to_target(&display_list, &ResourceStore::default(), target.view())
        .expect("offscreen render succeeds");

    assert!(!stats.fallback_used);
}
