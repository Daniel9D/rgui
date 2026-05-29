const PRODUCTION_FILES: &[&str] = &[
    "src/layout/intrinsic.rs",
    "src/layout/taffy.rs",
    "src/runtime/paint.rs",
    "src/runtime/overlay_pass.rs",
    "src/runtime/portal_pass.rs",
];

const ALLOWED_LITERAL_CONTEXTS: &[&str] = &[
    "0.0",
    "1.0",
    "-1",
    "z_index + 1",
    "z_index + 2",
    "Size::new(0.0, 0.0)",
    "Color::rgb(255, 255, 255)",
    "const DEFAULT_TEXT_COLOR",
    "const DEFAULT_WIDGET_SURFACE",
    "const DEFAULT_WIDGET_BORDER",
    "const PORTAL_WIDGET_BORDER",
    "const PORTAL_WIDGET_SURFACE",
];

#[test]
fn production_layout_paint_and_overlay_do_not_add_raw_visual_policy_literals() {
    let banned_patterns = [
        "Size::new(160.0, 36.0)",
        "Size::new(120.0, 36.0)",
        "Size::new(300.0, 180.0)",
        "Size::new(200.0, 180.0)",
        "Size::new(150.0, 180.0)",
        "Size::new(200.0, 150.0)",
        "Size::new(120.0, 32.0)",
        "font_size.unwrap_or(14.0)",
        ".unwrap_or(14.0)",
        "font_size = 14.0",
        "Color::rgb(20, 23, 28)",
        "Color::rgb(190, 198, 213)",
        "Color::rgb(249, 250, 252)",
        "rect.origin.x + 8.0",
        "rect.origin.y + 18.0",
        "label_len as f32 * 8.0",
        "height.max(48.0)",
        "owner_rect.size.width.max(160.0)",
        "println!(\"DEBUG",
        "paint_portal_children(",
        "let mut y = container.origin.y",
        "y += height +",
        "let height = 24.0",
        "panel_w = 320.0",
        "panel_h = 180.0",
        "portal_element_size(",
        "estimate_element_size(",
        "estimate_overlay_size(",
        "let row_height = 20.0",
        "* 20.0",
        "* 24.0",
        "Size::new(tab_w, 20.0)",
        ".max(48.0)",
        "current_x += tab_w + 8.0",
        "rect.origin.y + 8.0",
        "rect.origin.y + 4.0 +",
        "x + 4.0",
        "rect.origin.x + 4.0",
        "rect.size.width - 8.0",
        "rect.max_x() - 8.0",
        "track_y = rect.origin.y + 4.0",
        "track_h = (rect.size.height - 8.0)",
        "(rect.size.height - 8.0).max(1.0)",
        ".max(20.0).min(track.size.height)",
    ];

    for path in PRODUCTION_FILES {
        let source = std::fs::read_to_string(path).expect("production source reads");
        for pattern in banned_patterns {
            if ALLOWED_LITERAL_CONTEXTS
                .iter()
                .any(|allowed| pattern.contains(allowed))
            {
                continue;
            }
            for (line_index, line) in source.lines().enumerate() {
                if ALLOWED_LITERAL_CONTEXTS
                    .iter()
                    .any(|allowed| line.contains(allowed))
                {
                    continue;
                }
                assert!(
                    !line.contains(pattern),
                    "{path}:{} contains banned visual/layout hardcode: {pattern}",
                    line_index + 1
                );
            }
        }
    }
}

#[test]
fn runtime_scrollbar_geometry_uses_metrics_instead_of_raw_literals() {
    let source = std::fs::read_to_string("src/runtime/runtime.rs").expect("runtime source reads");
    for pattern in [
        "rect.max_x() - 8.0",
        "track_y = rect.origin.y + 4.0",
        "track_h = (rect.size.height - 8.0)",
        "(rect.size.height - 8.0).max(1.0)",
        ".max(20.0).min(track.size.height)",
    ] {
        assert!(
            !source.contains(pattern),
            "runtime scrollbar geometry should come from metrics, found {pattern}"
        );
    }
}

#[test]
fn audit_file_is_kept_as_traceability_input() {
    let audit = std::fs::read_to_string("rgui_taffy_hardcode_audit.md")
        .expect("audit source should remain available");
    assert!(audit.contains("Taffy is integrated"));
    assert!(audit.contains("WidgetMetrics"));
    assert!(audit.contains("Paint pass still contains hardcoded visual geometry"));
}
