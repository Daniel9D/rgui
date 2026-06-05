---
phase: 06
plan: 03
subsystem: docs
tags: [api-hardening, public-api, extension-points, widget-painter]
requires: []
provides:
  - API-04: `WidgetPainter: Send + Sync` documented in the custom-widget guide
  - CUST-01: `register_widget_painter` is a stable public contract
  - CUST-02: `unregister_widget_painter` exists with a documented use case
  - CUST-03: `docs/writing-a-custom-widget.md` walks the full lifecycle
  - Runnable example: `examples/custom_widget.rs`
  - End-to-end test: `tests/widget_painter_registry.rs`
affects:
  - docs/writing-a-custom-widget.md
  - examples/custom_widget.rs
  - Cargo.toml
  - tests/widget_painter_registry.rs
tech-stack:
  added: []
  patterns:
    - "Custom painters stored as `&'static dyn WidgetPainter` (process-global)"
    - "`AtomicUsize` as the side-channel for testing paint-call counts"
    - "Override a built-in `WidgetKind` (e.g. `WidgetKind::Badge`) until a public variant-add API exists"
key-files:
  created:
    - docs/writing-a-custom-widget.md
    - examples/custom_widget.rs
    - tests/widget_painter_registry.rs
  modified:
    - Cargo.toml
key-decisions:
  - "The plan's example sketch used `Arc<dyn WidgetPainter>` for registration; the actual API takes `&'static dyn WidgetPainter`. Used the correct `static` binding pattern in both the guide and the example."
  - "The example reuses `WidgetKind::Badge` (a built-in kind) rather than introducing a custom `WidgetKind` variant. The `WidgetKind` enum is `#[non_exhaustive]` with no public add-variant API, so this is the v1 workaround. The guide documents the limitation as a possible v1.x follow-up."
  - "The integration test asserts the painter is NOT invoked after `unregister_widget_painter`. This is the strongest pin of the contract — a no-op unregister would still pass the 'invoked' half of the test but fail this stronger assertion."
  - "Cargo.toml `[[example]]` entry for `custom_widget` is featureless (no `required-features`). The example uses only the default feature set, so no gating is needed."
  - "The guide is in `docs/writing-a-custom-widget.md` (user-facing), not in rustdoc. The trait is per-symbol and doesn't have a place for a multi-step narrative guide."
requirements-completed:
  - API-04
  - CUST-01
  - CUST-02
  - CUST-03
duration: 10 min
completed: 2026-06-05
---

# Phase 6 Plan 03: `WidgetPainter` extension contract + "writing a custom widget" guide Summary

## What Was Built

Confirmed the existing `WidgetPainter: Send + Sync` trait and the public `register_widget_painter` / `unregister_widget_painter` functions in `src/runtime/paint.rs` (no changes needed — Task 1 was pure verification). Wrote `docs/writing-a-custom-widget.md`, a 5-step user guide covering define → register → use → unregister → integration-test, with runnable code blocks using the actual `rgui` API. Added `examples/custom_widget.rs`, a runnable demo that registers a `StatusPillPainter` for `WidgetKind::Badge`, paints a `badge("online")` element, prints paint-command + invocation counts, and unregisters. Added `tests/widget_painter_registry.rs`, an end-to-end integration test that asserts the painter is invoked on `update()` after `register_widget_painter` and is NOT invoked after `unregister_widget_painter`.

## Tasks

| # | Task | Status | Commit |
|---|------|--------|--------|
| 1 | Confirm `WidgetPainter: Send + Sync` and `register_/unregister_widget_painter` are public | ✓ Verified (no change) | 871f4b2 |
| 2 | Write `docs/writing-a-custom-widget.md` | ✓ Done | 871f4b2 |
| 3 | Add `examples/custom_widget.rs` | ✓ Done | 871f4b2 |
| 4 | Add `tests/widget_painter_registry.rs` integration test | ✓ Done | 871f4b2 |
| 5 | Verify all changes pass | ✓ Done | 871f4b2 |

## Deviations from Plan

- **Example uses `&'static dyn WidgetPainter`, not `Arc<dyn WidgetPainter>`.** The plan's example sketch in Task 3 used `Arc::new(StatusPillPainter)` for registration, but the actual `register_widget_painter` API takes `&'static dyn WidgetPainter` (the registry stores `&'static` references; the process-global lifetime is required). Both the example and the guide use the `static FOO: MyPainter = MyPainter;` pattern.
- **Example reuses `WidgetKind::Badge`, not a new variant.** The plan acknowledged this is the v1 workaround. Both the example and the guide explain why (the `WidgetKind` enum is `#[non_exhaustive]` with no public add-variant API in v1).
- **Test asserts the painter is NOT invoked after `unregister_widget_painter`.** The plan's test sketch only checked the "invoked" half. The stronger "not invoked after unregister" assertion was added because it's the key proof that the round-trip works; without it, a buggy implementation that just leaks a stale painter would still pass.

## Self-Check

- [x] `cargo build --lib` succeeds.
- [x] `cargo test --lib --features default` passes (122 pre-existing tests, 0 regressions).
- [x] `cargo test --test widget_painter_registry --features default` passes.
- [x] `cargo build --example custom_widget` succeeds.
- [x] `cargo run --example custom_widget` runs and prints "Rendered 3 paint commands (StatusPillPainter invoked 1 time(s))".
- [x] `docs/writing-a-custom-widget.md` is renderable as markdown.
- [x] The guide's code blocks compile against the actual `rgui` API (verified by the example in `examples/custom_widget.rs` and the integration test in `tests/widget_painter_registry.rs`).
- [x] The `WidgetPainter` trait has `Send + Sync` supertraits (verified at `src/runtime/paint.rs:401`).
- [x] `register_widget_painter` and `unregister_widget_painter` are public functions (verified at `src/runtime/paint.rs:584, 597`).

## Key Files

- `docs/writing-a-custom-widget.md` — 5-step user guide (~190 lines).
- `examples/custom_widget.rs` — runnable example (~75 lines).
- `tests/widget_painter_registry.rs` — end-to-end test (~80 lines).
- `Cargo.toml:60-62` — new `[[example]]` entry for `custom_widget`.

## Ready For

- Phase 6 verification (consolidate all 3 plans, run the verifier).
