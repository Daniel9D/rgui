//! Shared test helpers for the rsgui integration test suite.
//!
//! Currently exports `build_50_widget_ui` (the canonical 50-widget
//! desktop UI used by `tests/frame_budget.rs` for the 8ms CPU-budget
//! regression test, plan 05-04). Other tests can opt-in by declaring
//! `mod common;` in their test file (Rust integration tests
//! auto-include `tests/common/mod.rs` if a `tests/common.rs` shim is
//! added per test file).

use rgui::widgets::{button, checkbox, input, list, text};
use rgui::Element;

/// Builds the canonical 50-widget desktop UI used by the
/// `tests/frame_budget.rs` integration test (plan 05-04, REND-04).
///
/// Composition (root + children):
///   - 1 root `Element::column`
///   - 1 toolbar `Element::row` containing 5 `button`s (1 + 5 = 6)
///   - 1 standalone `input` (= 7)
///   - 1 body `Element::column` containing:
///       - 10 `text` (label) widgets (= 17)
///       - 5 `Element::column` containers, each with 1 `text` and 1
///         `button` inside (5 + 5×2 = 15 more, = 32)
///       - 5 `list`s (= 37)
///       - 5 `checkbox`es (= 42)
///   - 1 footer `Element::row` with 2 `button`s (1 + 2 = 3, = 45)
///   - 5 extra labels at the bottom of the root column to reach 50
///     (= 50)
///
/// The mix mirrors a typical desktop UI: toolbar + search input +
/// scrolling data area (lists) + form controls (checkboxes) +
/// action buttons.
///
/// See `tests/frame_budget.rs` for the consumer.
pub fn build_50_widget_ui() -> Element {
    let mut body = Element::column()
        .key("frame-budget-body")
        .gap(6.0)
        .padding(8.0);

    // 10 text labels
    for i in 0..10 {
        body = body.child(text(format!("Label {i}")).key(format!("label-{i}")));
    }

    // 5 boxes, each with 1 text + 1 button (5 + 10 = 15)
    for i in 0..5 {
        body = body.child(
            Element::column()
                .key(format!("box-{i}"))
                .gap(4.0)
                .child(
                    text(format!("Box {i} title")).key(format!("box-{i}-title")),
                )
                .child(
                    button(format!("Box {i} action"))
                        .key(format!("box-{i}-action")),
                ),
        );
    }

    // 5 lists of 4 rows each
    for i in 0..5 {
        let items: Vec<String> = (0..4).map(|j| format!("L{i} R{j}")).collect();
        body = body.child(
            list()
                .key(format!("list-{i}"))
                .items(items)
                .default_selected_index(0),
        );
    }

    // 5 checkboxes
    for i in 0..5 {
        body = body.child(
            checkbox()
                .key(format!("checkbox-{i}"))
                .checked(i % 2 == 0),
        );
    }

    let toolbar = Element::row()
        .key("frame-budget-toolbar")
        .gap(8.0)
        .child(button("New").key("tb-new"))
        .child(button("Open").key("tb-open"))
        .child(button("Save").key("tb-save"))
        .child(button("Print").key("tb-print"))
        .child(button("Help").key("tb-help"));

    let footer = Element::row()
        .key("frame-budget-footer")
        .gap(8.0)
        .child(button("OK").key("ft-ok"))
        .child(button("Cancel").key("ft-cancel"));

    // 5 extra labels at the root to reach 50 widgets:
    // 1 root + 1 toolbar row + 5 toolbar buttons + 1 input + 1 body
    // column + (10 + 5 + 10 + 5 + 5) body children + 1 footer row + 2
    // footer buttons = 47. Add 3 more labels to reach 50.
    let mut root = Element::column()
        .key("frame-budget-root")
        .gap(10.0)
        .padding(12.0)
        .child(toolbar)
        .child(
            input()
                .key("frame-budget-input")
                .placeholder("Search..."),
        )
        .child(body)
        .child(footer);

    for i in 0..3 {
        root = root.child(text(format!("Footer note {i}")).key(format!("foot-{i}")));
    }

    root
}
