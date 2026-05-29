use rgui::Element;
use rgui::layout::{WidgetIntrinsicInput, intrinsic_widget_size};
use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::{list, table, tabs, text, tree, tree_item};
use rgui::{Size, Theme, WidgetKind};

#[test]
fn theme_exposes_widget_metrics_for_intrinsic_layout() {
    let theme = Theme::light();
    let metrics = &theme.widgets.metrics;

    assert_eq!(metrics.input.min_size, Size::new(160.0, 36.0));
    assert_eq!(metrics.select.trigger_min_size, Size::new(120.0, 36.0));
    assert_eq!(metrics.table.min_size, Size::new(300.0, 180.0));
    assert_eq!(metrics.list.min_size, Size::new(200.0, 180.0));
    assert_eq!(metrics.icon.default_size, Size::new(24.0, 24.0));
    assert_eq!(metrics.canvas.default_size, Size::new(200.0, 150.0));
}

#[test]
fn widget_metrics_can_return_kind_defaults() {
    let metrics = Theme::light().widgets.metrics;

    assert_eq!(
        metrics.min_size_for(WidgetKind::Input),
        Size::new(160.0, 36.0)
    );
    assert_eq!(
        metrics.min_size_for(WidgetKind::Select),
        Size::new(120.0, 36.0)
    );
    assert_eq!(
        metrics.min_size_for(WidgetKind::Canvas),
        Size::new(200.0, 150.0)
    );
}

#[test]
fn intrinsic_widget_size_uses_theme_metrics() {
    let mut theme = Theme::light();
    theme.widgets.metrics.input.min_size = Size::new(240.0, 44.0);
    theme.widgets.metrics.select.trigger_min_size = Size::new(180.0, 40.0);

    let input = intrinsic_widget_size(
        WidgetIntrinsicInput {
            widget_kind: WidgetKind::Input,
            label_width: None,
            known_width: None,
            known_height: None,
        },
        &theme.widgets.metrics,
    );
    let select = intrinsic_widget_size(
        WidgetIntrinsicInput {
            widget_kind: WidgetKind::Select,
            label_width: None,
            known_width: None,
            known_height: None,
        },
        &theme.widgets.metrics,
    );

    assert_eq!(input, Size::new(240.0, 44.0));
    assert_eq!(select, Size::new(180.0, 40.0));
}

#[test]
fn button_intrinsic_size_combines_label_width_with_metric_padding() {
    let mut theme = Theme::light();
    theme.widgets.metrics.button.horizontal_padding = 30.0;
    theme.widgets.metrics.button.min_width = 90.0;

    let button = intrinsic_widget_size(
        WidgetIntrinsicInput {
            widget_kind: WidgetKind::Button,
            label_width: Some(80.0),
            known_width: None,
            known_height: None,
        },
        &theme.widgets.metrics,
    );

    assert_eq!(button.width, 110.0);
    assert_eq!(button.height, theme.widgets.metrics.button.height);
}

#[test]
fn widget_intrinsic_known_width_is_single_source_for_minimum_size() {
    let metrics = rgui::WidgetMetrics::default();
    let size = rgui::layout::intrinsic_widget_size(
        rgui::layout::WidgetIntrinsicInput {
            widget_kind: rgui::WidgetKind::Button,
            label_width: Some(1.0),
            known_width: Some(24.0),
            known_height: Some(12.0),
        },
        &metrics,
    );

    assert_eq!(size.width, 24.0);
    assert_eq!(size.height, 12.0);
}

#[test]
fn theme_body_size_affects_text_layout_measurement() {
    let mut theme = Theme::light();
    theme.typography.body_size = 22.0;

    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column()
            .key("root")
            .child(text("Body").key("body")),
        viewport: Size::new(320.0, 160.0),
        theme,
        ..Default::default()
    });

    let body = output
        .snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.layout_box("body"))
        .expect("body layout");

    assert!(body.height >= 22.0);
}

#[test]
fn theme_metrics_affect_input_text_origin() {
    let mut theme = Theme::light();
    theme.widgets.metrics.input.horizontal_padding = 20.0;

    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: rgui::widgets::input().key("input").default_value("Hello"),
        viewport: Size::new(240.0, 100.0),
        theme,
        ..Default::default()
    });

    let input = output
        .snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.layout_box("input"))
        .expect("input layout");
    let text = output
        .display_list
        .commands()
        .iter()
        .find_map(|command| match command {
            rgui::PaintCommand::DrawText(cmd) if cmd.text == "Hello" => Some(cmd),
            _ => None,
        })
        .expect("input text command");

    assert!(text.rect.origin.x >= input.x + 20.0);
}

#[test]
fn overlay_metrics_control_popover_minimum_size() {
    let mut theme = Theme::light();
    theme.widgets.metrics.overlay.min_width = 220.0;
    theme.widgets.metrics.overlay.min_height = 72.0;

    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: rgui::widgets::button("Open").key("open").popover(
            rgui::widgets::popover()
                .open(true)
                .key("pop")
                .child(rgui::widgets::text("Small")),
        ),
        viewport: Size::new(360.0, 240.0),
        theme,
        ..Default::default()
    });

    let overlay = output
        .snapshot
        .as_ref()
        .and_then(|snapshot| {
            snapshot
                .overlays()
                .iter()
                .find(|overlay| overlay.key.as_deref() == Some("pop"))
        })
        .expect("popover overlay");

    assert!(overlay.rect.size.width >= 220.0);
    assert!(overlay.rect.size.height >= 72.0);
}

#[test]
fn tabs_paint_geometry_uses_theme_metrics() {
    let mut theme = Theme::light();
    theme.widgets.metrics.tabs.horizontal_padding = 24.0;
    theme.widgets.metrics.tabs.tab_gap = 14.0;
    theme.widgets.metrics.tabs.tab_height = 28.0;
    theme.widgets.metrics.tabs.tab_min_width = 72.0;

    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: tabs()
            .key("tabs")
            .tabs(["One", "Two"])
            .width(220.0)
            .height(80.0),
        viewport: Size::new(260.0, 120.0),
        theme,
        ..Default::default()
    });

    let tab_rects: Vec<_> = output
        .display_list
        .commands()
        .iter()
        .filter_map(|command| match command {
            rgui::PaintCommand::DrawRect(cmd)
                if cmd.z_index == 2 && (cmd.rect.size.height - 28.0).abs() < 0.01 =>
            {
                Some(cmd.rect)
            }
            _ => None,
        })
        .collect();

    assert_eq!(tab_rects.len(), 2);
    assert!(tab_rects[0].origin.x >= 24.0);
    assert!(tab_rects[0].size.width >= 72.0);
    assert!((tab_rects[1].origin.x - tab_rects[0].max_x() - 14.0).abs() < 0.01);
}

#[test]
fn collection_painters_use_theme_row_metrics() {
    let mut theme = Theme::light();
    theme.widgets.metrics.tree.row_height = 32.0;
    theme.widgets.metrics.table.row_height = 36.0;
    theme.widgets.metrics.table.cell_padding = 12.0;
    theme.widgets.metrics.list.row_height = 38.0;
    theme.widgets.metrics.list.item_padding = 14.0;

    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column()
            .child(
                tree()
                    .key("tree")
                    .items([tree_item("Root").expanded(true).child(tree_item("Leaf"))])
                    .height(96.0),
            )
            .child(
                table()
                    .key("table")
                    .columns(["Name"])
                    .rows([["First"], ["Second"]])
                    .default_selected_row(0)
                    .height(120.0),
            )
            .child(
                list()
                    .key("list")
                    .items(["Alpha", "Beta"])
                    .default_selected_index(0)
                    .height(96.0),
            ),
        viewport: Size::new(360.0, 360.0),
        theme,
        ..Default::default()
    });

    let tree_root = output
        .display_list
        .commands()
        .iter()
        .find_map(|command| match command {
            rgui::PaintCommand::DrawText(cmd) if cmd.text.contains("Root") => {
                Some(cmd.rect.origin.y)
            }
            _ => None,
        })
        .expect("tree root text");
    let tree_leaf = output
        .display_list
        .commands()
        .iter()
        .find_map(|command| match command {
            rgui::PaintCommand::DrawText(cmd) if cmd.text.contains("Leaf") => {
                Some(cmd.rect.origin.y)
            }
            _ => None,
        })
        .expect("tree leaf text");
    assert!((tree_leaf - tree_root - 32.0).abs() < 0.01);

    let table_selection = output
        .display_list
        .commands()
        .iter()
        .find_map(|command| match command {
            rgui::PaintCommand::DrawRect(cmd) if (cmd.rect.size.height - 36.0).abs() < 0.01 => {
                Some(cmd.rect)
            }
            _ => None,
        })
        .expect("table metric row");
    assert_eq!(table_selection.size.height, 36.0);

    let first_cell = output
        .display_list
        .commands()
        .iter()
        .find_map(|command| match command {
            rgui::PaintCommand::DrawText(cmd) if cmd.text == "First" => Some(cmd.rect.origin.x),
            _ => None,
        })
        .expect("first table cell text");
    let table_layout = output
        .snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.layout_box("table"))
        .expect("table layout");
    assert!(first_cell >= table_layout.x + 12.0);

    let list_selection = output
        .display_list
        .commands()
        .iter()
        .find_map(|command| match command {
            rgui::PaintCommand::DrawRect(cmd) if (cmd.rect.size.height - 34.0).abs() < 0.01 => {
                Some(cmd.rect)
            }
            _ => None,
        })
        .expect("list selected row");
    assert_eq!(list_selection.size.height, 34.0);

    let list_layout = output
        .snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.layout_box("list"))
        .expect("list layout");
    assert!(list_selection.origin.x >= list_layout.x + 14.0);
}
