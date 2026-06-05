---
phase: 06
plan: 02
subsystem: runtime
tags: [api-hardening, public-api, unwrap-policy, clippy]
requires: []
provides:
  - API-03: zero production-code `.unwrap()` in `src/runtime/`
  - Module-level clippy deny: `#![deny(clippy::unwrap_used)]` in `src/runtime/mod.rs`
  - Test-time regression gate: `tests/unwrap_audit.rs`
affects:
  - src/runtime/runtime.rs
  - src/runtime/mod.rs
  - src/runtime/debug.rs
  - src/runtime/state.rs
tech-stack:
  added: []
  patterns:
    - "Production code uses `.expect(\"invariant description\")` for infallible unwraps"
    - "Test fixtures may use `.unwrap()` (audit test strips `#[cfg(test)]` blocks)"
    - "Infallible operations on infallible types (e.g. `writeln!` to `String`) opt out via `#![allow(clippy::unwrap_used)]` at the file level"
key-files:
  created:
    - tests/unwrap_audit.rs
  modified:
    - src/runtime/runtime.rs
    - src/runtime/mod.rs
    - src/runtime/debug.rs
    - src/runtime/state.rs
key-decisions:
  - "Replaced `kind.unwrap()` with a temporary variable + `.expect(...)` carrying the matches! invariant — the line is long and the temp keeps the call site readable."
  - "Module-level deny (not crate-level) so `tests/`, `examples/`, and `benches/` retain the right to `.unwrap()`."
  - "Two enforcement layers: compile-time clippy deny + test-time grep audit. Either gate alone could be bypassed; both together is robust."
  - "Audit test respects file-level `#![allow(clippy::unwrap_used)]` opt-outs (e.g. `debug.rs` where `writeln!` to `String` is infallible)."
requirements-completed:
  - API-03
duration: 8 min
completed: 2026-06-05
---

# Phase 6 Plan 02: Audit + remove `unwrap()` in the runtime paint path Summary

## What Was Built

Removed the 1 production-code `.unwrap()` in `src/runtime/` (the `kind.unwrap()` at `src/runtime/runtime.rs:632` inside the text-hit-geometry path for `Input`/`Textarea` widgets) and replaced it with a `.expect(...)` carrying a descriptive invariant message. Added a module-level `#![deny(clippy::unwrap_used)]` in `src/runtime/mod.rs` so future PRs cannot add new unwraps under `src/runtime/` without a clippy error. Added `tests/unwrap_audit.rs` as a second enforcement layer: a grep-based regression test that strips `#[cfg(test)]` blocks from `src/runtime/**/*.rs` and asserts zero `.unwrap()` calls in production code.

## Tasks

| # | Task | Status | Commit |
|---|------|--------|--------|
| 1 | Replace `kind.unwrap()` with `.expect("...")` at `runtime.rs:632` | ✓ Done | 266949f |
| 2 | Add `#![deny(clippy::unwrap_used)]` at `runtime/mod.rs` | ✓ Done | 266949f |
| 3 | Add `tests/unwrap_audit.rs` regression test | ✓ Done | 266949f |
| 4 | Update `.clippy.toml` + add CI test (skipped per plan) | ⊘ Skipped | — |
| 5 | Verify all checks pass | ✓ Done | 266949f |

## Deviations from Plan

- **Added `#![allow(clippy::unwrap_used)]` to `src/runtime/debug.rs`**: the `writeln!` macro into a `String` is infallible, so the file-level allow is the right escape hatch. The audit test also recognises this opt-out and skips `debug.rs`. Per the plan's "Special case" note.
- **Added `#[allow(clippy::unwrap_used)]` to the `mod pointer_capture_release_tests` module in `state.rs`**: the two test-code unwraps at `state.rs:367, 394` are inside `#[cfg(test)] mod pointer_capture_release_tests`. The clippy deny catches them; the allow scopes the exception to that test module only. The audit test also strips `#[cfg(test)]` blocks, so the test passes either way.
- **Audit test recognises file-level `#![allow(clippy::unwrap_used)]`**: without this, the audit test would flag `debug.rs`'s `writeln!().unwrap()` calls and fail. The opt-out makes the audit test consistent with the clippy deny.

## Self-Check

- [x] `cargo build --lib` succeeds.
- [x] `cargo test --lib --features default` passes (122 pre-existing tests + 1 new lib test, 0 regressions).
- [x] `cargo test --test unwrap_audit` passes.
- [x] `cargo clippy --lib --all-features` emits zero `clippy::unwrap_used` errors.
- [x] `grep -n "\.unwrap()" src/runtime/*.rs` (excluding `#[cfg(test)]` blocks and `debug.rs`'s opted-out file) returns zero hits.

## Key Files

- `src/runtime/runtime.rs:632` — `kind.expect(...)` replaces the unwrap.
- `src/runtime/mod.rs:1` — `#![deny(clippy::unwrap_used)]` at the top.
- `src/runtime/debug.rs:1` — `#![allow(clippy::unwrap_used)]` opt-out.
- `src/runtime/state.rs:358` — `#[allow(clippy::unwrap_used)]` on the pointer-capture test module.
- `tests/unwrap_audit.rs` — new regression test (~120 lines).

## Ready For

- Plan 06-01: doctests for every public type at the crate root + `cargo doc` cleanup (independent of 06-02).
- Plan 06-03: `WidgetPainter` extension contract + "writing a custom widget" guide (independent of 06-02).
