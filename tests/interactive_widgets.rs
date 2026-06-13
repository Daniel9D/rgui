use rgui::runtime::{FrameInput, UiCommand, UiRuntime};
use rgui::widgets::{button, list, menu, option, select, table, tabs, text, tree, tree_item};
use rgui::{Element, LayerKind, Point, PointerButton, PointerEvent, Size, UiEvent};

fn update(runtime: &mut UiRuntime, root: Element) -> rgui::runtime::FrameOutput {
    runtime.update(FrameInput {
        root,
        viewport: Size::new(320.0, 180.0),
        ..FrameInput::default()
    })
}

fn click(runtime: &mut UiRuntime, point: Point) {
    runtime.dispatch(UiEvent::PointerDown(PointerEvent {
        position: point,
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));
    runtime.dispatch(UiEvent::PointerUp(PointerEvent {
        position: point,
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));
}

#[test]
fn select_click_opens_options_without_changing_selection() {
    let app = select()
        .key("choice")
        .options([
            option("One", "One"),
            option("Two", "Two"),
            option("Three", "Three"),
        ]);
    let mut runtime = UiRuntime::default();
    let output = update(&mut runtime, app.clone());
    let hit = output.hit_test.entries()[0].rect;

    click(
        &mut runtime,
        Point::new(hit.origin.x + 8.0, hit.origin.y + hit.size.height * 0.5),
    );
    let output = update(&mut runtime, app);

    assert_eq!(runtime.selected_index("choice"), Some(0));
    assert_eq!(runtime.selected_value("choice").as_deref(), Some("One"));
    assert!(
        output
            .hit_test
            .entries()
            .iter()
            .any(|entry| entry.key.as_deref() == Some("choice::__option::1")
                && entry.layer == LayerKind::Popover),
        "open select should expose option hit targets in an overlay"
    );
}

#[test]
fn select_option_click_updates_selected_index_and_closes_options() {
    let app = select()
        .key("choice")
        .options([
            option("One", "One"),
            option("Two", "Two"),
            option("Three", "Three"),
        ]);
    let mut runtime = UiRuntime::default();
    let output = update(&mut runtime, app.clone());
    let trigger = output.hit_test.entries()[0].rect;

    click(
        &mut runtime,
        Point::new(
            trigger.origin.x + 8.0,
            trigger.origin.y + trigger.size.height * 0.5,
        ),
    );
    let open = update(&mut runtime, app.clone());
    let option = open
        .hit_test
        .entries()
        .iter()
        .find(|entry| entry.key.as_deref() == Some("choice::__option::1"))
        .expect("second option hit target")
        .rect;

    click(
        &mut runtime,
        Point::new(
            option.origin.x + 8.0,
            option.origin.y + option.size.height * 0.5,
        ),
    );
    let closed = update(&mut runtime, app);

    assert_eq!(runtime.selected_index("choice"), Some(1));
    assert_eq!(runtime.selected_value("choice").as_deref(), Some("Two"));
    assert!(
        closed.hit_test.entries().iter().all(|entry| !entry
            .key
            .as_deref()
            .is_some_and(|key| key.starts_with("choice::__option::"))),
        "select options should close after choosing an option"
    );
}

#[test]
fn tabs_update_active_tab() {
    let app = tabs()
        .key("tabs")
        .tabs(["A", "B"])
        .child(text("A"))
        .child(text("B"));
    let mut runtime = UiRuntime::default();
    let output = update(&mut runtime, app.clone());
    let hit = output.hit_test.entries()[0].rect;

    click(
        &mut runtime,
        Point::new(hit.origin.x + 70.0, hit.origin.y + 12.0),
    );
    update(&mut runtime, app);

    assert_eq!(runtime.active_index("tabs"), Some(1));
}

#[test]
fn menu_item_emits_click_command() {
    let app = menu()
        .key("menu")
        .child(button("Delete").key("delete").on_click("delete"));
    let mut runtime = UiRuntime::default();
    let output = update(&mut runtime, app);
    let delete = output
        .hit_test
        .entries()
        .iter()
        .find(|entry| entry.key.as_deref() == Some("delete"))
        .expect("delete item")
        .rect;

    click(
        &mut runtime,
        Point::new(delete.origin.x + 4.0, delete.origin.y + 4.0),
    );

    assert!(runtime.drain_commands().iter().any(|cmd| {
        matches!(
            cmd,
            UiCommand::Click {
                key: Some(key),
                action: Some(action)
            } if key == "delete" && action == "delete"
        )
    }));
}

#[test]
fn tree_click_toggles_expanded_item_state() {
    let app = tree()
        .key("tree")
        .items([tree_item("Root").child(tree_item("Child"))]);
    let mut runtime = UiRuntime::default();
    let output = update(&mut runtime, app.clone());
    let hit = output.hit_test.entries()[0].rect;

    click(
        &mut runtime,
        Point::new(hit.origin.x + 8.0, hit.origin.y + 14.0),
    );
    update(&mut runtime, app);

    assert_eq!(runtime.tree_item_expanded("tree", 0), Some(true));
}

#[test]
fn table_click_updates_selected_row() {
    let app = table()
        .key("table")
        .columns(["Name"])
        .rows([["One"], ["Two"]]);
    let mut runtime = UiRuntime::default();
    let output = update(&mut runtime, app.clone());
    let hit = output.hit_test.entries()[0].rect;
    let metrics = rgui::Theme::light().metrics.table;

    click(
        &mut runtime,
        Point::new(
            hit.origin.x + metrics.cell_padding * 2.0,
            hit.origin.y + metrics.cell_padding + metrics.row_height * 2.5,
        ),
    );
    update(&mut runtime, app);

    assert_eq!(runtime.table_selected_row("table"), Some(1));
}

#[test]
fn list_click_updates_selected_index() {
    let app = list().key("list").items(["One", "Two", "Three"]);
    let mut runtime = UiRuntime::default();
    let output = update(&mut runtime, app.clone());
    let hit = output.hit_test.entries()[0].rect;

    click(
        &mut runtime,
        Point::new(hit.origin.x + 8.0, hit.origin.y + 36.0),
    );
    update(&mut runtime, app);

    assert_eq!(runtime.list_selected_index("list"), Some(1));
}
