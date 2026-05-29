use super::CommandQueue;
use crate::core::{
    DisplayList, Element, HitTestSnapshot, HitTestTree, LayerKind, RenderStats, ResourceStore,
    SemanticTree, Size, Theme, UiSnapshot,
};

#[derive(Clone, Debug)]
pub struct FrameInput {
    pub root: Element,
    pub viewport: Size,
    pub theme: Theme,
    pub scale_factor: f32,
}

impl Default for FrameInput {
    fn default() -> Self {
        Self {
            root: Element::column(),
            viewport: Size::new(800.0, 600.0),
            theme: Theme::light(),
            scale_factor: 1.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FrameOutput {
    pub display_list: DisplayList,
    pub resources: ResourceStore,
    pub semantics: SemanticTree,
    pub hit_test: HitTestTree,
    pub stats: RenderStats,
    pub layout_engine: &'static str,
    pub commands: CommandQueue,
    pub snapshot: Option<UiSnapshot>,
}

impl FrameOutput {
    pub fn debug_snapshot(&self) -> UiSnapshot {
        let mut snapshot = self.snapshot.clone().unwrap_or_default();
        snapshot.hit_test_entries = self
            .hit_test
            .entries()
            .iter()
            .map(|entry| {
                let rect = entry.hit_rect();
                HitTestSnapshot {
                    node: entry.node,
                    key: entry.key.clone(),
                    x: rect.origin.x,
                    y: rect.origin.y,
                    width: rect.size.width,
                    height: rect.size.height,
                    z_index: entry.z_index,
                    layer: match entry.layer {
                        LayerKind::Document => "Document".to_string(),
                        LayerKind::Floating => "Floating".to_string(),
                        LayerKind::Popover => "Popover".to_string(),
                        LayerKind::Tooltip => "Tooltip".to_string(),
                        LayerKind::ContextMenu => "ContextMenu".to_string(),
                        LayerKind::Modal => "Modal".to_string(),
                        LayerKind::Debug => "Debug".to_string(),
                    },
                }
            })
            .collect();
        snapshot
    }
}
