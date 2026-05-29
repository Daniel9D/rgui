use rgui::core::{ElementKind, FontStyle, FontWeight, PaintCommand, Point, Rect, Size};
use rgui::runtime::{Reconciler, paint};
use rgui::text_engine::{TextLayout, TextSystem};
use rgui::widgets::text;

#[test]
fn text_measure_width_scales_with_content() {
    let mut text = TextSystem::default();
    let short = text.measure("Save", 14.0, FontWeight::Normal, FontStyle::Normal, 320.0);
    let long = text.measure(
        "Save changes",
        14.0,
        FontWeight::Normal,
        FontStyle::Normal,
        320.0,
    );

    assert!(long.width > short.width);
    assert_eq!(short.glyph_count, 4);
    assert_eq!(long.glyph_count, 12);
}

#[test]
fn heading_text_is_larger_than_body_text() {
    let mut text = TextSystem::default();
    let body = text.measure("Title", 14.0, FontWeight::Normal, FontStyle::Normal, 320.0);
    let heading = text.measure("Title", 24.0, FontWeight::Bold, FontStyle::Normal, 320.0);

    assert!(heading.width > body.width);
    assert!(heading.height > body.height);
    assert!(heading.baseline > body.baseline);
}

#[test]
fn text_measurement_uses_content_font_size_and_wrapping() {
    let mut text = TextSystem::default();

    let short = text.measure("Hello", 16.0, FontWeight::Normal, FontStyle::Normal, 500.0);
    let long = text.measure(
        "Hello world from RGUI",
        16.0,
        FontWeight::Normal,
        FontStyle::Normal,
        500.0,
    );
    let heading = text.measure("Hello", 28.0, FontWeight::Bold, FontStyle::Normal, 500.0);
    let wrapped = text.measure(
        "one two three four five six seven eight",
        16.0,
        FontWeight::Normal,
        FontStyle::Normal,
        80.0,
    );

    assert!(long.width > short.width);
    assert!(heading.height > short.height);
    assert!(heading.baseline > short.baseline);
    assert!(wrapped.height >= short.height);
    assert!(wrapped.glyph_count >= short.glyph_count);
}

#[test]
fn text_layout_baseline_origin_rect_is_consistent() {
    let layout = TextLayout {
        text: "Hello".to_string(),
        font_px: 20.0,
        width: 50.0,
        height: 24.0,
        baseline: 16.0,
        line_height: 24.0,
        glyph_count: 5,
        lines: Vec::new(),
        glyph_runs: Vec::new(),
    };

    let rect = layout.rect_for_baseline_origin(Point::new(8.0, 32.0));
    assert_eq!(rect.origin.x, 8.0);
    assert_eq!(rect.origin.y, 16.0);
    assert_eq!(rect.size, Size::new(50.0, 24.0));
}

#[test]
fn text_layout_and_paint_use_same_font_size() {
    let mut reconciler = Reconciler::default();
    let tree = reconciler.reconcile(text("Title").heading().key("title"));
    let node = tree
        .nodes()
        .iter()
        .find(|node| matches!(node.kind, ElementKind::Text(_)))
        .expect("text node exists");

    let mut text_system = TextSystem::default();
    let painted = paint::paint_node_with_text(
        node,
        Rect::new(Point::new(10.0, 20.0), Size::new(100.0, 30.0)),
        0,
        &paint::VisualState::default(),
        &mut text_system,
    );

    let text_cmd = painted
        .iter()
        .find_map(|painted| match &painted.command {
            PaintCommand::DrawText(cmd) => Some(cmd),
            _ => None,
        })
        .expect("text command is painted");
    let layout = text_system.measure("Title", 24.0, FontWeight::Bold, FontStyle::Normal, 100.0);

    assert_eq!(text_cmd.size, layout.font_px);
    assert_eq!(
        text_cmd.rect.origin.y,
        20.0 + (30.0 - layout.height).max(0.0) * 0.5
    );
}

#[test]
fn text_layout_reports_caret_and_selection_rects() {
    let mut text = TextSystem::default();
    let layout = text.measure(
        "one two three four five six seven",
        16.0,
        FontWeight::Normal,
        FontStyle::Normal,
        80.0,
    );
    let origin = Point::new(10.0, 20.0);

    let caret = layout.caret_rect(3, origin);
    let selection = layout.selection_rects(0..7, origin);

    assert!(caret.origin.x >= origin.x);
    assert!(caret.origin.y >= origin.y);
    assert!(caret.size.height > 0.0);
    assert!(!selection.is_empty());
    assert!(selection.iter().all(|rect| rect.size.width > 0.0));
}

#[test]
fn wrapped_text_layout_exposes_line_and_glyph_runs() {
    let mut text = TextSystem::default();
    let layout = text.measure(
        "one two three four five six seven eight",
        16.0,
        FontWeight::Normal,
        FontStyle::Normal,
        80.0,
    );

    assert!(layout.lines.len() >= 1);
    assert_eq!(layout.glyph_runs.len(), layout.lines.len());
    assert!(
        layout
            .glyph_runs
            .iter()
            .all(|run| run.glyph_end >= run.glyph_start)
    );
}

#[test]
fn text_layout_caret_index_for_point_projection() {
    let mut text = TextSystem::default();
    let layout = text.measure(
        "Hello World",
        14.0,
        FontWeight::Normal,
        FontStyle::Normal,
        200.0,
    );
    let origin = Point::new(10.0, 20.0);

    // Clicking way to the left of the text should give index 0
    let idx_left = layout.caret_index_for_point(Point::new(0.0, 25.0), origin);
    assert_eq!(idx_left, 0);

    // Clicking way to the right of the text should give the last index (11)
    let idx_right = layout.caret_index_for_point(Point::new(300.0, 25.0), origin);
    assert_eq!(idx_right, 11);

    // Clicking around the middle of the text should give a valid index
    let idx_mid = layout.caret_index_for_point(Point::new(45.0, 25.0), origin);
    assert!(idx_mid <= 11);
}
