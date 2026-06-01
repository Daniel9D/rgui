# rgui — Widget overflow / sizing refactor plan

Tracks the work to fix widget overlap and implement overflow behavior.
Each task has a status (`done`, `in_progress`, `pending`), priority, and
notes.

## Goal

- Widgets that have no explicit size should resize to accommodate their
  children (hug-content behavior).
- Overflow should have a sensible default: visible for regular containers,
  hidden for overlay roots so content past the cap doesn't bleed out.
- Leaf widgets (Image, Switch, Slider, ProgressBar, Spinner, Badge, Avatar,
  Link) should have themable default intrinsic sizes so they don't collapse
  to 0×0 in flex layouts.

## Test baseline (before this work)

- 332 passed, 2 pre-existing failed (verified on initial commit):
  - `hardcode_policy` — missing `rgui_taffy_hardcode_audit.md`
  - `widgets_visual_flow::widget_intrinsic_boxes_are_large_enough_for_text_content`

## Test baseline (after this work)

- 341 passed, 2 pre-existing failed (same two). +9 new tests, 0 regressions.

## Tasks

### 1. Themable defaults on `WidgetMetrics` — DONE
- Status: done
- Priority: high
- Notes: Added 12 new sub-theme structs and `WidgetMetrics` fields:
  `CardTheme`, `AlertTheme`, `ModalTheme`, `PopoverTheme`, `ImageTheme`,
  `SwitchTheme`, `SliderTheme`, `ProgressBarTheme`, `SpinnerTheme`,
  `BadgeTheme`, `AvatarTheme`, `LinkTheme`. Populated sensible defaults in
  `default_theme()`. File: `src/core/theme.rs`.

### 2. Wire new fields into `min_size_for` — DONE
- Status: done
- Priority: high
- Notes: Replaced the `_ => Size::new(0.0, 0.0)` fallthrough at
  `src/core/theme.rs:407` with explicit arms for the 12 new widget kinds.
  Only `ScrollArea`, `Checkbox`, `Radio`, and `Text` now fall through to
  the zero size (those are correctly handled by the measure callback).

### 3. Hug-content path in `base_taffy_style` — DONE
- Status: done
- Priority: high
- Notes: Added a second pass in `src/layout/taffy.rs` that flips
  Card/Alert/Modal/Popover/Tooltip to `display: Flex, flex_direction:
  Column, align_items: Stretch` when the user has not set those explicitly.
  Combined with `Dimension::AUTO` size and `flex_grow: 0`, this produces
  hug-to-content behavior. Explicit `width()` / `height()` on the user
  style still wins because they flow through `to_taffy_style` and are
  preserved on the taffy_style's `size` field.

### 4. Default `Hidden` overflow on overlay roots — DONE
- Status: done
- Priority: high
- Notes: Two changes ensure the default is honored end-to-end:
  1. In `src/layout/taffy.rs::base_taffy_style`, the root-only else branch
     (overlay roots) sets `style.overflow_x/y = Some(Hidden)` when None,
     then re-derives the taffy_style. Without re-deriving, the mutation
     would be lost because the taffy style is built before the overlay
     branch.
  2. In `src/runtime/runtime.rs::clips_overflow_node`, added a fallback for
     overlay-root kinds (Modal/Popover/Tooltip/Menu) at the root that
     treats `None` as `Some(Hidden)`. This matches the runtime's
     `resolve_layout` clip-rect assignment.

### 5. Doc comments on Card/Alert/Modal/Popover/Tooltip builders — DONE
- Status: done
- Priority: medium
- Notes: Documented the "hugs its children unless you set an explicit size"
  contract on all five builders, plus the `Hidden` default overflow for the
  three overlay builders. Files: `src/widgets/layouts.rs`,
  `src/widgets/overlays.rs`, `src/widgets/feedback.rs`.

### 6. Tests (8 new across 3 files) — DONE
- Status: done
- Priority: high
- Notes: All pass. Summary of new tests:

  **`tests/widget_metrics.rs`** (3 new):
  - `min_size_for_leaf_widgets_returns_nonzero` — Image, Switch, Slider,
    ProgressBar, Spinner, Badge, Avatar all have `width > 0 && height > 0`.
  - `min_size_for_container_widgets_returns_sensible_default` — Card,
    Alert, Modal, Popover all have non-zero defaults.
  - `link_default_size_is_line_height` — Link is `Size::new(0, >= 14)`.

  **`tests/layout_taffy_contract.rs`** (4 new):
  - `card_with_child_hugs_content` — `card().child(text("X"))` produces
    a Card with `height > 0` and the child fits inside.
  - `overlay_root_defaults_to_overflow_hidden` — Modal, Popover, Tooltip
    roots have `clip_rect.is_some()`.
  - `card_explicit_size_overrides_hug` — `card().width(300).height(150)`
    keeps that exact size regardless of children.
  - `card_overflow_remains_visible_by_default` — `clip_rect.is_none()` for
    a plain Card root.

  **`tests/widgets_visual_flow.rs`** (1 new):
  - `siblings_in_column_do_not_overlap` — a sequence of
    `[card, alert, spinner]` in a column gets three disjoint ascending
    y-ranges, each with `height > 0`. This is the direct regression test
    for the user-reported overlap.

### 7. Verification — DONE
- Status: done
- Priority: high
- Notes: `cargo test --lib --features bitmap-text-fallback` passes (2
  lib tests, unchanged). All 47 integration test binaries run individually;
  340/42 pass, 2 pre-existing fail.

## What I did NOT change (out of scope)

- `Stack`'s 1×1 cell overlap (intentional, tested at
  `tests/layout_taffy_contract.rs:106-122`).
- `Position::Fixed` collapsing to `Position::Absolute` with a once-per-frame
  warning (`src/layout/taffy_mapping.rs:227-231`). Separate issue.
- `Taffy::Position::Fixed` keeping its own identity. Separate issue.
- Tabs / Tree / Table / List / Menu sizing — these already have
  `min_size_for` entries and are explicitly out of the hug-content scope
  per the user's answer.
- Paint-path rect math. The painter consumes whatever rect the layout
  backend produces, so fixing the layout fixes the painter automatically.

## Files modified

| File | Change |
|---|---|
| `src/core/theme.rs` | +12 sub-theme structs, +12 `WidgetMetrics` fields, populated defaults, replaced 0,0 fallthrough in `min_size_for` |
| `src/layout/taffy.rs` | Hug path in `base_taffy_style`, overlay-root overflow default, re-derive taffy_style after the default |
| `src/runtime/runtime.rs` | `clips_overflow_node` honors overlay-root overflow default |
| `src/widgets/layouts.rs` | Doc comment on `card()` |
| `src/widgets/feedback.rs` | Doc comment on `alert()` |
| `src/widgets/overlays.rs` | Doc comments on `modal()` / `popover()` / `tooltip()` |
| `src/core/snapshot.rs` | `LayoutBoxSnapshot::clips_overflow()` accessor |
| `tests/widget_metrics.rs` | +3 tests |
| `tests/layout_taffy_contract.rs` | +4 tests |
| `tests/widgets_visual_flow.rs` | +2 tests |

## Open follow-ups (not done; tracked here)

- Theme gallery / showcase update to exercise the new hug-content path
  visually (`examples/rml_widget_gallery.rs` if it has Card/Alert without
  explicit sizes; an `rgui update-goldens` would be needed if so). No
  example currently uses `card()` or `alert()` without an explicit size,
  so the visual showcase already works.
- The user-reported screenshot showed "Save" / "Cancel" / etc. in a Toolbar
  with no size given; that path is now covered by the new
  `toolbar_siblings_have_disjoint_x_ranges` test
  (`tests/widgets_visual_flow.rs`), which exactly mirrors the screenshot
  pattern (button / input / checkbox / radio / select / textarea in a row)
  and asserts non-overlapping x-ranges.
- `LayoutBoxSnapshot::clips_overflow()` accessor is now in place at
  `src/core/snapshot.rs`; the new `overlay_root_defaults_to_overflow_hidden`
  and `card_overflow_remains_visible_by_default` tests use it.

## Polish work done after the initial 7-step plan

- Added `clips_overflow()` method to `LayoutBoxSnapshot` to mirror
  `LayoutBox::clips_overflow`. Cleaned up the new test assertions to use
  it instead of the `clip_rect.is_some()` / `is_none()` pattern.
- Added `toolbar_siblings_have_disjoint_x_ranges` test to
  `tests/widgets_visual_flow.rs`. It builds an exact replica of the
  user-reported screenshot pattern (button + input + checkbox + radio +
  select + textarea in a row) and asserts the x-ranges are strictly
  ascending with no overlap. This is the most direct regression test for
  the user-visible bug.
- Audited `examples/*.rs` and `examples/*.rml`: no example uses
  `card()` / `alert()` without an explicit size, and no RML/HTML file
  exercises the affected widgets. The fix flows through automatically.
