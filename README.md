# rgui

`rgui` is an experimental retained-mode GUI library for Rust, built on top of
`wgpu`, `taffy`, and `glyphon`.

It is a playground for a Rust-native, GPU-first UI toolkit: an `Element` tree
goes in, a sorted `DisplayList` comes out, and the runtime keeps layout, paint,
event dispatch, text, and widget state moving together.

## Project Status

This project is intentionally experimental and AI-assisted.

Most of the implementation has been explored and iterated with AI coding
assistance, then tested, reviewed, and shaped into a working Rust codebase. That
makes `rgui` useful as a research project, learning tool, and prototype surface,
but it is not production-ready and should not be treated as a stable GUI
framework yet.

Expect:

- API changes while the design settles.
- incomplete or evolving widgets.
- occasional sharp edges in rendering, layout, and platform integration.
- tests and examples to carry more authority than long-term compatibility
  promises.

In other words: curious, capable, and still wearing a lab coat.

## What It Does

`rgui` provides a retained-mode UI model for desktop and embedded `wgpu`
applications. It focuses on producing a correct GPU-ready display list from a
declarative Rust widget tree, while keeping the public API approachable for
application code and examples.

Core ideas:

- retained `Element` trees instead of immediate-mode drawing calls.
- GPU rendering through `wgpu`.
- flex and grid layout through `taffy`.
- text shaping and rendering through `glyphon`.
- themed widgets with hover, disabled, checked, selected, focused, and active
  states.
- optional RML, an XML-style declarative markup format for UI experiments.

## Features

- GPU-accelerated rendering with `wgpu`.
- Windowing integration with `winit`.
- Flexbox and grid layout through `taffy`.
- Text shaping and rendering with `glyphon`.
- Widget coverage for buttons, inputs, textareas, sliders, checkboxes, radios,
  selects, tabs, menus, tables, lists, trees, cards, badges, avatars, alerts,
  spinners, progress bars, modals, popovers, tooltips, canvas primitives, and
  images.
- Optional RML parser for XML-style declarative UI.
- Accessibility scaffolding through the `accessibility` and `accesskit`
  features.
- Debug snapshots and render validation tests for inspecting runtime output.
- Clipboard support through `arboard`.
- Vector path support through `kurbo`.

## Quick Start

Use the crate locally while it is still experimental:

```toml
[dependencies]
rgui = { path = "path/to/rgui" }
```

Run a small window:

```bash
cargo run --example basic_window
```

Explore the widget showcase:

```bash
cargo run --example widgets
```

Try the RML examples:

```bash
cargo run --example rml_showcase --features rml
cargo run --example rml_widget_gallery --features rml
```

## Rust Widget Example

Widgets are described declaratively using Rust data structures and builders.
Application code creates an element tree; the runtime handles layout, paint, and
events.

```rust
use rgui::{Element, Size};
use rgui::runtime::{FrameInput, UiRuntime};

let root = Element::column()
    .child(Element::text("Hello from rgui"))
    .child(rgui::widgets::button("Click me"));

let mut runtime = UiRuntime::default();
let frame = runtime.update(FrameInput {
    root,
    viewport: Size::new(800.0, 600.0),
    ..Default::default()
});

assert!(frame.display_list.commands().len() > 0);
```

## RML Example

RML is optional and enabled with the `rml` feature. It is useful for demos,
fixtures, and exploring declarative UI syntax without writing Rust widget code
for every screen.

```xml
<Card width="320" padding="16">
  <Text>Hello from RML</Text>
  <Input placeholder="Type here..." />
  <Button label="Submit" />
</Card>
```

## Feature Flags

| Feature | Description |
| --- | --- |
| `text` | Text rendering support. Enabled by default. |
| `images` | Image loading and display. Enabled by default. |
| `svg` | Experimental SVG support. |
| `accessibility` | Accessibility scaffolding. Enabled by default. |
| `accesskit` | AccessKit integration. |
| `serde` | Serialization support for selected data structures. |
| `debug` | Debug utilities and overlays. Enabled by default. |
| `html` | Experimental HTML-oriented adapter surface. |
| `rml` | XML-style RML parser, backed by `quick-xml`. |
| `tailwind` | Tailwind-like adapter surface. |
| `css` | CSS adapter surface. |
| `canvas` | Canvas widget support. |
| `bitmap-text-fallback` | Bitmap text fallback path. |
| `vulkan-goldens` | Vulkan visual-golden test gate. |
| `validation-layers` | Enables expensive `wgpu` validation layers for testing. |

## Repository Layout

```text
rgui/
  src/
    core/          Core geometry, style, render, event, and snapshot types
    render/        wgpu renderer and render-item lowering
    runtime/       Frame runtime, reconciliation, state, paint, and events
    layout/        taffy layout integration
    text_engine/   glyphon-backed text measurement and shaping
    widgets/       Widget specs, builders, and painters
    rml/           RML parser and lowering
    adapters/      Minimal CSS, Tailwind, and HTML-style adapters
  examples/        Runnable examples and showcase apps
  tests/           Contract, integration, renderer, and visual-golden tests
  docs/            Public API notes, plans, and design documentation
```

## Development

Run the default test suite:

```bash
cargo test
```

Run RML-specific tests:

```bash
cargo test --features rml --test rml
cargo test --features rml --test widgets_example_showcase
```

Run visual goldens:

```bash
cargo test --test visual_goldens -j1
```

The project leans heavily on tests because the implementation is still moving.
When behavior changes intentionally, update tests and examples with the same
care as source code.

## Production Readiness

`rgui` is not ready for production use.

It may be useful if you want to study retained-mode UI architecture, experiment
with `wgpu` rendering, prototype Rust-native widgets, or inspect how a GUI
toolkit can turn a declarative tree into layout, paint, accessibility, and
event-dispatch data.

It is a bad fit today if you need semver stability, platform polish, a complete
accessibility story, mature text editing behavior, or long-term API guarantees.

## License

MIT. See [LICENSE](LICENSE).

## A Small Note From The Lab

This codebase started as an AI-assisted experiment and still carries that
spirit: fast iteration, lots of tests, a little weirdness, and a genuine attempt
to learn by building the thing directly.

Use it with curiosity. Trust it only after verification.
