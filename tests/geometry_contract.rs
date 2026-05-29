use rgui::{Point, Rect, Size, Vec2};

fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect::new(Point::new(x, y), Size::new(width, height))
}

#[test]
fn rect_intersection_returns_overlap() {
    let a = Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 10.0));
    let b = Rect::new(Point::new(4.0, 6.0), Size::new(10.0, 10.0));

    assert_eq!(
        a.intersect(b),
        Some(Rect::new(Point::new(4.0, 6.0), Size::new(6.0, 4.0)))
    );
}

#[test]
fn rect_intersection_returns_none_for_empty_overlap() {
    let a = Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 10.0));
    let b = Rect::new(Point::new(10.0, 0.0), Size::new(4.0, 4.0));

    assert_eq!(a.intersect(b), None);
}

#[test]
fn rect_union_covers_both_inputs() {
    let a = Rect::new(Point::new(2.0, 3.0), Size::new(5.0, 7.0));
    let b = Rect::new(Point::new(-1.0, 6.0), Size::new(2.0, 2.0));

    assert_eq!(
        a.union(b),
        Rect::new(Point::new(-1.0, 3.0), Size::new(8.0, 7.0))
    );
}

#[test]
fn rect_inflate_translation_and_pixel_snapping_are_stable() {
    let rect = Rect::new(Point::new(2.25, 3.75), Size::new(4.5, 5.1));

    assert_eq!(
        rect.inflate(1.0, 2.0),
        Rect::new(Point::new(1.25, 1.75), Size::new(6.5, 9.1))
    );
    assert_eq!(
        rect.translate(Vec2::new(3.0, -1.0)),
        Rect::new(Point::new(5.25, 2.75), Size::new(4.5, 5.1))
    );
    assert_eq!(
        rect.round_to_pixel(),
        Rect::new(Point::new(2.0, 3.0), Size::new(5.0, 6.0))
    );
}

#[test]
fn effective_clip_intersects_stack_with_viewport() {
    let viewport = rect(0.0, 0.0, 100.0, 100.0);
    let stack = vec![rect(10.0, 10.0, 80.0, 80.0), rect(20.0, 0.0, 50.0, 90.0)];

    assert_eq!(
        rgui::effective_clip(&stack, viewport),
        Some(rect(20.0, 10.0, 50.0, 80.0))
    );
}

#[test]
fn scroll_translate_offsets_rect_by_negative_scroll() {
    assert_eq!(
        rgui::scroll_translate(rect(10.0, 20.0, 30.0, 40.0), Vec2::new(3.0, 5.0)),
        rect(7.0, 15.0, 30.0, 40.0)
    );
}

#[test]
fn physical_pixel_snap_rounds_to_device_pixels() {
    assert_eq!(
        rgui::physical_pixel_snap(rect(0.2, 0.2, 10.3, 10.3), 2.0),
        rect(0.0, 0.0, 10.5, 10.5)
    );
}
