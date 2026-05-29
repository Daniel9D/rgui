use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::{button, canvas, checkbox, divider, icon, input, radio};
use rgui::{Element, PaintCommand, Size};

#[test]
fn widgets_example_is_a_runtime_gallery_for_all_widget_helpers() {
    let source = include_str!("../examples/widgets.rs");
    for expected in [
        "SurfaceRenderer",
        "UiRuntime",
        "FrameInput",
        ".update(",
        ".render(&output.display_list, &output.resources)",
        "button(",
        "input(",
        "checkbox(",
        "radio(",
        "select(",
        "textarea(",
        "tabs(",
        "tree(",
        "table(",
        "list(",
        "modal(",
        "popover(",
        "tooltip(",
        "canvas(",
        "icon(",
        "divider(",
        "text(",
        "Element::grid(",
        "Element::stack(",
        "Element::absolute(",
        "Align::Stretch",
        "CursorMoved",
        "MouseInput",
        "runtime.dispatch",
    ] {
        assert!(
            source.contains(expected),
            "examples/widgets.rs should showcase {expected}"
        );
    }

    for manual_preview_marker in [
        "DisplayList::default(",
        "hit_widget_index",
        "draw_text(",
        "glyph_pattern(",
    ] {
        assert!(
            !source.contains(manual_preview_marker),
            "examples/widgets.rs should use runtime output instead of {manual_preview_marker}"
        );
    }
}

#[test]
fn rml_showcase_is_a_markup_gallery_for_all_supported_tags() {
    let source = include_str!("../examples/rml_showcase.rs");
    for expected in [
        "<ScrollArea",
        "<Column",
        "<Row",
        "<Grid",
        "<Stack",
        "<Absolute",
        "<Text",
        "<Button",
        "<TextInput",
        "<Checkbox",
        "<Radio",
        "<Select",
        "<Option",
        "<Textarea",
        "<Tabs",
        "<Tab",
        "<Tree",
        "<TreeItem",
        "<Table",
        "<TableRow",
        "<List",
        "<Menu",
        "<MenuItem",
        "<ContextMenu",
        "<Popover",
        "<Tooltip",
        "<Modal",
        "<Icon",
        "<Divider",
        "<Canvas",
        "disabled",
        "loading",
        "password",
        "indeterminate",
        "align-items=\"stretch\"",
        "width=\"100%\"",
    ] {
        assert!(
            source.contains(expected),
            "examples/rml_showcase.rs should showcase {expected}"
        );
    }
}

#[test]
fn widgets_example_forwards_scroll_and_secondary_pointer_input() {
    let source = include_str!("../examples/widgets.rs");
    for expected in [
        "WindowEvent::MouseWheel",
        "MouseScrollDelta",
        "WheelEvent",
        "WheelDeltaMode",
        "MouseButton::Right",
    ] {
        assert!(
            source.contains(expected),
            "examples/widgets.rs should wire {expected} into UiRuntime"
        );
    }
}

#[test]
fn rml_showcase_forwards_scroll_and_secondary_pointer_input() {
    let source = include_str!("../examples/rml_showcase.rs");
    for expected in [
        "WindowEvent::MouseWheel",
        "MouseScrollDelta",
        "WheelEvent",
        "WheelDeltaMode",
        "MouseButton::Right",
    ] {
        assert!(
            source.contains(expected),
            "examples/rml_showcase.rs should wire {expected} into UiRuntime"
        );
    }
}

#[test]
fn widgets_showcase_toolbar_children_stay_inside_viewport() {
    let mut runtime = UiRuntime::default();
    let root = Element::row()
        .key("toolbar")
        .gap(8.0)
        .child(button("Save").key("save"))
        .child(input().key("search"))
        .child(checkbox().key("enabled"))
        .child(radio().key("choice"));

    let output = runtime.update(FrameInput {
        root,
        viewport: Size::new(360.0, 120.0),
        ..Default::default()
    });

    let snapshot = output.snapshot.as_ref().expect("snapshot exists");
    for key in ["save", "search", "enabled", "choice"] {
        let layout = snapshot.layout_box(key).expect("widget layout exists");
        assert!(
            layout.x + layout.width <= 360.0,
            "{key} should remain visible in the row"
        );
    }
}

#[test]
fn examples_rely_on_runtime_text_commands_not_manual_placeholder_blocks() {
    for path in [
        "examples/basic_window.rs",
        "examples/visual_showcase.rs",
        "examples/widgets.rs",
    ] {
        let source = std::fs::read_to_string(path).expect("example source reads");
        assert!(
            !source.contains("DrawRect(RectCmd"),
            "{path} should not draw manual text placeholder blocks"
        );
        assert!(
            source.contains("text(") || source.contains("button("),
            "{path} should exercise runtime text-producing APIs"
        );
    }
}

#[test]
fn primitive_helpers_emit_visible_paint_commands() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column()
            .child(icon("search").key("icon"))
            .child(divider().key("divider"))
            .child(canvas().named("chart").build().key("chart")),
        viewport: Size::new(320.0, 200.0),
        ..Default::default()
    });

    let snapshot = output.snapshot.as_ref().expect("snapshot exists");
    for key in ["icon", "divider", "chart"] {
        assert!(
            snapshot.layout_box(key).is_some(),
            "{key} should have layout"
        );
    }

    assert!(
        output.display_list.commands().len() >= 5,
        "icon, divider, and canvas should produce distinguishable paint"
    );
}

#[test]
fn icon_helper_paints_ascii_fallback_symbol() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column().child(icon("search").key("search-icon")),
        viewport: Size::new(120.0, 80.0),
        ..Default::default()
    });

    assert!(
        output
            .display_list
            .commands()
            .iter()
            .any(|command| { matches!(command, PaintCommand::DrawText(cmd) if cmd.text == "S") })
    );
}

#[test]
fn public_api_docs_include_visual_debug_and_full_runtime_examples() {
    let docs = std::fs::read_to_string("docs/public-api.md").expect("public API docs read");

    for expected in [
        "UiRuntime",
        "FrameInput",
        "DisplayList",
        "ResourceStore",
        "RGUI_DUMP_FRAME",
        "RGUI_DUMP_ITEMS",
        "RGUI_DEBUG_VISUAL",
        "button(",
        "checkbox(",
        "popover(",
        "canvas(",
        "divider(",
        "icon(",
    ] {
        assert!(docs.contains(expected), "docs should mention {expected}");
    }
}

#[test]
fn examples_include_debug_snapshot_and_visual_showcase_paths() {
    let debug = std::fs::read_to_string("examples/debug_snapshot.rs").expect("debug example reads");
    let visual =
        std::fs::read_to_string("examples/visual_showcase.rs").expect("visual example reads");
    let widgets = std::fs::read_to_string("examples/widgets.rs").expect("widgets example reads");

    assert!(debug.contains("RGUI_DUMP_FRAME") || debug.contains("debug"));
    assert!(visual.contains("heading()"));
    assert!(widgets.contains("popover("));
    assert!(widgets.contains("checked(true)") || widgets.contains("default_checked(true)"));
}

#[test]
fn examples_prefer_widget_builders_over_manual_specs_for_common_widgets() {
    for path in [
        "examples/widgets.rs",
        "examples/debug_snapshot.rs",
        "examples/visual_showcase.rs",
    ] {
        let source = std::fs::read_to_string(path).expect("example source reads");
        assert!(
            !source.contains("WidgetSpec::Select")
                && !source.contains("WidgetSpec::Tabs")
                && !source.contains("WidgetSpec::Table")
                && !source.contains("WidgetSpec::List"),
            "{path} should use builder-first APIs for common widgets"
        );
    }
}

#[test]
fn public_docs_cover_builder_parts_theme_and_runtime_state() {
    let docs = std::fs::read_to_string("docs/public-api.md").expect("public docs read");
    for expected in [
        "Widget Builders",
        "Runtime-Owned State",
        "Part Styling",
        "Theme Widget Variants",
        "Custom Widgets Future Direction",
        "selected_value",
        "option(\"medium\", \"Medium\")",
    ] {
        assert!(docs.contains(expected), "docs should mention {expected}");
    }
}

#[test]
fn public_docs_cover_widget_metrics_and_hardcode_policy() {
    let docs = std::fs::read_to_string("docs/public-api.md").expect("public docs read");
    for expected in [
        "Widget Metrics",
        "Theme::light().widgets.metrics",
        "Hardcode Policy",
        "visual constants",
        "LayoutDiagnostics",
    ] {
        assert!(docs.contains(expected), "docs should mention {expected}");
    }
}

#[test]
fn public_docs_cover_overlay_taffy_layout_ownership() {
    let docs = std::fs::read_to_string("docs/public-api.md").expect("public docs read");
    for expected in [
        "Overlay Taffy Layout",
        "Taffy owns overlay content size",
        "portal paint",
        "LayoutResult",
        "runtime centers the panel",
    ] {
        assert!(docs.contains(expected), "docs should mention {expected}");
    }
}
