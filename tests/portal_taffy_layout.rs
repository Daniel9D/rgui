use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::{button, popover, text};
use rgui::{Element, Size};

fn snapshot_rect(item: &rgui::LayoutBoxSnapshot) -> rgui::Rect {
    rgui::Rect::new(
        rgui::Point::new(item.x, item.y),
        rgui::Size::new(item.width, item.height),
    )
}

fn frame(root: Element) -> rgui::runtime::FrameOutput {
    let mut runtime = UiRuntime::default();
    runtime.update(FrameInput {
        root,
        viewport: Size::new(420.0, 260.0),
        ..FrameInput::default()
    })
}

#[test]
fn popover_children_use_column_gap_and_padding() {
    let app = Element::row().key("root").child(
        button("Open").key("open").popover(
            popover().open(true).key("menu").child(
                Element::column()
                    .key("menu-column")
                    .padding(12.0)
                    .gap(10.0)
                    .child(button("First").key("first"))
                    .child(button("Second").key("second")),
            ),
        ),
    );

    let output = frame(app);
    let first = output
        .hit_test
        .entries()
        .iter()
        .find(|entry| entry.key.as_deref() == Some("first"))
        .expect("first button");
    let second = output
        .hit_test
        .entries()
        .iter()
        .find(|entry| entry.key.as_deref() == Some("second"))
        .expect("second button");
    let panel = output
        .hit_test
        .entries()
        .iter()
        .find(|entry| entry.key.as_deref() == Some("menu"))
        .expect("popover panel");

    assert!(second.rect.origin.y >= first.rect.max_y() + 10.0);
    assert!(first.rect.origin.x >= panel.rect.origin.x + 20.0);
}

#[test]
fn nested_overlay_container_children_receive_stable_portal_ids() {
    let app = Element::row().key("root").child(
        button("Open").key("open").popover(
            popover().open(true).key("menu").child(
                Element::column()
                    .key("group")
                    .child(text("Nested").key("nested")),
            ),
        ),
    );

    let first = frame(app.clone());
    let second = frame(app);

    let first_node = first
        .semantics
        .nodes()
        .iter()
        .find(|node| node.key.as_deref() == Some("nested"))
        .map(|node| node.node);
    let second_node = second
        .semantics
        .nodes()
        .iter()
        .find(|node| node.key.as_deref() == Some("nested"))
        .map(|node| node.node);

    assert_eq!(first_node, second_node);
    assert!(first_node.is_some());
}

#[test]
fn portal_child_ids_do_not_overlap_document_ids() {
    let app = Element::row()
        .key("root")
        .child(button("Document").key("document-button"))
        .child(
            button("Open").key("open").popover(
                popover()
                    .open(true)
                    .key("menu")
                    .child(button("Portal").key("portal-button")),
            ),
        );

    let output = frame(app);
    let doc = output
        .hit_test
        .entries()
        .iter()
        .find(|entry| entry.key.as_deref() == Some("document-button"))
        .expect("document button");
    let portal = output
        .hit_test
        .entries()
        .iter()
        .find(|entry| entry.key.as_deref() == Some("portal-button"))
        .expect("portal button");

    assert_ne!(doc.node, portal.node);
    assert_ne!(portal.node.raw() & 0x8000_0000_0000_0000, 0);
}

#[test]
fn popover_anchor_uses_document_layout_result_bounds() {
    use rgui::runtime::{FrameInput, UiRuntime};
    use rgui::widgets::{button, popover, text};
    use rgui::{Element, Size};

    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column().padding(24.0).child(
            button("Open").key("anchor").popover(
                popover()
                    .open(true)
                    .key("pop")
                    .open(true)
                    .child(text("Body")),
            ),
        ),
        viewport: Size::new(320.0, 240.0),
        ..Default::default()
    });

    let snapshot = output.snapshot.as_ref().expect("snapshot exists");
    let anchor = snapshot.layout_box("anchor").expect("anchor layout");
    let overlay = snapshot
        .overlays()
        .iter()
        .find(|overlay| overlay.key.as_deref() == Some("pop"))
        .expect("popover overlay");

    assert!(overlay.rect.origin.x >= anchor.x);
    assert!(overlay.rect.origin.y >= anchor.y + anchor.height);
}

#[test]
fn popover_portal_child_bounds_match_taffy_layout_boxes() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: button("Open").key("open").popover(
            popover().open(true).key("menu").child(
                rgui::Element::column()
                    .key("menu-content")
                    .gap(10.0)
                    .padding(14.0)
                    .child(button("First").key("first"))
                    .child(button("Second").key("second")),
            ),
        ),
        viewport: Size::new(360.0, 260.0),
        ..Default::default()
    });

    let snapshot = output.snapshot.as_ref().expect("snapshot exists");
    let first_layout = snapshot.layout_box("first").expect("first layout");
    let second_layout = snapshot.layout_box("second").expect("second layout");

    assert!(second_layout.y >= first_layout.y + first_layout.height + 10.0);

    let first_hit = output
        .hit_test
        .entries()
        .iter()
        .find(|entry| entry.key.as_deref() == Some("first"))
        .expect("first hit-test entry");
    assert_eq!(first_hit.rect, snapshot_rect(first_layout));
}
