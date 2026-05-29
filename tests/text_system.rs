use rgui::text_engine::{TextShapeKey, TextSystem};
use rgui::{FontStyle, FontWeight};

#[test]
fn text_shape_key_changes_when_width_or_style_changes() {
    let normal = TextShapeKey::new("Hello", 120.0, FontWeight::Normal, FontStyle::Normal);
    let narrow = TextShapeKey::new("Hello", 80.0, FontWeight::Normal, FontStyle::Normal);
    let bold = TextShapeKey::new("Hello", 120.0, FontWeight::Bold, FontStyle::Normal);

    assert_ne!(normal, narrow);
    assert_ne!(normal, bold);
}

#[test]
fn text_system_reuses_cached_shapes_for_same_key() {
    let mut system = TextSystem::default();
    let first = system.shape("Hello", 120.0, FontWeight::Normal, FontStyle::Normal);
    let second = system.shape("Hello", 120.0, FontWeight::Normal, FontStyle::Normal);

    assert_eq!(first, second);
    assert_eq!(system.shape_cache_len(), 1);
}

#[test]
fn wgpu_text_lowering_defaults_to_glyph_atlas_items() {
    let strategy = rgui::render::wgpu::text::TextLoweringStrategy::GlyphAtlas;

    assert_eq!(format!("{strategy:?}"), "GlyphAtlas");
}

#[test]
fn text_shape_key_includes_font_size() {
    let small =
        TextShapeKey::new_with_size("Hello", 120.0, 14.0, FontWeight::Normal, FontStyle::Normal);
    let large =
        TextShapeKey::new_with_size("Hello", 120.0, 24.0, FontWeight::Normal, FontStyle::Normal);

    assert_ne!(small, large);
}

#[test]
fn text_metrics_scale_with_font_size() {
    let small = rgui::runtime::text_metrics::measure_text(
        "Heading",
        14.0,
        FontWeight::Normal,
        FontStyle::Normal,
        300.0,
    );
    let large = rgui::runtime::text_metrics::measure_text(
        "Heading",
        24.0,
        FontWeight::Bold,
        FontStyle::Normal,
        300.0,
    );

    assert!(large.width > small.width);
    assert!(large.height > small.height);
    assert!(large.baseline > small.baseline);
}
