# Roadmap: rsgui

## Overview

Eight phases to v1.0, structured as Vertical MVP slices. Each phase
delivers a coherent capability that the lib can demo. The P0
"robust" gates (incremental reconciliation, IME, multi-window,
render-path stress) are front-loaded; the P1 feature surface
(theme v2, animation, DnD) is middle; the P1 polish (virtualization,
canvas, diagnostics, i18n, custom widget) is the tail.

Granularity: **Fine** (8 phases, 3-5 plans each, MVP mode for
each phase's emit).

## Phases

- [x] **Phase 1: Incremental Reconciliation** — runtime updates only changed subtrees; layout runs incrementally
- [x] **Phase 2: Event & Input Hardening** — Tree/List/Tabs/Slider/Switch keyboard nav, full integration test coverage
- [x] **Phase 3: Text & IME** — preedit composition under real drivers, RTL / CJK shaping
- [x] **Phase 4: Multi-Window** — per-window `UiRuntime`, event routing, isolated snapshots (completed 2026-06-04)
- [ ] **Phase 5: Render Path Stress** — Vulkan + Metal goldens, validation layers, frame budget tests
- [ ] **Phase 6: Public API Hardening** — doctests, no-`unwrap` audit, custom widget contract
- [ ] **Phase 7: Theme v2 + Animation + DnD** — `ComponentTheme` lookup, tween/spring, drag source + drop target
- [ ] **Phase 8: Virtualization + Canvas + i18n + Docs** — windowed lists/tables, canvas layout, RTL, full doc polish

## Phase Details

### Phase 1: Incremental Reconciliation

**Goal**: The runtime diffs a new `Element` root against the prior `UiTree` and updates only changed subtrees; layout runs incrementally on dirty regions.
**Mode:** mvp
**Depends on**: Nothing (first phase)
**Requirements**: RECON-01, RECON-02, RECON-03, RECON-04
**Success Criteria** (what must be TRUE):

  1. Re-rendering the same `Element` tree for 100 frames costs < 1ms per frame on a 50-widget tree
  2. Changing one widget's label updates only that widget's paint path
  3. A node removed from the tree releases its `PointerCapture` automatically
  4. A widget whose `WidgetSpec` hash changes is re-mounted; otherwise its existing state survives

**Plans**: 4 plans

Plans:

- [x] 01-01: `Reconciler::diff` between prior and new `UiTree`
- [x] 01-02: Dirty-region tracking on `LayoutBox`; `taffy` partial re-layout
- [x] 01-03: Pointer-capture release on node removal
- [x] 01-04: `WidgetSpec` hash → re-mount; existing state preserved otherwise

### Phase 2: Event & Input Hardening

**Goal**: Every interactive widget has full keyboard navigation and is covered by the event-dispatch integration test suite.
**Mode:** mvp
**Depends on**: Phase 1
**Requirements**: EVNT-01, EVNT-02, EVNT-03, EVNT-04, EVNT-05, EVNT-06
**Success Criteria** (what must be TRUE):

  1. Tree / List / Tabs / Slider / Switch all pass integration tests for keyboard nav (arrow keys, Home / End, Tab)
  2. ProgressBar documents its no-op click behavior in a unit test
  3. Every interactive widget has at least one event-dispatch integration test (today: Button / Checkbox / Input / Select / Modal — five more needed)
  4. `cargo test` runs the new tests on the existing wgpu headless backend without changes

**Plans**: 4 plans

Plans:

- [x] 02-01: Tree / List keyboard navigation
- [x] 02-02: Tabs keyboard navigation (arrow keys + Ctrl-Tab)
- [x] 02-03: Slider + Switch keyboard + drag handlers
- [x] 02-04: ProgressBar click contract + integration test gaps closed

### Phase 3: Text & IME

**Goal**: Text input handles IME preedit composition under real drivers; text shaping round-trips for Latin, CJK, and one RTL script; the per-thread measure cache is observable.
**Mode:** mvp
**Depends on**: Phase 1
**Requirements**: TEXT-01, TEXT-02, TEXT-03, TEXT-04
**Success Criteria** (what must be TRUE):

  1. Typing into an `Input` with Windows IME (Chinese Pinyin) and macOS IME (Japanese Kana) both produce correct preedit + commit
  2. Text shaping with CJK + Arabic content renders without errors on the wgpu headless backend
  3. `TextCacheStats` shows > 80% hit rate on a stable-text benchmark
  4. `clear_metrics_cache()` is a public API and a unit test verifies it zeros the cache

**Plans**: 3 plans
Plans:

- [x] 03-01: IME preedit on Windows + macOS drivers
- [x] 03-02: CJK + RTL shaping in `glyphon` integration
- [x] 03-03: `TextCacheStats` observability + `clear_metrics_cache` public API

### Phase 4: Multi-Window

**Goal**: Multiple `UiRuntime` instances can coexist in one process; events route to the correct window; each window has its own `DisplayList` and `UiSnapshot`.
**Mode:** mvp
**Depends on**: Phase 1
**Requirements**: WIN-01, WIN-02, WIN-03, WIN-04
**Success Criteria** (what must be TRUE):

  1. An example app with two windows runs; clicks in window A do not affect window B
  2. `FrameInput.window_id` is a documented public field
  3. `UiRuntime` is `Send + Sync` (single runtime, multi-threaded host loop)
  4. Closing one window does not invalidate the other window's snapshot

**Plans**: 3 plans

Plans:

- [x] 04-01: `FrameInput.window_id` + per-window event routing
- [x] 04-02: Multi-runtime coexistence; `Send + Sync` on the runtime
- [x] 04-03: Per-window `DisplayList` + `UiSnapshot` isolation

### Phase 5: Render Path Stress

**Goal**: The wgpu render path is stable under real-world load; visual goldens pass on at least two GPU backends; the frame CPU budget is met for a 50-widget desktop UI.
**Mode:** mvp
**Depends on**: Phase 1
**Requirements**: REND-01, REND-02, REND-03, REND-04
**Success Criteria** (what must be TRUE):

  1. Visual goldens pass on Vulkan AND Metal (or Vulkan AND DX12) CI runners
  2. A stress-test scene (overlapping translucent layers, 10k-row windowed list, animated progress bars, IME, drag-and-drop) renders without errors
  3. Vulkan validation layers enabled in CI surface zero new buffer-alignment issues
  4. A 50-widget desktop UI's frame CPU budget is < 8ms on a modern laptop

**Plans**: 4 plans

Plans:

- [x] 05-01: Visual goldens on a second GPU backend
- [ ] 05-02: Stress-test scene in `examples/`
- [ ] 05-03: Vulkan validation layers in CI
- [ ] 05-04: Frame budget micro-benchmark test

### Phase 6: Public API Hardening

**Goal**: The public API is documented with doctests, the runtime paint path has no `unwrap()` under non-pathological inputs, and the `WidgetPainter` extension contract is stable.
**Mode:** mvp
**Depends on**: Phase 1
**Requirements**: API-01, API-02, API-03, API-04
**Success Criteria** (what must be TRUE):

  1. `cargo doc --document-private-items` builds with zero warnings
  2. Every public type in the `rgui` crate root has a doctest
  3. A grep for `unwrap()` in the runtime paint path returns zero hits
  4. `register_widget_painter` and `unregister_widget_painter` are documented public API with a "writing a custom widget" guide

**Plans**: 3 plans

Plans:

- [ ] 06-01: Doctests for every public type at the crate root
- [ ] 06-02: Audit + remove `unwrap()` in the runtime paint path
- [ ] 06-03: `WidgetPainter` extension contract + "writing a custom widget" guide

### Phase 7: Theme v2 + Animation + DnD

**Goal**: Per-widget style overrides go through `ComponentTheme`; tween + spring animations are public; drag-and-drop works across widgets.
**Mode:** mvp
**Depends on**: Phase 6
**Requirements**: THEM-01, THEM-02, THEM-03, THEM-04, ANIM-01, ANIM-02, ANIM-03, ANIM-04, DND-01, DND-02, DND-03, DND-04
**Success Criteria** (what must be TRUE):

  1. `theme.components.get(WidgetKind::Button)` overrides a button's style; flat `Theme::metrics` is deprecated
  2. An element's `Length` and `Color` can be animated with tween or spring
  3. A `Drag` source + `DropTarget` work across widgets with a payload
  4. Light / dark / high-contrast themes ship out of the box

**Plans**: 5 plans

Plans:

- [ ] 07-01: `ComponentTheme` lookup path + flat-field deprecation
- [ ] 07-02: Tween animations on `Length`, `Color`, `f32` style values
- [ ] 07-03: Spring animations with configurable stiffness / damping
- [ ] 07-04: `Drag` / `DropTarget` API + drop indicator
- [ ] 07-05: High-contrast theme variant + theme v2 migration guide

### Phase 8: Virtualization + Canvas + i18n + Docs

**Goal**: Large lists and tables are windowed; canvas widgets use absolute-coordinate layout; RTL works; docs are complete and current.
**Mode:** mvp
**Depends on**: Phase 7
**Requirements**: VIRT-01, VIRT-02, VIRT-03, VIRT-04, LOUT-01, LOUT-02, LOUT-03, DIAG-01, DIAG-02, DIAG-03, I18N-01, I18N-02, CUST-01, CUST-02, CUST-03
**Success Criteria** (what must be TRUE):

  1. A 10k-row `WindowedList` renders at 60 fps on a modern laptop
  2. A `Canvas` widget composes with flex / grid siblings without layout break
  3. RTL `taffy` layout flips correctly for an Arabic-language test scene
  4. `UiSnapshot` JSON schema is locked by a test; `to_debug_json()` round-trips safely
  5. `docs/public-api.md` is fully current (reflects the post-3.8 / post-8.5 cleanup)

**Plans**: 4 plans

Plans:

- [ ] 08-01: `WindowedList` + `WindowedTable` specs (sticky headers, multi-column)
- [ ] 08-02: `Canvas` widget + `ZStack` primitive
- [ ] 08-03: RTL `taffy` integration test
- [ ] 08-04: Doc polish (full pass against current public API)

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Incremental Reconciliation | 4/4 | Complete | 2026-06-03 |
| 2. Event & Input Hardening | 4/4 | Complete | 2026-06-03 |
| 3. Text & IME | 3/3 | Complete | 2026-06-04 |
| 4. Multi-Window | 3/3 | Complete    | 2026-06-04 |
| 5. Render Path Stress | 1/4 | In Progress|  |
| 6. Public API Hardening | 0/3 | Not started | - |
| 7. Theme v2 + Animation + DnD | 0/5 | Not started | - |
| 8. Virtualization + Canvas + i18n + Docs | 0/4 | Not started | - |

---
*Roadmap created: 2026-06-03*
