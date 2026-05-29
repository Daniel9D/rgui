use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::{
    alert, avatar, badge, button, card, icon, image, input, link, progress_bar, slider, spinner,
    switch, text,
};
use rgui::{Size, WidgetSpec};

#[test]
fn new_widgets_batch1_are_runtime_compatible() {
    let root = rgui::Element::column()
        .child(image("logo.png").alt("Company logo").key("img"))
        .child(switch().label("Enable feature").key("sw"))
        .child(slider().key("sl"))
        .child(progress_bar().key("pb"))
        .child(spinner().key("spin"))
        .child(badge("New").key("badge"))
        .child(avatar().key("av"))
        .child(link("Documentation").key("lnk"))
        .child(alert().key("alert"))
        .child(card().key("card").child(text("Card content")))
        .child(button("Submit").key("btn"))
        .child(icon("check").key("icn"))
        .child(input().key("inp"));

    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root,
        viewport: Size::new(640.0, 480.0),
        ..Default::default()
    });

    // All new widgets should produce at least one display item
    assert!(
        !output.display_list.commands().is_empty(),
        "new widgets should produce display commands"
    );

    // Verify specs are attached correctly
    let img = output
        .semantics
        .by_key("img")
        .expect("image should be in semantics");
    assert_eq!(img.role, rgui::core::Role::Image);
}

#[test]
fn image_spec_stores_src_and_alt() {
    let el = image("avatar.png").alt("User avatar");
    let Some(WidgetSpec::Image(spec)) = el.widget_spec else {
        panic!("expected ImageSpec");
    };
    assert_eq!(spec.src, Some("avatar.png".to_string()));
    assert_eq!(spec.alt, Some("User avatar".to_string()));
}

#[test]
fn badge_spec_stores_text() {
    let el = badge("Beta");
    let Some(WidgetSpec::Badge(spec)) = el.widget_spec else {
        panic!("expected BadgeSpec");
    };
    assert_eq!(spec.text, "Beta");
}

#[test]
fn switch_spec_is_default_unchecked() {
    let el = switch();
    let Some(WidgetSpec::Switch(spec)) = el.widget_spec else {
        panic!("expected SwitchSpec");
    };
    assert!(!spec.checked);
    assert!(!spec.disabled);
}

#[test]
fn slider_spec_has_sensible_defaults() {
    let el = slider();
    let Some(WidgetSpec::Slider(spec)) = el.widget_spec else {
        panic!("expected SliderSpec");
    };
    assert_eq!(spec.min, 0.0);
    assert_eq!(spec.max, 0.0);
    assert_eq!(spec.value, 0.0);
    assert!(spec.step.is_none());
}

#[test]
fn progress_bar_spec_defaults() {
    let el = progress_bar();
    let Some(WidgetSpec::ProgressBar(spec)) = el.widget_spec else {
        panic!("expected ProgressBarSpec");
    };
    assert_eq!(spec.value, 0.0);
    assert_eq!(spec.max, 0.0);
    assert!(!spec.indeterminate);
}

#[test]
fn card_and_alert_are_containers() {
    let card_el = card().child(text("Inside card"));
    assert_eq!(card_el.children.len(), 1);

    let alert_el = alert().child(text("Inside alert"));
    assert_eq!(alert_el.children.len(), 1);
}
