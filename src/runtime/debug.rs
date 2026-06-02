use crate::core::{
    BorderCmd, Color, DisplayList, LayerKind, LayerSpec, Paint, PaintCommand, Rect, RectCmd, Size,
};
use crate::runtime::FrameOutput;
use std::fmt::Write as _;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DebugVisualMode {
    pub show_bounds: bool,
    pub show_hit_test: bool,
    pub show_clip_rects: bool,
    pub show_paint_order: bool,
    pub show_text_boxes: bool,
    pub show_overlay_layers: bool,
}

impl DebugVisualMode {
    pub fn parse(value: &str) -> Self {
        let mut mode = Self::default();
        for flag in value.split(',').map(str::trim) {
            match flag {
                "bounds" => mode.show_bounds = true,
                "hit-test" => mode.show_hit_test = true,
                "clips" => mode.show_clip_rects = true,
                "paint-order" => mode.show_paint_order = true,
                "text" => mode.show_text_boxes = true,
                "overlays" => mode.show_overlay_layers = true,
                "all" => {
                    mode.show_bounds = true;
                    mode.show_hit_test = true;
                    mode.show_clip_rects = true;
                    mode.show_paint_order = true;
                    mode.show_text_boxes = true;
                    mode.show_overlay_layers = true;
                }
                "" => {}
                _ => {}
            }
        }
        mode
    }

    pub fn from_env() -> Self {
        std::env::var("RGUI_DEBUG_VISUAL")
            .ok()
            .map(|value| Self::parse(&value))
            .unwrap_or_default()
    }

    pub fn is_empty(self) -> bool {
        self == Self::default()
    }
}

/// Format a human-readable dump of the frame for diagnostics.
///
/// **Allocation warning:** this function eagerly formats every
/// `PaintCommand` and style entry. Do *not* call it on every
/// frame in release builds; gate the call behind an env-var
/// check (see `runtime.rs` `RGUI_DUMP_FRAME`) or a debug flag.
///
/// Bug fix 2.11: the previous signature took an `enabled: bool`
/// parameter and returned `String::new()` if it was `false`. The
/// allocation still happened at the call site (a `String` is built
/// regardless), defeating the early-return. The current contract
/// is "caller checks the gate; this function always dumps".
///
/// Bug fix 2.11 (continued): each line was previously formatted
/// with `format!("…{x:?}\n")`, which allocates a fresh `String`
/// per line and then copies it into the output buffer. We now use
/// `write!` against a single `String` so the only allocations are
/// the buffer's internal growth and the per-line Debug string the
/// std library itself produces for the value.
pub fn format_frame_dump(output: &FrameOutput) -> String {
    let mut dump = String::new();
    writeln!(dump, "=== FRAME ===").unwrap();
    writeln!(dump, "layout_engine: {}", output.layout_engine).unwrap();

    writeln!(dump, "=== DISPLAY LIST ===").unwrap();
    for (index, command) in output.display_list.commands().iter().enumerate() {
        writeln!(dump, "[{index:03}] {command:?}").unwrap();
    }

    if let Some(snapshot) = &output.snapshot {
        writeln!(dump, "=== STYLES ===").unwrap();
        for style in &snapshot.styles {
            writeln!(dump, "{style:?}").unwrap();
        }

        writeln!(dump, "=== MEASURE ===").unwrap();
        for measure in &snapshot.measure {
            writeln!(dump, "{measure:?}").unwrap();
        }

        writeln!(dump, "=== LAYOUT ===").unwrap();
        for layout in &snapshot.layout {
            writeln!(dump, "{layout:?}").unwrap();
        }

        writeln!(dump, "=== PAINT ===").unwrap();
        for paint in &snapshot.display_list {
            writeln!(dump, "{paint:?}").unwrap();
        }

        writeln!(dump, "=== HIT TEST ===").unwrap();
        for entry in &snapshot.hit_test_entries {
            writeln!(dump, "{entry:?}").unwrap();
        }

        writeln!(dump, "=== SEMANTICS ===").unwrap();
        for semantic in &snapshot.semantics {
            writeln!(dump, "{semantic:?}").unwrap();
        }

        writeln!(dump, "=== OVERLAYS ===").unwrap();
        for overlay in snapshot.overlays() {
            writeln!(dump, "{overlay:?}").unwrap();
        }

        writeln!(dump, "=== STATS ===").unwrap();
        writeln!(dump, "{:?}", snapshot.performance).unwrap();
    }

    dump
}

pub fn push_debug_rect(display_list: &mut DisplayList, rect: Rect, color: Color, z_index: i32) {
    display_list.push(PaintCommand::PushLayer(LayerSpec::new(LayerKind::Debug)));
    display_list.push(PaintCommand::DrawBorder(BorderCmd {
        rect,
        color,
        width: 1.0,
        radius: 0.0,
        z_index,
    }));
    display_list.push(PaintCommand::PopLayer);
}

pub fn push_debug_label_backplate(display_list: &mut DisplayList, rect: Rect, z_index: i32) {
    display_list.push(PaintCommand::PushLayer(LayerSpec::new(LayerKind::Debug)));
    display_list.push(PaintCommand::DrawRect(RectCmd {
        rect: Rect::new(
            rect.origin,
            Size::new(rect.size.width.max(1.0), rect.size.height.max(1.0)),
        ),
        paint: Paint::Solid(Color::rgba(255, 255, 0, 80)),
        radius: 0.0,
        opacity: 1.0,
        z_index,
    }));
    display_list.push(PaintCommand::PopLayer);
}
