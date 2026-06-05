# Writing a Custom Widget

`rgui` ships a fixed set of widget kinds (button, input, list, etc.) and
matches each to a built-in `WidgetPainter` during paint dispatch. If you
need a widget that the built-in set does not provide — a status pill, a
custom progress dial, a domain-specific chart — you can register your
own `WidgetPainter` for any existing `WidgetKind` without modifying the
`rgui` crate.

The extension point lives in `rgui::runtime::paint`. This guide walks
through the full lifecycle: define, register, use, unregister, and test.

## The `WidgetPainter` trait

`WidgetPainter` is the per-widget paint dispatch. The runtime calls it
once per widget per frame, after layout has resolved the widget's
geometry and the theme + style have been resolved.

```rust
pub trait WidgetPainter: Send + Sync {
    fn background_color(&self, ctx: &PaintCtx<'_>) -> Color { ctx.style.background }
    fn has_border(&self) -> bool { true }
    fn paint_content(&self, ctx: &mut PaintCtx<'_>, cmds: &mut Vec<PaintedCommand>) {}
    // Template method `paint(...)` is also on the trait but uses
    // default-impl composition — you only override the hooks above.
}
```

Two supertraits:

- `Send` — your painter may be moved across thread boundaries.
- `Sync` — your painter may be shared (`&dyn WidgetPainter`) across
  threads at the same time.

`rgui` stores registered painters in a process-global `RwLock<HashMap>`
indexed by `WidgetKind`. The lock is held only for the duration of a
single insert / remove; paint dispatch reads through `&'static`, so the
hot path is lock-free.

## Step 1: define your painter

```rust
use rgui::core::{Color, Rect};
use rgui::runtime::paint::{PaintCtx, PaintedCommand};
use rgui::runtime::WidgetPainter;

pub struct StatusPillPainter;

impl WidgetPainter for StatusPillPainter {
    fn background_color(&self, _ctx: &PaintCtx<'_>) -> Color {
        // A solid green background that ignores the resolved style.
        Color::rgb(0, 128, 0)
    }

    fn paint_content(&self, ctx: &mut PaintCtx<'_>, cmds: &mut Vec<PaintedCommand>) {
        // Emit a single rounded-rect command at the painter's
        // background z-layer + 2 (the trait contract requires
        // content commands to start at z + 2 or higher).
        cmds.push(ctx.draw_rect(ctx.rect, Color::rgb(0, 200, 0), 8.0, 0));
    }
}
```

The painter is a plain zero-sized type. It has no constructor because
there is no state to set up — every method is a pure function of the
`PaintCtx` it receives.

## Step 2: register your painter

`register_widget_painter` takes a `&'static dyn WidgetPainter` because
the registry stores pointers into static memory. The simplest way to
satisfy the bound is a `static` binding:

```rust
use rgui::runtime::{register_widget_painter, WidgetKind};

static STATUS_PILL_PAINTER: StatusPillPainter = StatusPillPainter;

fn install() {
    register_widget_painter(WidgetKind::Badge, &STATUS_PILL_PAINTER);
}
```

The registry is process-global. Calling `register_widget_painter` twice
for the same `WidgetKind` replaces the prior painter (and returns it).
There is no `&mut` requirement, so multiple crates can collaborate.

## Step 3: use your painter in an `Element`

`register_widget_painter` overrides paint dispatch for the named
`WidgetKind`. Any element that resolves to that kind — built-in
builders, your own builders, or programmatic `WidgetSpec` — will use
your painter.

```rust
use rgui::widgets::badge;

let _my_widget = badge("online");
```

When the runtime paints `my_widget`, it looks up `WidgetKind::Badge` in
the painter registry, finds `StatusPillPainter`, and calls its hooks
instead of the built-in `BadgePainter`.

## Step 4: unregister on shutdown

The registry lives for the lifetime of the process. If you replace a
built-in painter for a kind that is also used by other parts of the
app, you may want to restore the original on shutdown:

```rust
use rgui::runtime::unregister_widget_painter;

fn uninstall() {
    unregister_widget_painter(WidgetKind::Badge);
}
```

`unregister_widget_painter` returns the painter that was previously
registered, if any. After unregistering, the next paint dispatch falls
back to the built-in painter for that kind.

## Step 5: integration-test your painter

`rgui`'s test infrastructure lets you exercise a painter without a GPU
device. `UiRuntime::update` walks the element tree, resolves theme +
style, and calls your painter's `background_color` / `paint_content`
hooks with a fresh `PaintCtx`. The output is a `DisplayList` you can
inspect:

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use rgui::core::Color;
use rgui::runtime::paint::{PaintCtx, PaintedCommand};
use rgui::runtime::{register_widget_painter, unregister_widget_painter,
                    FrameInput, UiRuntime, WidgetKind, WidgetPainter};
use rgui::widgets::badge;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

struct CountingPainter;

impl WidgetPainter for CountingPainter {
    fn background_color(&self, _ctx: &PaintCtx<'_>) -> Color { Color::rgb(0, 0, 0) }
    fn paint_content(&self, _ctx: &mut PaintCtx<'_>, _cmds: &mut Vec<PaintedCommand>) {
        COUNTER.fetch_add(1, Ordering::SeqCst);
    }
}

static COUNTING_PAINTER: CountingPainter = CountingPainter;

#[test]
fn counting_painter_invoked_once() {
    register_widget_painter(WidgetKind::Badge, &COUNTING_PAINTER);
    COUNTER.store(0, Ordering::SeqCst);

    let mut runtime = UiRuntime::default();
    let _ = runtime.update(FrameInput {
        root: badge("online"),
        ..Default::default()
    });

    assert!(COUNTER.load(Ordering::SeqCst) > 0,
        "CountingPainter should be invoked at least once");

    unregister_widget_painter(WidgetKind::Badge);
}
```

`AtomicUsize` is the simplest way to observe paint calls from outside
the painter — the trait methods take `&self`, so shared atomic state
is the natural side-channel.

## When to write a custom widget

Use the extension point when:

- You need a widget the built-in set does not provide.
- The change is paint-only (no new event dispatch, no new layout
  primitive, no new accessibility role).
- The new widget can be expressed as "different paint for an existing
  `WidgetKind`".

## When NOT to write a custom widget

Reach for a different extension point when:

- You need a new event type (e.g. a new keyboard shortcut or pointer
  gesture). Add a new `UiEvent` variant in the runtime instead.
- You need a new layout primitive (e.g. a `ZStack`). Add a new
  `PrimitiveKind` instead.
- You need a new accessibility role. Add a new `Role` in `rgui::a11y`
  instead.

These three are the natural extension seams. The `WidgetPainter` hook
is the paint-only seam; mixing paint + event + layout changes into a
single custom widget produces a widget that is hard to test and hard
to evolve.

## Reusing a `WidgetKind` vs adding a new variant

The `WidgetKind` enum is `#[non_exhaustive]`, so adding a new variant
from outside the `rgui` crate requires a public `add_variant` API that
does not exist in v1. The workaround is to reuse an existing
`WidgetKind` (e.g. `WidgetKind::Badge` for a "status pill" or
`WidgetKind::Button` for a generic labeled control). If your widget is
fundamentally a new concept and the `WidgetKind` reuse is a stretch,
file an issue describing the widget — the v1.x follow-up may add a
public variant-add API.
