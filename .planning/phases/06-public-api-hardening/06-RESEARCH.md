# Phase 6: Public API Hardening - Research

**Gathered:** 2026-06-04
**Status:** Ready for planning

## Domain Investigation

### Crate-root public API surface

The `rgui` crate root (`src/lib.rs`) exposes the following public API:

**Direct public modules** (`pub mod ...`):
- `a11y` — accessibility primitives
- `adapters` — adapter layer (e.g., serde)
- `core` — `Element`, `DisplayList`, `RenderStats`, `UiSnapshot`, etc.
- `debug` — debug dump formatters
- `images` — image widget support
- `layout` — layout engine
- `render` — wgpu renderer
- `runtime` — `UiRuntime`, `FrameInput`, `FrameOutput`
- `state` — runtime state arena
- `svg` — SVG support
- `text_engine` — text shaping + measurement
- `widgets` — widget builders (`button()`, `input()`, `list()`, `scroll_area()`, etc.)

**Explicit re-exports** (`pub use widgets::spec::{...}`):
- 30+ widget spec types: `AlertSpec`, `AvatarSpec`, `BadgeSpec`, `ButtonSpec`, `CardSpec`, `CheckboxSpec`, `IconSpec`, `ImageSpec`, `InputSpec`, `LinkSpec`, `ListSpec`, `MenuSpec`, `MenuItemSpec`, `ModalSpec`, `PopoverSpec`, `ProgressBarSpec`, `RadioSpec`, `SelectOption`, `SelectPartStyles`, `SelectSpec`, `SliderSpec`, `SpinnerSpec`, `SwitchSpec`, `TableSpec`, `TabsSpec`, `TextareaSpec`, `TooltipSpec`, `TreeSpec`, `TreeItemSpec`, `WidgetSpec`, plus variants (`AlertVariant`, `BadgeVariant`, `AvatarSize`, `ImageFit`).

**Hidden re-exports** (`#[doc(hidden)] pub use core::*;`):
- 190+ types from `core` that resolve at the crate root (e.g., `rgui::Color`, `rgui::Point`, `rgui::Size`) but are not part of the documented public surface. The bug-fix 8.5 comment explains this is intentional but `#[doc(hidden)]` keeps them working.

**API-01 implication:** the doctest plan should cover the 30+ explicit `widgets::spec` re-exports and the public modules' top-level types. The `#[doc(hidden)] pub use core::*` items are excluded (they're explicitly hidden).

### Doctest conventions in Rust 2024

- **Doctest syntax**: `///` rustdoc comment + ` ```rust ` block. The `cargo test --doc` invocation runs them.
- **Three execution modes**:
  - No annotation: runs the example. The example must compile AND the assertions must pass.
  - ` ```no_run `: compiles but doesn't run. Use for examples that have side effects (windowing, network).
  - ` ```ignore `: neither compiles nor runs. Use for pseudo-code or for examples that need features not enabled by default.
- **Smoke tests vs full examples**:
  - **Smoke test** (idiomatic for opaque types): `let _ = Type::default();`. Just proves the type's name + `Default::default()` resolve.
  - **Full usage example** (idiomatic for builder types): shows the construction + a representative use. E.g., for `list()`:
    ```rust
    let items = vec!["a".to_string(), "b".to_string()];
    let _list = rgui::widgets::list().items(items).default_selected_index(0);
    ```
- **API-01 application**: doctests must be "runnable" (not `no_run`/`ignore` for most types). The crate-root types that have side effects (e.g., the winit example adapter) are `no_run`'d.

### `cargo doc --document-private-items` warnings

The pre-existing warnings are:
- `src/render/wgpu/mod.rs:165` — `/// of [\`new_headless_for_tests\`]`. The symbol `new_headless_for_tests` exists in the same module but rustdoc can't resolve it (possibly because of the orphan-rule or because it's behind a `#[cfg(test)]` gate). Fix: escape the brackets (`\[` / `\]`) or rewrite as a relative path.
- `src/runtime/paint.rs:3` — `//! walks the [\`UiTree\`]`. The symbol `UiTree` is not imported into the module's namespace (it's `crate::runtime::tree::UiTree`, accessed via `super::UiNode`). Fix: use a fully-qualified path or add `use super::UiTree;` at the module level.

Both fixes are 1-2 line changes. The `cargo doc --document-private-items` build must then emit zero warnings (per the API-02 success criterion).

### Unwrap audit findings (deep dive)

A comprehensive grep for `\.unwrap\(\)` in `src/runtime/**/*.rs` (excluding `#[cfg(test)]` blocks) finds:

| File | Line | Code | Status |
|------|------|------|--------|
| `src/runtime/state.rs` | 367 | `let cancel = cancel.unwrap();` | in `#[cfg(test)]` block — excluded |
| `src/runtime/state.rs` | 394 | `let cancel = cancel.unwrap();` | in `#[cfg(test)]` block — excluded |
| `src/runtime/runtime.rs` | 632 | `kind.unwrap()` | **production code — target** |

The `kind.unwrap()` at `runtime.rs:632`:
```rust
if matches!(kind, Some(WidgetKind::Input | WidgetKind::Textarea)) {
    // ... [state arena setup] ...
    if let Some(state) = self.state_arena.get_mut::<InputState>(hit.node) {
        let (text_top_left, measure_width, style) =
            text_hit_geometry_for_widget(kind.unwrap(), hit.rect, &self.theme);
```

The `matches!(kind, Some(WidgetKind::Input | WidgetKind::Textarea))` check above the unwrap proves `kind` is `Some(...)` for `Input` or `Textarea`. The unwrap is reachable (the matches! returns true) and provably safe (the Some is one of the matched variants). The replacement per D-02:
```rust
let (text_top_left, measure_width, style) =
    text_hit_geometry_for_widget(
        kind.expect("WidgetKind is Some(WidgetKind::Input | Textarea) when matches!(kind, Some(WidgetKind::Input | Textarea)) is true"),
        hit.rect,
        &self.theme,
    );
```

This is the only production-code unwrap in `src/runtime/`. The plan replaces it in 1 commit.

### `clippy::unwrap_used` behavior

`clippy::unwrap_used` is a clippy lint that flags `.unwrap()` calls. It can be configured at the lint level:
- `allow` (default — no warning)
- `warn` (warning, doesn't fail build)
- `deny` (error, fails `cargo build`)
- `forbid` (cannot be overridden with `#[allow(...)]`)

For the Phase 6 use case, `deny` is the right level: it fails the build, but individual functions can still `#[allow(clippy::unwrap_used)]` if needed. The deny is added at the module level via `#![deny(clippy::unwrap_used)]` in `src/runtime/mod.rs` (or in each runtime submodule).

Trade-off: `clippy::unwrap_used` may flag `.unwrap()` calls in `clippy` itself or in macro-expanded code. Need to verify it doesn't false-positive in the runtime's other `expect()` / `?` operators. The 5 existing `.expect()` calls are NOT flagged by `clippy::unwrap_used` (only `.unwrap()` is).

### `WidgetPainter` extension contract

The `WidgetPainter` trait (in `src/runtime/paint.rs:380` approximately) is the per-widget paint dispatch:
```rust
pub trait WidgetPainter: Send + Sync {
    fn background_color(&self, ctx: &PaintCtx<'_>) -> Color;
    fn paint_content(&self, ctx: &mut PaintCtx<'_>, cmds: &mut Vec<PaintedCommand>);
}
```

The trait is `Send + Sync` (Phase 4 D-17). The runtime maintains a `static` registry of `Arc<dyn WidgetPainter>` instances keyed by `WidgetKind`. The registry is `Mutex`-protected (the 5 `.expect("widget painter registry poisoned")` calls).

The `register_widget_painter(WidgetKind, Arc<dyn WidgetPainter>)` and `unregister_widget_painter(WidgetKind)` functions are public API (or are made public in 06-03). The "writing a custom widget" guide is a markdown doc that walks through:
1. Define a `MyPainter` struct that implements `WidgetPainter`.
2. Call `register_widget_painter(MyKind, Arc::new(MyPainter))` at app startup.
3. Use `MyKind` in an `Element` widget spec.
4. The painter is invoked during paint.
5. Call `unregister_widget_painter(MyKind)` on app shutdown.

The guide should be runnable: a copy-paste example, with the painter registered and used in a `UiRuntime::default()` test. The example can live in `examples/custom_widget.rs`.

## Patterns Established by Other Phases

- **Phase 5's culling fix** added a `clip_rect: Option<Rect>` field to `VisualState` and used it in `ListPainter::paint_content`. This is a new field, a new method body. The Phase 6 unwrap audit reads the resulting `ListPainter::paint_content` (no new unwraps there).
- **Phase 4's `Send + Sync` propagation** (D-17) added the `unsafe impl Send + Sync` for `TaffyLayoutBackend` with a SAFETY block explaining the invariant. Phase 6's guide can reference this as the model for documenting `Send + Sync` extension points.
- **Phase 5's CI workflow** uses `RUSTFLAGS=-D warnings` on the clippy + doc jobs. Phase 6's added `#![deny(clippy::unwrap_used)]` aligns with this — both fail the build on a warning. The CI workflow doesn't need to change for Phase 6; the deny is enforced locally by the rustc/clippy linter.

## Risks and Pitfalls

- **clippy::unwrap_used false positives in test code**: if the deny is added at the module level (`src/runtime/mod.rs`), the test modules inside `src/runtime/state.rs` etc. are also subject to the deny. The deny is added at the **crate root** level (`src/lib.rs`) would be too broad (covers tests/, examples/, benches/). The right scope is `src/runtime/` only.

- **Unwrap regression test brittleness**: a `tests/unwrap_audit.rs` test that reads file contents and checks for `.unwrap()` substrings can be fooled by `.unwrap_or` (which has a different name) or by `writeln!` macros that contain the substring `writeln` (not `unwrap`). The test must use word-boundary matching: `\b\.unwrap\(\)\b` (Rust regex syntax). A comment-style or string-literal `".unwrap()"` is not flagged (it's a string, not a call).

- **cargo doc warnings can hide behind existing warnings**: the pre-existing warnings `new_headless_for_tests` and `UiTree` are in `src/render/wgpu/mod.rs` and `src/runtime/paint.rs` respectively. The fix is in the same modules. The Phase 5 culling fix added code to `src/runtime/paint.rs` that includes new rustdoc comments (the culling-fix docstring). Need to verify those don't introduce new warnings.

- **Doctest execution time**: 30+ doctests, each compiling + running a small example. Total doctest execution time is typically 5-30 seconds. CI budget is acceptable.

## Recommended Plan Allocation (informs planner)

| Plan | API-01..04 mapping | Estimated work |
|------|--------------------|----------------|
| 06-01 Doctests | API-01, API-02 | 30+ doctests + 2 broken-link fixes |
| 06-02 Unwrap audit | API-03 | 1 unwrap replacement + clippy deny + grep test |
| 06-03 Painter guide | API-04, CUST-01..03 | 1 guide markdown + 1 example + confirm `Send + Sync` |

Each plan is a vertical slice (delivers an end-to-end working change for one or two requirements). 06-02 is the smallest plan (1 file change + 1 new test + 1 module-attribute change). 06-01 is the largest (30+ doctests across multiple files). 06-03 is documentation-heavy.

The plans are independent (no file overlap between 06-01 and 06-02; 06-03 touches `docs/` and `examples/`). They can all run in Wave 1 in parallel, or sequentially in order 06-02 -> 06-01 -> 06-03 (smallest first for fast feedback).

## Files to Modify (informative — planner decides exact)

- `src/lib.rs` — may add `#![doc = include_str!("../README.md")]` or other crate-level docs (Claude's discretion; not required for API-01/02).
- `src/runtime/mod.rs` — add `#![deny(clippy::unwrap_used)]` at the module level.
- `src/runtime/runtime.rs:632` — replace `kind.unwrap()` with `kind.expect("...")`.
- `src/render/wgpu/mod.rs:165` — fix the rustdoc broken-link warning.
- `src/runtime/paint.rs:3` — fix the rustdoc broken-link warning.
- All public type definitions in the `pub use widgets::spec::{...}` list — add doctests.
- `tests/unwrap_audit.rs` — new test that greps for `.unwrap()` in `src/runtime/`.
- `docs/writing-a-custom-widget.md` — new guide.
- `examples/custom_widget.rs` — new runnable example for the guide.
- `STATE.md` — update with Phase 6 decisions.

## Verification Targets

- `cargo doc --document-private-items` exits 0 with zero warnings.
- `cargo test --doc` runs all doctests and they pass.
- `cargo test --test unwrap_audit` exits 0 (no new unwraps in `src/runtime/`).
- `cargo clippy --all-targets --all-features -- -D warnings` exits 0 (no clippy errors, including the new `clippy::unwrap_used` deny).
- The "writing a custom widget" guide is renderable as markdown (no broken links, all code blocks compile).
