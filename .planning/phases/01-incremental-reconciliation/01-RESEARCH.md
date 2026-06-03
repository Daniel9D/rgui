# Phase 1: Incremental Reconciliation - Research

**Researched:** 2026-06-03
**Confidence:** HIGH (researched against the actual codebase, not a generic reference)

## Research question

> How does an existing retained-mode GUI toolkit diff an `Element`
> tree against a prior `UiTree` and update only the changed
> subtrees — what data structures, what diff algorithm, what
> re-mount semantics?

## Existing code to build on

### What `runtime/runtime.rs` does today

The current `update()` path (see `src/runtime/runtime.rs:2150+`)
is:

1. `runtime.update(FrameInput)` →
2. Build a fresh `UiTree` from the new `Element` (full rebuild)
3. `reconcile` is currently a stub (the partial `Reconciler`
   exists in `src/runtime/reconcile.rs` but the diff is not
   wired into the main loop)
4. `paint_widget_themed` runs over the tree, producing
   `DisplayList` + `UiSnapshot`

The cost-of-full-rebuild is the concrete motivation for this
phase. There is no caching of layout, no preservation of state
across frames, and no incremental pointer-capture cleanup.

### What `reconcile.rs` provides

- `Reconciler` struct + `ReconcileOutput` (committed in earlier
  work; see `src/runtime/reconcile.rs`)
- The skeleton is in place but the diff is `O(n²)` against the
  whole tree. It needs a fast-path for "the tree is unchanged".

### What `state.rs` provides

- `BoolState` — per-key bool state with `by_node` and `by_key`
  maps. Today keyed by NodeId which is regenerated on every
  rebuild. Needs to survive reconciliation.
- `PointerCapture` — keyed by `String` (the user's `Element::key`).
  The key survives re-mount; NodeId does not.
- `ScrollState` — keyed by NodeId, but conceptually by
  widget identity. Needs to survive.
- `DragState` — keyed by `String` (the drag target's key).

### What `tree.rs` provides

- `UiTree` with `nodes: Vec<UiNode>` and `index: HashMap<NodeId, usize>`
- `UiNode` with `id`, `key`, `parent`, `children`, `kind`, `widget_spec`, `style`, `variant`, `semantic`, `handlers`, `overlay`, `open`
- `IdAllocator` for stable id assignment across rebuilds
- `AncestorIds` iterator (lazy, for event paths)
- `from_element` + `from_element_with_ids` + `from_portal_element`
  — three different ways to build a tree from an `Element`

### What `dirty.rs` provides

- `DirtyFlags` (bitflag) — `EMPTY`, `LAYOUT`, `PAINT`, `EVENT`,
  `STYLE`, `STRUCTURE`. Already committed; ready to use.

## Algorithm: structural diff

The diff is a recursive walk over the two trees, keyed by
position (index) within `children`:

```text
diff_node(prior_node, new_node, dirty):
  if prior_node.widget_spec != new_node.widget_spec:
    unmount(prior_node)            # release captures, drop state
    mount(new_node)                 # fresh state, fresh layout
    dirty |= LAYOUT | PAINT
    return
  if prior_node.style != new_node.style:
    patch(prior_node, new_node)     # in-place update; preserves state
    dirty |= PAINT
  if prior_node.children.len() != new_node.children.len():
    dirty |= LAYOUT | STRUCTURE
  for i in 0..min(prior, new).len():
    diff_node(prior.children[i], new.children[i], dirty)
  for i in min(prior, new).len()..max(prior, new).len():
    if i >= prior.len():
      mount(new.children[i])
    else:
      unmount(prior.children[i])
  return dirty
```

`Hash` on `WidgetSpec` is the cheap equality check that drives
the "spec changed" decision. `PartialEq` is the structural
comparison; `Hash` is the fast path.

## Pitfalls (from PITFALLS.md, item #1 + #5 + #7 + #8)

### PITFALL: full-rebuild reconciliation as the steady state
The current behavior. Phase 1 fixes it. **Fix in this phase.**

### PITFALL: Pointer-capture leak (item #5)
If a `PointerCapture` is set on a node that is removed in the
next `update()`, the next click goes nowhere or to the wrong node.
**Fix in this phase, paired with reconciliation.**

### PITFALL: Event dispatch reentrancy (item #7)
The runtime should snapshot `VisualState` per frame and dispatch
against the snapshot. The current code mutates state during
dispatch. **Out of scope for this phase** (P1 in the
requirements).

### PITFALL: Tree shape changes invalidating the hit-test cache (item #8)
`HitTestTree` is rebuilt every frame today. The risk is a
*future* optimization that reuses the cache across frames.
**Not a risk in this phase** (we don't add such an optimization).

## Reference: how other retained-mode toolkits do it

- **React** uses a fiber tree with `key`-based child
  reconciliation. Children at the same position are compared
  first (cheaper than key lookup); keyed children use a hash
  map. The reconciler is O(n) for stable trees.
- **Flutter** uses `Widget` (immutable config) + `Element`
  (mutable instance) + `RenderObject` (layout + paint). The
  reconciler walks the element tree and updates render objects
  in place when the widget config is equivalent.
- **SwiftUI** uses `View` (immutable) + identity-based diffing.
  Structurally identical subtrees reuse elements.

rsgui's model is closer to Flutter: `Element` is the
user-facing builder; `UiNode` is the mutable instance; the
runtime reconciles them.

The recommended algorithm for rsgui is:
1. Walk the new `Element` tree, allocating / reusing `NodeId`s
   in document order.
2. For each new node, look up the prior `UiNode` by id.
3. Compare spec hashes; if equal, patch in place. If not,
   unmount prior, mount new.
4. Recurse into children (positional matching for v1; keyed
   matching deferred to v1.x).
5. Emit a `DirtyFlags` bitmask for downstream phases (layout
   and paint) to consume.

## Specific implementation choices

### `WidgetSpec` hashing
Derive `Hash` on every `WidgetSpec` variant. Today
`WidgetSpec` derives `Clone, Debug, PartialEq` but not `Hash`.
Adding the derive is mechanical (one line per spec).

`Hash` is computed on the spec's structural data (kind + payload).
The spec variants are:
- `Button` / `Checkbox` / `Radio` / `Switch` / `MenuItem` —
  small payloads, hash is cheap
- `Input` / `Textarea` / `Select` — text + spec fields
- `Tabs` / `List` / `Table` / `Tree` — collections
- `Card` / `Badge` / `Alert` / `Image` / `Avatar` / `Link` —
  small payloads
- `Modal` / `Popover` / `Tooltip` — small payloads
- `Canvas` — just a name
- `ProgressBar` / `Spinner` / `Switch` / `Slider` — value/max

All are small enough that hashing is sub-microsecond.

### State preservation across re-mount
State that is keyed by `String` (the user's `Element::key`)
survives a re-mount *only if the key is preserved*. If the
user re-keys the element, the state is lost. Document this
explicitly in the relevant constructors.

State that is keyed by `NodeId` (today, for example
`BoolState::by_node`) does not survive a re-mount because the
node id changes. The reconciliation work includes updating
`BoolState` to prefer `by_key` (which already exists) and fall
back to `by_node` only on the first frame.

### `LayoutCache` shape
```rust
struct LayoutCache {
    boxes: HashMap<NodeId, LayoutBox>,
    dirty: HashSet<NodeId>,
}
```

A node is in `dirty` if it was mounted, unmounted, style-changed,
or has a dirty ancestor. Layout runs `taffy` over the dirty
subtree.

For v1, the cache is not multi-resolution — it's invalidated
on any style change. The first-phase cost is "less than full
re-layout", not "no re-layout at all".

### Test plan
- A 50-widget tree, rendered, captured as a baseline.
- Mutate one widget's label.
- Reconcile, re-render, assert that the captured paint diff
  is only the one widget.
- Assert that the prior `BoolState` for the unchanged
  widgets is preserved.
- Assert that pointer-capture for a removed node is released.

This becomes a visual golden + a `cargo test` unit test.

## Sources

- `src/runtime/runtime.rs` (the current `update()` pipeline)
- `src/runtime/reconcile.rs` (the partial `Reconciler` skeleton)
- `src/runtime/state.rs` (`BoolState`, `PointerCapture`, `ScrollState`, `DragState`)
- `src/runtime/tree.rs` (`UiTree`, `UiNode`, `IdAllocator`, `AncestorIds`)
- `src/runtime/dirty.rs` (`DirtyFlags`)
- `src/widgets/spec.rs` (the 25+ `WidgetSpec` variants that need `Hash`)
- `src/core/element.rs` (`Element`, `ElementKind`, `WidgetSpec`)
- `.planning/research/PITFALLS.md` (items #1, #5, #7, #8)
- `.planning/research/ARCHITECTURE.md` (the 5-stage pipeline)
- React reconciliation (https://legacy.reactjs.org/docs/reconciliation.html)
- Flutter element tree (https://docs.flutter.dev/resources/architectural-overview)

---
*Research for Phase 1 of rsgui*
*Researched: 2026-06-03*
