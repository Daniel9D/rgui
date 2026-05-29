use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::{list, menu, menu_item, option, select, tab, table, tabs, tree, tree_item};
use rgui::{Size, WidgetSpec};

#[test]
fn public_widget_builders_cover_common_app_code() {
    let root = rgui::Element::column()
        .child(
            select()
                .key("priority")
                .options([
                    option("low", "Low"),
                    option("medium", "Medium"),
                    option("high", "High"),
                ])
                .default_value("medium")
                .placeholder("Priority"),
        )
        .child(
            tabs()
                .key("settings")
                .tabs(["General", "Advanced"])
                .default_active_index(0)
                .child(tab("General"))
                .child(tab("Advanced")),
        )
        .child(
            tree()
                .key("project")
                .items([tree_item("src").expanded(true).child(tree_item("lib.rs"))]),
        )
        .child(
            table()
                .key("jobs")
                .columns(["Name", "Status"])
                .rows([["Runtime", "Ready"], ["Renderer", "Ready"]])
                .default_selected_row(0),
        )
        .child(
            list()
                .key("inbox")
                .items(["Inbox", "Today", "Done"])
                .default_selected_index(1),
        )
        .child(
            menu()
                .key("actions")
                .items([menu_item("Archive").on_click("archive")]),
        );

    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root,
        viewport: Size::new(640.0, 480.0),
        ..Default::default()
    });

    assert!(output.semantics.by_key("priority").is_some());
    assert_eq!(
        runtime.selected_value("priority").as_deref(),
        Some("medium")
    );
    assert_eq!(runtime.active_index("settings"), Some(0));
    assert_eq!(runtime.table_selected_row("jobs"), Some(0));
    assert_eq!(runtime.list_selected_index("inbox"), Some(1));
}

#[test]
fn select_builder_stores_value_label_options_in_spec() {
    let element = select().options([option("low", "Low"), option("high", "High")]);
    let Some(WidgetSpec::Select(spec)) = element.widget_spec else {
        panic!("select spec");
    };

    assert_eq!(spec.options[0].value, "low");
    assert_eq!(spec.options[0].label, "Low");
    assert_eq!(spec.options[1].value, "high");
    assert_eq!(spec.options[1].label, "High");
}
