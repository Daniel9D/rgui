use rgui::rml::{RmlAttributeStatus, parse, rml_attribute_status};

#[test]
fn disabled_attributes_are_implemented_after_disabled_fix() {
    for tag in [
        "Button",
        "TextInput",
        "Textarea",
        "Checkbox",
        "Radio",
        "Select",
        "Option",
    ] {
        assert_eq!(
            rml_attribute_status(tag, "disabled"),
            RmlAttributeStatus::Implemented,
            "{tag} disabled attribute should be implemented"
        );
    }
}

#[test]
fn modal_dismiss_attrs_are_implemented() {
    assert_eq!(
        rml_attribute_status("Modal", "close-on-escape"),
        RmlAttributeStatus::Implemented
    );
    assert_eq!(
        rml_attribute_status("Modal", "close-on-outside-click"),
        RmlAttributeStatus::Implemented
    );
}

#[test]
fn style_attribute_is_warning_status() {
    assert_eq!(
        rml_attribute_status("Column", "style"),
        RmlAttributeStatus::Warning
    );
}

#[test]
fn parsed_only_attributes_emit_warnings() {
    let parsed = parse(
        r#"<Modal key="dialog" open initial-focus="ok"><Button key="ok">OK</Button></Modal>"#,
    )
    .expect("RML parses");

    assert!(
        parsed
            .warnings
            .iter()
            .any(|warning| warning.message.contains("initial-focus"))
    );
}
