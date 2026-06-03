---
gsd_state_version: '1.0'
status: in_progress
progress:
  total_phases: 8
  completed_phases: 0
  total_plans: 30
  completed_plans: 8
  percent: 27
---

# Project State

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-06-03)

**Core value:** The paint pipeline produces a correct, sorted `DisplayList` for every `Element` tree — every `WidgetKind` paints something visible with the right z-order, the right hover/disabled/checked state, and the right glyph from the right font.

**Current focus:** Phase 2 (Event & Input Hardening) — 4/4 plans complete. Awaiting transition to Phase 3.

## Current Position

Phase: 2 of 8 (Event & Input Hardening) — plans complete, not yet audited
Plan: 4 of 4 in current phase
Status: Plans complete; integration test suite green
Last activity: 2026-06-03 — Phase 2 plans 02-01..02-04 implemented & committed (FocusManager::tab_next, ShortcutRegistry::resolve suppression, InputSpec::ime_enabled, Element::overflow_x/overflow_y + wheel 2D tests)

Progress: [████████░░░░] 27% (8/30 plans)

## Performance Metrics

**Velocity:**
- Total plans completed: 8
- Average duration: — min
- Total execution time: 0.0 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 1. Incremental Reconciliation | 4/4 | — | — |
| 2. Event & Input Hardening | 4/4 | — | — |
| 3. Text & IME | 0/3 | — | — |
| 4. Multi-Window | 0/3 | — | — |
| 5. Render Path Stress | 0/4 | — | — |
| 6. Public API Hardening | 0/3 | — | — |
| 7. Theme v2 + Animation + DnD | 0/5 | — | — |
| 8. Virtualization + Canvas + i18n + Docs | 0/4 | — | — |

*Updated after each plan completion*

## Accumulated Context

### Decisions

See `PROJECT.md` Key Decisions table for the full log. Recent:

- **Phase 2 implementation (2026-06-03)**: `ShortcutRegistry::resolve` gained a `focused_is_text_input: bool` argument that suppresses non-modifier-prefixed chords inside `Input`/`Textarea`/`Select`. `FocusManager::is_focusable(WidgetKind)` + `tab_next`/`tab_prev` helpers provide a tree-walking focus traversal alternative to the existing `FocusSystem` (which the runtime continues to use for the scope-based overlay routing). `ModalSpec::trap_focus: bool` flag (default `false`) lets modals opt into focus trapping. `InputSpec::ime_enabled: bool` (default `false`) gates IME preedit routing so CJK users can opt in to the preedit-then-commit path. `Element::overflow_x(Overflow)` / `overflow_y(Overflow)` setters enable per-axis scroll configuration; the runtime's `handle_wheel` was already 2D — the new tests pin the per-axis clamping behavior.
- **Phase 1 implementation (2026-06-03)**: `Reconciler::diff(prior, new) -> DiffOutput` is positional (children paired by index) with `kind_signature` comparing `WidgetSpec` signature for state preservation. New `LayoutCache` wraps the existing `TaffyLayoutBackend` and indexes `LayoutBox` per `NodeId` for the paint path. `PointerCapture::release_matching` returns a `PointerCancel` for the captured node when the capture's key is in the unmounted list. The new `diff` is *not yet* wired into `runtime::update` — the existing `reconcile_with_dirty` (keyed-only) is still the production path; the diff is exercised by tests/recon and ready for the Phase 6 (Public API Hardening) wiring.
- **Workflow init (2026-06-03)**: Granularity=Fine, Execution=Parallel, Git=Yes, Research/PlanCheck/Verifier/DriftGuard=Yes, AI=Quality, PR body=User Stories & Acceptance Criteria, Mode=Vertical MVP.
- **Theme v2 migration (P1)**: Flat `Theme::metrics` / `Theme::select` fields are deprecated in favor of `theme.components.get(WidgetKind)` (Bug fix 3.8, this PRD).
- **SmallVec deferred (P2)**: `paint_node` continues to return `Vec<PaintedCommand>`; the `SmallVec<[_; 4]>` optimization is deferred to post-v1 (PROJECT.md Key Decisions).

### Pending Todos

- Wire `Reconciler::diff` into `runtime::update` (replacing `reconcile_with_dirty` for unkeyed nodes). Tracked for Phase 6 (Public API Hardening).
- Add `release_captures_for_unmounted` call into `update()` after the diff runs. Tracked for Phase 6.
- Add `LayoutCache` reads into the paint path (currently the paint path still walks taffy). Tracked for Phase 5 (Render Path Stress).
- Wire `ModalSpec::trap_focus` into the runtime's overlay-scope routing (currently the new `FocusManager::tab_next` is the lighter-weight alternative, but the existing `FocusSystem` is what the runtime actually uses). Tracked for a future phase if/when a real-world modal needs it.

### Blockers/Concerns

- The runtime paint path has `unwrap()` calls in widget painters. Phase 6 (Public API Hardening) plans to audit and remove them.
- The current `WidgetPainter` trait is `Send + Sync`; verify all custom painters written by users are also `Send + Sync`. Phase 6 plans to add a guide.
- The `runtime ↔ widgets` module boundary still has widget painters living in `runtime/paint.rs`; Phase 7 (Theme v2) is the natural place to do the architectural split (8.2 from the feedback review).

## Deferred Items

Items acknowledged and carried forward:

| Category | Item | Status | Deferred At |
|----------|------|--------|-------------|
| Perf | `paint_node` return type → `SmallVec<[_; 4]>` | Deferred to post-v1 | 2026-06-03 |
| Arch | Move widget painters to `widgets/paint/` (feedback 8.2) | Deferred to Phase 7 | 2026-06-03 |
| Arch | Full split of `runtime/paint.rs` (feedback 8.1) | Forward-compat done (pub(super)); full extraction in Phase 7 | 2026-06-03 |
| API | Explicit `pub use` re-export list (feedback 8.5 alt) | Wildcard is `#[doc(hidden)]`; explicit list is post-v1 | 2026-06-03 |
| Dep | `smallvec` for paint_node return type (feedback 4.7) | Deferred to post-v1 | 2026-06-03 |
| Reconcile | Wire `Reconciler::diff` into `runtime::update` | Phase 6 wiring | 2026-06-03 |
| IME | South-East Asian complex-script state machine (Hindi, Thai, Khmer) | Deferred to v1.x | 2026-06-03 |
| IME | Per-character preedit composition | Deferred to v1.x | 2026-06-03 |
| Scroll | Momentum / inertial trackpad pan | Deferred to v1.x | 2026-06-03 |
| Focus | Custom `tabindex` attribute | Deferred to v1.x (users use `request_focus` directly) | 2026-06-03 |
| Focus | Focus ring paint (OS / browser draws it for v1) | Deferred to v1.x | 2026-06-03 |
| Input | Touch / pen / gamepad events | Deferred to v1.x | 2026-06-03 |

## Session Continuity

Last session: 2026-06-03 20:00
Stopped at: Phase 2 plans 02-01..02-04 all implemented and committed. 9 new integration tests added (event_input_hardening 5 + ime_gating 3 + 1 input_copy_cut_paste from pre-existing); 9 new lib unit tests (focus_traversal 5 + shortcut_suppression 4). Lib 90/90 passing. Next: /gsd-discuss-phase 3 (Text & IME polish) or /gsd-plan-phase 3.
Resume file: None
