# Phase 1: Incremental Reconciliation - Context

**Gathered:** 2026-06-03
**Status:** Ready for planning
**Source:** Inline synthesis (GSD `gsd-project-researcher` / `gsd-phase-researcher` / `gsd-pattern-mapper` subagents unavailable in this runtime; reasoning done by the orchestrator against the actual codebase)

<domain>

## Phase Boundary

The runtime currently builds a fresh `UiTree` from the `Element`
root on every `update()` and re-runs the full paint / layout /
hit-test pipeline against the new tree. That's the cost ceiling
today — every frame pays the full cost of reconciliation, layout,
and paint even when the UI hasn't changed.

This phase delivers **incremental reconciliation**: the runtime
diffs a new `Element` root against the prior `UiTree` and updates
only changed subtrees. Layout runs incrementally on dirty regions.
Pointer captures held against removed nodes are released. Widget
spec hash changes trigger a re-mount of the affected node;
otherwise existing per-node state survives.

The phase is the floor of "robust" in the PRD. Without it, every
other capability — themes, events, a11y — still pays the
full-rebuild tax every frame, which is what blocks a v1.0
performance claim.

</domain>

<decisions>

## Implementation Decisions

The decisions below are derived from `PROJECT.md` and the
`prd.md` "What Needs To Be A Robust wgpu GUI Lib" P0 list. They
are locked unless explicitly marked otherwise.

### Tree diffing strategy

- **RECON-01: Tree diffing algorithm.** The reconciler walks
  the prior and new trees in parallel, keyed by node id
  (or by key when present, with id as fallback). A node that
  exists in both with the same `WidgetSpec` hash and children
  list is reused; a node whose spec hash changed is re-mounted
  (existing children reset); a node only in the new tree is
  mounted; a node only in the old tree is unmounted.
- **RECON-01: Subtree reuse.** If a node's spec hash and
  children list are both unchanged, the existing `UiNode`,
  `LayoutBox`, and widget-specific state (`ScrollState`, `BoolState`,
  pointer capture) are preserved verbatim. Only the visual
  properties that genuinely changed (e.g. `label` of a button)
  are patched in-place.
- **RECON-01: Child ordering.** Children are matched by
  position (index). Reordering children is treated as
  "remove + insert" — keyed matching is a v1.x optimization
  once we have a stable key system.

### Layout

- **RECON-03: Layout dirty regions.** A node is "layout dirty"
  if it was mounted, unmounted, or had its `style` (lengths,
  padding, gap, etc.) changed. Its children are layout-dirty
  transitively. Layout runs the standard taffy pass on the
  dirty subtree, not the whole tree. The first phase is OK
  with re-running taffy over the full dirty region; per-node
  cached layout is a v1.x follow-up.
- **RECON-03: Cache keys.** Layout cache is keyed on the
  `WidgetSpec` hash + style + available width. Hit rate > 50%
  on a stable frame is the bar; the cache lives in `runtime/state.rs`
  as `LayoutCache: HashMap<NodeId, LayoutBox>`.

### Pointer capture

- **RECON-02: Capture release on unmount.** When the reconciler
  unmounts a node, it checks `PointerCapture` for any keys
  matching that node and releases them. The release emits a
  synthetic `PointerEvent::Cancel` so the receiving widget
  can clean up drag state.
- **RECON-02: Capture by key, not by node id.** Today's
  `PointerCapture` is keyed by `String` (the key the user
  set on the element). After re-mount, the same key captures
  the same logical slot; the reconciler preserves the
  pointer-capture entry across a re-mount if the key survives.
  If the key is removed entirely, the capture is released.

### Widget spec hash

- **RECON-04: Hash function.** `WidgetSpec` derives `Hash` (it
  doesn't today; this is a small new derive). The hash is
  computed via `std::collections::hash_map::DefaultHasher` for
  the spec's structural data (kind + payload). Style changes
  do *not* trigger a re-mount; they trigger a partial repaint
  (RECON-01's visual-properties patch).
- **RECON-04: Hash collision tolerance.** A hash collision
  (extremely unlikely with the spec sizes we have) would
  incorrectly mark a node as unchanged. We accept this risk
  for v1; the spec count is small (~25 kinds × small payloads).

### What this phase does NOT do

- ✗ **No per-component reconciliation.** All widgets
  reconcile against their spec + style + children. The
  `Component` element kind is a v1.x follow-up.
- ✗ **No child reordering by key.** Reordered children
  fall back to remove + insert at the new position.
- ✗ **No per-node cached layout.** Phase 1 runs taffy over
  the full dirty region. Per-node cached layout is a v1.x
  follow-up.
- ✗ **No animation system.** Reconciliation just *updates*
  values; transitions are a v1.x follow-up.

</decisions>

<canonical_refs>

## Canonical References

**Downstream agents and executors MUST read these before
planning or implementing.**

### Project context
- `.planning/PROJECT.md` — full project context, Validated / Active / Out-of-scope requirements
- `.planning/prd.md` — the original idea document with the P0/P1/P2/P3 priority lists
- `.planning/REQUIREMENTS.md` — the full requirement list with REQ-IDs
- `.planning/ROADMAP.md` — the 8-phase roadmap

### Research grounding
- `.planning/research/PITFALLS.md` — pitfall #1 (full-rebuild reconcile) is the reason this phase exists
- `.planning/research/ARCHITECTURE.md` — the 5-stage pipeline and component boundaries
- `.planning/research/FEATURES.md` — the table-stakes feature list

### Code to read
- `src/runtime/runtime.rs` — the orchestrator (the `update()` entry point)
- `src/runtime/tree.rs` — `UiTree` + `UiNode` + the `AncestorIds` iterator
- `src/runtime/state.rs` — `BoolState`, `DragState`, `PointerCapture`, `ScrollState` (the per-node state)
- `src/runtime/reconcile.rs` — the existing partial `Reconciler` (skeleton to extend)
- `src/core/element.rs` — `Element`, `ElementKind`, `WidgetSpec`
- `src/widgets/spec.rs` — the 25+ `*Spec` structs that need `Hash` derives
- `src/runtime/dirty.rs` — `DirtyFlags` (the bitflag infrastructure already in place)

</canonical_refs>

<specifics>

## Specific Ideas

- The reconciler should be a method on `UiRuntime` (or a
  dedicated `Reconciler` struct) that takes a `&mut UiTree`
  and `&Element` and produces an updated `UiTree`. The
  per-frame loop in `runtime.rs` calls it before layout.
- A useful debug API: `reconciler.stats()` returns a struct
  with `mounted`, `unmounted`, `reused`, `re_mounted` counts.
  This becomes part of `UiSnapshot` (DIAG-01 in the
  requirements).
- The visual golden test for reconciliation: a snapshot test
  that builds a 50-widget tree, renders it, mutates one
  widget's label, renders again, and asserts the paint diff
  touches only the one widget. This becomes the canonical
  performance regression test for the phase.

</specifics>

<deferred>

## Deferred Ideas

- Per-node cached layout (a separate phase after v1)
- Child reordering by key (v1.x)
- Animation system integration with reconciliation (v1.x)
- Generic `Component` element reconciliation (v1.x)
- Cross-window reconciliation (Phase 4 multi-window)

</deferred>

---
*Phase: 01-incremental-reconciliation*
*Context gathered: 2026-06-03 via inline synthesis (GSD subagents unavailable in this runtime)*
