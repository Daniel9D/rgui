use crate::{
    AlertSpec, Element, ElementKind, ProgressBarSpec, SpinnerSpec, WidgetKind,
    WidgetSpec,
};

/// Creates a progress bar element. Use `.value(0.0..=1.0)` to set
/// the fill fraction and `.max(n)` to change the upper bound.
pub fn progress_bar() -> Element {
    Element::new(ElementKind::Widget(WidgetKind::ProgressBar))
        .widget_spec(WidgetSpec::ProgressBar(ProgressBarSpec::default()))
}

/// Creates a spinner (loading indicator) element. Use
/// `.label("…")` to show hint text alongside the spinner.
pub fn spinner() -> Element {
    Element::new(ElementKind::Widget(WidgetKind::Spinner))
        .widget_spec(WidgetSpec::Spinner(SpinnerSpec::default()))
}

/// Creates an alert / banner element for inline status messages.
///
/// Sizing: by default the alert **hugs its children** (auto-sized as a
/// vertical flex container). To force a fixed size, set an explicit `width()`
/// and/or `height()` on the element.
pub fn alert() -> Element {
    Element::new(ElementKind::Widget(WidgetKind::Alert))
        .widget_spec(WidgetSpec::Alert(AlertSpec::default()))
}
