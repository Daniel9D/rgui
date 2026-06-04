use serde::Serialize;

use crate::{LayoutDebugSnapshot, NodeId};
use crate::text_engine::TextCacheStats;

#[derive(Clone, Debug, PartialEq, Serialize)]
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
    /// Phase 3 / Plan 03-03: shape / layout text cache stats at
    /// the moment the snapshot was built. Populated by
    /// `UiRuntime::update` after the frame is built.
    pub text_cache: TextCacheStats,
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
            text_cache: TextCacheStats::default(),
        }
    }
}

impl UiSnapshot {
    pub fn overlays(&self) -> &[OverlaySnapshot] {
        &self.overlays
    }

    /// Serialize the snapshot to JSON for debug dumps. Bug fix 2.8:
    /// the previous hand-rolled `format!` only emitted counts and
    /// would break as soon as a string field (e.g. `role`, `key`,
    /// `kind`) was added — it didn't escape `"` or `\` in the
    /// values. Now we go through `serde_json`, which handles
    /// every field type and gives stable, parseable output.
    ///
    /// On a `serde_json` failure (which can happen if the
    /// snapshot contains a non-`Serialize` type — none today),
    /// the function falls back to a stringified error so the
    /// caller still gets a printable result.
    pub fn to_debug_json(&self) -> String {
        match serde_json::to_string(self) {
            Ok(json) => json,
            Err(err) => format!(
                "{{\"error\":\"snapshot serialization failed: {err}\"}}"
            ),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct UiDiagnostics {
    pub layout_errors: Vec<String>,
    pub layout_warnings: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct OverlaySnapshot {
    pub key: Option<String>,
    pub layer: crate::LayerKind,
    pub rect: crate::Rect,
    pub modal: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
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

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ResolvedStyleSnapshot {
    pub node: NodeId,
    pub z_index: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct MeasureSnapshot {
    pub node: NodeId,
    pub key: Option<String>,
    pub preferred_width: f32,
    pub preferred_height: f32,
    pub content_width: f32,
    pub content_height: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
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

impl LayoutBoxSnapshot {
    /// Returns `true` when this box has an active clip rect, meaning content
    /// that falls outside is clipped rather than drawn. Mirrors
    /// `crate::core::layout::LayoutBox::clips_overflow` so tests can use the
    /// same vocabulary on both representations.
    pub fn clips_overflow(&self) -> bool {
        self.clip_rect.is_some()
    }
}

impl UiSnapshot {
    pub fn layout_box(&self, key: &str) -> Option<&LayoutBoxSnapshot> {
        self.layout
            .iter()
            .find(|item| item.key.as_deref() == Some(key))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PaintCommandSnapshot {
    pub kind: String,
    pub z_index: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SemanticSnapshot {
    pub node: NodeId,
    pub role: String,
    pub label: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct EventTraceSnapshot {
    pub node: NodeId,
    pub phase: String,
    pub event: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct AccessibilityMetrics {
    pub semantic_node_count: usize,
    pub accesskit_update_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LayerKind;

    // Bug fix 2.8: `to_debug_json` now goes through
    // `serde_json`. The previous hand-rolled `format!` only
    // emitted counts and would break as soon as a string
    // field (e.g. `kind`, `role`, `key`) was added — it
    // didn't escape `"` or `\` in the values. Verify:
    // (a) the output is parseable as JSON;
    // (b) string fields round-trip with escaping intact.

    #[test]
    fn to_debug_json_emits_parseable_json() {
        let snapshot = UiSnapshot::default();
        let json = snapshot.to_debug_json();
        let parsed: serde_json::Value =
            serde_json::from_str(&json).expect("output must be valid JSON");
        // Vec fields are JSON arrays; scalars are scalars.
        // Note: the new schema uses the Rust field names
        // (`display_list`, `hit_test_entries`, `performance`)
        // rather than the abbreviated names (`paint`,
        // `hit_test`, `stats`) the hand-rolled format! used.
        // This is a deliberate, documented improvement.
        assert_eq!(parsed["tree_nodes"], serde_json::json!([]));
        assert_eq!(parsed["styles"], serde_json::json!([]));
        assert_eq!(parsed["performance"]["display_command_count"], 0);
        assert_eq!(parsed["performance"]["batch_count"], 0);
    }

    #[test]
    fn to_debug_json_escapes_string_fields() {
        // Build a snapshot with string fields that include
        // characters which the old hand-rolled format!
        // would have emitted un-escaped.
        let mut snapshot = UiSnapshot::default();
        snapshot.display_list.push(PaintCommandSnapshot {
            kind: "DrawText".to_string(),
            z_index: 0,
        });
        snapshot.semantics.push(SemanticSnapshot {
            node: NodeId::from_raw(7),
            role: "button".to_string(),
            label: Some("a \"quoted\" label".to_string()),
        });
        snapshot.overlays.push(OverlaySnapshot {
            key: Some("tab\nbreak".to_string()),
            layer: LayerKind::Modal,
            rect: crate::Rect::new(
                crate::Point::new(0.0, 0.0),
                crate::Size::new(10.0, 20.0),
            ),
            modal: true,
        });
        let json = snapshot.to_debug_json();
        let parsed: serde_json::Value =
            serde_json::from_str(&json).expect("output must be valid JSON");
        // The strings round-trip with their embedded characters.
        assert_eq!(parsed["display_list"][0]["kind"], "DrawText");
        assert_eq!(parsed["semantics"][0]["role"], "button");
        assert_eq!(
            parsed["semantics"][0]["label"],
            "a \"quoted\" label"
        );
        assert_eq!(parsed["overlays"][0]["key"], "tab\nbreak");
        assert_eq!(parsed["overlays"][0]["layer"], "Modal");
    }

    // Phase 5 / Plan 05-04 (REND-04): unit test that pins the
    // `frame_time_ms` field is now populated by `UiRuntime::update`,
    // not zeroed by `..PerformanceMetrics::default()`. The test
    // builds a non-trivial element tree (one `Element::row` with
    // three child labels), runs `update` once, and asserts
    // `frame_time_ms > 0.0`. This is the sanity check for the
    // Phase 4 deviation that plan 05-04 fixes.

    #[test]
    fn frame_time_ms_field_is_populated_by_runtime() {
        let mut runtime = crate::runtime::UiRuntime::default();
        let output = runtime.update(crate::runtime::FrameInput {
            root: crate::Element::row()
                .key("frame-budget-test")
                .gap(8.0)
                .child(crate::widgets::text("alpha"))
                .child(crate::widgets::text("beta"))
                .child(crate::widgets::text("gamma")),
            ..Default::default()
        });
        let frame_time_ms = output
            .debug_snapshot()
            .performance
            .frame_time_ms;
        assert!(
            frame_time_ms > 0.0,
            "PerformanceMetrics.frame_time_ms must be populated by UiRuntime::update, \
             not zeroed. Got {frame_time_ms}.",
        );
    }
}
