//! Integration tests for the RML parser (requires `--features rml`).

#[cfg(feature = "rml")]
mod rml_tests {
    use rgui::rml;
    use rgui::{Align, Element, Length, Overflow, WidgetSpec};

    // ── helpers ──────────────────────────────────────────────────────────────

    /// Run `f` in a thread with 8 MiB stack so that tests don't
    /// stack-overflow in debug (unoptimised) builds where `Element` values
    /// have large stack frames.
    fn with_large_stack<F, R>(f: F) -> R
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(f)
            .expect("thread spawn failed")
            .join()
            .expect("thread panicked")
    }

    fn parse_ok(src: &'static str) -> Element {
        with_large_stack(move || rml::parse(src).expect("parse should succeed").element)
    }

    fn parse_warn(src: &'static str) -> (Element, Vec<rml::RmlWarning>) {
        with_large_stack(move || {
            let result = rml::parse(src).expect("parse should succeed");
            (result.element, result.warnings)
        })
    }

    fn parse_err(src: &'static str) -> rml::RmlError {
        with_large_stack(move || rml::parse(src).expect_err("parse should fail"))
    }

    // ── Phase 1: Layout ───────────────────────────────────────────────────────

    #[test]
    fn phase1_column_text_row() {
        let el = parse_ok(
            r#"
            <Column key="settings" padding="16" gap="8">
                <Text heading>Settings</Text>
                <Row gap="8" align-items="center">
                    <Text>Name</Text>
                </Row>
            </Column>
        "#,
        );
        // Top-level is a column
        assert!(matches!(
            el.kind,
            rgui::ElementKind::Primitive(rgui::PrimitiveKind::Column)
        ));
        assert_eq!(el.key.as_ref().map(|k| k.as_str()), Some("settings"));
        assert_eq!(el.style.padding, Some(rgui::Edge::all(Length::Px(16.0))));
        assert_eq!(el.style.gap, Some(Length::Px(8.0)));

        // First child: Text with heading style
        let text_child = &el.children[0];
        assert!(matches!(text_child.kind, rgui::ElementKind::Text(_)));
        let ts = text_child.style.text.as_ref().unwrap();
        assert_eq!(ts.size, Length::Px(24.0));

        // Second child: Row
        let row = &el.children[1];
        assert!(matches!(
            row.kind,
            rgui::ElementKind::Primitive(rgui::PrimitiveKind::Row)
        ));
        assert_eq!(row.style.gap, Some(Length::Px(8.0)));
        assert_eq!(row.style.align_items, Some(Align::Center));
    }

    #[test]
    fn phase1_scroll_area() {
        let el = parse_ok(
            r#"
            <ScrollArea key="log" height="160">
                <Text height="40">Line 1</Text>
                <Text height="40">Line 2</Text>
            </ScrollArea>
        "#,
        );
        // scroll_area() is column + overflow scroll
        assert!(matches!(
            el.kind,
            rgui::ElementKind::Primitive(rgui::PrimitiveKind::Column)
        ));
        assert_eq!(el.style.overflow_y, Some(Overflow::Scroll));
        assert_eq!(el.style.height, Some(Length::Px(160.0)));
        assert_eq!(el.children.len(), 2);
    }

    #[test]
    fn phase1_scroll_area_axis_y_sets_vertical_scroll_only() {
        let el = parse_ok(r#"<ScrollArea axis="y" />"#);
        assert_eq!(el.style.overflow_x, Some(Overflow::Hidden));
        assert_eq!(el.style.overflow_y, Some(Overflow::Scroll));
    }

    #[test]
    fn phase1_scroll_area_axis_x_sets_horizontal_scroll_only() {
        let el = parse_ok(r#"<ScrollArea axis="x" />"#);
        assert_eq!(el.style.overflow_x, Some(Overflow::Scroll));
        assert_eq!(el.style.overflow_y, Some(Overflow::Hidden));
    }

    #[test]
    fn phase1_scroll_area_axis_both_keeps_both_axes_scrollable() {
        let el = parse_ok(r#"<ScrollArea axis="both" />"#);
        assert_eq!(el.style.overflow_x, Some(Overflow::Scroll));
        assert_eq!(el.style.overflow_y, Some(Overflow::Scroll));
    }

    #[test]
    fn phase1_grid() {
        let el = parse_ok(
            r#"
            <Grid key="dash" grid-template-columns="1fr 1fr" gap="12">
                <Text>Left</Text>
                <Text>Right</Text>
            </Grid>
        "#,
        );
        assert!(matches!(
            el.kind,
            rgui::ElementKind::Primitive(rgui::PrimitiveKind::Grid)
        ));
        assert!(el.style.grid_template_columns.is_some());
    }

    #[test]
    fn phase1_canvas() {
        let el = parse_ok(r#"<Canvas key="chart" name="chart-v2" width="320" height="180" />"#);
        assert!(matches!(el.kind, rgui::ElementKind::Canvas(_)));
        if let rgui::ElementKind::Canvas(ref cs) = el.kind {
            assert_eq!(cs.name, "chart-v2");
        }
    }

    #[test]
    fn phase1_stack_and_absolute() {
        parse_ok(
            r#"
            <Stack width="240" height="160">
                <Text>Base</Text>
                <Button z-index="2">Overlay</Button>
            </Stack>
        "#,
        );
        parse_ok(
            r#"
            <Absolute width="320" height="200">
                <Button position="absolute" inset="16 0 0 16">Top Left</Button>
            </Absolute>
        "#,
        );
    }

    // ── Phase 2: Forms ────────────────────────────────────────────────────────

    #[test]
    fn phase2_button_label_from_child() {
        let el = parse_ok(r#"<Button key="save" primary on-click="save">Save</Button>"#);
        assert!(matches!(el.widget_spec, Some(WidgetSpec::Button(_))));
        if let Some(WidgetSpec::Button(ref bs)) = el.widget_spec {
            assert_eq!(bs.label.as_deref(), Some("Save"));
        }
        assert!(el.variant.is_some());
        assert_eq!(el.event_handlers.on_click_action.as_deref(), Some("save"));
    }

    #[test]
    fn phase2_button_disabled_loading() {
        let el = parse_ok(r#"<Button disabled loading>Click</Button>"#);
        if let Some(WidgetSpec::Button(ref bs)) = el.widget_spec {
            assert!(bs.disabled);
            assert!(bs.loading);
        }
    }

    #[test]
    fn phase2_text_input() {
        let el = parse_ok(
            r#"<TextInput key="name" placeholder="Name" default-value="Alice" aria-label="User name" />"#,
        );
        assert!(matches!(el.widget_spec, Some(WidgetSpec::Input(_))));
        if let Some(WidgetSpec::Input(ref spec)) = el.widget_spec {
            assert_eq!(spec.placeholder.as_deref(), Some("Name"));
            assert_eq!(spec.default_value.as_deref(), Some("Alice"));
            assert_eq!(spec.aria_label.as_deref(), Some("User name"));
        }
    }

    #[test]
    fn phase2_input_alias() {
        // <Input /> is an alias for <TextInput />
        let el = parse_ok(r#"<Input placeholder="Search" />"#);
        assert!(matches!(el.widget_spec, Some(WidgetSpec::Input(_))));
    }

    #[test]
    fn phase2_checkbox() {
        let el = parse_ok(r#"<Checkbox key="enabled" label="Enabled" checked />"#);
        assert!(matches!(el.widget_spec, Some(WidgetSpec::Checkbox(_))));
        assert_eq!(el.checked, Some(true));
        if let Some(WidgetSpec::Checkbox(ref spec)) = el.widget_spec {
            assert_eq!(spec.label.as_deref(), Some("Enabled"));
        }
    }

    #[test]
    fn phase2_checkbox_text_child_label() {
        let el = parse_ok(r#"<Checkbox key="terms">Accept terms</Checkbox>"#);
        if let Some(WidgetSpec::Checkbox(ref spec)) = el.widget_spec {
            assert_eq!(spec.label.as_deref(), Some("Accept terms"));
        }
    }

    #[test]
    fn phase2_radio() {
        let el = parse_ok(r#"<Radio key="choice-a" value="a" label="Choice A" />"#);
        assert!(matches!(el.widget_spec, Some(WidgetSpec::Radio(_))));
        if let Some(WidgetSpec::Radio(ref spec)) = el.widget_spec {
            assert_eq!(spec.value.as_deref(), Some("a"));
            assert_eq!(spec.label.as_deref(), Some("Choice A"));
        }
    }

    #[test]
    fn phase2_select_with_options() {
        let el = parse_ok(
            r#"
            <Select key="priority" placeholder="Priority" default-value="medium">
                <Option value="low">Low</Option>
                <Option value="medium">Medium</Option>
                <Option value="high" disabled>High</Option>
            </Select>
        "#,
        );
        assert!(matches!(el.widget_spec, Some(WidgetSpec::Select(_))));
        if let Some(WidgetSpec::Select(ref spec)) = el.widget_spec {
            assert_eq!(spec.placeholder.as_deref(), Some("Priority"));
            assert_eq!(spec.default_value.as_deref(), Some("medium"));
            assert_eq!(spec.options.len(), 3);
            assert_eq!(spec.options[0].value, "low");
            assert_eq!(spec.options[0].label, "Low");
            assert!(!spec.options[0].disabled);
            assert!(spec.options[2].disabled);
        }
    }

    #[test]
    fn phase2_select_option_alias() {
        let el = parse_ok(
            r#"
            <Select>
                <SelectOption value="a">A</SelectOption>
            </Select>
        "#,
        );
        if let Some(WidgetSpec::Select(ref spec)) = el.widget_spec {
            assert_eq!(spec.options.len(), 1);
        }
    }

    #[test]
    fn phase2_select_style_children_apply_part_styles() {
        let el = parse_ok(
            r##"
            <Select key="priority">
                <SelectStyle part="trigger" height="32" />
                <SelectStyle part="popover" width="220" />
                <SelectStyle part="item" padding="8" />
                <Option value="low">Low</Option>
            </Select>
            "##,
        );

        if let Some(WidgetSpec::Select(ref spec)) = el.widget_spec {
            assert_eq!(
                spec.styles.trigger.as_ref().and_then(|s| s.height.clone()),
                Some(Length::Px(32.0))
            );
            assert_eq!(
                spec.styles.popover.as_ref().and_then(|s| s.width.clone()),
                Some(Length::Px(220.0))
            );
            assert!(
                spec.styles
                    .item
                    .as_ref()
                    .and_then(|s| s.padding.clone())
                    .is_some()
            );
        } else {
            panic!("expected select spec");
        }
    }

    #[test]
    fn phase2_textarea() {
        let el = parse_ok(
            r#"<Textarea key="notes" placeholder="Notes" default-value="Initial" rows="4" />"#,
        );
        assert!(matches!(el.widget_spec, Some(WidgetSpec::Textarea(_))));
        if let Some(WidgetSpec::Textarea(ref spec)) = el.widget_spec {
            assert_eq!(spec.placeholder.as_deref(), Some("Notes"));
            assert_eq!(spec.default_value.as_deref(), Some("Initial"));
            assert_eq!(spec.rows, Some(4));
        }
    }

    // ── Phase 3: Collections ──────────────────────────────────────────────────

    #[test]
    fn phase3_tabs_from_children() {
        let el = parse_ok(
            r#"
            <Tabs key="tabs" active-index="0">
                <Tab label="General" />
                <Tab label="Advanced" />
            </Tabs>
        "#,
        );
        assert!(matches!(el.widget_spec, Some(WidgetSpec::Tabs(_))));
        if let Some(WidgetSpec::Tabs(ref spec)) = el.widget_spec {
            assert_eq!(spec.tabs, vec!["General", "Advanced"]);
            assert_eq!(spec.active_index, Some(0));
        }
    }

    #[test]
    fn phase3_tabs_compact() {
        let el = parse_ok(r#"<Tabs key="tabs" tabs="General,Advanced" active-index="1" />"#);
        if let Some(WidgetSpec::Tabs(ref spec)) = el.widget_spec {
            assert_eq!(spec.tabs, vec!["General", "Advanced"]);
            assert_eq!(spec.active_index, Some(1));
        }
    }

    #[test]
    fn phase3_list_from_children() {
        let el = parse_ok(
            r#"
            <List key="list" selected-index="1">
                <ListItem>Inbox</ListItem>
                <ListItem>Today</ListItem>
                <ListItem>Done</ListItem>
            </List>
        "#,
        );
        assert!(matches!(el.widget_spec, Some(WidgetSpec::List(_))));
        if let Some(WidgetSpec::List(ref spec)) = el.widget_spec {
            assert_eq!(spec.items, vec!["Inbox", "Today", "Done"]);
            assert_eq!(spec.selected_index, Some(1));
        }
    }

    #[test]
    fn phase3_list_compact() {
        let el = parse_ok(r#"<List items="Inbox,Today,Done" selected-index="0" />"#);
        if let Some(WidgetSpec::List(ref spec)) = el.widget_spec {
            assert_eq!(spec.items, vec!["Inbox", "Today", "Done"]);
        }
    }

    #[test]
    fn phase3_table_from_children() {
        let el = parse_ok(
            r#"
            <Table key="table" selected-row="0">
                <Columns>
                    <ColumnDef>Name</ColumnDef>
                    <ColumnDef>Status</ColumnDef>
                </Columns>
                <Row>
                    <Cell>Runtime</Cell>
                    <Cell>Ready</Cell>
                </Row>
                <Row>
                    <Cell>Renderer</Cell>
                    <Cell>Ready</Cell>
                </Row>
            </Table>
        "#,
        );
        assert!(matches!(el.widget_spec, Some(WidgetSpec::Table(_))));
        if let Some(WidgetSpec::Table(ref spec)) = el.widget_spec {
            assert_eq!(spec.columns, vec!["Name", "Status"]);
            assert_eq!(spec.rows.len(), 2);
            assert_eq!(spec.rows[0], vec!["Runtime", "Ready"]);
            assert_eq!(spec.selected_row, Some(0));
        }
    }

    #[test]
    fn phase3_table_compact() {
        let el = parse_ok(
            r#"
            <Table columns="Name,Status" selected-row="0">
                <TableRow values="Runtime,Ready" />
                <TableRow values="Renderer,Ready" />
            </Table>
        "#,
        );
        if let Some(WidgetSpec::Table(ref spec)) = el.widget_spec {
            assert_eq!(spec.columns, vec!["Name", "Status"]);
            assert_eq!(spec.rows[1], vec!["Renderer", "Ready"]);
        }
    }

    #[test]
    fn phase3_tree() {
        let el = parse_ok(
            r#"
            <Tree key="tree">
                <TreeItem label="Project" expanded>
                    <TreeItem label="src" />
                </TreeItem>
            </Tree>
        "#,
        );
        assert!(matches!(el.widget_spec, Some(WidgetSpec::Tree(_))));
        if let Some(WidgetSpec::Tree(ref spec)) = el.widget_spec {
            assert_eq!(spec.items.len(), 1);
            assert_eq!(spec.items[0].label, "Project");
            assert!(spec.items[0].expanded);
            assert_eq!(spec.items[0].children[0].label, "src");
        }
    }

    #[test]
    fn phase3_menu() {
        let el = parse_ok(
            r#"
            <Menu key="menu">
                <MenuItem key="archive" on-click="archive" shortcut="Ctrl+A">Archive</MenuItem>
                <MenuItem key="delete" action="delete" disabled>Delete</MenuItem>
            </Menu>
        "#,
        );
        assert!(matches!(el.widget_spec, Some(WidgetSpec::Menu(_))));
        if let Some(WidgetSpec::Menu(ref spec)) = el.widget_spec {
            assert_eq!(spec.items.len(), 2);
            assert_eq!(spec.items[0].label, "Archive");
            assert_eq!(spec.items[0].action.as_deref(), Some("archive"));
            assert_eq!(spec.items[0].shortcut.as_deref(), Some("Ctrl+A"));
            assert!(!spec.items[0].disabled);
            assert_eq!(spec.items[1].action.as_deref(), Some("delete"));
            assert!(spec.items[1].disabled);
        }
        assert_eq!(el.children.len(), 2);
        let item = &el.children[0];
        assert_eq!(
            item.event_handlers.on_click_action.as_deref(),
            Some("archive")
        );
    }

    #[test]
    fn phase3_context_menu_items_populate_menu_spec() {
        let el = parse_ok(
            r#"
            <ContextMenu key="context">
                <MenuItem key="copy" action="copy" shortcut="Ctrl+C">Copy</MenuItem>
            </ContextMenu>
        "#,
        );
        if let Some(WidgetSpec::Menu(ref spec)) = el.widget_spec {
            assert_eq!(spec.items.len(), 1);
            assert_eq!(spec.items[0].label, "Copy");
            assert_eq!(spec.items[0].action.as_deref(), Some("copy"));
            assert_eq!(spec.items[0].shortcut.as_deref(), Some("Ctrl+C"));
        } else {
            panic!("expected context menu to lower as menu spec");
        }
    }

    // ── Phase 4: Overlays ─────────────────────────────────────────────────────

    #[test]
    fn phase4_popover_slot() {
        let el = parse_ok(
            r#"
            <Button key="menu">
                Menu
                <Popover slot="popover" key="menu-popover">
                    <Button key="refresh">Refresh</Button>
                </Popover>
            </Button>
        "#,
        );
        assert!(matches!(el.widget_spec, Some(WidgetSpec::Button(_))));
        // The popover should have been attached via .popover(), not as a child
        assert!(el.overlay.is_some());
        let overlay = el.overlay.as_ref().unwrap();
        assert!(!overlay.open); // trigger popovers open from runtime interaction
        assert!(matches!(overlay.widget_spec, Some(WidgetSpec::Popover(_))));
    }

    #[test]
    fn phase4_context_menu_slot() {
        let el = parse_ok(
            r#"
            <Button key="btn">
                Right-click me
                <ContextMenu slot="context-menu" key="row-menu">
                    <MenuItem key="delete" action="delete">Delete</MenuItem>
                </ContextMenu>
            </Button>
        "#,
        );
        assert!(el.overlay.is_some());
        let overlay = el.overlay.as_ref().unwrap();
        assert!(!overlay.open); // context_menu is open=false
    }

    #[test]
    fn phase4_tooltip() {
        let el = parse_ok(r#"<Tooltip key="help" text="Helpful text" />"#);
        assert!(matches!(el.widget_spec, Some(WidgetSpec::Tooltip(_))));
        if let Some(WidgetSpec::Tooltip(ref spec)) = el.widget_spec {
            assert_eq!(spec.text.as_deref(), Some("Helpful text"));
        }
    }

    #[test]
    fn phase4_modal() {
        let el = parse_ok(
            r#"
            <Modal key="confirm" title="Confirmation" open="false" close-on-escape close-on-outside-click>
                <Column gap="8">
                    <Text>Modal content.</Text>
                    <Button primary>OK</Button>
                </Column>
            </Modal>
        "#,
        );
        assert!(matches!(el.widget_spec, Some(WidgetSpec::Modal(_))));
        if let Some(WidgetSpec::Modal(ref spec)) = el.widget_spec {
            assert_eq!(spec.title.as_deref(), Some("Confirmation"));
            assert!(spec.close_on_escape);
            assert!(spec.close_on_outside_click);
        }
        assert!(!el.open);
    }

    // ── Phase 5: Style coverage ───────────────────────────────────────────────

    #[test]
    fn phase5_style_attributes() {
        let el = parse_ok(
            r##"
            <Column key="panel" width="320" padding="16" gap="8"
                background="#f5f7fa" radius="8" opacity="0.95"
            >
                <Text font-size="24" font-weight="bold">Settings</Text>
            </Column>
        "##,
        );
        assert_eq!(el.style.width, Some(Length::Px(320.0)));
        assert_eq!(el.style.opacity, Some(0.95));
        assert!(el.style.background.is_some());
        assert!(el.style.radius.is_some());
        if let Some(ref r) = el.style.radius {
            assert_eq!(r.top_left, 8.0);
        }

        let text_child = &el.children[0];
        let ts = text_child.style.text.as_ref().unwrap();
        assert_eq!(ts.size, Length::Px(24.0));
        assert_eq!(ts.weight, rgui::FontWeight::Bold);
    }

    #[test]
    fn phase5_shadow_attribute_lowers_to_style_shadow() {
        let el = parse_ok(r#"<Column shadow="0 4 12 0 rgba(15,23,42,0.25)" />"#);
        let shadows = el.style.shadow.expect("shadow should lower");
        assert_eq!(shadows.len(), 1);
        assert_eq!(shadows[0].offset_x, 0.0);
        assert_eq!(shadows[0].offset_y, 4.0);
        assert_eq!(shadows[0].blur, 12.0);
        assert_eq!(shadows[0].spread, 0.0);
    }

    #[test]
    fn phase5_transform_attribute_lowers_to_style_transform() {
        let el = parse_ok(r#"<Column transform="translate(8,12) scale(1.5,2) rotate(0.5)" />"#);
        let transform = el.style.transform.expect("transform should lower");
        assert_eq!(transform.translate_x, 8.0);
        assert_eq!(transform.translate_y, 12.0);
        assert_eq!(transform.scale_x, 1.5);
        assert_eq!(transform.scale_y, 2.0);
        assert_eq!(transform.rotate_radians, 0.5);
    }

    #[test]
    fn phase5_style_attribute_reports_warning() {
        let (_el, warnings) = parse_warn(r#"<Column style="padding: 8px" />"#);
        assert!(
            warnings
                .iter()
                .any(|warning| warning.message.contains("style")),
            "expected style warning, got {warnings:?}"
        );
    }

    #[test]
    fn phase5_duplicate_keys_report_warning() {
        let (_el, warnings) = parse_warn(
            r#"
            <Column>
                <Button key="same">One</Button>
                <Button key="same">Two</Button>
            </Column>
            "#,
        );
        assert!(
            warnings
                .iter()
                .any(|warning| warning.message.contains("duplicate key")),
            "expected duplicate key warning, got {warnings:?}"
        );
    }

    #[test]
    fn phase5_percentage_length() {
        let el = parse_ok(r#"<Column width="50%" height="100%" />"#);
        assert_eq!(el.style.width, Some(Length::Percent(0.5)));
        assert_eq!(el.style.height, Some(Length::Percent(1.0)));
    }

    #[test]
    fn phase5_edge_shorthand_two_values() {
        let el = parse_ok(r#"<Column padding="8 16" />"#);
        let pad = el.style.padding.unwrap();
        assert_eq!(pad.top, Length::Px(8.0));
        assert_eq!(pad.right, Length::Px(16.0));
        assert_eq!(pad.bottom, Length::Px(8.0));
        assert_eq!(pad.left, Length::Px(16.0));
    }

    #[test]
    fn phase5_edge_shorthand_four_values() {
        let el = parse_ok(r#"<Column padding="1 2 3 4" />"#);
        let pad = el.style.padding.unwrap();
        assert_eq!(pad.top, Length::Px(1.0));
        assert_eq!(pad.right, Length::Px(2.0));
        assert_eq!(pad.bottom, Length::Px(3.0));
        assert_eq!(pad.left, Length::Px(4.0));
    }

    // ── Complete MVP example (§12) ─────────────────────────────────────────────

    #[test]
    fn full_mvp_example() {
        let el = parse_ok(
            r#"
            <Column key="settings" padding="16" gap="8" width="320">
                <Text key="title" heading>Settings</Text>

                <Row key="form-row" gap="8" align-items="center">
                    <TextInput key="name" placeholder="Name" default-value="Alice" />
                    <Checkbox key="enabled" checked label="Enabled" />
                    <Button key="save" primary on-click="save">Save</Button>
                </Row>

                <ScrollArea key="content" height="96">
                    <Text key="large-content" height="180">Large content</Text>
                </ScrollArea>

                <Button key="menu">
                    Menu
                    <Popover slot="popover" key="menu-popover">
                        <Column gap="4">
                            <Button key="refresh">Refresh</Button>
                            <Text key="pop-settings">Settings</Text>
                            <Button key="archive">Archive</Button>
                        </Column>
                    </Popover>
                </Button>
            </Column>
        "#,
        );

        // Root
        assert!(matches!(
            el.kind,
            rgui::ElementKind::Primitive(rgui::PrimitiveKind::Column)
        ));
        assert_eq!(el.key.as_ref().map(|k| k.as_str()), Some("settings"));
        assert_eq!(el.style.width, Some(Length::Px(320.0)));

        // Children: title, form-row, scroll-area, menu button
        assert_eq!(el.children.len(), 4);

        // form-row
        let form_row = &el.children[1];
        assert_eq!(form_row.children.len(), 3);

        // menu button has popover overlay
        let menu_btn = &el.children[3];
        assert!(menu_btn.overlay.is_some());
    }

    // ── Error cases ────────────────────────────────────────────────────────────

    #[test]
    fn error_unknown_tag() {
        let err = parse_err(r#"<Foobar />"#);
        assert!(err.message.contains("Foobar"), "message: {}", err.message);
    }

    #[test]
    fn error_missing_canvas_name() {
        let err = parse_err(r#"<Canvas width="100" />"#);
        assert!(err.message.contains("name"), "message: {}", err.message);
    }

    #[test]
    fn error_missing_icon_name() {
        let err = parse_err(r#"<Icon />"#);
        assert!(err.message.contains("name"), "message: {}", err.message);
    }

    #[test]
    fn error_invalid_length() {
        let err = parse_err(r#"<Column width="notanumber" />"#);
        assert!(
            err.message.contains("notanumber") || err.message.contains("length"),
            "message: {}",
            err.message
        );
    }

    #[test]
    fn error_invalid_bool() {
        let err = parse_err(r#"<Button disabled="maybe" />"#);
        assert!(
            err.message.contains("bool") || err.message.contains("maybe"),
            "message: {}",
            err.message
        );
    }

    #[test]
    fn error_multiple_roots() {
        let err = parse_err(r#"<Column /><Row />"#);
        assert!(
            err.message.contains("root") || err.message.contains("2"),
            "message: {}",
            err.message
        );
    }

    #[test]
    fn error_text_with_element_child() {
        let err = parse_err(r#"<Text><Button>Bad</Button></Text>"#);
        assert!(
            err.message.contains("Text") || err.message.contains("child"),
            "message: {}",
            err.message
        );
    }

    #[test]
    fn error_invalid_enum_align() {
        let err = parse_err(r#"<Row align-items="middle" />"#);
        assert!(
            err.message.contains("middle") || err.message.contains("align"),
            "message: {}",
            err.message
        );
    }

    // ── Bool shorthand ────────────────────────────────────────────────────────

    #[test]
    fn bool_shorthand_disabled() {
        let el = parse_ok(r#"<Button disabled>Click</Button>"#);
        if let Some(WidgetSpec::Button(ref spec)) = el.widget_spec {
            assert!(spec.disabled);
        } else {
            panic!("expected Button spec");
        }
    }

    #[test]
    fn bool_explicit_false() {
        let el = parse_ok(r#"<Button disabled="false">Click</Button>"#);
        if let Some(WidgetSpec::Button(ref spec)) = el.widget_spec {
            assert!(!spec.disabled);
        }
    }

    // ── Primitives ────────────────────────────────────────────────────────────

    #[test]
    fn primitive_icon() {
        let el = parse_ok(r#"<Icon key="search-icon" name="search" />"#);
        assert!(matches!(el.widget_spec, Some(WidgetSpec::Icon(_))));
        if let Some(WidgetSpec::Icon(ref spec)) = el.widget_spec {
            assert_eq!(spec.name, "search");
        }
    }

    #[test]
    fn primitive_divider() {
        let el = parse_ok(r#"<Divider key="section-divider" />"#);
        assert!(matches!(el.widget_spec, Some(WidgetSpec::Divider)));
    }

    // ── Warnings ──────────────────────────────────────────────────────────────

    #[test]
    fn warning_unknown_select_child() {
        let (el, warnings) = parse_warn(
            r#"
            <Select>
                <Option value="a">A</Option>
                <SomeUnknown />
            </Select>
        "#,
        );
        assert!(!warnings.is_empty(), "expected a warning for unknown child");
        assert!(matches!(el.widget_spec, Some(WidgetSpec::Select(_))));
    }

    // ── Value parser edge cases ────────────────────────────────────────────────

    #[test]
    fn color_hex_rgb() {
        let el = parse_ok(r##"<Text color="#ff0000">Red</Text>"##);
        let ts = el.style.text.unwrap();
        let c = ts.color;
        // Color::rgb(255,0,0)
        assert_eq!(c, rgui::Color::rgb(255, 0, 0));
    }

    #[test]
    fn color_hex_short() {
        let el = parse_ok(r##"<Text color="#f00">Red</Text>"##);
        let ts = el.style.text.unwrap();
        assert_eq!(ts.color, rgui::Color::rgb(255, 0, 0));
    }

    #[test]
    fn color_rgba_function() {
        let el = parse_ok(r#"<Column background="rgba(255,0,0,0.5)" />"#);
        assert!(el.style.background.is_some());
    }

    #[test]
    fn length_auto() {
        let el = parse_ok(r#"<Column width="auto" />"#);
        assert_eq!(el.style.width, Some(Length::Auto));
    }

    #[test]
    fn length_fr() {
        use rgui::GridTrack;
        let el = parse_ok(r#"<Grid grid-template-columns="1fr 2fr" />"#);
        let cols = el.style.grid_template_columns.unwrap();
        assert_eq!(cols[0], GridTrack::Fraction(1.0));
        assert_eq!(cols[1], GridTrack::Fraction(2.0));
    }
}
