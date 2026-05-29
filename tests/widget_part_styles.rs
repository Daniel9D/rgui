use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::{option, select};
use rgui::{Color, Edge, FontWeight, Length, PaintCommand, Size, Style, Theme};

#[test]
fn style_new_supports_numeric_widget_part_helpers() {
    let style = Style::new()
        .width(220.0)
        .height(32.0)
        .padding(8.0)
        .font_weight(FontWeight::Bold)
        .background(Color::rgb(10, 20, 30));

    assert_eq!(style.width, Some(Length::Px(220.0)));
    assert_eq!(style.height, Some(Length::Px(32.0)));
    assert_eq!(style.padding, Some(Edge::all(Length::Px(8.0))));
    assert_eq!(
        style.text.as_ref().map(|text| text.weight),
        Some(FontWeight::Bold)
    );
    assert!(style.background.is_some());
}

#[test]
fn style_merge_over_uses_last_writer_for_option_fields() {
    let base = Style::new().width(120.0).height(24.0);
    let override_style = Style::new().height(36.0);
    let merged = base.merge_over(override_style);

    assert_eq!(merged.width, Some(Length::Px(120.0)));
    assert_eq!(merged.height, Some(Length::Px(36.0)));
}

#[test]
fn local_select_part_styles_affect_trigger_and_selected_item() {
    let mut runtime = UiRuntime::default();
    let root = select()
        .key("priority")
        .options([option("low", "Low"), option("medium", "Medium")])
        .default_value("medium")
        .styles(|s| {
            s.trigger(Style::new().height(40.0));
            s.item_selected(Style::new().font_weight(FontWeight::Bold));
        });

    let output = runtime.update(FrameInput {
        root,
        viewport: Size::new(240.0, 120.0),
        ..Default::default()
    });

    let trigger = output
        .snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.layout_box("priority"))
        .expect("select layout");
    assert_eq!(trigger.height, 40.0);

    assert!(output.display_list.commands().iter().any(|command| {
        matches!(
            command,
            PaintCommand::DrawText(cmd)
                if cmd.text == "Medium" && cmd.font_weight == FontWeight::Bold
        )
    }));
}

#[test]
fn select_theme_variant_applies_part_styles_and_local_styles_win() {
    let mut theme = Theme::light();
    theme.widgets.select.variant("priority", |v| {
        v.trigger(
            Style::new()
                .height(36.0)
                .background(Color::rgb(200, 210, 220)),
        );
        v.item_selected(Style::new().font_weight(FontWeight::Bold));
    });

    let root = select()
        .key("priority")
        .variant("priority")
        .options([option("medium", "Medium")])
        .default_value("medium")
        .styles(|s| {
            s.trigger(Style::new().height(44.0));
        });

    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root,
        viewport: Size::new(240.0, 120.0),
        theme,
        ..Default::default()
    });

    let trigger = output
        .snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.layout_box("priority"))
        .expect("select layout");
    assert_eq!(trigger.height, 44.0);

    assert!(output.display_list.commands().iter().any(|command| {
        matches!(
            command,
            PaintCommand::DrawText(cmd)
                if cmd.text == "Medium" && cmd.font_weight == FontWeight::Bold
        )
    }));
}
