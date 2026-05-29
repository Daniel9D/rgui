use rgui::text_engine::TextSystem;
use rgui::{FontStyle, FontWeight, Point};

#[test]
fn caret_hit_testing_uses_glyph_advances_not_linear_projection() {
    let mut text = TextSystem::default();
    let layout = text.measure("Wiiii", 24.0, FontWeight::Normal, FontStyle::Normal, 400.0);
    let origin = Point::new(0.0, 0.0);

    let first_glyph_middle = layout.lines[0].glyph_positions[0].advance_x - 0.1;
    let index = layout.caret_index_for_point(Point::new(first_glyph_middle, 10.0), origin);

    assert_eq!(index, 0);
}
