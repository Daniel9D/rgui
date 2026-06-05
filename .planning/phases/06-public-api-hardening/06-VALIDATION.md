---
phase: 06-public-api-hardening
nyquist: enabled
strategy: standard-rustdoc-and-runtime-audit
---

# Phase 6 — Validation Strategy

**Mode:** standard (rustdoc + runtime audit; no benchmarks, no AI eval, no visual goldens)

## Dimensions Covered

| Dimension | Validator | Source |
|-----------|-----------|--------|
| 1. Rustdoc compliance | `cargo doc --document-private-items` | API-02 success criterion |
| 2. Doctest coverage | `cargo test --doc` | API-01 success criterion |
| 3. Unwrap audit | `tests/unwrap_audit.rs` + `clippy::unwrap_used` deny | API-03 success criterion |
| 4. WidgetPainter extension | trait compile + `register_widget_painter` test + `docs/writing-a-custom-widget.md` review | API-04 success criterion |

## Validation Plan per Plan

### 06-01 (Doctests)

**Validation: `cargo test --doc` runs all doctests and they pass.**

- All doctests in `pub use widgets::spec::{...}` re-exports compile and run.
- The 2 pre-existing rustdoc broken-link warnings (`new_headless_for_tests`, `UiTree`) are fixed.
- `cargo doc --document-private-items` emits zero warnings.
- A lib test asserts `cargo doc --document-private-items` exits 0 (catches future regressions).

### 06-02 (Unwrap audit)

**Validation: zero `.unwrap()` in `src/runtime/` production code.**

- The 1 found `kind.unwrap()` at `src/runtime/runtime.rs:632` is replaced with `kind.expect("...")`.
- `#![deny(clippy::unwrap_used)]` is added at the `src/runtime/` module level.
- `tests/unwrap_audit.rs` grep-asserts zero `.unwrap()` substrings in `src/runtime/**/*.rs` (excluding `#[cfg(test)]` blocks).
- A lib test asserts `cargo clippy --lib --all-features -- -D warnings` exits 0 (catches clippy::unwrap_used denials).

### 06-03 (Painter guide)

**Validation: the guide is runnable, the trait is `Send + Sync`, the API exists.**

- `WidgetPainter` trait confirms `Send + Sync` (it does, from Phase 4 D-17).
- `register_widget_painter` and `unregister_widget_painter` are public API.
- `docs/writing-a-custom-widget.md` is renderable (no broken links, all code blocks compile).
- `examples/custom_widget.rs` runs without panic (compiles + opens a window in `winit` examples if winit is available, or compiles only if not).
- A lib test asserts the guide's code blocks compile (by including them as a doc test in the guide file).

## Cross-cutting Validations

- `cargo build --lib` and `cargo build --tests` both succeed.
- `cargo test --lib` (the lib test suite) passes 100% of the time.
- `cargo clippy --all-targets --all-features -- -D warnings` exits 0 (catches the new `clippy::unwrap_used` deny + any other regressions).
- `cargo doc --document-private-items --no-deps` exits 0 with zero warnings (catches the broken-link fixes).
- `cargo test --release --test frame_budget` (Phase 5) still passes — no regression to the 8ms budget.

## What is NOT validated here

- Visual goldens (no UI changes in Phase 6).
- Frame budget (Phase 5; the changes in Phase 6 are docs + small refactors that don't affect the paint path).
- Validation layers / CI (Phase 5; the CI workflow doesn't change).
- Performance benchmarks (no runtime perf changes in Phase 6).

## Validation Budget

- Total validation time: ~5 minutes (lib tests + clippy + doctest + doc + the new unwrap_audit test + frame_budget regression check).
- CI budget per push: ~10 minutes (lib + tests + clippy + doc + vulkan-goldens + validation-layers + frame_budget).
