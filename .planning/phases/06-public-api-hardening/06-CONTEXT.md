# Phase 6: Public API Hardening - Context

**Gathered:** 2026-06-04
**Status:** Ready for planning
**Source:** Inline synthesis (GSD `gsd-phase-researcher` / `gsd-pattern-mapper` subagents unavailable in this runtime; reasoning done by the orchestrator against the actual codebase)

<domain>

## Phase Boundary

The public API surface of the `rgui` crate is currently documented but not enforced: doctests are absent on most crate-root public types, the `cargo doc` build emits pre-existing rustdoc-broken-link warnings, the runtime paint path has at least one reachable `.unwrap()` (the `kind.unwrap()` at `src/runtime/runtime.rs:632` — provably safe by the `matches!` check above it but still an unwrap), and the `WidgetPainter` extension contract exists but is not part of a documented "writing a custom widget" guide.

This phase hardens the public API in four ways (one per requirement):

1. **API-01 (doctests for every public type at the crate root)** — every public type in the `rgui` crate root gains a runnable doctest. Smoke tests for compile-only types, full usage examples for the builder / state-machine types.
2. **API-02 (`cargo doc --document-private-items` builds without warnings)** — fix the pre-existing rustdoc-broken-link warnings (`new_headless_for_tests` at `src/render/wgpu/mod.rs:165`, `UiTree` at `src/runtime/paint.rs:3`) and any new warnings introduced by the Phase 5 culling fix.
3. **API-03 (no `unwrap()` in the runtime paint path under non-pathological inputs)** — replace the 1 found `.unwrap()` in `src/runtime/` (production code) with `.expect("descriptive message")`, add a `clippy::unwrap_used = "deny"` lint at the `src/runtime/` module level so future PRs can't regress, and add a `tests/unwrap_audit.rs` test that grep-asserts zero new unwraps in `src/runtime/` (excludes `#[cfg(test)]` blocks).
4. **API-04 (`WidgetPainter` is `Send + Sync`; custom-painter docs explain the contract)** — confirm the `WidgetPainter` trait already requires `Send + Sync` (it does, from Phase 4 D-17 / Phase 5's painter work), and write a `docs/writing-a-custom-widget.md` guide that walks a user through `register_widget_painter` / `unregister_widget_painter` with a runnable example.

This phase is in `mvp` mode per ROADMAP.md. The 3 plans (per ROADMAP.md) deliver:
- 06-01: Doctests for every public type at the crate root
- 06-02: Audit + remove `unwrap()` in the runtime paint path
- 06-03: `WidgetPainter` extension contract + "writing a custom widget" guide

What's out of scope (later phases):
- Component-level doctests (the widget builders' full usage examples); Phase 6 covers crate-root types only. Component-level doctests are a v1.x follow-up.
- A `pub use` explicit re-export list (replacing the `#[doc(hidden)]` wildcard). Deferred to post-v1 per PROJECT.md.
- Moving widget painters out of `runtime/paint.rs` to `widgets/paint/`. Deferred to Phase 7 (Theme v2) per STATE.md.

</domain>

<decisions>

## Implementation Decisions

### Unwrap replacement policy (06-02)

The discussion covered how to enforce API-03. The user picked the strictest combination: replace all production-code unwraps in `src/runtime/` (excluding `#[cfg(test)]` blocks) with `.expect("descriptive message")`, AND add a `clippy::unwrap_used = "deny"` lint at the `src/runtime/` module level so future PRs cannot add new unwraps without a deliberate override.

- **D-01 (audit scope: `src/runtime/`, production code only).** The audit covers `src/runtime/**/*.rs` but excludes `#[cfg(test)] mod tests` blocks (where `unwrap()` on test fixtures is idiomatic). The pre-existing 5 `.expect()` calls for the widget painter registry mutex in `src/runtime/paint.rs:587,597,606,614,672` are in good shape and need no change. The 1 reachable `.unwrap()` found at `src/runtime/runtime.rs:632` (`text_hit_geometry_for_widget(kind.unwrap(), ...)`) is the only production-code target.
- **D-02 (replacement style: `.expect("descriptive message")`).** Each `.unwrap()` becomes `.expect("descriptive message that names the invariant being violated")`. For the `kind.unwrap()` at `runtime.rs:632`, the replacement is `kind.expect("WidgetKind must be Some when matches!(kind, Some(WidgetKind::Input | Textarea)) is true")`. The matches! check above the unwrap is the invariant; the expect message names it so a future test failure points directly at the violated invariant. Standard Rust idiom; panics with a useful message.
- **D-03 (audit threshold: every production-code unwrap in `src/runtime/`).** The plan replaces the 1 found unwrap. The plan also adds `tests/unwrap_audit.rs` that grep-asserts zero new `.unwrap()` calls in `src/runtime/**/*.rs` (excluding `#[cfg(test)]` blocks) at the file level — a regression-test that fails the build if a future PR adds one. The audit is a pure-string assertion on file contents; no AST parsing, no `cargo expand`. Runs in `cargo test --test unwrap_audit`.
- **D-04 (enforcement: `clippy::unwrap_used = "deny"` at the module level).** The `src/runtime/mod.rs` (or each runtime submodule) gets a `#![deny(clippy::unwrap_used)]` attribute so the rustc/clippy linter catches new unwraps at compile time. This is a stronger guard than the grep test (catches at `cargo build`, not `cargo test`). Combined with D-03's grep test, the unwrap budget is enforced at two layers.

### Claude's Discretion (06-01, 06-03)

The user did not select the other three gray areas (doctest scope + format, custom-painter API + guide, MVP organization). The planner has flexibility on:

- **06-01 — Doctest scope and format.** Doctests can be smoke tests (`let _ = Type::default();` to ensure the example compiles) or full usage examples (showing how to construct + use the type). The standard Rust library convention is: full usage examples for builder / state-machine types (e.g., `Element::column()`), and one-line "this compiles" for opaque types (e.g., `Color`, `Point`). The exact per-type decision is the planner's. The `cargo doc --document-private-items` cleanup (the `new_headless_for_tests` and `UiTree` broken-link fixes) is also Claude's discretion — use `\[` / `\]` escaping, or rename the referenced symbol, or move the doctest to a place where the symbol resolves.
- **06-03 — `WidgetPainter` extension contract and guide.** The trait already requires `Send + Sync` (Phase 4 D-17). The `register_widget_painter` / `unregister_widget_painter` functions are gated by the `widget-painter-registry` feature (or the equivalent — the planner reads `src/runtime/paint.rs` to confirm). The guide is a markdown file in `docs/` (e.g., `docs/writing-a-custom-widget.md`) that walks through: define a `WidgetPainter` impl, register it, unregister it, integration-test it. The guide can include a runnable example in `examples/custom_widget.rs`. The guide's exact structure is Claude's discretion; the goal is that a new user can write a custom widget by reading the guide.
- **MVP organization.** The 3 plans (06-01, 06-02, 06-03) are the natural horizontal layers. For mvp mode, each plan should still deliver a "vertical slice" where possible. 06-02's vertical slice is: pick the 1 unwrap, replace it, add the lint, add the test — all in one plan, fully shippable. 06-01 and 06-03 are doctest/guide work which is naturally horizontal but the user benefits from per-type / per-section commits. The planner organizes accordingly.

The planner is free to make these choices without re-asking; the executor will document the chosen approach in each plan's SUMMARY.

### Folded Todos

None — discussion stayed within phase scope.

</decisions>

<canonical_refs>

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project context
- `.planning/PROJECT.md` — full project context, constraints (60fps, no unwrap in paint path, no serde on hot path)
- `.planning/REQUIREMENTS.md` — API-01..04 (lines 47-50) and CUST-01..03 (lines 100-102) are the v1 scope for this phase
- `.planning/ROADMAP.md` — Phase 6 entry (line 22) and 3 plan slots (06-01..03)

### Prior phase decisions that apply here
- `.planning/phases/01-incremental-reconciliation/01-CONTEXT.md` — the diffing model + `Element` builder. Phase 6's doctests on `Element` reflect this API.
- `.planning/phases/02-event-input-hardening/02-CONTEXT.md` — `ModalSpec::trap_focus`, `InputSpec::ime_enabled`. Phase 6's doctests on these specs show the new fields.
- `.planning/phases/03-text-ime/03-CONTEXT.md` — `ImeHostDriver: Send + Sync`, `TextCacheStats`. Phase 6's `WidgetPainter` guide references these as examples of `Send + Sync` extension points.
- `.planning/phases/04-multi-window/04-CONTEXT.md` D-17 — `ImeHostDriver: Send + Sync` and `AccessibilityBackend: Send + Sync` were pulled forward. Phase 6 confirms the same for `WidgetPainter`.
- `.planning/phases/05-render-path-stress/05-CONTEXT.md` — the culling fix added code to `ListPainter::paint_content` in `src/runtime/paint.rs`. Phase 6's unwrap audit includes this newly-added code. Also D-22 of Phase 5: `RendererOptions::default().backends == wgpu::Backends::PRIMARY` — a doc-example candidate for Phase 6's doctest plan.

### Code to read
- `src/lib.rs` — the `pub use` re-exports that define the "crate root public API". Phase 6's doctest plan covers these types.
- `src/runtime/mod.rs` — the `pub mod paint;` line. Phase 6's `clippy::unwrap_used` deny is added at this module level (or per-submodule).
- `src/runtime/paint.rs:587,597,606,614,672` — the 5 pre-existing `.expect("widget painter registry poisoned")` calls. These are the pattern Phase 6's unwrap replacement follows.
- `src/runtime/runtime.rs:632` — the 1 found `.unwrap()` (`kind.unwrap()`). The single target of the unwrap audit.
- `src/runtime/state.rs:367,394` — 2 `.unwrap()` calls in `#[cfg(test)]` blocks. Excluded from the audit (test code).
- `src/core/element.rs:57-62` — `Element::row()` / `Element::column()` builders. Doctest candidates for 06-01.
- `src/widgets/collections.rs:56-58` — `list()` builder. Doctest candidate.
- `src/widgets/forms.rs:40,49,185` — `input()`, `checkbox()`, `slider()` builders. Doctest candidates.
- `src/render/wgpu/mod.rs:165` — pre-existing rustdoc broken-link warning (`new_headless_for_tests`). API-02 success criterion target.
- `src/runtime/paint.rs:3` — pre-existing rustdoc broken-link warning (`UiTree`). API-02 success criterion target.

### External references
- **Rust 2024 doctests** — the standard `///` doc comment + ` ``` ` (no_run, ignore, should_panic) syntax. The `cargo test --doc` invocation runs them.
- **clippy::unwrap_used** — the `clippy.toml` linter config + `#![deny(clippy::unwrap_used)]` module attribute. The lint catches `.unwrap()` calls at compile time.
- **`docs/` convention** — the project's existing docs directory (if any). Phase 6 adds `docs/writing-a-custom-widget.md`.

</canonical_refs>

<codebase_context>

## Existing Code Insights

### Reusable Assets
- `src/runtime/paint.rs:587,597,606,614,672` — 5 pre-existing `.expect("widget painter registry poisoned")` calls. The pattern Phase 6's unwrap replacement follows.
- `src/lib.rs` `pub use` list — defines the "crate root public API" surface that 06-01's doctest plan covers.
- `tests/` directory — has 70+ integration tests; the 06-02 `unwrap_audit` test follows the same pattern (a `#[test]` that fails on a string-regex assertion).
- `.planning/STATE.md` — has the Phase 5 culling-fix decision (the `// TODO(phase-7)` and `// TODO(phase-8)` annotations are in `tests/stress_scene.rs`; Phase 6's unwrap audit reads this file too).

### Established Patterns
- **Per-task atomic commits** — Phase 5 set the standard: `feat(NN-MM):` / `test(NN-MM):` / `refactor(NN-MM):` / `docs(NN-MM):` / `ci(NN-MM):`. Phase 6 follows.
- **`#![deny(...)]` module attributes** — the codebase already uses `#![deny(rustdoc::broken_intra_doc_links)]` in some modules. The 06-02 `clippy::unwrap_used` deny follows the same pattern.
- **`/// rustdoc` on public types** — most crate-root public types have a `///` doc comment, but few have a runnable doctest. Phase 6's 06-01 plan adds the missing doctests.
- **`docs/` directory** — `.planning/ROADMAP.md` and `.planning/PROJECT.md` are in `.planning/`, not `docs/`. The Phase 6 guide goes in `docs/` (a new convention) or `.planning/` (existing convention). Claude's discretion.

### Integration Points
- `src/runtime/mod.rs:21` — the `pub mod paint;` line. The 06-02 `clippy::unwrap_used` deny is added at this level (or per-submodule). The exact placement is Claude's discretion.
- `src/lib.rs:1-50` — the `pub use` re-exports. Phase 6's 06-01 plan does not change this list; it just adds doctests to the re-exported types.
- `Cargo.toml:21-35` — the feature flags. The 06-02 unwrap_audit test does not need a new feature (it runs in default features).
- `src/runtime/paint.rs:580-680` — the widget painter registry. 06-03's guide references this code as the example for the `register_widget_painter` / `unregister_widget_painter` flow.

</codebase_context>

<specifics>

## Specific Ideas

- **The 5 pre-existing `.expect("widget painter registry poisoned")` calls** in `src/runtime/paint.rs` are the pattern Phase 6's unwrap replacement follows. The expect message names the violated invariant ("registry poisoned" = "the mutex that protects the painter registry is in a poisoned state, which only happens if a thread panicked while holding the lock"). Phase 6's `kind.unwrap()` -> `kind.expect("WidgetKind must be Some when matches!(kind, Some(WidgetKind::Input | Textarea)) is true")` follows the same template.
- **The `tests/unwrap_audit.rs` test** can be a simple file-content scan: read each `.rs` file under `src/runtime/`, skip `#[cfg(test)] mod tests { ... }` blocks, and assert no `.unwrap()` substring. About 30 lines of code. Runs in <100ms.
- **The `clippy::unwrap_used = "deny"` deny** at the `src/runtime/mod.rs` level. The deny catches `.unwrap()` calls at compile time (faster feedback than the grep test). The deny may need `#![allow(clippy::unwrap_used)]` on individual functions that legitimately use unwrap (none expected for Phase 6; the existing `.expect()` calls don't trigger the lint).
- **The "writing a custom widget" guide** should be a `docs/writing-a-custom-widget.md` markdown file (or similar) that includes:
  1. The `WidgetPainter` trait overview (1 paragraph)
  2. A minimal example: a `CustomPainter` that draws a colored rect (20 lines of code)
  3. `register_widget_painter(MyKind, Arc::new(MyPainter))` (1 paragraph)
  4. `unregister_widget_painter(MyKind)` (1 paragraph)
  5. Integration-testing the painter with a `WidgetKind` that's not in the standard set (1 paragraph)
  The guide should be runnable: copy-paste the example, register it, see it paint. The example painter can be added to `examples/custom_widget.rs`.
- **The pre-existing `cargo doc` warnings** (`new_headless_for_tests` at `src/render/wgpu/mod.rs:165`, `UiTree` at `src/runtime/paint.rs:3`) are the API-02 success-criterion targets. The fix is straightforward: either rename the symbol to match, or escape the brackets (`\[` / `\]`), or rewrite the rustdoc reference to use a path that resolves.

</specifics>

<deferred>

## Deferred Ideas

- **Component-level doctests** (doctests on the widget builders' full usage examples). Phase 6 covers crate-root types only. Component-level doctests are a v1.x follow-up.
- **A `pub use` explicit re-export list** (replacing the `#[doc(hidden)]` wildcard in `src/lib.rs`). Deferred to post-v1 per PROJECT.md's Key Decisions.
- **Moving widget painters out of `runtime/paint.rs` to `widgets/paint/`** (the architectural split). Deferred to Phase 7 (Theme v2) per STATE.md's "Pending Todos" section.
- **API documentation as a `docs.rs` page** (publishing the rustdoc to docs.rs). Deferred to v1.0 release prep.

### Reviewed Todos (not folded)

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 06-public-api-hardening*
*Context gathered: 2026-06-04 via inline discussion (GSD subagents unavailable in this runtime; discussed 1 of 4 gray areas)*
