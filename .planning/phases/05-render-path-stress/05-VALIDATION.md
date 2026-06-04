---
phase: 05
slug: render-path-stress
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-04
---

# Phase 5 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) — `lib` tests + `tests/*.rs` integration tests |
| **Config file** | `Cargo.toml` (workspace) |
| **Quick run command** | `cargo test --lib` (~2s) |
| **Full suite command** | `cargo test --lib --test visual_goldens --test stress_scene --test frame_budget --test multi_window_coexistence --test multi_window_event_routing --test multi_window_snapshot_isolation --test multi_window_shared_a11y --test render_wgpu_render_items --test render_wgpu_offscreen_render` (~10s) |
| **Estimated runtime** | ~10 seconds (lib + targeted integration tests) |

> **Note:** the existing pre-Phase-5 test errors (in `tests/interactive_widgets.rs`, `tests/rml_attribute_matrix.rs`, `tests/render_validation.rs`) are **out of Phase 5 scope** (confirmed in `STATE.md`). They will not block Phase 5 validation.

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib` (lib tests; ~2s)
- **After every plan wave:** Run the full suite command above
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~10 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 05-01-01 | 01 | 1 | REND-01 | — | N/A | integration | `cargo test --test visual_goldens_vulkan` (or equivalent second-backend test) | ❌ W0 | ⬜ pending |
| 05-02-01 | 02 | 1 | REND-02 | — | N/A | integration | `cargo test --test stress_scene` | ❌ W0 | ⬜ pending |
| 05-03-01 | 03 | 1 | REND-03 | — | N/A | build + ci | `cargo build --features validation-layers` + `.github/workflows/ci.yml` runs | ❌ W0 | ⬜ pending |
| 05-04-01 | 04 | 2 | REND-04 | — | N/A | integration | `cargo test --test frame_budget` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/visual_goldens_vulkan.rs` (or `#[cfg(...)]` gate inside `tests/visual_goldens.rs`) — second-backend test (plan 05-01)
- [ ] `tests/stress_scene.rs` — headless stress test (plan 05-02)
- [ ] `.github/workflows/ci.yml` — CI workflow with `validation-layers` feature (plan 05-03)
- [ ] `tests/frame_budget.rs` — 50-widget frame-budget test (plan 05-04)

*Existing infrastructure covers the rest: `WgpuRenderer::new_headless_for_tests`, `RendererOptions.backends`, `pixel_diff_stats`, `PerformanceMetrics.display_command_count`, `output.stats.command_count`, `output.display_list.commands().len()`.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| None | — | — | — |

*All Phase 5 behaviors have automated verification (cargo test + CI).*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
