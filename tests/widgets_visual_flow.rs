use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::{
    button, checkbox, context_menu, input, list, menu, menu_item, option, radio, scroll_area,
    select, table, tabs, text, textarea, tree, tree_item,
};
use rgui::{Element, FontWeight, LayoutBoxSnapshot, PaintCommand, Rect, Size, Style};

fn showcase_tree() -> Element {
    Element::column()
        .key("widget-showcase")
        .padding(16.0)
        .gap(10.0)
        .child(
            text("Interactive Widget Showcase")
                .heading()
                .key("showcase-title"),
        )
        .child(state_bar())
        .child(toolbar_section())
        .child(data_section())
}

fn state_bar() -> Element {
    Element::row()
        .key("state-bar")
        .gap(12.0)
        .child(state_chip("Clicks", "0"))
        .child(state_chip("Enabled", "on"))
        .child(state_chip("Query", ""))
        .child(state_chip("Focus", ""))
}

fn state_chip(label: &str, value: &str) -> Element {
    Element::row()
        .gap(4.0)
        .child(text(label).key(format!("{}-label", label.to_lowercase())))
        .child(text(value).key(format!("{}-value", label.to_lowercase())))
}

fn toolbar_section() -> Element {
    Element::column()
        .key("toolbar-section")
        .gap(6.0)
        .child(text("Toolbar").heading().key("toolbar-title"))
        .child(
            Element::row()
                .key("toolbar")
                .gap(8.0)
                .child(button("Save").key("save").primary())
                .child(input().key("search"))
                .child(checkbox().key("enabled").checked(true))
                .child(radio().key("choice")),
        )
}

fn data_section() -> Element {
    Element::column()
        .key("data-section")
        .gap(8.0)
        .child(text("Data & Collections").heading().key("data-title"))
        .child(
            Element::row()
                .key("pickers")
                .gap(8.0)
                .child(
                    select()
                        .key("select")
                        .options([
                            option("low", "Low"),
                            option("medium", "Medium"),
                            option("high", "High"),
                        ])
                        .default_value("medium")
                        .placeholder("Priority")
                        .styles(|s| {
                            s.trigger(Style::new().height(32.0));
                            s.popover(Style::new().width(220.0));
                            s.item(Style::new().padding(8.0));
                            s.item_selected(Style::new().font_weight(FontWeight::Bold));
                        }),
                )
                .child(textarea().key("notes"))
                .child(
                    tabs()
                        .key("tabs")
                        .tabs(["General", "Advanced"])
                        .default_active_index(0)
                        .child(text("General Content"))
                        .child(text("Advanced Content")),
                ),
        )
        .child(
            Element::row()
                .key("collections")
                .gap(8.0)
                .child(
                    tree()
                        .key("tree")
                        .items([tree_item("Project").expanded(true).child(tree_item("src"))]),
                )
                .child(
                    table()
                        .key("table")
                        .columns(["Name", "Status"])
                        .rows([["Runtime", "Ready"], ["Renderer", "Ready"]])
                        .default_selected_row(0),
                )
                .child(
                    list()
                        .key("list")
                        .items(["Inbox", "Today", "Done"])
                        .default_selected_index(1),
                )
                .child(
                    menu()
                        .key("menu")
                        .child(menu_item("Archive").key("archive").on_click("archive")),
                ),
        )
        .child(
            scroll_area()
                .key("log_scroll")
                .height(160.0)
                .child(text("Line 1").key("line-1").height(40.0))
                .child(text("Line 2").key("line-2").height(40.0))
                .child(text("Line 3").key("line-3").height(40.0))
                .child(text("Line 4").key("line-4").height(40.0))
                .child(text("Line 5").key("line-5").height(40.0)),
        )
        .child(
            button("Right-click me").key("context-btn").context_menu(
                context_menu()
                    .key("row_menu")
                    .child(menu_item("Delete").key("delete")),
            ),
        )
}

fn frame() -> rgui::runtime::FrameOutput {
    let mut runtime = UiRuntime::default();
    runtime.update(FrameInput {
        root: showcase_tree(),
        viewport: Size::new(808.0, 823.0),
        ..Default::default()
    })
}

fn layout<'a>(snapshot: &'a rgui::UiSnapshot, key: &str) -> &'a LayoutBoxSnapshot {
    snapshot
        .layout_box(key)
        .unwrap_or_else(|| panic!("missing layout box for key {key}"))
}

fn bottom(box_: &LayoutBoxSnapshot) -> f32 {
    box_.y + box_.height
}

fn right(box_: &LayoutBoxSnapshot) -> f32 {
    box_.x + box_.width
}

fn assert_below(snapshot: &rgui::UiSnapshot, upper: &str, lower: &str) {
    let upper_box = layout(snapshot, upper);
    let lower_box = layout(snapshot, lower);
    assert!(
        lower_box.y >= bottom(upper_box),
        "{lower} should start below {upper}: {lower}.y={} {upper}.bottom={}",
        lower_box.y,
        bottom(upper_box)
    );
}

fn assert_after(snapshot: &rgui::UiSnapshot, left: &str, right_key: &str) {
    let left_box = layout(snapshot, left);
    let right_box = layout(snapshot, right_key);
    assert!(
        right_box.x >= right(left_box),
        "{right_key} should start after {left}: {right_key}.x={} {left}.right={}",
        right_box.x,
        right(left_box)
    );
}

fn text_rects<'a>(output: &'a rgui::runtime::FrameOutput, text: &str) -> Vec<Rect> {
    output
        .display_list
        .commands()
        .iter()
        .filter_map(|command| match command {
            PaintCommand::DrawText(cmd) if cmd.text == text => Some(cmd.rect),
            _ => None,
        })
        .collect()
}

#[test]
fn widget_showcase_sections_stack_without_overlap() {
    let output = frame();
    let snapshot = output.snapshot.as_ref().expect("snapshot exists");

    assert_below(snapshot, "showcase-title", "state-bar");
    assert_below(snapshot, "state-bar", "toolbar-section");
    assert_below(snapshot, "toolbar-section", "data-section");
    assert_below(snapshot, "data-title", "pickers");
    assert_below(snapshot, "pickers", "collections");
    assert_below(snapshot, "collections", "log_scroll");
    assert_below(snapshot, "log_scroll", "context-btn");
}

#[test]
fn toolbar_and_picker_rows_advance_children_horizontally() {
    let output = frame();
    let snapshot = output.snapshot.as_ref().expect("snapshot exists");

    assert_after(snapshot, "save", "search");
    assert_after(snapshot, "search", "enabled");
    assert_after(snapshot, "enabled", "choice");
    assert_after(snapshot, "select", "notes");
    assert_after(snapshot, "notes", "tabs");
    assert_after(snapshot, "tree", "table");
    assert_after(snapshot, "table", "list");
    assert_after(snapshot, "list", "menu");
}

#[test]
fn scroll_area_lines_have_distinct_vertical_positions() {
    let output = frame();
    let snapshot = output.snapshot.as_ref().expect("snapshot exists");

    assert_below(snapshot, "line-1", "line-2");
    assert_below(snapshot, "line-2", "line-3");
    assert_below(snapshot, "line-3", "line-4");
    assert_below(snapshot, "line-4", "line-5");
}

fn layout_rect(snapshot: &rgui::UiSnapshot, key: &str) -> Rect {
    let box_ = layout(snapshot, key);
    Rect::new(
        rgui::Point::new(box_.x, box_.y),
        Size::new(box_.width, box_.height),
    )
}

const RECT_EPS: f32 = 0.5;

fn contains_rect(outer: Rect, inner: Rect) -> bool {
    inner.origin.x + RECT_EPS >= outer.origin.x
        && inner.origin.y + RECT_EPS >= outer.origin.y
        && inner.max_x() <= outer.max_x() + RECT_EPS
        && inner.max_y() <= outer.max_y() + RECT_EPS
}

fn assert_text_inside(output: &rgui::runtime::FrameOutput, text: &str, owner_key: &str) {
    let snapshot = output.snapshot.as_ref().expect("snapshot exists");
    let owner = layout_rect(snapshot, owner_key);
    let rects = text_rects(output, text);
    assert!(!rects.is_empty(), "missing DrawText command for {text}");
    assert!(
        rects.iter().any(|rect| contains_rect(owner, *rect)),
        "{text} should paint inside {owner_key}; owner={owner:?} text_rects={rects:?}"
    );
}

#[test]
fn section_heading_text_paints_inside_its_layout_box() {
    let output = frame();

    assert_text_inside(&output, "Interactive Widget Showcase", "showcase-title");
    assert_text_inside(&output, "Toolbar", "toolbar-title");
    assert_text_inside(&output, "Data & Collections", "data-title");
}

#[test]
fn toolbar_button_text_stays_inside_button_rect() {
    let output = frame();

    assert_text_inside(&output, "Save", "save");
}

#[test]
fn menu_and_tab_text_stay_inside_widget_rects() {
    let output = frame();

    assert_text_inside(&output, "Archive", "menu");
    assert_text_inside(&output, "General", "tabs");
    assert_text_inside(&output, "Advanced", "tabs");
}

#[test]
fn text_nodes_have_positive_layout_height_in_showcase() {
    let output = frame();
    let snapshot = output.snapshot.as_ref().expect("snapshot exists");

    for key in [
        "showcase-title",
        "toolbar-title",
        "data-title",
        "line-1",
        "line-2",
    ] {
        let box_ = layout(snapshot, key);
        assert!(
            box_.height > 0.0,
            "{key} should have positive height: {box_:?}"
        );
    }
}

#[test]
fn widget_intrinsic_boxes_are_large_enough_for_text_content() {
    let output = frame();
    let snapshot = output.snapshot.as_ref().expect("snapshot exists");

    let save = layout(snapshot, "save");
    assert!(
        save.height >= 28.0,
        "save button height should fit label: {save:?}"
    );

    let menu = layout(snapshot, "menu");
    assert!(
        menu.height >= 32.0,
        "menu should reserve item height: {menu:?}"
    );

    let tabs = layout(snapshot, "tabs");
    assert!(
        tabs.height >= 32.0,
        "tabs should reserve tab header height: {tabs:?}"
    );
}

#[test]
fn button_label_has_vertical_inset_inside_button() {
    let output = frame();
    let snapshot = output.snapshot.as_ref().expect("snapshot exists");
    let owner = layout_rect(snapshot, "save");
    let rect = text_rects(&output, "Save")
        .into_iter()
        .find(|rect| contains_rect(owner, *rect))
        .expect("Save text inside button");

    assert!(
        rect.origin.y > owner.origin.y,
        "Save text should not touch button top: owner={owner:?} text={rect:?}"
    );
    assert!(
        rect.max_y() < owner.max_y(),
        "Save text should not touch button bottom: owner={owner:?} text={rect:?}"
    );
}

#[test]
fn menu_item_text_has_top_padding() {
    let output = frame();
    let snapshot = output.snapshot.as_ref().expect("snapshot exists");
    let owner = layout_rect(snapshot, "archive");
    let rect = text_rects(&output, "Archive")
        .into_iter()
        .find(|rect| contains_rect(owner, *rect))
        .expect("Archive text inside menu item");

    assert!(
        rect.origin.y >= owner.origin.y,
        "Archive text should sit within its menu item: owner={owner:?} text={rect:?}"
    );
    assert!(
        rect.max_y() <= owner.max_y() + 0.5,
        "Archive text should stay inside its menu item: owner={owner:?} text={rect:?}"
    );
}

#[test]
fn tab_labels_have_distinct_origins_inside_tabs() {
    let output = frame();
    let snapshot = output.snapshot.as_ref().expect("snapshot exists");
    let owner = layout_rect(snapshot, "tabs");

    let general = text_rects(&output, "General");
    let advanced = text_rects(&output, "Advanced");
    assert_eq!(
        general.len(),
        1,
        "General tab should paint once: {general:?}"
    );
    assert_eq!(
        advanced.len(),
        1,
        "Advanced tab should paint once: {advanced:?}"
    );
    assert!(contains_rect(owner, general[0]), "General inside tabs");
    assert!(contains_rect(owner, advanced[0]), "Advanced inside tabs");
    assert_ne!(
        general[0].origin, advanced[0].origin,
        "tab labels should not overlap"
    );
}

#[test]
fn checked_checkbox_mark_is_inset_from_outer_box() {
    let output = frame();
    let snapshot = output.snapshot.as_ref().expect("snapshot exists");
    let checkbox = layout_rect(snapshot, "enabled");

    let mark = output
        .display_list
        .commands()
        .iter()
        .filter_map(|command| match command {
            PaintCommand::DrawRect(cmd) if contains_rect(checkbox, cmd.rect) => Some(cmd.rect),
            _ => None,
        })
        .find(|rect| {
            rect.origin.x > checkbox.origin.x
                && rect.origin.y > checkbox.origin.y
                && rect.max_x() < checkbox.max_x()
                && rect.max_y() < checkbox.max_y()
        })
        .expect("checked mark should be inset inside checkbox");

    assert!(
        mark.size.width <= checkbox.size.width * 0.75,
        "checked mark should not fill the checkbox: checkbox={checkbox:?} mark={mark:?}"
    );
}

#[test]
fn siblings_in_column_do_not_overlap() {
    // Regression test: a sequence of distinct widgets in a Column should
    // each get a non-zero, non-overlapping y range. Before the widget-sizing
    // fix, Card / Alert / Spinner all collapsed to 0x0 via the
    // `min_size_for` fallthrough, so the next sibling stacked at the
    // parent's y=0 and the three rendered on top of each other.
    use rgui::widgets::{alert, card, spinner};

    let root = Element::column()
        .key("root")
        .padding(8.0)
        .gap(4.0)
        .child(card().key("a").child(text("A").key("a-text")))
        .child(alert().key("b").child(text("B").key("b-text")))
        .child(spinner().key("c"));

    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root,
        viewport: Size::new(400.0, 400.0),
        ..Default::default()
    });

    let snapshot = output.snapshot.as_ref().expect("snapshot exists");
    let a = snapshot.layout_box("a").expect("a layout");
    let b = snapshot.layout_box("b").expect("b layout");
    let c = snapshot.layout_box("c").expect("c layout");

    // Each widget has a non-zero size.
    assert!(a.height > 0.0 && b.height > 0.0 && c.height > 0.0,
            "all three widgets should have non-zero height (a={}, b={}, c={})",
            a.height, b.height, c.height);

    // They are stacked in order, no overlap.
    assert!(a.y < b.y, "a.y={} should be < b.y={}", a.y, b.y);
    assert!(b.y < c.y, "b.y={} should be < c.y={}", b.y, c.y);
    let a_max = a.y + a.height;
    let b_max = b.y + b.height;
    assert!(a_max <= b.y + 0.01,
            "a (y={}..{}) should not overlap b (y={}..{})",
            a.y, a_max, b.y, b_max);
    assert!(b_max <= c.y + 0.01,
            "b (y={}..{}) should not overlap c (y={}..{})",
            b.y, b_max, c.y, c.y + c.height);
}

#[test]
fn toolbar_siblings_have_disjoint_x_ranges() {
    // Direct regression test for the user-reported screenshot: a horizontal
    // row of mixed widgets (button / input / checkbox / radio / select /
    // textarea) should produce layout boxes with strictly ascending, non-
    // overlapping x-ranges. Before the widget-sizing fix, widgets whose
    // `min_size_for` fell through to Size::new(0, 0) would collapse to 0
    // wide, causing the next sibling to stack at the same x as the previous
    // one (visible overlap).
    use rgui::widgets::{button, checkbox, input, radio, select, textarea};

    let root = Element::row()
        .key("bar")
        .gap(8.0)
        .child(button("Save").key("save").primary())
        .child(input().key("search"))
        .child(checkbox().key("enabled").checked(true))
        .child(radio().key("choice"))
        .child(select().key("filter").options([option("a", "Active")]))
        .child(textarea().key("notes").width(120.0));

    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root,
        viewport: Size::new(800.0, 200.0),
        ..Default::default()
    });

    let snapshot = output.snapshot.as_ref().expect("snapshot exists");
    let keys = ["save", "search", "enabled", "choice", "filter", "notes"];
    let boxes: Vec<_> = keys
        .iter()
        .map(|k| {
            (
                *k,
                snapshot
                    .layout_box(k)
                    .unwrap_or_else(|| panic!("{k} layout")),
            )
        })
        .collect();

    // Each box has a positive width and height.
    for (k, b) in &boxes {
        assert!(b.width > 0.0 && b.height > 0.0,
                "{k} should have non-zero size (got {}x{})", b.width, b.height);
    }

    // Strictly ascending x with no horizontal overlap.
    for window in boxes.windows(2) {
        let (k1, prev) = &window[0];
        let (k2, next) = &window[1];
        let prev_max_x = prev.x + prev.width;
        assert!(prev_max_x <= next.x + 0.01,
                "{k1} (x={}..{}) should not overlap {k2} (x={}..{})",
                prev.x, prev_max_x, next.x, next.x + next.width);
        assert!(next.x > prev.x,
                "{k2}.x ({}) should be > {k1}.x ({})", next.x, prev.x);
    }
}
