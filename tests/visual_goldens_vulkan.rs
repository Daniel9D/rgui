//! Cross-backend visual goldens for the Vulkan backend (REND-01).
//!
//! Mirrors the 8 scenes in `tests/visual_goldens.rs` but renders each one
//! through `wgpu::Backends::VULKAN` instead of the host's `Backends::PRIMARY`
//! (DX12 on Windows, Metal on macOS). The same PNG baselines under
//! `tests/goldens/` are used as the expected reference — the goal is to
//! prove that the wgpu render path is stable across backends.
//!
//! # Run
//!
//! ```text
//! cargo test --features vulkan-goldens --test visual_goldens_vulkan
//! ```
//!
//! The expected outcome on a Vulkan-capable runner (`windows-latest` or
//! `ubuntu-latest` in CI, plan 05-03) is that all 8 per-scene tests plus
//! the aggregate diagnostic test pass within the cross-backend tolerance
//! constants documented below.
//!
//! # Tolerances
//!
//! Cross-backend variance is real (driver text rasterizers, sub-pixel
//! rounding, sRGB blend rounding all differ between Vulkan and DX12 /
//! Metal). `MAX_ABS_DIFF_LIMIT` is `15` here (vs. `5` in the same-backend
//! suite at `tests/visual_goldens.rs`) — bump it further in **this file
//! only** if a future driver bump pushes diffs higher; do NOT loosen the
//! `Backends::PRIMARY` suite.
//!
//! # CI gate
//!
//! Plan 05-03 wires the `validation-layers` feature and the CI workflow
//! that invokes this test target on Vulkan-capable runners. This file is
//! the artifact 05-03 imports.

#![cfg(feature = "vulkan-goldens")]

use rgui::render::wgpu::{OffscreenTarget, WgpuRenderer};
use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::{
    button, canvas, checkbox, context_menu, divider, icon, input, list, menu, menu_item, option,
    popover, radio, scroll_area, select, table, tabs, text, textarea, tree, tree_item,
};
use rgui::{Element, ElementKind, Length, PrimitiveKind, Size, SizeU32};
use std::path::{Path, PathBuf};

/// Per-channel absolute difference tolerated before a pixel is counted as
/// "changed". `1` covers quantization noise; anything above this on a
/// meaningful fraction of pixels is a real regression.
const PIXEL_TOLERANCE: u8 = 1;

/// Maximum fraction of pixels allowed to differ by more than the per-pixel
/// tolerance. Loosened vs. the same-backend suite (which uses `0.0001`)
/// because cross-backend text rasterization can drift a few thousand
/// pixels on a 640x480 frame.
const CHANGED_PIXEL_RATIO_LIMIT: f64 = 0.005;

/// Maximum single-channel drift tolerated anywhere in the frame. Bumped
/// from `5` (same-backend) to `15` (cross-backend) per the plan; raise
/// further in this file only if a driver bump pushes diffs higher.
const MAX_ABS_DIFF_LIMIT: u8 = 15;

/// Renders an element tree through the Vulkan backend and returns the
/// RGBA8 pixels. Uses the test seam introduced by plan 05-01 task 1.
fn render_runtime_rgba_vulkan(root: Element, size: SizeU32) -> Vec<u8> {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root,
        viewport: Size::new(size.width as f32, size.height as f32),
        ..Default::default()
    });

    let mut renderer = WgpuRenderer::new_headless_for_tests_with_backends(
        size,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        wgpu::Backends::VULKAN,
    );
    let target = OffscreenTarget::new(renderer.context(), size);
    renderer
        .render_to_target(&output.display_list, &output.resources, target.view())
        .expect("runtime frame renders");
    pollster::block_on(target.read_rgba8(renderer.context())).expect("readback works")
}

fn golden_paths(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let expected = Path::new("tests")
        .join("goldens")
        .join(format!("{name}.png"));
    let actual = Path::new("target")
        .join("rgui-goldens-vulkan")
        .join("actual")
        .join(format!("{name}.png"));
    let diff = Path::new("target")
        .join("rgui-goldens-vulkan")
        .join("diff")
        .join(format!("{name}.png"));
    (expected, actual, diff)
}

fn save_png(path: &Path, size: SizeU32, pixels: &[u8]) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("png parent directory is created");
    }
    let image = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(
        size.width,
        size.height,
        pixels.to_vec(),
    )
    .expect("rgba image buffer");
    image.save(path).expect("png writes");
}

fn load_png_rgba(path: &Path) -> (SizeU32, Vec<u8>) {
    let image = image::open(path)
        .unwrap_or_else(|err| panic!("failed to open golden {}: {err}", path.display()))
        .to_rgba8();
    (
        SizeU32::new(image.width(), image.height()),
        image.into_raw(),
    )
}

fn diff_rgba(expected: &[u8], actual: &[u8]) -> (usize, Vec<u8>) {
    let mut changed = 0usize;
    let mut diff = Vec::with_capacity(actual.len());
    for (expected_px, actual_px) in expected.chunks_exact(4).zip(actual.chunks_exact(4)) {
        if expected_px == actual_px {
            diff.extend_from_slice(&[0, 0, 0, 0]);
        } else {
            changed += 1;
            diff.extend_from_slice(&[255, 0, 255, 255]);
        }
    }
    (changed, diff)
}

#[derive(Debug, Clone, Copy)]
struct PixelDiffStats {
    changed_pixels: usize,
    max_abs_diff: u8,
    total_pixels: usize,
}

impl PixelDiffStats {
    fn changed_ratio(&self) -> f64 {
        if self.total_pixels == 0 {
            return 0.0;
        }
        self.changed_pixels as f64 / self.total_pixels as f64
    }
}

fn pixel_diff_stats(expected: &[u8], actual: &[u8]) -> PixelDiffStats {
    debug_assert_eq!(expected.len(), actual.len());
    let mut stats = PixelDiffStats {
        changed_pixels: 0,
        max_abs_diff: 0,
        total_pixels: expected.len() / 4,
    };
    for (expected_px, actual_px) in expected.chunks_exact(4).zip(actual.chunks_exact(4)) {
        let mut pixel_changed = false;
        for (&e, &a) in expected_px.iter().zip(actual_px.iter()) {
            let d = (e as i32 - a as i32).unsigned_abs() as u8;
            if d > stats.max_abs_diff {
                stats.max_abs_diff = d;
            }
            if d > PIXEL_TOLERANCE {
                pixel_changed = true;
            }
        }
        if pixel_changed {
            stats.changed_pixels += 1;
        }
    }
    stats
}

fn assert_visual_matches_vulkan(name: &str, size: SizeU32, actual_pixels: &[u8]) {
    let (expected_path, actual_path, diff_path) = golden_paths(name);
    save_png(&actual_path, size, actual_pixels);
    assert!(
        expected_path.exists(),
        "missing golden {}; capture baselines via the same-backend suite first \
         (RGUI_UPDATE_GOLDENS=1 cargo test --test visual_goldens)",
        expected_path.display()
    );

    let (expected_size, expected_pixels) = load_png_rgba(&expected_path);
    assert_eq!(expected_size, size, "golden size changed for {name}");

    if expected_pixels == actual_pixels {
        return;
    }

    let (strict_changed, diff_pixels) = diff_rgba(&expected_pixels, actual_pixels);
    let stats = pixel_diff_stats(&expected_pixels, actual_pixels);
    save_png(&diff_path, size, &diff_pixels);

    let within_ratio = stats.changed_ratio() <= CHANGED_PIXEL_RATIO_LIMIT;
    let within_max = stats.max_abs_diff <= MAX_ABS_DIFF_LIMIT;

    if !(within_ratio && within_max) {
        panic!(
            "vulkan visual golden {name} changed beyond cross-backend tolerance \
             (changed_pixels={}/{} changed_ratio={:.6} max_abs_diff={} \
             strict_changed={} limits: ratio<={:.6} max<={}); \
             actual={} diff={}",
            stats.changed_pixels,
            stats.total_pixels,
            stats.changed_ratio(),
            stats.max_abs_diff,
            strict_changed,
            CHANGED_PIXEL_RATIO_LIMIT,
            MAX_ABS_DIFF_LIMIT,
            actual_path.display(),
            diff_path.display(),
        );
    }
}

fn scene_text_hierarchy() -> Element {
    Element::column()
        .child(text("Title Case Heading").heading().key("title"))
        .child(text("Readable body text").key("body"))
}

fn scene_toolbar() -> Element {
    Element::row()
        .gap(8.0)
        .child(button("Save").key("save"))
        .child(checkbox().checked(true).key("enabled"))
}

fn scene_popover() -> Element {
    Element::column().child(
        button("Menu").key("menu").popover(
            popover()
                .open(true)
                .key("menu-popover")
                .child(text("Profile")),
        ),
    )
}

fn scene_scroll_clip() -> Element {
    Element::new(ElementKind::Primitive(PrimitiveKind::ScrollArea))
        .key("scroll")
        .height(Length::Px(96.0))
        .child(text("Line one"))
        .child(text("Line two"))
        .child(text("Line three"))
        .child(text("Line four"))
        .child(text("Line five"))
}

fn scene_full_widgets() -> Element {
    Element::column()
        .gap(10.0)
        .child(text("Controls").heading())
        .child(
            Element::row()
                .gap(8.0)
                .child(button("Save").primary().key("save"))
                .child(button("Cancel").key("cancel"))
                .child(checkbox().checked(true).key("enabled")),
        )
        .child(text("The public examples should render visible output."))
}

fn scene_widgets_collections() -> Element {
    Element::column()
        .padding(16.0)
        .gap(10.0)
        .child(text("Interactive Widget Showcase").heading())
        .child(
            Element::row()
                .key("pickers")
                .gap(8.0)
                .child(
                    select()
                        .key("select")
                        .options([
                            option("low", "Low"),
                            option("medium", "Medium"),
                            option("high", "High"),
                        ])
                        .default_value("medium"),
                )
                .child(textarea().key("notes"))
                .child(
                    tabs()
                        .key("tabs")
                        .tabs(["General", "Advanced"])
                        .default_active_index(0),
                ),
        )
        .child(
            Element::row()
                .key("collections")
                .gap(8.0)
                .child(
                    tree()
                        .key("tree")
                        .items([tree_item("Project").expanded(true).child(tree_item("src"))]),
                )
                .child(
                    table()
                        .key("table")
                        .columns(["Name", "Status"])
                        .rows([["Runtime", "Ready"], ["Renderer", "Ready"]])
                        .default_selected_row(0),
                )
                .child(
                    list()
                        .key("list")
                        .items(["Inbox", "Today", "Done"])
                        .default_selected_index(1),
                )
                .child(
                    menu()
                        .key("menu")
                        .child(menu_item("Archive").key("archive")),
                ),
        )
        .child(
            Element::row()
                .key("media")
                .gap(8.0)
                .child(icon("search").key("icon-search"))
                .child(icon("settings").key("icon-settings"))
                .child(icon("home").key("icon-home"))
                .child(divider().key("divider"))
                .child(canvas("chart").key("chart")),
        )
}

fn scene_widget_showcase_flow() -> Element {
    Element::column()
        .padding(16.0)
        .gap(10.0)
        .child(text("Interactive Widget Showcase").heading())
        .child(
            Element::row()
                .gap(12.0)
                .child(text("Clicks"))
                .child(text("0"))
                .child(text("Enabled"))
                .child(text("on"))
                .child(text("Query"))
                .child(text("Focus")),
        )
        .child(
            Element::column()
                .gap(6.0)
                .child(text("Toolbar").heading())
                .child(
                    Element::row()
                        .gap(8.0)
                        .child(button("Save").primary().key("save"))
                        .child(input().key("search"))
                        .child(checkbox().checked(true).key("enabled"))
                        .child(radio().key("choice")),
                ),
        )
        .child(
            Element::column()
                .gap(8.0)
                .child(text("Data & Collections").heading())
                .child(
                    Element::row()
                        .gap(8.0)
                        .child(
                            select()
                                .key("select")
                                .options([
                                    option("low", "Low"),
                                    option("medium", "Medium"),
                                    option("high", "High"),
                                ])
                                .default_value("medium")
                                .placeholder("Priority"),
                        )
                        .child(textarea().key("notes"))
                        .child(
                            tabs()
                                .key("tabs")
                                .tabs(["General", "Advanced"])
                                .default_active_index(0),
                        ),
                )
                .child(
                    Element::row()
                        .gap(8.0)
                        .child(tree().key("tree").items([
                            tree_item("Project").expanded(true).child(tree_item("src")),
                        ]))
                        .child(
                            table()
                                .key("table")
                                .columns(["Name", "Status"])
                                .rows([["Runtime", "Ready"], ["Renderer", "Ready"]])
                                .default_selected_row(0),
                        )
                        .child(
                            list()
                                .key("list")
                                .items(["Inbox", "Today", "Done"])
                                .default_selected_index(1),
                        )
                        .child(
                            menu()
                                .key("menu")
                                .child(menu_item("Archive").key("archive")),
                        ),
                )
                .child(
                    scroll_area()
                        .key("log_scroll")
                        .height(160.0)
                        .child(text("Line 1").height(40.0))
                        .child(text("Line 2").height(40.0))
                        .child(text("Line 3").height(40.0)),
                )
                .child(
                    button("Right-click me")
                        .key("context-btn")
                        .context_menu(context_menu().child(menu_item("Delete"))),
                ),
        )
}

fn scene_new_painters() -> Element {
    use rgui::widgets::{
        alert, avatar, badge, card, image as rgui_image, link, progress_bar, slider, spinner,
        switch,
    };
    Element::column()
        .padding(12.0)
        .gap(8.0)
        .child(
            Element::row()
                .key("row1")
                .gap(8.0)
                .child(card().key("card1").width(120.0).height(80.0))
                .child(badge("New").key("badge1"))
                .child(link("Read more").key("link1"))
                .child(alert().key("alert1").width(160.0)),
        )
        .child(
            Element::row()
                .key("row2")
                .gap(8.0)
                .child(progress_bar().key("progress1").width(200.0))
                .child(spinner().key("spinner1").width(24.0).height(24.0))
                .child(switch().checked(true).key("switch1").width(40.0))
                .child(slider().key("slider1").width(120.0))
                .child(
                    rgui_image("placeholder")
                        .key("image1")
                        .width(48.0)
                        .height(48.0),
                )
                .child(avatar().key("avatar1").width(36.0).height(36.0)),
        )
}

#[test]
fn golden_text_hierarchy_320x160_vulkan() {
    let size = SizeU32::new(320, 160);
    let pixels = render_runtime_rgba_vulkan(scene_text_hierarchy(), size);
    assert_visual_matches_vulkan("golden_text_hierarchy_320x160", size, &pixels);
}

#[test]
fn golden_toolbar_360x120_vulkan() {
    let size = SizeU32::new(360, 120);
    let pixels = render_runtime_rgba_vulkan(scene_toolbar(), size);
    assert_visual_matches_vulkan("golden_toolbar_360x120", size, &pixels);
}

#[test]
fn golden_popover_320x200_vulkan() {
    let size = SizeU32::new(320, 200);
    let pixels = render_runtime_rgba_vulkan(scene_popover(), size);
    assert_visual_matches_vulkan("golden_popover_320x200", size, &pixels);
}

#[test]
fn golden_scroll_clip_320x200_vulkan() {
    let size = SizeU32::new(320, 200);
    let pixels = render_runtime_rgba_vulkan(scene_scroll_clip(), size);
    assert_visual_matches_vulkan("golden_scroll_clip_320x200", size, &pixels);
}

#[test]
fn golden_full_widgets_640x480_vulkan() {
    let size = SizeU32::new(640, 480);
    let pixels = render_runtime_rgba_vulkan(scene_full_widgets(), size);
    assert_visual_matches_vulkan("golden_full_widgets_640x480", size, &pixels);
}

#[test]
fn golden_widgets_collections_640x480_vulkan() {
    let size = SizeU32::new(640, 480);
    let pixels = render_runtime_rgba_vulkan(scene_widgets_collections(), size);
    assert_visual_matches_vulkan("golden_widgets_collections_640x480", size, &pixels);
}

#[test]
fn golden_widget_showcase_flow_808x823_vulkan() {
    let size = SizeU32::new(808, 823);
    let pixels = render_runtime_rgba_vulkan(scene_widget_showcase_flow(), size);
    assert_visual_matches_vulkan("golden_widget_showcase_flow_808x823", size, &pixels);
}

#[test]
fn golden_new_painters_640x320_vulkan() {
    let size = SizeU32::new(640, 320);
    let pixels = render_runtime_rgba_vulkan(scene_new_painters(), size);
    assert_visual_matches_vulkan("golden_new_painters_640x320", size, &pixels);
}
