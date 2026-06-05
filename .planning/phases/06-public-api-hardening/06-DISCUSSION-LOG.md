# Phase 6: Public API Hardening — Discussion Log

**Gathered:** 2026-06-04
**Mode:** inline synthesis (GSD subagents unavailable in this runtime)

## Gray Areas Identified

| # | Area | Selected? |
|---|------|-----------|
| 1 | Doctest scope + format (API-01, API-02) | no |
| 2 | Unwrap replacement policy (API-03) | **yes** |
| 3 | Custom-painter API + "writing a custom widget" guide (API-04, CUST-01..03) | no |
| 4 | MVP mode organization | no |

The user selected only area #2. The other three are Claude's Discretion in CONTEXT.md.

## Area #2: Unwrap replacement policy

### Q1: Audit scope
**Question:** Which unwraps are in scope for API-03?
**Selected:** All unwraps in `src/runtime/` (production code, excluding `#[cfg(test)]` blocks)
**Rationale:** The ROADMAP success criterion is "a grep for `unwrap()` in the runtime paint path returns zero hits". The runtime paint path lives in `src/runtime/`, so the audit covers `src/runtime/**/*.rs`. Test code (`#[cfg(test)]` blocks) is excluded because unwrap-on-test-fixtures is idiomatic.

### Q2: Replacement style
**Question:** Replacement style for each found unwrap?
**Selected:** `.expect("descriptive message")`
**Rationale:** Standard Rust pattern. The expect message names the violated invariant so a test failure points directly at the issue. For the `kind.unwrap()` at `runtime.rs:632`, the replacement is `kind.expect("WidgetKind must be Some when matches!(kind, Some(WidgetKind::Input | Textarea)) is true")`.

### Q3: Audit threshold
**Question:** Audit threshold?
**Selected:** All non-test unwraps in the audit scope, plus a `tests/unwrap_audit.rs` regression test
**Rationale:** The plan replaces the 1 found unwrap. The regression test greps for `.unwrap()` in `src/runtime/` (excluding `#[cfg(test)]` blocks) and fails if any are found. This catches future regressions.

### Q4: Enforcement level
**Question:** What does the success_criteria look like?
**Selected:** `clippy::unwrap_used = "deny"` at the module level
**Rationale:** Strongest enforcement. Catches new unwraps at `cargo build` time (faster feedback than the grep test which runs at `cargo test`). Combined with Q3's grep test, the unwrap budget is enforced at two layers.

## Decisions Captured (see CONTEXT.md)

- **D-01**: Audit scope = `src/runtime/`, production code only (exclude `#[cfg(test)]` blocks)
- **D-02**: Replacement style = `.expect("descriptive message")` (naming the violated invariant)
- **D-03**: Audit threshold = all production-code unwraps in `src/runtime/`; add `tests/unwrap_audit.rs` regression test
- **D-04**: Enforcement = `#![deny(clippy::unwrap_used)]` at the `src/runtime/` module level

## Claude's Discretion

- **06-01 — Doctest scope and format**: standard Rust convention (full usage examples for builder types, one-line for opaque types). The pre-existing `cargo doc` broken-link warnings are fixed in this plan.
- **06-03 — `WidgetPainter` extension contract and guide**: the trait already requires `Send + Sync` (Phase 4 D-17). The guide is `docs/writing-a-custom-widget.md` (or similar) with a minimal example, register/unregister, and integration-test sections.
- **MVP organization**: 3 horizontal plans (06-01, 06-02, 06-03) with vertical-slice commits within each.

## Deferred Ideas

See CONTEXT.md `## Deferred Ideas` section for the full list (component-level doctests, `pub use` re-export list, painter module split, docs.rs publishing).
