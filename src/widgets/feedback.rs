use crate::{
    AlertSpec, Element, ElementKind, ProgressBarSpec, SpinnerSpec, WidgetKind,
    WidgetSpec,
};

/// Creates a progress bar element.
pub fn progress_bar() -> Element {
    Element::new(ElementKind::Widget(WidgetKind::ProgressBar))
        .widget_spec(WidgetSpec::ProgressBar(ProgressBarSpec::default()))
}

/// Creates a spinner (loading indicator) element.
pub fn spinner() -> Element {
    Element::new(ElementKind::Widget(WidgetKind::Spinner))
        .widget_spec(WidgetSpec::Spinner(SpinnerSpec::default()))
}

/// Creates an alert / banner element for inline status messages.
pub fn alert() -> Element {
    Element::new(ElementKind::Widget(WidgetKind::Alert))
        .widget_spec(WidgetSpec::Alert(AlertSpec::default()))
}
