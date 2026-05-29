use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::button;
use rgui::{
    Color, DisplayList, Paint, PaintCommand, PathCmd, Point, Rect, RectCmd, Size, TextCmd, Theme,
};

#[test]
fn display_list_accepts_zero_size_rect_geometry() {
    let mut list = DisplayList::default();
    list.push(PaintCommand::DrawRect(RectCmd {
        rect: Rect::new(Point::new(0.0, 0.0), Size::new(0.0, 10.0)),
        paint: Paint::Solid(Color::rgb(0, 0, 0)),
        radius: 0.0,
        opacity: 1.0,
        z_index: 0,
    }));

    assert_eq!(list.validate(), Ok(()));
}

#[test]
fn display_list_rejects_invalid_rect_geometry() {
    let mut list = DisplayList::default();
    list.push(PaintCommand::DrawRect(RectCmd {
        rect: Rect::new(Point::new(0.0, 0.0), Size::new(-1.0, 10.0)),
        paint: Paint::Solid(Color::rgb(0, 0, 0)),
        radius: 0.0,
        opacity: 1.0,
        z_index: 0,
    }));

    let error = list.validate().expect_err("negative width rect is invalid");
    assert!(error.contains("width"));
}

#[test]
fn display_list_accepts_valid_rect_geometry() {
    let mut list = DisplayList::default();
    list.push(PaintCommand::DrawRect(RectCmd {
        rect: Rect::new(Point::new(0.0, 0.0), Size::new(1.0, 10.0)),
        paint: Paint::Solid(Color::rgb(0, 0, 0)),
        radius: 0.0,
        opacity: 1.0,
        z_index: 0,
    }));

    assert_eq!(list.validate(), Ok(()));
}

#[test]
fn display_list_rejects_invalid_text_geometry() {
    let mut list = DisplayList::default();
    list.push(PaintCommand::DrawText(TextCmd {
        text: "Bad".to_string(),
        rect: Rect::new(Point::new(f32::NAN, 0.0), Size::new(0.0, 17.0)),
        color: Color::rgb(0, 0, 0),
        size: 14.0,
        font_weight: rgui::FontWeight::Normal,
        font_style: rgui::FontStyle::Normal,
        line_height: Some(17.0),
        z_index: 0,
    }));

    assert!(
        list.validate()
            .expect_err("invalid text origin")
            .contains("point")
    );
}

#[test]
fn display_list_rejects_invalid_path_geometry() {
    let mut list = DisplayList::default();
    list.push(PaintCommand::DrawPath(PathCmd {
        points: vec![Point::new(0.0, 0.0), Point::new(f32::INFINITY, 1.0)],
        color: Color::rgb(0, 0, 0),
        width: -1.0,
        z_index: 0,
    }));

    assert!(list.validate().expect_err("invalid path").contains("path"));
}

#[test]
fn button_primary_color_comes_from_theme() {
    let root = button("Save").key("save").primary();
    let mut runtime = UiRuntime::default();

    let light = runtime.update(FrameInput {
        root: root.clone(),
        theme: Theme::light(),
        viewport: Size::new(240.0, 100.0),
        ..Default::default()
    });
    let dark = runtime.update(FrameInput {
        root,
        theme: Theme::dark(),
        viewport: Size::new(240.0, 100.0),
        ..Default::default()
    });

    let light_primary = primary_button_background(light.display_list.commands());
    let dark_primary = primary_button_background(dark.display_list.commands());

    assert_ne!(light_primary, dark_primary);
}

fn primary_button_background(commands: &[PaintCommand]) -> Option<Color> {
    commands.iter().find_map(|command| match command {
        PaintCommand::DrawRect(cmd) if cmd.z_index == 0 => match cmd.paint {
            Paint::Solid(color) => Some(color),
            _ => None,
        },
        _ => None,
    })
}
