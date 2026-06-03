# Phase 3: Text & IME

**Phase:** 3 of 8
**Depends on:** Phase 1 (Incremental Reconciliation) — state preservation keyed on NodeId means an `Input`'s preedit can survive a parent patch; Phase 2 (Event & Input Hardening) — `ime_enabled` flag is the opt-in gate, focus traversal gets the cursor into a text input before IME ever fires.

## Phase Boundary

Phases 1 + 2 made the runtime efficient and well-behaved for keyboard / wheel. Phase 2 also wired the *receive* side of IME: `ImePreedit` / `ImeCommit` events are now routed to focused `Input` elements that have opted in via `ime_enabled(true)` (covered by `tests/ime_gating.rs`). What Phase 2 did *not* do:

- **No host driver abstraction.** Tests synthesize `UiEvent::ImePreedit` directly. Real host integration (winit's `WindowEvent::Ime`, AppKit's `NSTextInputClient`, web's `compositionupdate`) is left to each app. The "real drivers" success criterion is aspirational, not testable today.
- **No CJK / RTL shaping coverage.** `glyphon_text.rs` uses `Shaping::Advanced` and the default `Attrs::new().family(SansSerif)`. Tests only shape Latin. A CJK character returns zero glyphs on a host without an Asian font; RTL bidirectional text has no test pinning its behavior.
- **`TextCacheStats` is internal.** `TextSystem::cache_stats()` exists but isn't reachable through `UiRuntime` or `UiSnapshot`. An app that wants to log hit rate has to call it on the `TextSystem` directly, which is a private dependency.
- **`clear_metrics_cache()` is half-public.** It's a `pub fn` in `runtime::text_metrics`, but the per-thread `MetricsCache` it clears is the *heuristic* cache, not the *shape/layout* cache on `TextSystem`. The two paths have two unrelated clearing functions; an app doesn't know which to call.

This phase closes those gaps.

## Implementation Decisions (locked in this context)

1. **`ImeHostDriver` is a producer-side trait.** The runtime owns the trait object; each frame it calls `driver.poll(&mut sink)`, where `sink: &mut dyn ImeEventSink` is a thin abstraction the runtime implements to push `UiEvent::ImePreedit` / `UiEvent::ImeCommit` into the runtime's event queue. Real host drivers (winit, AppKit) are *not* part of v1 — apps that want them write a 30-line adapter. `MockDriver` ships in-tree for tests and replays a `Vec<ImeOp>` script. This mirrors winit's `EventLoop::run` poll pattern.
2. **No bundled fonts; rely on system fonts.** Tests assume Noto CJK (`fonts-noto-cjk` on apt, `noto-sans-cjk` on dnf) and Noto Arabic / Naskh are installed on the CI host. Documented in `CONTRIBUTING.md` and in a `scripts/ci-install-fonts.sh` that runs in CI. Skips the legal review of bundling CC-licensed fonts and keeps the repo small. Tests fail with a clear "Noto CJK not found" message if the host is missing fonts.
3. **Arabic is the v1 RTL reference script.** The shaping test pins three behaviors: (a) isolated form renders alone, (b) connected letters "بسم" produce joined glyphs with greater total width than the sum of isolates (contextual shaping actually happened), (c) mixed bidi "Hello بسم" renders with correct paragraph order (Latin LTR, Arabic RTL, no `NaN` width). Hebrew / Persian are v1.x follow-ups; the test surface generalizes once we add a second script.
4. **`TextCacheStats` surfaces in three places.** (a) `UiRuntime::text_cache_stats() -> TextCacheStats` public method, returning the shape+layout cache snapshot. (b) `UiSnapshot.text_cache: TextCacheStats` field, so debug JSON shows it. (c) Existing `RendererStats` (which already has `glyphon_enabled`) gains a `text_cache: TextCacheStats` field. The heuristic `text_metrics::clear_metrics_cache()` stays a free function; we add a sibling `text_metrics::metrics_cache_stats() -> TextMetricsCacheStats` (the struct already exists in `text_metrics.rs:93`) so the two caches are observable symmetrically. We do *not* unify the two caches — they have different keys and lifetimes, and merging them would force one cache to grow with the other's input domain.
5. **Preedit is held in the existing `InputState.preedit: Option<ImePreedit>` field.** No new state. The IME *driver* is the only new abstraction; everything below it already exists.
6. **`Shaping::Advanced` is the v1 default for all scripts.** `glyphon_text.rs:183` already uses it. The shaping test asserts that CJK + Arabic shape with this setting on real system fonts. (No `Shaping::Basic` fallback — Advanced is the correct choice for CJK + RTL.)

## Canonical References

- **`src/core/event.rs:49-74`**: `ImePreedit` struct, `UiEvent::ImePreedit` / `UiEvent::ImeCommit` variants — the events the driver sink pushes.
- **`src/core/event.rs:72-73`**: the two event variants the driver must be able to deliver.
- **`src/widgets/spec.rs:78-93`**: `InputSpec` with `ime_enabled: bool` (Phase 2) — the gate that decides whether the runtime accepts preedit for a given `Input`.
- **`src/runtime/runtime.rs:650-689`**: `handle_ime_preedit` and `is_focused_ime_enabled()` — the receive-side logic. Phase 3 leaves this alone; it just adds the driver that *produces* the events.
- **`src/state/input.rs:11`**: `InputState.preedit: Option<ImePreedit>` — where the preedit text is stored once the runtime accepts it. Already there.
- **`src/text_engine/system.rs:49-56`**: `TextCacheStats` struct (shape/layout hits, misses, entries).
- **`src/text_engine/system.rs:84-99`**: `TextSystem::cache_stats() -> TextCacheStats` — the source of truth for stats.
- **`src/runtime/text_metrics.rs:73-101`**: per-thread `METRICS_CACHE`, `clear_metrics_cache()`, `TextMetricsCacheStats`, `metrics_cache_stats()` — the heuristic path. We add the missing `pub fn` for the stats getter.
- **`src/render/wgpu/glyphon_text.rs:8-72`**: `GlyphonTextBridge` with `font_system: glyphon::FontSystem`, `cache: glyphon::Cache`, `atlas: glyphon::TextAtlas`. The CJK + Arabic shaping test exercises this path.
- **`src/render/wgpu/glyphon_text.rs:183`**: `glyphon::Shaping::Advanced` — the shaping strategy that must work for CJK + Arabic.
- **`src/render/wgpu/glyphon_text.rs:210`**: existing `RendererStats.glyphon_enabled: bool` — the field we extend with `text_cache: TextCacheStats`.
- **`tests/ime_gating.rs`**: 3 existing tests (preedit-routes / dropped / commit-always). Phase 3 extends, doesn't replace.
- **`src/runtime/text_metrics.rs:260-281`**: existing `clear_metrics_cache_evicts_everything` unit test for the heuristic path.

## Specific Ideas

- **Plan 03-01 (IME host driver)**:
  - New module `src/runtime/ime_host.rs` with:
    ```rust
    pub trait ImeHostDriver {
        /// Called once per frame. Implementations push events via the sink.
        fn poll(&mut self, sink: &mut dyn ImeEventSink);
    }
    pub trait ImeEventSink {
        fn preedit(&mut self, text: String, cursor: Option<(usize, usize)>);
        fn commit(&mut self, text: String);
    }
    ```
  - `ImeOp` enum for the test script: `Begin`, `Preedit(text, cursor)`, `Commit(text)`, `End`. `MockDriver { script: Vec<ImeOp>, cursor: usize }` impls `ImeHostDriver` by replaying one op per `poll()` call (or all-at-once on the first poll for a simple scripted test).
  - `UiRuntime` gains a `driver: Box<dyn ImeHostDriver>` field, defaulting to `Box::new(NoopDriver)` (an empty driver for apps that don't need IME).
  - The runtime's `update()` / frame pump calls `driver.poll(&mut self)` *before* processing the input event queue, so the driver-sourced events land in the same queue as host-sourced events.
  - `NoopDriver` keeps v1 backward-compatible: apps that don't construct an `ImeHostDriver` get nothing.
  - 4 new tests in `tests/ime_host_driver.rs`:
    1. `mock_driver_replays_preedit_then_commit` — driver emits `Preedit("konni")` then `Commit("こんにちは")`; assert the focused `Input` ends with `"こんにちは"` and `preedit` is `None`.
    2. `mock_driver_preedit_replaces_previous_preedit` — driver emits two `Preedit` ops in sequence; assert the second replaces the first (no concatenation).
    3. `noop_driver_emits_no_events` — `UiRuntime::default()` processes a frame; assert no `ImePreedit` / `ImeCommit` events were dispatched.
    4. `driver_events_ignored_when_focus_is_not_text_input` — driver emits `Preedit`; the focused widget is a `Button`; assert no preedit was stored on any `InputState` (the receive-side gate from Phase 2 is the second line of defense).

- **Plan 03-02 (CJK + Arabic shaping)**:
  - `tests/text_shaping_cjk_rtl.rs` — 3 tests:
    1. `cjk_string_shapes_to_glyphs_on_system_fonts` — `"日本語"` shapes successfully; `shaped.glyphs.len() > 0`; each glyph has a non-zero `x_advance`. Skip the test (with a `tracing::warn!`) if `fonts-noto-cjk` is not installed; CI is expected to install it.
    2. `arabic_isolated_letter_shapes_correctly` — single letter `"ب"` shapes alone; assert `glyphs.len() == 1`, glyph index is non-zero.
    3. `arabic_contextual_shaping_produces_joined_glyphs` — `"بسم"` shapes with `Shaping::Advanced`; assert *total width* of the three connected glyphs is greater than the sum of the three isolated letters' widths. This proves contextual substitution actually happened.
    4. `arabic_latin_bidi_renders_in_correct_order` — mixed string `"Hello بسم"`; assert no glyph has `NaN` x_advance, glyphs are in linear order along the baseline (no overlap), and total width > 0.
  - New helper `tests/text_shaping_helpers.rs` (or inline): `shape_string(text: &str) -> Vec<ShapedGlyph>` that drives `GlyphonTextBridge` with a 200x40 box. Refactor the existing Latin shaping test (if any) to use it.
  - `scripts/ci-install-fonts.sh` — one-line shell: `apt-get install -y fonts-noto-cjk fonts-noto` (with a docstring for `brew install --cask font-noto-sans-cjk font-noto-naskh-arabic` on macOS). Wired into the GitHub Actions CI job that runs the shaping tests.
  - `CONTRIBUTING.md` gains a "System fonts" section pointing at the script.

- **Plan 03-03 (TextCacheStats observability)**:
  - `UiRuntime::text_cache_stats(&self) -> TextCacheStats` — public method that returns `self.text_system.cache_stats()`.
  - `UiSnapshot` gains `pub text_cache: TextCacheStats` field. The existing `to_debug_json()` is updated to include it (alongside any other snapshot fields).
  - `RendererStats` gains `pub text_cache: TextCacheStats`. Existing visual-golden test machinery (which already snapshots `RendererStats`) is regenerated.
  - New `pub fn` in `runtime::text_metrics`: `pub fn text_metrics_cache_stats() -> TextMetricsCacheStats` (the struct exists at `text_metrics.rs:93`; only the `pub fn` is missing). Symmetric getter so apps can log both caches.
  - New `pub fn` in `runtime::text_metrics`: `pub fn clear_text_cache() -> usize` that delegates to the *shape/layout* cache (currently `TextSystem` has no public clear — add `TextSystem::clear_caches() -> (usize, usize)` returning `(shape_evicted, layout_evicted)` and expose it through runtime). Both caches are now clearable: `clear_metrics_cache()` for the heuristic, `clear_text_cache()` for the shape/layout.
  - 4 new tests in `tests/text_cache_observability.rs`:
    1. `text_cache_stats_reports_shape_hits_after_repeated_render` — call `runtime.update(&element_with_text)` 5 times with the same text; assert `stats.shape_hits >= 4` (first miss, subsequent 4 hits).
    2. `clear_text_cache_evicts_shape_and_layout_entries` — render text, then `runtime.clear_text_cache()`; assert `stats.shape_entries == 0` and `stats.layout_entries == 0`; hit counters unchanged.
    3. `ui_snapshot_includes_text_cache_stats` — call `runtime.snapshot()`; assert `snapshot.text_cache.shape_entries > 0` after a render.
    4. `text_metrics_cache_stats_reports_entries` — call `measure_text()` 3 times with different args; assert `metrics_cache_stats().entries == 3`.

## Deferred Ideas (out of scope for v1)

- **Winit / AppKit / browser IME adapters**: Apps integrate directly. v1.x ships a `winit` feature behind `cfg` that wires `WindowEvent::Ime` to `ImeHostDriver::poll`.
- **Hebrew + Persian + Urdu shaping**: Only Arabic. The test surface generalizes to RTL scripts with one new fixture per script.
- **South-East Asian complex-script state machine (Hindi, Thai, Khmer)**: Same as Phase 2 deferral. v1 covers CJK preedit-then-commit.
- **Preedit underline rendering**: `ImePreedit` carries the text but the paint path doesn't yet underline it. v1.x follows once a real driver shows the visual the user expects. (For v1 the preedit shows in the text field as plain text — same color as committed — which is correct, just unstyled.)
- **LRU eviction on `TextSystem` caches**: Plain `HashMap` is intentional. The shape/layout caches are bounded in practice by the number of distinct strings × (size, weight, style, width) combinations an app actually uses; for desktop UIs that's well under 10k entries. LRU is a v1.x follow-up if a real app shows runaway growth.
- **Cached font discovery**: cosmic-text's `FontSystem::new()` walks the system font db on every `TextSystem::default()`. For a 60fps app, this happens once. v1.x may cache the db handle if profiling shows it.

## How to know this phase is done

- A test using `MockDriver` simulates a Japanese IME: preedit `"konni"` arrives, then commit `"こんにちは"`; the focused `Input` shows the committed text, no preedit remains.
- A test shapes `"بسم"` and asserts the contextual (joined) glyphs are wider than the isolated letters' sum.
- A test shapes `"日本語"` and asserts non-zero glyphs (or skips with a warning if Noto CJK is missing).
- A test mixed-bidi-shapes `"Hello بسم"` and asserts the glyphs are in a sane order with no NaN widths.
- A test renders the same text 5 times and asserts `TextCacheStats.shape_hits >= 4`.
- A test calls `clear_text_cache()` and asserts the next render is a miss (shape_entries = 1 after a clear, then a re-render).
- All scenarios are covered by integration tests under `tests/ime_host_driver.rs`, `tests/text_shaping_cjk_rtl.rs`, and `tests/text_cache_observability.rs`.
- `cargo test --features rml,bitmap-text-fallback` shows ≥ 11 new tests, all green.
- `cargo doc --document-private-items` builds with zero new warnings (the new `ImeHostDriver` trait has rustdoc).

## Requirements covered

This phase closes the v1 requirements: **TEXT-01**, **TEXT-02**, **TEXT-03**, **TEXT-04**. See `REQUIREMENTS.md` for the full text. `TEXT-01` is upgraded from `Partial` to `Complete` once Plan 03-01 lands (runtime side done in Phase 2; host driver abstraction done in Phase 3).

## Risks

- **System fonts on CI are non-portable.** If a contributor runs the shaping tests on a host without Noto CJK, they get a skip-with-warning, not a failure. The risk is that a CI host quietly changes its base image and tests start skipping silently. Mitigation: the `tracing::warn!` in the skip path includes "set `RGUI_REQUIRE_FONTS=1` to fail instead of skip" for CI to opt into strict mode.
- **MockDriver can drift from real driver behavior.** A real IME delivers cursor positions, character-level updates, and per-character composition. MockDriver only models the high-level script. Mitigation: MockDriver's `ImeOp` variants are the *minimum* the runtime needs to handle; a winit adapter in v1.x will catch any gap.
- **`RendererStats` change affects visual goldens.** Adding a field to `RendererStats` may shift byte offsets in the existing test fixtures. Mitigation: the visual_goldens test in Phase 5 regenerates the affected fixtures, and Phase 3's plan explicitly includes a "rerun visual goldens" step.
- **Arabic bidi test is hard to assert without a real layout engine.** Glyphon + cosmic-text do shaping, not full bidi reordering. The test asserts "no NaN, glyphs in order, total width > 0", which is correct but minimal. A v1.x follow-up could integrate `unicode-bidi` to assert the paragraph direction.
- **`clear_text_cache` is a public API change in spirit, not in name.** The function didn't exist before. Apps that relied on unbounded cache growth (none in the test suite) will see eviction on explicit clear. Mitigation: clearing is opt-in; no implicit eviction.
