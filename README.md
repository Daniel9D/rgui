# rgui

> ⚗️ **Vibe Coded · Experimental · Don't @ me**

A GPU-accelerated, retained-mode GUI library for Rust — built by vibes, shipped by chaos, and not ready for production (but maybe someday?).

---

## ⚠️ Disclaimer

This is a **vibe-coded experimental project**. That means:

- It was built by following intuition more than specification.
- The API **will** break. Probably often.
- There are dragons 🐉 lurking in the corners of the codebase.
- No stability guarantees. No semver promises. No warranties expressed or implied.
- Issues and PRs are welcome, but expect replies written at 2 AM fueled by curiosity.

Use it to learn, tinker, or get inspired — but **don't ship it to production** unless you're the kind of person who enjoys living dangerously.

---

## What Is This?

`rgui` is an experimental Rust GUI toolkit that renders everything through **wgpu** (a modern, cross-platform GPU API). It aims to be a batteries-included widget library with its own layout engine, text rendering, and even a declarative markup language.

Think of it as a playground for exploring what a Rust-native, GPU-first UI framework could look like.

---

## ✨ Features (so far)

- 🎨 **GPU-accelerated rendering** via [`wgpu`](https://github.com/gfx-rs/wgpu)
- 🪟 **Windowing** via [`winit`](https://github.com/rust-windowing/winit)
- 📐 **Flex/grid layout** powered by [`taffy`](https://github.com/DioxusLabs/taffy)
- ✍️ **Text rendering** with [`glyphon`](https://github.com/grovesNL/glyphon)
- 🧩 **Rich widget set** — buttons, inputs, sliders, checkboxes, modals, popovers, tooltips, tabs, tables, trees, cards, badges, avatars, alerts, spinners, progress bars, selects, and more
- 🖼️ **Image support** via the `images` feature
- 🔣 **SVG support** (experimental, `svg` feature)
- ♿ **Accessibility scaffolding** (`a11y` + `accesskit` features)
- 📝 **RML** — a custom XML-based markup language for declaring UIs declaratively (think JSX but make it Rust)
- 🐛 **Debug tooling** built-in (`debug` feature)
- 📋 **Clipboard** support via [`arboard`](https://github.com/1Password/arboard)
- 🎭 **Vector path rendering** via [`kurbo`](https://github.com/linebender/kurbo)

---

## 🗂️ Project Structure

```
rgui/
├── src/
│   ├── core/          # Core abstractions and types
│   ├── render/        # wgpu rendering pipeline
│   ├── widgets/       # Widget specs and implementations
│   │   ├── spec.rs        # All widget specs (the declarative API)
│   │   ├── primitives.rs  # Box, text, image primitives
│   │   ├── forms.rs       # Input, select, checkbox, radio, slider…
│   │   ├── feedback.rs    # Alert, spinner, progress bar, badge…
│   │   ├── overlays.rs    # Modal, popover, tooltip
│   │   ├── navigation.rs  # Tabs, menu
│   │   ├── collections.rs # Table, list, tree
│   │   └── canvas.rs      # Canvas widget
│   ├── layout/        # Layout engine integration (taffy)
│   ├── text_engine/   # Text shaping & rendering (glyphon)
│   ├── rml/           # RML markup language parser & evaluator
│   ├── runtime/       # Event loop & application runtime
│   ├── state/         # Application state management
│   ├── adapters/      # Platform adapters
│   ├── a11y.rs        # Accessibility support
│   ├── debug.rs       # Debug utilities
│   ├── images.rs      # Image loading
│   └── svg.rs         # SVG rendering (stub)
├── examples/
│   ├── basic_window.rs        # Hello world window
│   ├── widgets.rs             # Widget showcase
│   ├── visual_showcase.rs     # Visual demo
│   ├── rml_showcase.rs        # RML markup demo
│   ├── rml_widget_gallery.rs  # Full widget gallery via RML
│   └── debug_snapshot.rs      # Debug rendering snapshot
└── docs/
```

---

## 🚀 Getting Started

```toml
# Cargo.toml
[dependencies]
rgui = { path = "path/to/rgui" }
```

Run one of the examples to see it in action:

```bash
# Basic window
cargo run --example basic_window

# Widget showcase
cargo run --example widgets

# RML markup showcase (requires rml feature)
cargo run --example rml_showcase --features rml

# Full widget gallery via RML
cargo run --example rml_widget_gallery --features rml
```

---

## 🧱 The Widget System

Widgets are described declaratively using **specs** — plain Rust structs that describe what should be rendered. No macros required (though RML gives you a markup option).

```rust
use rgui::{ButtonSpec, WidgetSpec};

let my_button = WidgetSpec::Button(ButtonSpec {
    label: "Click me".into(),
    ..Default::default()
});
```

---

## 📝 RML — Rust Markup Language

`rgui` ships with an optional XML-based declarative UI format called **RML**, enabled via the `rml` feature. It lets you describe your UI in a familiar tag-based syntax:

```xml
<Button label="Click me" />
<Card>
  <Text>Hello from RML!</Text>
  <Input placeholder="Type here..." />
</Card>
```

---

## 🛠️ Feature Flags

| Feature              | Description                              |
|----------------------|------------------------------------------|
| `text`               | Text rendering support (default on)      |
| `images`             | Image loading & display (default on)     |
| `svg`                | SVG rendering (experimental)             |
| `accessibility`      | Accessibility scaffolding (default on)   |
| `accesskit`          | AccessKit integration                    |
| `rml`                | RML markup language parser               |
| `serde`              | Serialization support for widget specs   |
| `debug`              | Debug utilities & overlays (default on)  |
| `html`               | HTML rendering (planned)                 |
| `tailwind`           | Tailwind-like styling (planned)          |
| `css`                | CSS support (planned)                    |
| `canvas`             | Canvas widget (planned)                  |
| `bitmap-text-fallback` | Bitmap font fallback                   |

---

## 🤷 Why?

Because why not? Sometimes the best way to learn how GUI frameworks work is to build one from scratch, let the vibes guide the architecture, and see what emerges.

This project is less about shipping a polished product and more about exploring the design space of Rust GUI. Every file in here is a question asked through code.

---

## 📜 License

TBD — for now, treat it like it's MIT. Or Apache-2.0. Honestly, just don't be weird about it.

---

<div align="center">

*Built with ✨ vibes ✨, Rust, and a healthy disregard for sleep.*

</div>
