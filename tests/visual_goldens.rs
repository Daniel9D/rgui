use rgui::render::wgpu::{OffscreenTarget, RendererOptions, WgpuRenderer};
use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::{
    button, canvas, checkbox, context_menu, divider, icon, input, list, menu, menu_item, option,
    popover, radio, scroll_area, select, table, tabs, text, textarea, tree, tree_item,
};
use rgui::{Element, ElementKind, Length, PrimitiveKind, Size, SizeU32};
use std::path::{Path, PathBuf};

fn render_runtime_rgba(root: Element, size: SizeU32) -> Vec<u8> {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root,
        viewport: Size::new(size.width as f32, size.height as f32),
        ..Default::default()
    });

    let mut renderer = pollster::block_on(WgpuRenderer::new_headless(RendererOptions {
        initial_size: size,
        ..RendererOptions::default()
    }))
    .expect("headless renderer initializes");
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
        .join("rgui-goldens")
        .join("actual")
        .join(format!("{name}.png"));
    let diff = Path::new("target")
        .join("rgui-goldens")
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

/// Per-channel diff statistics computed from two RGBA8 frames.
///
/// These are the inputs to a *perceptual* tolerance check: small
/// GPU/driver-level non-determinism (1-2 units of channel difference
/// on a few thousand pixels) should be tolerated, while genuine
/// regressions (whole regions redrawn) should fail loudly. A pixel
/// is counted as "changed" when any of its four channels differs by
/// more than `PIXEL_TOLERANCE` (configured below) from the expected
/// value.
#[derive(Debug, Clone, Copy)]
struct PixelDiffStats {
    /// Number of pixels that differ by more than the per-channel
    /// tolerance threshold.
    changed_pixels: usize,
    /// Largest single-channel absolute difference observed.
    max_abs_diff: u8,
    /// Total pixel count.
    total_pixels: usize,
}

impl PixelDiffStats {
    /// Fraction of pixels that exceed the tolerance, range `[0.0, 1.0]`.
    fn changed_ratio(&self) -> f64 {
        if self.total_pixels == 0 {
            return 0.0;
        }
        self.changed_pixels as f64 / self.total_pixels as f64
    }
}

/// Per-channel absolute difference that a single pixel may have
/// before it is counted as "changed". `1` covers quantization noise
/// and minor sub-pixel rasterization differences; anything above
/// this on a meaningful fraction of pixels is a real regression.
const PIXEL_TOLERANCE: u8 = 1;

/// Maximum fraction of pixels allowed to differ by more than the
/// per-pixel tolerance. `0.0001` = 0.01% of pixels, e.g. ~30
/// pixels on a 640×480 frame. Anything more is a real regression.
const CHANGED_PIXEL_RATIO_LIMIT: f64 = 0.0001;

/// Maximum single-channel drift tolerated anywhere in the frame.
/// `5` catches the case where a tiny fraction of pixels drift by a
/// lot (which the ratio gate would otherwise miss) — e.g. a 1-pixel
/// brush stroke rendering at a different sub-pixel offset on a
/// different driver.
const MAX_ABS_DIFF_LIMIT: u8 = 5;

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

fn assert_visual_matches(name: &str, size: SizeU32, actual_pixels: &[u8]) {
    let (expected_path, actual_path, diff_path) = golden_paths(name);
    if std::env::var_os("RGUI_UPDATE_GOLDENS").is_some() {
        save_png(&expected_path, size, actual_pixels);
        return;
    }

    save_png(&actual_path, size, actual_pixels);
    assert!(
        expected_path.exists(),
        "missing golden {}; run RGUI_UPDATE_GOLDENS=1 cargo test --test visual_goldens",
        expected_path.display()
    );

    let (expected_size, expected_pixels) = load_png_rgba(&expected_path);
    assert_eq!(expected_size, size, "golden size changed for {name}");

    // Strict path: bytes match exactly. This is the common case on
    // a deterministic software rasterizer; if we reach this, the
    // golden is locked in and we don't need to consider tolerance.
    if expected_pixels == actual_pixels {
        return;
    }

    let (strict_changed, diff_pixels) = diff_rgba(&expected_pixels, actual_pixels);
    let stats = pixel_diff_stats(&expected_pixels, actual_pixels);
    save_png(&diff_path, size, &diff_pixels);

    // Tolerance path: a small amount of per-pixel noise is OK so
    // the goldens don't flake on driver / GPU non-determinism. If
    // either gate fails we treat the diff as a real regression.
    let within_ratio = stats.changed_ratio() <= CHANGED_PIXEL_RATIO_LIMIT;
    let within_max = stats.max_abs_diff <= MAX_ABS_DIFF_LIMIT;

    if !(within_ratio && within_max) {
        panic!(
            "visual golden {name} changed beyond tolerance \
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

#[test]
fn pixel_diff_stats_reports_clean_match_as_zero() {
    let pixels = vec![10u8, 20, 30, 255, 40, 50, 60, 128];
    let stats = pixel_diff_stats(&pixels, &pixels);
    assert_eq!(stats.changed_pixels, 0);
    assert_eq!(stats.max_abs_diff, 0);
    assert_eq!(stats.total_pixels, 2);
    assert!(stats.changed_ratio() <= CHANGED_PIXEL_RATIO_LIMIT);
}

#[test]
fn pixel_diff_stats_counts_tolerated_drift_as_unflagged() {
    // Two pixels, each channel drifts by 1 (within tolerance).
    let expected = vec![10u8, 20, 30, 255, 40, 50, 60, 128];
    let actual = vec![11u8, 21, 31, 255, 41, 51, 61, 128];
    let stats = pixel_diff_stats(&expected, &actual);
    assert_eq!(
        stats.changed_pixels, 0,
        "drift within tolerance should not flag pixels"
    );
    assert_eq!(stats.max_abs_diff, 1);
    assert!(stats.changed_ratio() <= CHANGED_PIXEL_RATIO_LIMIT);
    assert!(stats.max_abs_diff <= MAX_ABS_DIFF_LIMIT);
}

#[test]
fn pixel_diff_stats_flags_regression_above_tolerance() {
    // One pixel of 8 channels drifts by 50 in red; the rest match.
    let mut expected = vec![0u8; 32];
    let mut actual = vec![0u8; 32];
    actual[0] = 50; // first pixel red channel: drift = 50
    let stats = pixel_diff_stats(&expected, &actual);
    assert_eq!(stats.changed_pixels, 1, "single out-of-tolerance pixel");
    assert_eq!(stats.max_abs_diff, 50);
    assert!(stats.changed_ratio() > CHANGED_PIXEL_RATIO_LIMIT);
    assert!(stats.max_abs_diff > MAX_ABS_DIFF_LIMIT);
}

#[test]
fn golden_text_hierarchy_320x160_matches() {
    let size = SizeU32::new(320, 160);
    let pixels = render_runtime_rgba(
        Element::column()
            .child(text("Title Case Heading").heading().key("title"))
            .child(text("Readable body text").key("body")),
        size,
    );
    assert_visual_matches("golden_text_hierarchy_320x160", size, &pixels);
}

#[test]
fn golden_toolbar_360x120_matches() {
    let size = SizeU32::new(360, 120);
    let pixels = render_runtime_rgba(
        Element::row()
            .gap(8.0)
            .child(button("Save").key("save"))
            .child(checkbox().checked(true).key("enabled")),
        size,
    );
    assert_visual_matches("golden_toolbar_360x120", size, &pixels);
}

#[test]
fn golden_popover_320x200_matches() {
    let size = SizeU32::new(320, 200);
    let pixels = render_runtime_rgba(
        Element::column().child(
            button("Menu").key("menu").popover(
                popover()
                    .open(true)
                    .key("menu-popover")
                    .child(text("Profile")),
            ),
        ),
        size,
    );
    assert_visual_matches("golden_popover_320x200", size, &pixels);
}

#[test]
fn golden_scroll_clip_320x200_matches() {
    let size = SizeU32::new(320, 200);
    let pixels = render_runtime_rgba(
        Element::new(ElementKind::Primitive(PrimitiveKind::ScrollArea))
            .key("scroll")
            .height(Length::Px(96.0))
            .child(text("Line one"))
            .child(text("Line two"))
            .child(text("Line three"))
            .child(text("Line four"))
            .child(text("Line five")),
        size,
    );
    assert_visual_matches("golden_scroll_clip_320x200", size, &pixels);
}

#[test]
fn golden_full_widgets_640x480_matches() {
    let size = SizeU32::new(640, 480);
    let pixels = render_runtime_rgba(
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
            .child(text("The public examples should render visible output.")),
        size,
    );
    assert_visual_matches("golden_full_widgets_640x480", size, &pixels);
}

#[test]
fn golden_widgets_collections_640x480_matches() {
    let size = SizeU32::new(640, 480);
    let pixels = render_runtime_rgba(
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
            ),
        size,
    );
    assert_visual_matches("golden_widgets_collections_640x480", size, &pixels);
}

#[test]
fn golden_widget_showcase_flow_808x823_matches() {
    let size = SizeU32::new(808, 823);
    let pixels = render_runtime_rgba(
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
            ),
        size,
    );
    assert_visual_matches("golden_widget_showcase_flow_808x823", size, &pixels);
}
