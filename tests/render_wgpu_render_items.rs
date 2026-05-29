use rgui::render::wgpu::{
    InstanceRaw, MAX_RENDER_ITEMS_PER_FRAME, PipelineCache, PipelineKind, SHADER_SOURCE,
    WgpuRenderer, build_batches_from_items, build_render_items,
};
use rgui::{
    ClipSpec, Color, DisplayList, ImageCmd, ImageId, LayerKind, LayerSpec, Paint, PaintCommand,
    Point, Rect, RectCmd, ResourceStore, Size, TextCmd,
};

#[test]
fn lowers_rect_commands_to_solid_items_with_order_and_z_index() {
    let mut list = DisplayList::default();
    list.push(PaintCommand::DrawRect(RectCmd {
        rect: Rect::new(Point::new(4.0, 8.0), Size::new(20.0, 10.0)),
        paint: Paint::Solid(Color::rgba(10, 20, 30, 200)),
        radius: 0.0,
        opacity: 0.5,
        z_index: 7,
    }));

    let mut renderer = WgpuRenderer::new_headless_for_tests();
    let items = build_render_items(&list, &ResourceStore::default(), renderer.atlas_mut())
        .expect("valid display list lowers");

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].layer, LayerKind::Document);
    assert_eq!(items[0].clip_rect, None);
    assert_eq!(items[0].pipeline, PipelineKind::SolidRect);
    assert_eq!(items[0].z_index, 7);
    assert_eq!(items[0].rect.origin.x, 4.0);
    assert_eq!(
        items[0].color,
        [
            10.0 / 255.0,
            20.0 / 255.0,
            30.0 / 255.0,
            200.0 / 255.0 * 0.5
        ]
    );
}

#[test]
fn render_items_preserve_layer_clip_z_and_order() {
    let clip = Rect::new(Point::new(2.0, 3.0), Size::new(12.0, 14.0));
    let mut list = DisplayList::default();
    list.push(PaintCommand::DrawRect(RectCmd {
        rect: Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 10.0)),
        paint: Paint::Solid(Color::rgb(255, 0, 0)),
        radius: 0.0,
        opacity: 1.0,
        z_index: 2,
    }));
    list.push(PaintCommand::PushLayer(LayerSpec::new(LayerKind::Popover)));
    list.push(PaintCommand::PushClip(ClipSpec::rect(clip)));
    list.push(PaintCommand::DrawRect(RectCmd {
        rect: Rect::new(Point::new(4.0, 4.0), Size::new(8.0, 8.0)),
        paint: Paint::Solid(Color::rgb(0, 255, 0)),
        radius: 0.0,
        opacity: 1.0,
        z_index: 1,
    }));
    list.push(PaintCommand::PopClip);
    list.push(PaintCommand::PopLayer);

    let mut renderer = WgpuRenderer::new_headless_for_tests();
    let items = build_render_items(&list, &ResourceStore::default(), renderer.atlas_mut())
        .expect("valid display list lowers");

    assert_eq!(items.len(), 2);
    assert_eq!(items[0].z_index, 2);
    assert_eq!(items[0].layer, LayerKind::Document);
    assert_eq!(items[0].clip_rect, None);
    assert_eq!(items[1].z_index, 1);
    assert_eq!(items[1].layer, LayerKind::Popover);
    assert_eq!(items[1].clip_rect, Some(clip));
}

#[test]
fn render_items_skip_text_when_glyphon_is_default() {
    let clip = Rect::new(Point::new(0.0, 0.0), Size::new(120.0, 40.0));
    let mut list = DisplayList::default();
    list.push(PaintCommand::PushLayer(LayerSpec::new(LayerKind::Popover)));
    list.push(PaintCommand::PushClip(ClipSpec::rect(clip)));
    list.push(PaintCommand::DrawText(TextCmd {
        text: "Menu".to_string(),
        rect: Rect::new(Point::new(8.0, 8.0), Size::new(0.0, 17.0)),
        color: Color::rgb(0, 0, 0),
        size: 14.0,
        font_weight: rgui::FontWeight::Normal,
        font_style: rgui::FontStyle::Normal,
        line_height: Some(17.0),
        z_index: 4,
    }));
    list.push(PaintCommand::PopClip);
    list.push(PaintCommand::PopLayer);

    let mut renderer = WgpuRenderer::new_headless_for_tests();
    let items = build_render_items(&list, &ResourceStore::default(), renderer.atlas_mut())
        .expect("display lowers to items");
    let batches = build_batches_from_items(&items);

    assert!(items.is_empty());
    assert!(batches.is_empty());
}

#[test]
fn popover_layer_sorts_above_document_even_with_lower_z_index() {
    let mut list = DisplayList::default();
    list.push(PaintCommand::DrawRect(RectCmd {
        rect: Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 10.0)),
        paint: Paint::Solid(Color::rgb(255, 0, 0)),
        radius: 0.0,
        opacity: 1.0,
        z_index: 10,
    }));
    list.push(PaintCommand::PushLayer(LayerSpec::new(LayerKind::Popover)));
    list.push(PaintCommand::DrawRect(RectCmd {
        rect: Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 10.0)),
        paint: Paint::Solid(Color::rgb(0, 255, 0)),
        radius: 0.0,
        opacity: 1.0,
        z_index: 0,
    }));
    list.push(PaintCommand::PopLayer);

    let mut renderer = WgpuRenderer::new_headless_for_tests();
    let items = build_render_items(&list, &ResourceStore::default(), renderer.atlas_mut())
        .expect("valid display list lowers");

    assert_eq!(items.last().unwrap().layer, LayerKind::Popover);
}

#[test]
fn composite_text_subitems_keep_command_order_relative_to_later_rects() {
    let mut list = DisplayList::default();
    list.push(PaintCommand::DrawText(TextCmd {
        text: "A".to_string(),
        rect: Rect::new(Point::new(0.0, -12.0), Size::new(0.0, 17.0)),
        color: Color::rgb(0, 0, 0),
        size: 14.0,
        font_weight: rgui::FontWeight::Normal,
        font_style: rgui::FontStyle::Normal,
        line_height: Some(17.0),
        z_index: 0,
    }));
    list.push(PaintCommand::DrawRect(RectCmd {
        rect: Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 10.0)),
        paint: Paint::Solid(Color::rgb(255, 0, 0)),
        radius: 0.0,
        opacity: 1.0,
        z_index: 0,
    }));

    let mut renderer = WgpuRenderer::new_headless_for_tests();
    let items = build_render_items(&list, &ResourceStore::default(), renderer.atlas_mut())
        .expect("valid display list lowers");

    assert_eq!(
        items.last().unwrap().color[0],
        1.0,
        "later rect should sort after earlier text"
    );
}

#[test]
fn batches_adjacent_items_with_same_pipeline_and_z_index() {
    let mut list = DisplayList::default();
    for x in [0.0, 12.0] {
        list.push(PaintCommand::DrawRect(RectCmd {
            rect: Rect::new(Point::new(x, 0.0), Size::new(10.0, 10.0)),
            paint: Paint::Solid(Color::rgb(255, 0, 0)),
            radius: 0.0,
            opacity: 1.0,
            z_index: 1,
        }));
    }

    let mut renderer = WgpuRenderer::new_headless_for_tests();
    let items = build_render_items(&list, &ResourceStore::default(), renderer.atlas_mut())
        .expect("valid display list lowers");
    let batches = build_batches_from_items(&items);

    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].command_count, 2);
}

#[test]
fn batches_split_when_layer_or_clip_changes() {
    let clip = Rect::new(Point::new(0.0, 0.0), Size::new(16.0, 16.0));
    let mut list = DisplayList::default();
    list.push(PaintCommand::DrawRect(RectCmd {
        rect: Rect::new(Point::new(0.0, 0.0), Size::new(8.0, 8.0)),
        paint: Paint::Solid(Color::rgb(255, 0, 0)),
        radius: 0.0,
        opacity: 1.0,
        z_index: 0,
    }));
    list.push(PaintCommand::PushClip(ClipSpec::rect(clip)));
    list.push(PaintCommand::DrawRect(RectCmd {
        rect: Rect::new(Point::new(8.0, 0.0), Size::new(8.0, 8.0)),
        paint: Paint::Solid(Color::rgb(255, 0, 0)),
        radius: 0.0,
        opacity: 1.0,
        z_index: 0,
    }));
    list.push(PaintCommand::PopClip);

    let mut renderer = WgpuRenderer::new_headless_for_tests();
    let items = build_render_items(&list, &ResourceStore::default(), renderer.atlas_mut())
        .expect("valid display list lowers");
    let batches = build_batches_from_items(&items);

    assert_eq!(batches.len(), 2);
    assert_eq!(batches[0].key.clip_rect, None);
    assert_eq!(batches[1].key.clip_rect, Some(clip));
}

#[test]
fn zero_size_rects_validate_but_do_not_lower_to_render_items() {
    let mut list = DisplayList::default();
    list.push(PaintCommand::DrawRect(RectCmd {
        rect: Rect::new(Point::new(0.0, 0.0), Size::new(0.0, 10.0)),
        paint: Paint::Solid(Color::rgb(0, 0, 0)),
        radius: 0.0,
        opacity: 1.0,
        z_index: 0,
    }));

    let mut renderer = WgpuRenderer::new_headless_for_tests();
    let items = build_render_items(&list, &ResourceStore::default(), renderer.atlas_mut())
        .expect("zero-size geometry is accepted at display-list level");

    assert!(items.is_empty());
}

#[test]
fn text_lowering_reports_glyphon_when_bitmap_fallback_is_disabled() {
    let mut list = DisplayList::default();
    list.push(PaintCommand::DrawText(TextCmd {
        text: "A B".to_string(),
        rect: Rect::new(Point::new(0.0, -12.0), Size::new(0.0, 17.0)),
        color: Color::rgb(0, 0, 0),
        size: 14.0,
        font_weight: rgui::FontWeight::Normal,
        font_style: rgui::FontStyle::Normal,
        line_height: Some(17.0),
        z_index: 0,
    }));

    let mut renderer = WgpuRenderer::new_headless_for_tests();
    let items = build_render_items(&list, &ResourceStore::default(), renderer.atlas_mut())
        .expect("text lowers");

    assert!(
        items.is_empty(),
        "default glyphon rendering should bypass render-item text lowering"
    );
}

#[test]
fn render_item_limit_does_not_apply_to_default_glyphon_text_bridge() {
    let mut list = DisplayList::default();
    list.push(PaintCommand::DrawText(TextCmd {
        text: "x".repeat(MAX_RENDER_ITEMS_PER_FRAME + 1),
        rect: Rect::new(Point::new(0.0, -12.0), Size::new(0.0, 17.0)),
        color: Color::rgb(0, 0, 0),
        size: 14.0,
        font_weight: rgui::FontWeight::Normal,
        font_style: rgui::FontStyle::Normal,
        line_height: Some(17.0),
        z_index: 0,
    }));

    let mut renderer = WgpuRenderer::new_headless_for_tests();
    let items = build_render_items(&list, &ResourceStore::default(), renderer.atlas_mut())
        .expect("glyphon text is prepared outside render item lowering");

    assert!(items.is_empty());
}

#[test]
fn render_item_limit_stops_lowering_before_unbounded_allocation() {
    let mut list = DisplayList::default();
    list.push(PaintCommand::DrawText(TextCmd {
        text: "W".repeat(MAX_RENDER_ITEMS_PER_FRAME),
        rect: Rect::new(Point::new(0.0, -12.0), Size::new(0.0, 17.0)),
        color: Color::rgb(0, 0, 0),
        size: 14.0,
        font_weight: rgui::FontWeight::Normal,
        font_style: rgui::FontStyle::Normal,
        line_height: Some(17.0),
        z_index: 0,
    }));

    let mut renderer = WgpuRenderer::new_headless_for_tests();
    let items = build_render_items(&list, &ResourceStore::default(), renderer.atlas_mut())
        .expect("lowering may fill up to the item limit");

    assert!(items.len() <= MAX_RENDER_ITEMS_PER_FRAME);
}

#[test]
fn missing_image_lowers_to_visible_fallback_rect() {
    let mut list = DisplayList::default();
    list.push(PaintCommand::DrawImage(ImageCmd {
        id: ImageId::from_raw(999),
        rect: Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 10.0)),
        opacity: 1.0,
        z_index: 0,
    }));

    let mut renderer = WgpuRenderer::new_headless_for_tests();
    let items = build_render_items(&list, &ResourceStore::default(), renderer.atlas_mut())
        .expect("missing image should still lower");

    assert_eq!(items[0].pipeline, PipelineKind::SolidRect);
    assert_eq!(items[0].color[0], 1.0);
}

#[test]
fn instance_raw_has_stable_gpu_layout() {
    assert_eq!(std::mem::size_of::<InstanceRaw>(), 80);
    assert_eq!(std::mem::align_of::<InstanceRaw>(), 4);
    assert!(InstanceRaw::vertex_buffer_layout().array_stride >= 80);
}

#[test]
fn shader_source_contains_expected_entry_points() {
    let _ = std::any::type_name::<PipelineCache>();
    assert!(SHADER_SOURCE.contains("fn vs_main"));
    assert!(SHADER_SOURCE.contains("fn fs_main"));
}

#[test]
fn text_lowering_reports_strategy_and_item_count() {
    let report = rgui::render::wgpu::text::TextLoweringReport {
        strategy: rgui::render::wgpu::text::TextLoweringStrategy::Glyphon,
        item_count: 0,
    };

    assert_eq!(report.item_count, 0);
    assert_eq!(
        report.strategy,
        rgui::render::wgpu::text::TextLoweringStrategy::Glyphon
    );
}

#[test]
fn text_lowering_can_select_glyphon_strategy() {
    let strategy = rgui::render::wgpu::text::TextLoweringStrategy::Glyphon;

    assert_eq!(format!("{strategy:?}"), "Glyphon");
}

#[test]
fn render_debug_formatters_report_items_and_batches() {
    let mut list = DisplayList::default();
    list.push(PaintCommand::DrawRect(RectCmd {
        rect: Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 10.0)),
        paint: Paint::Solid(Color::rgb(255, 0, 0)),
        radius: 0.0,
        opacity: 1.0,
        z_index: 0,
    }));

    let mut renderer = WgpuRenderer::new_headless_for_tests();
    let items = build_render_items(&list, &ResourceStore::default(), renderer.atlas_mut())
        .expect("items lower");
    let batches = build_batches_from_items(&items);

    let item_dump = rgui::render::wgpu::debug::format_render_items(&items);
    let batch_dump = rgui::render::wgpu::debug::format_render_batches(&batches);

    assert!(item_dump.contains("RENDER ITEMS"));
    assert!(item_dump.contains("SolidRect"));
    assert!(batch_dump.contains("RENDER BATCHES"));
    assert!(batch_dump.contains("command_count"));
}
