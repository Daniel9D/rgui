use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::core::{FontStyle, FontWeight, Size};
use crate::render::wgpu::constants::{TEXT_WIDTH_HEURISTIC, TEXT_WIDTH_HEURISTIC_BOLD};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextMetrics {
    pub width: f32,
    pub height: f32,
    pub baseline: f32,
}

/// Cache key for the heuristic `measure_text` path.
///
/// Bug fix 4.2: the heuristic was recomputed on every call. The
/// function is now fronted by a thread-local cache keyed on the
/// full argument tuple (text, font_size, weight, style, max_width).
/// `f32`s are hashed via their bit pattern so `NaN`/`±0.0` are
/// stable (the heuristic clamps inputs so we never produce `NaN`,
/// but the bit hash keeps the cache sound if the input ever does).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct MetricKey {
    text_hash: u64,
    size_bits: u32,
    width_bits: u32,
    weight: FontWeight,
    style: FontStyle,
}

impl MetricKey {
    fn new(text: &str, size: f32, width: f32, weight: FontWeight, style: FontStyle) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        text.hash(&mut hasher);
        Self {
            text_hash: hasher.finish(),
            size_bits: size.to_bits(),
            width_bits: width.to_bits(),
            weight,
            style,
        }
    }
}

/// Per-thread cache for the heuristic `measure_text` path.
///
/// Bug fix 4.2: shared across the caller's thread (text rendering
/// happens on a single thread in normal use, so a per-thread cache
/// hits the same hot path as a process-global one). The cache is
/// plain (not LRU) and grows unbounded; in long-running apps a
/// clear call (`clear_metrics_cache`) is appropriate after a
/// theme switch or locale change.
#[derive(Default)]
struct MetricsCache {
    entries: HashMap<MetricKey, TextMetrics>,
}

impl MetricsCache {
    fn get_or_insert_with<F>(&mut self, key: MetricKey, compute: F) -> TextMetrics
    where
        F: FnOnce() -> TextMetrics,
    {
        if let Some(&cached) = self.entries.get(&key) {
            return cached;
        }
        let value = compute();
        self.entries.insert(key, value);
        value
    }
}

thread_local! {
    static METRICS_CACHE: RefCell<MetricsCache> = RefCell::new(MetricsCache::default());
}

/// Clear the per-thread `measure_text` cache. Intended for tests
/// and for theme/locale change handlers in long-running apps.
/// Returns the number of entries that were evicted.
pub fn clear_metrics_cache() -> usize {
    METRICS_CACHE.with(|cell| {
        let mut cache = cell.borrow_mut();
        let n = cache.entries.len();
        cache.entries.clear();
        n
    })
}

/// Cache statistics for the heuristic `measure_text` path. Mirrors
/// the `text_engine::TextCacheStats` shape so the two paths can be
/// reported side-by-side in observability tools.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextMetricsCacheStats {
    pub entries: usize,
}

pub fn metrics_cache_stats() -> TextMetricsCacheStats {
    METRICS_CACHE.with(|cell| TextMetricsCacheStats {
        entries: cell.borrow().entries.len(),
    })
}

pub fn measure_text(
    text: &str,
    font_size: f32,
    weight: FontWeight,
    style: FontStyle,
    max_width: f32,
) -> TextMetrics {
    // Bug fix 4.2: cache the heuristic on a per-thread HashMap
    // keyed on the full argument tuple. The hot path is now an
    // O(1) HashMap lookup for repeated calls with the same inputs,
    // which is the common case (same label, same size, same style,
    // re-rendered every frame).
    let key = MetricKey::new(text, font_size, max_width, weight, style);
    METRICS_CACHE.with(|cell| {
        cell.borrow_mut()
            .get_or_insert_with(key, || {
                compute_metrics(text, font_size, weight, style, max_width)
            })
    })
}

fn compute_metrics(
    text: &str,
    font_size: f32,
    weight: FontWeight,
    style: FontStyle,
    max_width: f32,
) -> TextMetrics {
    // Bug fix 4.1 note: `chars().count()` is O(n) and walks UTF-8.
    // The width heuristic is itself an approximation, so the
    // function is "fast and approximate" overall. The real shape cache
    // in `text_engine` is the source of truth for layout; this function
    // exists for callers that need a width estimate without paying the
    // full shaping cost. Cached at the call site (above) for the
    // common re-render case.
    let glyph_count = text.chars().count() as f32;
    let weight_scale = if matches!(weight, FontWeight::Bold) {
        1.08
    } else {
        1.0
    };
    let style_scale = if matches!(style, FontStyle::Italic) {
        1.04
    } else {
        1.0
    };
    let width_heuristic = if matches!(weight, FontWeight::Bold) {
        TEXT_WIDTH_HEURISTIC_BOLD
    } else {
        TEXT_WIDTH_HEURISTIC
    };
    let width = (glyph_count * font_size * width_heuristic * weight_scale * style_scale)
        .max(font_size)
        .min(max_width.max(font_size));
    let height = (font_size * 1.25).max(1.0);
    let baseline = font_size * 0.9;

    TextMetrics {
        width,
        height,
        baseline,
    }
}

pub fn metrics_size(metrics: TextMetrics) -> Size {
    Size::new(metrics.width, metrics.height)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Bug fix 4.2: cache hit on identical inputs. Two calls with
    // the same argument tuple must return the same TextMetrics
    // (by value, no recomputation) and the cache entry count
    // must not grow on the second call.
    //
    // Each test uses a unique text string so that parallel test
    // execution doesn't race on the process-global cache.
    #[test]
    fn measure_text_caches_identical_calls() {
        // Snapshot the entry count for this unique key by reading
        // before and after; the second call must be a no-op insert.
        let m1 = measure_text(
            "tt-cache-identical",
            14.0,
            FontWeight::Normal,
            FontStyle::Normal,
            200.0,
        );
        let entries_after_first = metrics_cache_stats().entries;
        let m2 = measure_text(
            "tt-cache-identical",
            14.0,
            FontWeight::Normal,
            FontStyle::Normal,
            200.0,
        );
        assert_eq!(m1, m2);
        assert_eq!(
            metrics_cache_stats().entries,
            entries_after_first,
            "second call with same key should hit cache"
        );
    }

    #[test]
    fn measure_text_different_size_is_a_distinct_key() {
        let entries_before = metrics_cache_stats().entries;
        let _ = measure_text(
            "tt-distinct-size",
            14.0,
            FontWeight::Normal,
            FontStyle::Normal,
            200.0,
        );
        let _ = measure_text(
            "tt-distinct-size",
            20.0,
            FontWeight::Normal,
            FontStyle::Normal,
            200.0,
        );
        let entries_after = metrics_cache_stats().entries;
        assert_eq!(
            entries_after - entries_before,
            2,
            "size difference should add a second cache entry"
        );
    }

    #[test]
    fn measure_text_different_weight_is_a_distinct_key() {
        let entries_before = metrics_cache_stats().entries;
        let _ = measure_text(
            "tt-distinct-weight",
            14.0,
            FontWeight::Normal,
            FontStyle::Normal,
            200.0,
        );
        let _ = measure_text(
            "tt-distinct-weight",
            14.0,
            FontWeight::Bold,
            FontStyle::Normal,
            200.0,
        );
        let entries_after = metrics_cache_stats().entries;
        assert_eq!(
            entries_after - entries_before,
            2,
            "weight difference should add a second cache entry"
        );
    }

    #[test]
    fn clear_metrics_cache_evicts_everything() {
        let _ = measure_text(
            "tt-clear-a",
            14.0,
            FontWeight::Normal,
            FontStyle::Normal,
            100.0,
        );
        let _ = measure_text(
            "tt-clear-b",
            14.0,
            FontWeight::Normal,
            FontStyle::Normal,
            100.0,
        );
        let before = metrics_cache_stats().entries;
        assert!(before >= 2, "fixture should populate at least 2 entries");
        let evicted = clear_metrics_cache();
        assert_eq!(evicted, before);
        assert_eq!(metrics_cache_stats().entries, 0);
    }
}
