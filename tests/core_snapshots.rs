use rgui::{
    AccessibilityMetrics, EventTraceSnapshot, HitTestSnapshot, LayoutBoxSnapshot, OverlaySnapshot,
    PaintCommandSnapshot, PerformanceMetrics, ResolvedStyleSnapshot, SemanticSnapshot, UiSnapshot,
};
use rgui::{AxisSet, GridTrack, LayoutBox, Length, ScrollState, Vec2};
use rgui::{
    ClipSpec, Color, DisplayList, ImageId, LayerKind, LayerSpec, Paint, PaintCommand, RectCmd,
    RenderStats, RendererBackend, ResourceStore, SizeU32,
};
use rgui::{Component, ComponentCx, Edge, Element, ElementKind};
use rgui::{
    ComponentTheme, ComponentThemeMap, Theme, ThemeMode, ThemeScope, VariantId, WidgetKind,
};
use rgui::{DefaultStyleMode, Display, StateFlags, Style, StyleResolver};
use rgui::{EventResult, FocusManager, Shortcut, ShortcutRegistry, ShortcutScope};
use rgui::{HitTestEntry, HitTestTree, OverlayManager, OverlaySpec};
use rgui::{KeyboardNav, Role, SemanticAction, SemanticNode, SemanticStates, SemanticTree};
use rgui::{NodeId, Point, Rect, Size};
use rgui::{TextInputState, TextPosition, TextRange, TextSelection};

#[test]
fn geometry_and_ids_are_stable() {
    let rect = Rect::new(Point::new(10.0, 20.0), Size::new(100.0, 50.0));

    assert_eq!(NodeId::from_raw(7).raw(), 7);
    assert_eq!(rect.max_x(), 110.0);
    assert_eq!(rect.max_y(), 70.0);
    assert!(rect.contains(Point::new(30.0, 40.0)));
    assert!(!rect.contains(Point::new(111.0, 40.0)));
}

#[test]
fn display_list_tracks_balanced_layers_and_clips() {
    let mut list = DisplayList::default();
    list.push(PaintCommand::PushLayer(LayerSpec::new(LayerKind::Document)));
    list.push(PaintCommand::PushClip(ClipSpec::rect(Rect::new(
        Point::new(0.0, 0.0),
        Size::new(50.0, 50.0),
    ))));
    list.push(PaintCommand::DrawRect(RectCmd {
        rect: Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 10.0)),
        paint: Paint::Solid(Color::rgb(255, 0, 0)),
        radius: 0.0,
        opacity: 1.0,
        z_index: 0,
    }));
    list.push(PaintCommand::PopClip);
    list.push(PaintCommand::PopLayer);

    assert_eq!(list.validate(), Ok(()));
    assert_eq!(list.commands().len(), 5);
}

#[test]
fn display_list_rejects_unbalanced_clips() {
    let mut list = DisplayList::default();
    list.push(PaintCommand::PopClip);

    assert!(list.validate().is_err());
}

#[test]
fn ui_snapshot_debug_json_reports_measure_and_stats() {
    let mut snapshot = UiSnapshot::default();
    snapshot.measure.push(rgui::MeasureSnapshot {
        node: NodeId::from_raw(1),
        key: Some("title".to_string()),
        preferred_width: 48.0,
        preferred_height: 20.0,
        content_width: 48.0,
        content_height: 20.0,
    });
    snapshot.performance.display_command_count = 3;

    let json = snapshot.to_debug_json();

    assert!(json.contains("\"measure\":1"));
    assert!(json.contains("\"stats\""));
    assert!(json.contains("\"display_command_count\":3"));
}

struct CountingRenderer;

impl RendererBackend for CountingRenderer {
    fn resize(&mut self, _size: SizeU32) {}

    fn render(&mut self, display_list: &DisplayList, _resources: &ResourceStore) -> RenderStats {
        RenderStats {
            command_count: display_list.commands().len(),
            batch_count: 1,
            atlas_upload_bytes: 0,
            render_item_count: 0,
            text_item_count: 0,
            clip_batch_count: 0,
            glyphon_enabled: false,
            text_area_count: 0,
            clipped_text_area_count: 0,
            skipped_text_area_count: 0,
            glyph_count: 0,
            fallback_used: false,
        }
    }
}

#[test]
fn renderer_backend_consumes_display_list_only() {
    let mut renderer = CountingRenderer;
    let mut list = DisplayList::default();
    list.push(PaintCommand::DrawImage(rgui::ImageCmd {
        id: ImageId::from_raw(2),
        rect: Rect::new(Point::new(0.0, 0.0), Size::new(20.0, 20.0)),
        opacity: 1.0,
        z_index: 3,
    }));

    let stats = renderer.render(&list, &ResourceStore::default());

    assert_eq!(stats.command_count, 1);
}

#[test]
fn typed_lengths_convert_to_css_like_values_without_strings() {
    assert_eq!(Length::Px(12.0).resolve(200.0), Some(12.0));
    assert_eq!(Length::Percent(0.5).resolve(200.0), Some(100.0));
    assert_eq!(Length::Auto.resolve(200.0), None);
    assert_eq!(GridTrack::fr(2.0).fraction(), Some(2.0));
}

#[test]
fn layout_box_separates_viewport_and_content_size() {
    let layout = LayoutBox {
        node: NodeId::from_raw(1),
        key: None,
        local_rect: Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 40.0)),
        world_rect: Rect::new(Point::new(5.0, 5.0), Size::new(100.0, 40.0)),
        content_size: Size::new(100.0, 300.0),
        padding_rect: Rect::new(Point::new(5.0, 5.0), Size::new(100.0, 40.0)),
        content_rect: Rect::new(Point::new(5.0, 5.0), Size::new(100.0, 40.0)),
        clip_rect: Some(Rect::new(Point::new(5.0, 5.0), Size::new(100.0, 40.0))),
        scroll_offset: Vec2::new(0.0, 0.0),
        z_index: 0,
    };

    assert_eq!(layout.viewport_size(), Size::new(100.0, 40.0));
    assert_eq!(layout.content_size.height, 300.0);
}

#[test]
fn scroll_state_clamps_offsets() {
    let mut state = ScrollState::new(AxisSet::vertical());
    state.viewport_size = Size::new(100.0, 50.0);
    state.content_size = Size::new(100.0, 200.0);

    state.scroll_by(Vec2::new(0.0, 500.0));
    assert_eq!(state.offset.y, 150.0);

    state.scroll_by(Vec2::new(0.0, -600.0));
    assert_eq!(state.offset.y, 0.0);
}

#[test]
fn style_resolution_order_uses_last_writer_for_option_fields() {
    let resolver = StyleResolver::new(DefaultStyleMode::Full);
    let base = Style::default().display(Display::Flex).opacity(0.5);
    let local = Style::default().opacity(0.75).z_index(9);

    let resolved = resolver.resolve_layers([base, local]);

    assert_eq!(resolved.display, Display::Flex);
    assert_eq!(resolved.opacity, 0.75);
    assert_eq!(resolved.z_index, 9);
}

#[test]
fn state_flags_are_compact_and_composable() {
    let flags = StateFlags::HOVER | StateFlags::FOCUS | StateFlags::INVALID;

    assert!(flags.contains(StateFlags::HOVER));
    assert!(flags.contains(StateFlags::FOCUS));
    assert!(!flags.contains(StateFlags::DISABLED));
}

#[test]
fn built_in_themes_support_light_and_dark_modes() {
    let light = Theme::light();
    let dark = Theme::dark();

    assert_eq!(light.mode, ThemeMode::Light);
    assert_eq!(dark.mode, ThemeMode::Dark);
    assert_ne!(light.colors.background, dark.colors.background);
}

#[test]
fn local_theme_scope_overrides_tokens_without_strings() {
    let theme = Theme::light();
    let scoped = ThemeScope::new(theme).with_primary(Color::rgb(80, 120, 255));

    assert_eq!(scoped.theme().colors.primary, Color::rgb(80, 120, 255));
}

#[test]
fn component_theme_resolves_variant_style() {
    let mut map = ComponentThemeMap::default();
    let variant = VariantId::new("primary");
    map.insert(
        WidgetKind::Button,
        ComponentTheme::default().with_variant(variant.clone(), Style::default().z_index(5)),
    );

    let style = map
        .get(WidgetKind::Button)
        .and_then(|theme| theme.variant(&variant))
        .cloned()
        .expect("button primary variant exists");

    assert_eq!(style.z_index, Some(5));
}

#[test]
fn event_result_tracks_stop_and_prevent_default() {
    let result = EventResult::handled().stop_propagation().prevent_default();

    assert!(result.handled);
    assert!(result.stop_propagation);
    assert!(result.prevent_default);
}

#[test]
fn focus_manager_tracks_one_node_per_window() {
    let mut focus = FocusManager::default();
    focus.request_focus(NodeId::from_raw(10));

    assert_eq!(focus.focused(), Some(NodeId::from_raw(10)));
}

#[test]
fn shortcuts_resolve_local_scope_before_window_scope() {
    let mut registry = ShortcutRegistry::default();
    registry.register(Shortcut::new(
        "Ctrl+S",
        ShortcutScope::Window,
        "window-save",
    ));
    registry.register(Shortcut::new(
        "Ctrl+S",
        ShortcutScope::FocusedNode(NodeId::from_raw(2)),
        "node-save",
    ));

    let action = registry
        .resolve("Ctrl+S", Some(NodeId::from_raw(2)))
        .expect("shortcut resolves");

    assert_eq!(action, "node-save");
}

#[test]
fn typed_builders_create_retained_element_trees() {
    let tree = Element::column()
        .key("account-form")
        .style(Style::default().padding_edge(Edge::all(Length::Px(16.0))))
        .child(Element::text("Account"))
        .child(Element::row().child(Element::text("Email")));

    assert_eq!(tree.children.len(), 2);
    assert_eq!(
        tree.key.as_ref().map(|key| key.as_str()),
        Some("account-form")
    );
}

struct TitleComponent;

impl Component for TitleComponent {
    fn render(&self, _cx: &mut ComponentCx) -> Element {
        Element::text("Title")
    }
}

#[test]
fn composed_components_render_to_elements() {
    let mut cx = ComponentCx::new(NodeId::from_raw(1), Theme::light());
    let element = TitleComponent.render(&mut cx);

    assert!(matches!(element.kind, ElementKind::Text(_)));
}

#[test]
fn text_input_state_applies_ime_commit_at_selection() {
    let mut state = TextInputState::new("ac");
    state.selection = TextSelection::caret(TextPosition::new(1));
    state.commit_text("b");

    assert_eq!(state.text, "abc");
    assert_eq!(state.selection.head.byte_offset, 2);
}

#[test]
fn text_selection_normalizes_anchor_and_head() {
    let selection = TextSelection {
        anchor: TextPosition::new(8),
        head: TextPosition::new(3),
    };

    assert_eq!(selection.range(), TextRange::new(3, 8));
}

#[test]
fn hit_testing_prefers_topmost_overlay_entry() {
    let mut tree = HitTestTree::default();
    tree.push(HitTestEntry::new(
        NodeId::from_raw(1),
        Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 100.0)),
        0,
        LayerKind::Document,
    ));
    tree.push(HitTestEntry::new(
        NodeId::from_raw(2),
        Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 100.0)),
        0,
        LayerKind::Popover,
    ));

    assert_eq!(
        tree.hit_test(Point::new(5.0, 5.0)),
        Some(NodeId::from_raw(2))
    );
}

#[test]
fn overlay_manager_orders_modal_above_popover() {
    let mut manager = OverlayManager::default();
    manager.register(OverlaySpec::new(NodeId::from_raw(1), LayerKind::Popover));
    manager.register(OverlaySpec::new(NodeId::from_raw(2), LayerKind::Modal));

    let ordered = manager.ordered();

    assert_eq!(
        ordered.last().map(|spec| spec.owner),
        Some(NodeId::from_raw(2))
    );
}

#[test]
fn semantic_tree_preserves_role_label_state_and_bounds() {
    let mut tree = SemanticTree::default();
    tree.push(SemanticNode {
        node: NodeId::from_raw(1),
        key: Some("save".to_string()),
        role: Role::Button,
        label: Some("Save".to_string()),
        description: None,
        value: None,
        states: SemanticStates {
            focused: true,
            disabled: false,
            checked: false,
            expanded: None,
        },
        actions: vec![SemanticAction::Press],
        focusable: true,
        focus_order: Some(1),
        keyboard_navigation: KeyboardNav::TabStop,
        bounds: Rect::new(Point::new(0.0, 0.0), Size::new(80.0, 32.0)),
    });

    assert_eq!(tree.nodes()[0].role, Role::Button);
    assert_eq!(tree.nodes()[0].label.as_deref(), Some("Save"));
    assert!(tree.nodes()[0].states.focused);
}

#[test]
fn semantic_roles_and_actions_map_to_public_accessibility_names() {
    assert_eq!(rgui::a11y::role_to_str(Role::Button), "button");
    assert_eq!(rgui::a11y::role_to_str(Role::TextInput), "text-input");
    assert_eq!(rgui::a11y::action_to_str(SemanticAction::Press), "press");
    assert_eq!(
        rgui::a11y::action_to_str(SemanticAction::SetValue),
        "set-value"
    );
}

#[test]
fn semantic_actions_map_to_ui_commands() {
    let node = SemanticNode {
        node: NodeId::from_raw(7),
        key: Some("save".to_string()),
        role: Role::Button,
        label: Some("Save".to_string()),
        description: None,
        value: None,
        states: SemanticStates::default(),
        actions: vec![SemanticAction::Press],
        focusable: true,
        focus_order: None,
        keyboard_navigation: KeyboardNav::TabStop,
        bounds: Rect::new(Point::new(0.0, 0.0), Size::new(80.0, 32.0)),
    };

    assert!(matches!(
        rgui::a11y::command_for_action(&node, SemanticAction::Press, None),
        Some(rgui::runtime::UiCommand::Click { key: Some(key), .. }) if key == "save"
    ));
    assert_eq!(
        rgui::a11y::command_for_action(&node, SemanticAction::Focus, None),
        Some(rgui::runtime::UiCommand::Focus {
            key: "save".to_string()
        })
    );
}

#[test]
fn ui_snapshot_collects_all_regression_surfaces() {
    let snapshot = UiSnapshot {
        tree_nodes: vec!["root".to_string()],
        styles: vec![ResolvedStyleSnapshot {
            node: NodeId::from_raw(1),
            z_index: 0,
        }],
        measure: vec![rgui::MeasureSnapshot {
            node: NodeId::from_raw(1),
            key: Some("root".to_string()),
            preferred_width: 10.0,
            preferred_height: 10.0,
            content_width: 10.0,
            content_height: 10.0,
        }],
        layout: vec![LayoutBoxSnapshot {
            node: NodeId::from_raw(1),
            key: Some("root".to_string()),
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
            content_width: 10.0,
            content_height: 10.0,
            clip_rect: None,
        }],
        display_list: vec![PaintCommandSnapshot {
            kind: "DrawRect".to_string(),
            z_index: 0,
        }],
        semantics: vec![SemanticSnapshot {
            node: NodeId::from_raw(1),
            role: "Button".to_string(),
            label: Some("Save".to_string()),
        }],
        events: vec![EventTraceSnapshot {
            node: NodeId::from_raw(1),
            phase: "Target".to_string(),
            event: "PointerDown".to_string(),
        }],
        overlays: vec![OverlaySnapshot {
            key: Some("menu".to_string()),
            layer: LayerKind::Popover,
            rect: Rect::new(Point::new(0.0, 40.0), Size::new(120.0, 80.0)),
            modal: false,
        }],
        hit_test_entries: vec![HitTestSnapshot {
            node: NodeId::from_raw(1),
            key: Some("save".to_string()),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 40.0,
            z_index: 0,
            layer: "Document".to_string(),
        }],
        layout_debug: Default::default(),
        performance: PerformanceMetrics::default(),
        diagnostics: Default::default(),
    };

    assert_eq!(snapshot.display_list[0].kind, "DrawRect");
    assert_eq!(snapshot.semantics[0].label.as_deref(), Some("Save"));
}

#[test]
fn performance_metrics_track_required_mvp_numbers() {
    let metrics = PerformanceMetrics {
        frame_time_ms: 12.0,
        node_count: 5000,
        style_cache_hit_rate: 0.95,
        layout_recompute_count: 42,
        display_command_count: 250,
        batch_count: 8,
        atlas_upload_bytes: 4096,
        atlas_eviction_count: 0,
        text_shape_cache_hit_rate: 0.90,
        hit_test_time_ms: 0.2,
        accessibility: AccessibilityMetrics {
            semantic_node_count: 120,
            accesskit_update_count: 1,
        },
    };

    assert!(metrics.frame_time_ms <= 16.7);
    assert!(metrics.style_cache_hit_rate >= 0.90);
}

#[test]
fn accessibility_backend_sync_with_ui_runtime() {
    use rgui::runtime::{FrameInput, UiRuntime};
    use rgui::widgets::button;

    let mut runtime = UiRuntime::default();
    let backend = rgui::a11y::RealAccessibilityBackend::new();
    runtime.a11y_backend = Some(Box::new(backend));

    let output = runtime.update(FrameInput {
        root: Element::column().child(button("Submit").key("submit-btn")),
        ..Default::default()
    });

    assert_eq!(
        output
            .snapshot
            .as_ref()
            .unwrap()
            .performance
            .accessibility
            .accesskit_update_count,
        1
    );
    assert_eq!(
        output
            .snapshot
            .as_ref()
            .unwrap()
            .performance
            .accessibility
            .semantic_node_count,
        2
    ); // column + button
}

#[test]
fn debug_snapshot_derives_hit_entries_from_frame_hit_tree() {
    let mut runtime = rgui::runtime::UiRuntime::default();
    let frame = runtime.update(rgui::runtime::FrameInput {
        root: rgui::widgets::button("A").key("a"),
        viewport: rgui::Size::new(120.0, 60.0),
        ..Default::default()
    });
    let snapshot = frame.debug_snapshot();
    assert_eq!(
        snapshot.hit_test_entries.len(),
        frame.hit_test.entries().len()
    );
}
