use rgui::runtime::{FrameInput, UiRuntime};
use rgui::runtime::paint::{
    PaintCtx, PaintedCommand, WidgetPainter, register_widget_painter, unregister_widget_painter,
};
use rgui::widgets::{badge, button, input, modal, popover, text, tooltip};
use rgui::{
    Color, Element, KeyEvent, PaintCommand, Point, PointerButton, PointerEvent, Rect, RectCmd, Size,
    Theme, UiEvent, WidgetKind,
};

struct OversizedBadgePainter;

impl WidgetPainter for OversizedBadgePainter {
    fn background_color(&self, _ctx: &PaintCtx<'_>) -> Color {
        Color::rgba(0, 0, 0, 0)
    }

    fn paint_content(&self, ctx: &mut PaintCtx<'_>, cmds: &mut Vec<PaintedCommand>) {
        let oversized = Rect::new(
            ctx.rect.origin,
            rgui::Size::new(ctx.rect.size.width * 2.0, ctx.rect.size.height),
        );
        cmds.push(ctx.draw_rect(oversized, Color::rgb(255, 0, 0), 0.0, 0));
    }
}

static OVERSIZED_BADGE_PAINTER: OversizedBadgePainter = OversizedBadgePainter;

#[test]
fn runtime_update_returns_all_frame_artifacts() {
    let mut runtime = UiRuntime::default();
    let root = Element::column().child(text("Hello")).child(button("Save"));

    let output = runtime.update(FrameInput {
        root,
        viewport: Size::new(320.0, 240.0),
        theme: Theme::light(),
        scale_factor: 1.0,
    });

    assert!(!output.display_list.commands().is_empty());
    assert!(output.hit_test.entries().len() >= 2);
    assert!(output.semantics.nodes().len() >= 2);
    assert!(output.snapshot.is_some());
    assert!(output.stats.command_count >= 1);
}

#[test]
fn runtime_stats_report_commands_items_batches_and_text() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column()
            .child(text("Stats").key("title"))
            .child(button("Save").key("save")),
        viewport: Size::new(320.0, 240.0),
        ..Default::default()
    });

    assert!(output.stats.command_count > 0);
    assert!(output.stats.render_item_count >= output.stats.command_count);
    assert!(output.stats.text_item_count > 0);
}

#[test]
fn runtime_snapshot_includes_style_measure_layout_and_stats() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column()
            .gap(8.0)
            .child(text("Title").heading().key("title"))
            .child(button("Save").key("save")),
        viewport: Size::new(320.0, 240.0),
        theme: Theme::light(),
        scale_factor: 1.0,
    });

    let snapshot = output.snapshot.as_ref().expect("snapshot exists");
    assert!(
        !snapshot.styles.is_empty(),
        "style snapshots should be populated"
    );
    assert!(
        !snapshot.measure.is_empty(),
        "measure snapshots should be populated"
    );
    assert_eq!(snapshot.measure.len(), snapshot.layout.len());
    assert!(snapshot.to_debug_json().contains("\"measure\""));
    assert_eq!(
        snapshot.performance.display_command_count,
        output.stats.command_count
    );
}

#[test]
fn runtime_preserves_node_id_for_keyed_elements_across_reorder() {
    let mut runtime = UiRuntime::default();
    let viewport = Size::new(320.0, 240.0);

    runtime.update(FrameInput {
        root: Element::column()
            .child(text("A").key("a"))
            .child(button("B").key("b")),
        viewport,
        scale_factor: 1.0,
        theme: Theme::light(),
    });
    let a_before = runtime.node_for_key("a").expect("node for key a");
    let b_before = runtime.node_for_key("b").expect("node for key b");

    runtime.update(FrameInput {
        root: Element::column()
            .child(button("B").key("b"))
            .child(text("A").key("a")),
        viewport,
        scale_factor: 1.0,
        theme: Theme::light(),
    });

    assert_eq!(runtime.node_for_key("a"), Some(a_before));
    assert_eq!(runtime.node_for_key("b"), Some(b_before));
}

#[test]
fn default_runtime_frame_reports_taffy_layout_engine() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column().child(text("Hello").key("hello")),
        viewport: Size::new(240.0, 120.0),
        theme: Theme::light(),
        scale_factor: 1.0,
    });

    assert_eq!(output.layout_engine, "taffy_first");
    assert!(
        output
            .snapshot
            .as_ref()
            .expect("snapshot")
            .layout_box("hello")
            .is_some(),
        "default frame should produce Taffy-backed layout data"
    );
}

#[test]
fn runtime_layout_debug_reports_incremental_path() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column()
            .key("root")
            .child(text("Hello").key("label")),
        viewport: Size::new(240.0, 120.0),
        ..Default::default()
    });
    let snapshot = output.debug_snapshot();

    assert_eq!(snapshot.layout_debug.engine, "taffy_first");
    assert_eq!(snapshot.layout_debug.incremental_layout_count, 1);
    assert_eq!(snapshot.layout_debug.full_rebuild_count, 0);
}

#[test]
fn runtime_paints_visible_nodes_from_layout() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column().child(text("Hello")).child(button("Save")),
        viewport: Size::new(320.0, 240.0),
        theme: Theme::light(),
        scale_factor: 1.0,
    });

    assert!(
        output
            .display_list
            .commands()
            .iter()
            .any(|command| matches!(command, PaintCommand::DrawText(_)))
    );
    assert!(
        output
            .display_list
            .commands()
            .iter()
            .any(|command| matches!(
                command,
                PaintCommand::DrawRect(_) | PaintCommand::DrawBorder(_)
            ))
    );
}

#[test]
fn runtime_paints_document_background_for_window_examples() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column().child(text("Hello")).child(button("Save")),
        viewport: Size::new(320.0, 240.0),
        theme: Theme::light(),
        scale_factor: 1.0,
    });

    assert!(matches!(
        output.display_list.commands().first(),
        Some(PaintCommand::DrawRect(_))
    ));
}

#[test]
fn overflow_hidden_clips_node_own_paint_commands() {
    let old = unregister_widget_painter(WidgetKind::Badge);
    register_widget_painter(WidgetKind::Badge, &OVERSIZED_BADGE_PAINTER);

    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: badge("wide")
            .width(20.0)
            .height(20.0)
            .overflow(rgui::Overflow::Hidden),
        viewport: Size::new(64.0, 32.0),
        ..Default::default()
    });

    let commands = output.display_list.commands();
    let push_index = commands
        .iter()
        .position(|cmd| matches!(cmd, PaintCommand::PushClip(_)))
        .expect("overflow-hidden should push a clip");
    let oversized_rect_index = commands
        .iter()
        .position(|cmd| {
            matches!(
                cmd,
                PaintCommand::DrawRect(RectCmd { rect, .. })
                    if rect.size.width > 20.0
            )
        })
        .expect("custom oversized painter should emit a wide rect");
    let pop_index = commands
        .iter()
        .position(|cmd| matches!(cmd, PaintCommand::PopClip))
        .expect("overflow-hidden should pop the clip");

    unregister_widget_painter(WidgetKind::Badge);
    if let Some(old) = old {
        register_widget_painter(WidgetKind::Badge, old);
    }

    assert!(
        push_index < oversized_rect_index && oversized_rect_index < pop_index,
        "own paint must be bracketed by overflow clip"
    );
}

#[test]
fn basic_window_uses_ui_runtime_frame_output() {
    let source = include_str!("../examples/basic_window.rs");

    assert!(source.contains("UiRuntime"));
    assert!(source.contains("FrameInput"));
    assert!(source.contains(".update("));
    assert!(!source.contains("DisplayList::default()"));
}

#[test]
fn debug_snapshot_contains_pipeline_surfaces() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column().child(text("Hello")).child(button("Save")),
        viewport: Size::new(320.0, 240.0),
        theme: Theme::light(),
        scale_factor: 1.0,
    });

    let snapshot = output.snapshot.as_ref().unwrap();
    assert!(!snapshot.layout.is_empty());
    assert!(!snapshot.display_list.is_empty());
    assert!(!output.debug_snapshot().hit_test_entries.is_empty());
    assert!(snapshot.performance.frame_time_ms >= 0.0);
    assert!(snapshot.to_debug_json().contains("\"hit_test_entries\""));
}

#[test]
fn runtime_snapshot_reports_no_layout_errors_for_valid_frame() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: rgui::widgets::text("Body").key("body"),
        viewport: Size::new(240.0, 120.0),
        ..Default::default()
    });

    let snapshot = output.snapshot.as_ref().expect("snapshot exists");
    assert!(snapshot.diagnostics.layout_errors.is_empty());
    assert!(snapshot.diagnostics.layout_warnings.is_empty());
}

#[test]
fn debug_snapshot_exposes_layout_debug_counters() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput::default());
    let snapshot = output.debug_snapshot();

    assert_eq!(snapshot.layout_debug.engine, "taffy_first");
    assert!(snapshot.layout_debug.taffy_node_count >= snapshot.layout.len());
    assert_eq!(snapshot.layout_debug.layout_error_count, 0);
    assert_eq!(snapshot.layout_debug.layout_warning_count, 0);
}

#[test]
fn runtime_reports_recompute_when_keyed_kind_changes() {
    let mut runtime = UiRuntime::default();
    let viewport = Size::new(320.0, 240.0);

    runtime.update(FrameInput {
        root: Element::column().child(text("Name").key("field")),
        viewport,
        scale_factor: 1.0,
        theme: Theme::light(),
    });

    let output = runtime.update(FrameInput {
        root: Element::column().child(button("Name").key("field")),
        viewport,
        scale_factor: 1.0,
        theme: Theme::light(),
    });

    let snapshot = output.snapshot.as_ref().unwrap();
    assert!(snapshot.performance.layout_recompute_count >= 1);
}

#[test]
fn input_text_state_appears_in_paint_commands() {
    let mut runtime = UiRuntime::default();
    let root = Element::column().child(input().key("name"));

    runtime.update(FrameInput {
        root: root.clone(),
        viewport: Size::new(320.0, 240.0),
        theme: Theme::light(),
        scale_factor: 1.0,
    });
    runtime.dispatch(UiEvent::KeyDown(KeyEvent {
        key: "Tab".to_string(),
        modifiers: 0,
        repeat: false,
    }));
    runtime.dispatch(UiEvent::TextInput("A".to_string()));

    let output = runtime.update(FrameInput {
        root,
        viewport: Size::new(320.0, 240.0),
        theme: Theme::light(),
        scale_factor: 1.0,
    });

    assert!(
        output
            .display_list
            .commands()
            .iter()
            .any(|command| { matches!(command, PaintCommand::DrawText(cmd) if cmd.text == "A") })
    );
}

#[test]
fn click_input_focuses_input_and_text_goes_only_to_focused_input() {
    let mut runtime = UiRuntime::default();
    let root = Element::column()
        .child(input().key("first"))
        .child(input().key("second"));
    let first_frame = runtime.update(FrameInput {
        root: root.clone(),
        viewport: Size::new(320.0, 200.0),
        theme: Theme::light(),
        scale_factor: 1.0,
    });
    let second_bounds = first_frame
        .hit_test
        .entries()
        .iter()
        .find(|entry| entry.key.as_deref() == Some("second"))
        .expect("second input hit entry")
        .rect;

    runtime.dispatch(UiEvent::PointerDown(PointerEvent {
        position: Point::new(second_bounds.origin.x + 4.0, second_bounds.origin.y + 4.0),
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));
    runtime.dispatch(UiEvent::TextInput("x".to_string()));

    runtime.update(FrameInput {
        root,
        viewport: Size::new(320.0, 200.0),
        theme: Theme::light(),
        scale_factor: 1.0,
    });

    assert_eq!(runtime.focused_key().as_deref(), Some("second"));
    assert_eq!(runtime.text_state("second").as_deref(), Some("x"));
    assert_eq!(runtime.text_state("first"), None);
}

#[test]
fn runtime_text_commands_survive_until_wgpu_lowering() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column().child(text("Settings").key("title")),
        viewport: Size::new(320.0, 240.0),
        theme: Theme::light(),
        scale_factor: 1.0,
    });

    assert!(output.display_list.commands().iter().any(|command| {
        matches!(command, PaintCommand::DrawText(cmd) if cmd.text == "Settings")
    }));
}

#[test]
fn button_label_is_painted_inside_button_without_child_hit_target() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column().child(button("Save").key("save")),
        viewport: Size::new(240.0, 120.0),
        theme: Theme::light(),
        scale_factor: 1.0,
    });

    let text_count = output
        .display_list
        .commands()
        .iter()
        .filter(|command| matches!(command, PaintCommand::DrawText(cmd) if cmd.text == "Save"))
        .count();
    assert_eq!(text_count, 1);

    let snapshot = output.snapshot.as_ref().expect("snapshot exists");
    assert_eq!(
        snapshot.layout.len(),
        2,
        "root and button should be the only document-flow layout nodes"
    );
    let debug_snapshot = output.debug_snapshot();
    assert_eq!(
        debug_snapshot.hit_test_entries.len(),
        2,
        "button label should not add its own hit-test target"
    );
    assert_eq!(
        debug_snapshot
            .hit_test_entries
            .iter()
            .filter(|entry| entry.key.as_deref() == Some("save"))
            .count(),
        1
    );
    assert!(
        runtime.node_for_key("save").is_some(),
        "button key should remain the interactive node"
    );
}

#[test]
fn closed_overlay_widgets_do_not_paint_in_document_flow() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column()
            .child(modal().key("modal").open(false))
            .child(popover().key("popover").open(false))
            .child(tooltip().key("tooltip").open(false)),
        viewport: Size::new(320.0, 240.0),
        theme: Theme::light(),
        scale_factor: 1.0,
    });

    let non_background_rects = output
        .display_list
        .commands()
        .iter()
        .filter(|command| matches!(command, PaintCommand::DrawRect(cmd) if cmd.z_index >= 0))
        .count();
    assert_eq!(non_background_rects, 0);
}

#[test]
fn debug_visual_mode_parses_comma_separated_flags() {
    let mode = rgui::runtime::debug::DebugVisualMode::parse("bounds,clips,hit-test,text,overlays");

    assert!(mode.show_bounds);
    assert!(mode.show_clip_rects);
    assert!(mode.show_hit_test);
    assert!(mode.show_text_boxes);
    assert!(mode.show_overlay_layers);
    assert!(!mode.show_paint_order);
}

#[test]
fn frame_debug_dump_smoke_test_does_not_panic() {
    // Bug fix 2.11: the `format_frame_dump` function used to take an
    // `enabled: bool` parameter and return `String::new()` when
    // disabled. The allocation still happened at the call site, so
    // the "early return" was a no-op. The current contract is
    // "always dump; caller checks the env var". This smoke test
    // ensures the function still returns a non-empty string for
    // a real frame.
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column().child(text("Hello").key("hello")),
        viewport: Size::new(240.0, 120.0),
        theme: Theme::light(),
        scale_factor: 1.0,
    });

    let dump = rgui::runtime::debug::format_frame_dump(&output);
    assert!(!dump.is_empty());
    assert!(dump.contains("FRAME"));
}

#[test]
fn frame_debug_dump_includes_layout_paint_hit_test_and_overlays() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column().child(text("Hello").key("hello")),
        viewport: Size::new(240.0, 120.0),
        theme: Theme::light(),
        scale_factor: 1.0,
    });

    let dump = rgui::runtime::debug::format_frame_dump(&output);

    assert!(dump.contains("DISPLAY LIST"));
    assert!(dump.contains("layout_engine: taffy_first"));
    assert!(dump.contains("LAYOUT"));
    assert!(dump.contains("HIT TEST"));
    assert!(dump.contains("OVERLAYS"));
}

#[test]
fn frame_debug_dump_includes_styles_measure_semantics_and_stats() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column().child(text("Hello").key("hello")),
        viewport: Size::new(240.0, 120.0),
        theme: Theme::light(),
        scale_factor: 1.0,
    });

    let dump = rgui::runtime::debug::format_frame_dump(&output);

    assert!(dump.contains("STYLES"));
    assert!(dump.contains("MEASURE"));
    assert!(dump.contains("SEMANTICS"));
    assert!(dump.contains("STATS"));
}
