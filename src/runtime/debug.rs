use crate::core::{
    BorderCmd, Color, DisplayList, LayerKind, LayerSpec, Paint, PaintCommand, Rect, RectCmd, Size,
};
use crate::runtime::FrameOutput;

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

pub fn format_frame_dump(output: &FrameOutput, enabled: bool) -> String {
    if !enabled {
        return String::new();
    }

    let mut dump = String::new();
    dump.push_str("=== FRAME ===\n");
    dump.push_str(&format!("layout_engine: {}\n", output.layout_engine));

    dump.push_str("=== DISPLAY LIST ===\n");
    for (index, command) in output.display_list.commands().iter().enumerate() {
        dump.push_str(&format!("[{index:03}] {command:?}\n"));
    }

    if let Some(snapshot) = &output.snapshot {
        dump.push_str("=== STYLES ===\n");
        for style in &snapshot.styles {
            dump.push_str(&format!("{style:?}\n"));
        }

        dump.push_str("=== MEASURE ===\n");
        for measure in &snapshot.measure {
            dump.push_str(&format!("{measure:?}\n"));
        }

        dump.push_str("=== LAYOUT ===\n");
        for layout in &snapshot.layout {
            dump.push_str(&format!("{layout:?}\n"));
        }

        dump.push_str("=== PAINT ===\n");
        for paint in &snapshot.display_list {
            dump.push_str(&format!("{paint:?}\n"));
        }

        dump.push_str("=== HIT TEST ===\n");
        for entry in &snapshot.hit_test_entries {
            dump.push_str(&format!("{entry:?}\n"));
        }

        dump.push_str("=== SEMANTICS ===\n");
        for semantic in &snapshot.semantics {
            dump.push_str(&format!("{semantic:?}\n"));
        }

        dump.push_str("=== OVERLAYS ===\n");
        for overlay in snapshot.overlays() {
            dump.push_str(&format!("{overlay:?}\n"));
        }

        dump.push_str("=== STATS ===\n");
        dump.push_str(&format!("{:?}\n", snapshot.performance));
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
