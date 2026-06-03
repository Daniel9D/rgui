<!-- GSD:project-start source:PROJECT.md -->

## Project

**rsgui**

A Rust GUI library for `wgpu` that gives desktop and embedded-wgpu
applications a retained-mode, GPU-accelerated, themeable, accessible
UI surface — `Element` tree in, `DisplayList` out, with a clean
separation between layout, paint, and event dispatch. Targeted at
developers who want the GPU story of `wgpu` without giving up the
ergonomics of a retained-mode toolkit.

**Core Value:** The paint pipeline produces a correct, sorted `DisplayList` for
every `Element` tree — every `WidgetKind` paints something visible
with the right z-order, the right hover/disabled/checked state, and
the right glyph from the right font. If paint is wrong, nothing
else matters; every other capability exists to make that output
useful.

### Constraints

- **Tech stack**: Rust + `wgpu` + `taffy` + `glyphon`. The retained-mode model + Rust-native style system is non-negotiable (the design's whole point).
- **Performance**: 60 fps for typical desktop UIs on integrated GPUs. Each frame's full pipeline (layout + paint + render) must fit in ~8ms of CPU budget on a modern laptop.
- **Compatibility**: Must build and run on Windows / macOS / Linux. Mobile / WASM are not v1 targets but the lib must not preclude them.
- **Backwards compatibility**: Public API changes need a deprecation alias in the same release. We don't have a 1.0 yet; pre-1.0 breaking changes need a CHANGELOG note and a migration entry in the docs.
- **No runtime dependencies** outside the chosen stack. No `tokio`, no `serde` for the hot path (serde is allowed in debug dumps only).
- **Test discipline**: Every interactive widget must be covered by the event dispatch integration test suite. No `unwrap()` in the runtime paint path under non-pathological inputs.

<!-- GSD:project-end -->

<!-- GSD:stack-start source:research/STACK.md -->

## Technology Stack

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Rust | 2024 edition | Host language | Memory safety + zero-cost abstractions + the strongest GPU binding story via `wgpu` |
| `wgpu` | 29 | GPU rendering | The de-facto Rust GPU API; Vulkan / Metal / DX12 / WebGPU backends; the only mature choice in the ecosystem |
| `taffy` | 0.10.1 | Layout | Flexbox + CSS Grid algorithm; the standard layout engine for Rust UIs (`egui`, `iced`, `druid` all evaluate taffy) |
| `glyphon` | 0.11.0 | Text shaping | The standard `wgpu`-native text shaper; integrates with `cosmic-text` for shaping and our `wgpu` render path for atlas upload |
| `kurbo` | 0.11 | 2D geometry math | The path / bezier math used by `piet` and `druid`; correct for canvas / vector primitives |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `bytemuck` | 1.25 | `Pod` / `Zeroable` derives | Required for `wgpu` vertex / index / uniform buffers |
| `image` | 0.25 | Image decode | `Image` widget source (PNG, JPEG, etc.) |
| `quick-xml` | 0.37 | RML parsing | Behind the `rml` feature; the lightweight choice for declarative XML |
| `serde` + `serde_json` | 1 | Debug dumps only | `to_debug_json()` for `UiSnapshot`; never on the hot path |
| `thiserror` | 2 | Error type derives | Used in `RendererError` and `DisplayListError` |
| `pollster` | 0.4 | Blocking-on-async helper | `wgpu::request_adapter` and `request_device` in tests; production code is async |
| `winit` | 0.30 | Windowing + events | The standard `wgpu` host loop |
| `arboard` | 3.4 | Clipboard | For cut / copy / paste on `Input` / `Textarea` |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| `cargo test --test <name>` | Per-test-file run | The 51 integration test files give fast feedback on a single surface area |
| `RGUI_UPDATE_GOLDENS=1 cargo test --test visual_goldens` | Update PNG baselines | Used when intentional paint changes ship |
| `cargo doc --document-private-items` | Catch missing docs | Will be wired into CI pre-v1 |
| `cargo clippy --all-targets` | Lint pass | Currently clean; will be a CI gate pre-v1 |

## Installation

# The stack is already pinned in Cargo.toml. To build:

# For tests (lib + visual goldens):

# For the examples:

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| `wgpu` | Direct Vulkan / Metal / DX12 | When you need backend-specific extensions that `wgpu` doesn't expose (rare for a GUI lib) |
| `taffy` | Custom flex impl | When you only need a subset and taffy's full algorithm is too heavy; not the case for rsgui |
| `glyphon` | Direct `cosmic-text` + custom atlas | When you need fine-grained atlas control; we already do this internally via `text_engine::TextSystem` |
| `wgpu` 29 | `wgpu` 28 or earlier | When you target a specific older toolchain; 29 is current |
| `kurbo` | `euclid` | `euclid` is more general; `kurbo` is the `piet` / `druid` ecosystem choice with native bezier support |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| `egui` crate | Immediate-mode design point; different from retained-mode. Mixing them costs more than it saves. | `rgui`'s own `Element` tree |
| `iced` | Elm-style `Message` update model; not compatible with our spec-based `Element` builder. | `rgui`'s `WidgetSpec` structs |
| `druid` | Unmaintained since 2022; Piet is the underlying render layer and not `wgpu`-native. | `wgpu` directly (via this stack) |
| `web-view` (Tauri / wry) | Embeds a web engine; defeats the Rust-native style system. | `rgui`'s own `Element` tree + `RML` if you need declarative |
| `tokio` on the hot path | The paint path is sync; async runtime would add overhead. | `pollster` for the rare async step (device request) |
| `serde` on the hot path | The hot path is the paint / event loop. `serde_json` is allowed in `to_debug_json` only. | Plain field access for performance-critical reads |

## Stack Patterns by Variant

- Drop `winit` + `arboard`
- Render directly to an `OffscreenTarget` (already supported)
- Skip the `accessibility` feature
- Add a `window_id` to `FrameInput` / `FrameOutput`
- Route events per-window
- The current single-window architecture needs a `Window` registry
- `wgpu` 29 already supports WebGPU backend via `wgpu::Backends::BROWSER_WEBGPU`
- The `taffy` and `kurbo` deps are already cross-platform
- Glyphon's WebGPU support is improving but watch for font-loading gotchas

## Version Compatibility

| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| `wgpu 29` | `glyphon 0.11` | Matched in `Cargo.toml`; check glyphon release notes when bumping wgpu |
| `taffy 0.10` | `wgpu 29` | Independent; taffy is layout-only |
| `serde 1` + `serde_json 1` | Any | Used only in `to_debug_json`; never on the hot path |
| `thiserror 2` | Rust 2024 | `thiserror 2.x` is the current generation |

## Sources

- `Cargo.toml` (committed at `Cargo.toml`) — current pinned versions
- `feedback.md` (Mavis code review) — what NOT to use surfaced in the API design notes
- `docs/public-api.md` — current public surface; pre-`3.8` / pre-`8.5` cleanup still pending

<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->

## Conventions

Conventions not yet established. Will populate as patterns emerge during development.
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->

## Architecture

Architecture not yet mapped. Follow existing patterns found in the codebase.
<!-- GSD:architecture-end -->

<!-- GSD:skills-start source:skills/ -->

## Project Skills

No project skills found. Add skills to any of: `.claude/skills/`, `.agents/skills/`, `.cursor/skills/`, `.github/skills/`, or `.codex/skills/` with a `SKILL.md` index file.
<!-- GSD:skills-end -->

<!-- GSD:workflow-start source:GSD defaults -->

## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:

- `/gsd-quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd-debug` for investigation and bug fixing
- `/gsd-execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->

<!-- GSD:profile-start -->

## Developer Profile

> Profile not yet configured. Run `/gsd-profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
