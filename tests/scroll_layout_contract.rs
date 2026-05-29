use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::text;
use rgui::{Element, Overflow, Size, Vec2};

#[test]
fn scroll_area_clips_and_reports_layout_owned_content_size() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column()
            .key("scroll")
            .height(80.0)
            .overflow(Overflow::Scroll)
            .child(text("A").height(120.0).key("a"))
            .child(text("B").height(120.0).key("b")),
        viewport: Size::new(240.0, 120.0),
        ..Default::default()
    });

    let snapshot = output.snapshot.as_ref().expect("snapshot exists");
    let scroll = snapshot.layout_box("scroll").expect("scroll layout");

    assert_eq!(scroll.height, 80.0);
    assert!(scroll.clip_rect.is_some());
    assert!(scroll.content_height >= 240.0);
}

#[test]
fn scroll_offset_translates_child_snapshot_rects() {
    let mut runtime = UiRuntime::default();
    runtime.set_scroll_offset_for_key("scroll", Vec2::new(0.0, 20.0));

    let output = runtime.update(FrameInput {
        root: Element::column()
            .key("scroll")
            .height(60.0)
            .overflow(Overflow::Scroll)
            .child(text("Tall").height(180.0).key("tall")),
        viewport: Size::new(240.0, 120.0),
        ..Default::default()
    });

    let snapshot = output.snapshot.as_ref().expect("snapshot exists");
    let scroll = snapshot.layout_box("scroll").expect("scroll layout");
    let tall = snapshot.layout_box("tall").expect("tall layout");

    assert_eq!(tall.y, scroll.y - 20.0);
}

#[test]
fn nested_scroll_content_size_uses_post_enrichment_child_coordinates() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column().key("root").padding(10.0).child(
            Element::column().key("wrapper").padding(12.0).child(
                Element::column()
                    .key("scroll")
                    .height(80.0)
                    .overflow(Overflow::Scroll)
                    .child(text("A").key("a").height(120.0))
                    .child(text("B").key("b").height(120.0)),
            ),
        ),
        viewport: Size::new(240.0, 160.0),
        ..Default::default()
    });

    let snapshot = output.snapshot.as_ref().expect("snapshot exists");
    let scroll = snapshot.layout_box("scroll").expect("scroll layout");

    assert!(scroll.content_height >= 240.0);
    assert!(scroll.content_height < 300.0);
}
