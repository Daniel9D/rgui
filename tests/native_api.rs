use rgui::widgets::{button, checkbox, text};
use rgui::{Align, Element, Length};

#[test]
fn typed_native_api_builds_layout_and_style_without_html() {
    let element = Element::column()
        .key("settings")
        .padding(16.0)
        .gap(12.0)
        .child(text("Settings").heading())
        .child(
            Element::row()
                .align_center()
                .gap(8.0)
                .child(checkbox().checked(true))
                .child(text("Enable notifications")),
        )
        .child(button("Save").primary());

    assert_eq!(element.children.len(), 3);
    assert_eq!(
        element.style.padding.as_ref().unwrap().top,
        Length::Px(16.0)
    );
    assert_eq!(element.style.gap, Some(Length::Px(12.0)));
    assert_eq!(element.children[1].style.align_items, Some(Align::Center));
}
