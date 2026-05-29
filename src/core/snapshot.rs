use crate::{LayoutDebugSnapshot, NodeId};

#[derive(Clone, Debug, PartialEq)]
pub struct UiSnapshot {
    pub tree_nodes: Vec<String>,
    pub styles: Vec<ResolvedStyleSnapshot>,
    pub measure: Vec<MeasureSnapshot>,
    pub layout: Vec<LayoutBoxSnapshot>,
    pub display_list: Vec<PaintCommandSnapshot>,
    pub semantics: Vec<SemanticSnapshot>,
    pub events: Vec<EventTraceSnapshot>,
    pub overlays: Vec<OverlaySnapshot>,
    pub hit_test_entries: Vec<HitTestSnapshot>,
    pub layout_debug: LayoutDebugSnapshot,
    pub performance: PerformanceMetrics,
    pub diagnostics: UiDiagnostics,
}

impl Default for UiSnapshot {
    fn default() -> Self {
        Self {
            tree_nodes: Vec::new(),
            styles: Vec::new(),
            measure: Vec::new(),
            layout: Vec::new(),
            display_list: Vec::new(),
            semantics: Vec::new(),
            events: Vec::new(),
            overlays: Vec::new(),
            hit_test_entries: Vec::new(),
            layout_debug: LayoutDebugSnapshot::default(),
            performance: PerformanceMetrics::default(),
            diagnostics: UiDiagnostics::default(),
        }
    }
}

impl UiSnapshot {
    pub fn overlays(&self) -> &[OverlaySnapshot] {
        &self.overlays
    }

    pub fn to_debug_json(&self) -> String {
        format!(
            "{{\"tree_nodes\":{},\"styles\":{},\"measure\":{},\"layout\":{},\"paint\":{},\"hit_test\":{},\"semantics\":{},\"overlays\":{},\"stats\":{{\"display_command_count\":{},\"batch_count\":{},\"atlas_upload_bytes\":{}}}}}",
            self.tree_nodes.len(),
            self.styles.len(),
            self.measure.len(),
            self.layout.len(),
            self.display_list.len(),
            self.hit_test_entries.len(),
            self.semantics.len(),
            self.overlays.len(),
            self.performance.display_command_count,
            self.performance.batch_count,
            self.performance.atlas_upload_bytes,
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiDiagnostics {
    pub layout_errors: Vec<String>,
    pub layout_warnings: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OverlaySnapshot {
    pub key: Option<String>,
    pub layer: crate::LayerKind,
    pub rect: crate::Rect,
    pub modal: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HitTestSnapshot {
    pub node: NodeId,
    pub key: Option<String>,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub z_index: i32,
    pub layer: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedStyleSnapshot {
    pub node: NodeId,
    pub z_index: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MeasureSnapshot {
    pub node: NodeId,
    pub key: Option<String>,
    pub preferred_width: f32,
    pub preferred_height: f32,
    pub content_width: f32,
    pub content_height: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayoutBoxSnapshot {
    pub node: NodeId,
    pub key: Option<String>,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub content_width: f32,
    pub content_height: f32,
    pub clip_rect: Option<crate::Rect>,
}

impl UiSnapshot {
    pub fn layout_box(&self, key: &str) -> Option<&LayoutBoxSnapshot> {
        self.layout
            .iter()
            .find(|item| item.key.as_deref() == Some(key))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaintCommandSnapshot {
    pub kind: String,
    pub z_index: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SemanticSnapshot {
    pub node: NodeId,
    pub role: String,
    pub label: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EventTraceSnapshot {
    pub node: NodeId,
    pub phase: String,
    pub event: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AccessibilityMetrics {
    pub semantic_node_count: usize,
    pub accesskit_update_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PerformanceMetrics {
    pub frame_time_ms: f32,
    pub node_count: usize,
    pub style_cache_hit_rate: f32,
    pub layout_recompute_count: usize,
    pub display_command_count: usize,
    pub batch_count: usize,
    pub atlas_upload_bytes: usize,
    pub atlas_eviction_count: usize,
    pub text_shape_cache_hit_rate: f32,
    pub hit_test_time_ms: f32,
    pub accessibility: AccessibilityMetrics,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            frame_time_ms: 0.0,
            node_count: 0,
            style_cache_hit_rate: 0.0,
            layout_recompute_count: 0,
            display_command_count: 0,
            batch_count: 0,
            atlas_upload_bytes: 0,
            atlas_eviction_count: 0,
            text_shape_cache_hit_rate: 0.0,
            hit_test_time_ms: 0.0,
            accessibility: AccessibilityMetrics {
                semantic_node_count: 0,
                accesskit_update_count: 0,
            },
        }
    }
}
