use crate::core::{Size, WidgetKind, WidgetMetrics};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WidgetIntrinsicInput {
    pub widget_kind: WidgetKind,
    pub label_width: Option<f32>,
    pub known_width: Option<f32>,
    pub known_height: Option<f32>,
}

pub fn intrinsic_widget_size(input: WidgetIntrinsicInput, metrics: &WidgetMetrics) -> Size {
    let default = match input.widget_kind {
        WidgetKind::Button => {
            let width = input
                .label_width
                .map(|width| width + metrics.button.horizontal_padding)
                .unwrap_or(metrics.button.min_width);
            Size::new(width.max(metrics.button.min_width), metrics.button.height)
        }
        WidgetKind::Checkbox | WidgetKind::Radio => {
            let control = metrics.input.min_size.height.min(24.0);
            let width = input
                .label_width
                .map(|w| control + 6.0 + w)
                .unwrap_or(control);
            Size::new(width, control)
        }
        WidgetKind::Tabs => metrics.tabs.min_size,
        WidgetKind::Menu => metrics.menu.min_size,
        kind => metrics.min_size_for(kind),
    };

    Size::new(
        input.known_width.unwrap_or(default.width),
        input.known_height.unwrap_or(default.height),
    )
}
