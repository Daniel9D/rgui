use rgui::adapters::{css, html, minimal_css, minimal_html, minimal_tailwind, tailwind};
use rgui::{Display, ElementKind, Length, Overflow, PrimitiveKind};

#[test]
fn html_adapter_outputs_typed_elements_without_runtime_privileges() {
    let element = html::parse_element("<div>Save</div>").unwrap();
    assert!(matches!(
        element.kind,
        ElementKind::Primitive(PrimitiveKind::Column)
            | ElementKind::Primitive(PrimitiveKind::Row)
            | ElementKind::Text(_)
    ));
}

#[test]
fn minimal_html_adapter_keeps_existing_button_smoke_test() {
    let element = minimal_html::parse_element("<button>Save</button>").unwrap();
    assert!(matches!(
        element.kind,
        ElementKind::Widget(rgui::WidgetKind::Button)
    ));
}

#[test]
fn css_adapter_maps_simple_properties_to_style_values() {
    let style = css::css_to_style("padding: 8px; gap: 4px; width: 120px").unwrap();

    assert_eq!(style.padding.as_ref().unwrap().top, Length::Px(8.0));
    assert_eq!(style.gap, Some(Length::Px(4.0)));
    assert_eq!(style.width, Some(Length::Px(120.0)));
}

#[test]
fn css_adapter_maps_overflow_properties_to_style_values() {
    let style =
        css::css_to_style("overflow: hidden; overflow-x: auto; overflow-y: scroll").unwrap();

    assert_eq!(style.overflow_x, Some(Overflow::Auto));
    assert_eq!(style.overflow_y, Some(Overflow::Scroll));

    let visible = css::css_to_style("overflow: visible").unwrap();
    assert_eq!(visible.overflow_x, Some(Overflow::Visible));
    assert_eq!(visible.overflow_y, Some(Overflow::Visible));
}

#[test]
fn minimal_css_adapter_keeps_existing_style_smoke_test() {
    let style = minimal_css::css_to_style("height: 24px").unwrap();
    assert_eq!(style.height, Some(Length::Px(24.0)));
}

#[test]
fn minimal_css_adapter_maps_overflow_properties_to_style_values() {
    let style =
        minimal_css::css_to_style("overflow: clip; overflow-x: hidden; overflow-y: auto")
            .unwrap();

    assert_eq!(style.overflow_x, Some(Overflow::Hidden));
    assert_eq!(style.overflow_y, Some(Overflow::Auto));
}

#[test]
fn tailwind_adapter_maps_layout_classes_to_style_values() {
    let style = tailwind::classes_to_style("flex flex-col gap-2 p-3").unwrap();

    assert_eq!(style.display, Some(Display::Flex));
    assert_eq!(style.gap, Some(Length::Px(8.0)));
    assert_eq!(style.padding.as_ref().unwrap().top, Length::Px(12.0));
}

#[test]
fn tailwind_adapter_maps_overflow_utilities_to_style_values() {
    let style =
        tailwind::classes_to_style("overflow-hidden overflow-x-auto overflow-y-scroll").unwrap();

    assert_eq!(style.overflow_x, Some(Overflow::Auto));
    assert_eq!(style.overflow_y, Some(Overflow::Scroll));

    let visible = tailwind::classes_to_style("overflow-visible").unwrap();
    assert_eq!(visible.overflow_x, Some(Overflow::Visible));
    assert_eq!(visible.overflow_y, Some(Overflow::Visible));

    let scroll = tailwind::classes_to_style("overflow-scroll").unwrap();
    assert_eq!(scroll.overflow_x, Some(Overflow::Scroll));
    assert_eq!(scroll.overflow_y, Some(Overflow::Scroll));
}

#[test]
fn minimal_tailwind_adapter_keeps_existing_layout_smoke_test() {
    let style = minimal_tailwind::classes_to_style("grid gap-4").unwrap();
    assert_eq!(style.display, Some(Display::Grid));
    assert_eq!(style.gap, Some(Length::Px(16.0)));
}

#[test]
fn minimal_tailwind_adapter_maps_every_overflow_utility() {
    let cases = [
        ("overflow-auto", Some(Overflow::Auto), Some(Overflow::Auto)),
        (
            "overflow-hidden",
            Some(Overflow::Hidden),
            Some(Overflow::Hidden),
        ),
        (
            "overflow-visible",
            Some(Overflow::Visible),
            Some(Overflow::Visible),
        ),
        (
            "overflow-scroll",
            Some(Overflow::Scroll),
            Some(Overflow::Scroll),
        ),
        ("overflow-x-auto", Some(Overflow::Auto), None),
        ("overflow-y-auto", None, Some(Overflow::Auto)),
        ("overflow-x-hidden", Some(Overflow::Hidden), None),
        ("overflow-y-hidden", None, Some(Overflow::Hidden)),
        ("overflow-x-visible", Some(Overflow::Visible), None),
        ("overflow-y-visible", None, Some(Overflow::Visible)),
        ("overflow-x-scroll", Some(Overflow::Scroll), None),
        ("overflow-y-scroll", None, Some(Overflow::Scroll)),
    ];

    for (class, expected_x, expected_y) in cases {
        let style = minimal_tailwind::classes_to_style(class).unwrap();
        assert_eq!(style.overflow_x, expected_x, "{class} overflow-x");
        assert_eq!(style.overflow_y, expected_y, "{class} overflow-y");
    }
}
