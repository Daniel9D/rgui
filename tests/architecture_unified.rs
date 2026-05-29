#[test]
fn cargo_manifest_defines_one_rgui_crate_without_workspace_members() {
    let manifest = include_str!("../Cargo.toml");

    assert!(manifest.contains("name = \"rgui\""));
    assert!(!manifest.contains("[workspace]"));
    assert!(!manifest.contains("rgui-core"));
    assert!(!manifest.contains("rgui-render-wgpu"));
    assert!(!manifest.contains("rgui-widgets"));
}

#[test]
fn public_examples_import_the_unified_rgui_crate() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for path in [
        "examples/basic_window.rs",
        "examples/render_rects.rs",
        "examples/visual_showcase.rs",
        "examples/widgets.rs",
    ] {
        let source = std::fs::read_to_string(root.join(path)).expect("example source reads");

        assert!(source.contains("use rgui::"), "{path} should import rgui");
        assert!(
            !source.contains("rgui_core"),
            "{path} should not import rgui_core"
        );
        assert!(
            !source.contains("rgui_render_wgpu"),
            "{path} should not import rgui_render_wgpu"
        );
        assert!(
            !source.contains("rgui_widgets"),
            "{path} should not import rgui_widgets"
        );
        assert!(
            !source.contains("rgui_runtime"),
            "{path} should not import rgui_runtime"
        );
    }
}

#[test]
fn manifest_pins_requested_renderer_layout_and_text_versions() {
    let manifest = include_str!("../Cargo.toml");

    assert!(manifest.contains("taffy = \"0.10.1\""));
    assert!(manifest.contains("wgpu = \"29\""));
    assert!(manifest.contains("glyphon = \"0.11.0\""));
}
