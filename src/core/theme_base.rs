//! Absolute z-index and hit-test-order base values used by the overlay
//! system. These are *absolute* z-indices (not the relative
//! [`crate::core::LayerKind::order`] used to sort layers within a single
//! paint pass). They guarantee overlays always sit above document content.
//!
//! For the per-layer ordering, see [`crate::core::LayerKind::order`].

/// Z-index used as the base for the overlay panel. Anything at or above
/// this is considered an overlay. Document content uses z-indices below
/// this value.
pub const OVERLAY_PANEL_Z_BASE: i32 = 1000;

/// Z-index used for content drawn inside an overlay panel.
pub const OVERLAY_CONTENT_Z_BASE: i32 = OVERLAY_PANEL_Z_BASE + 1;

/// Z-index used for the modal panel itself. Equals `OVERLAY_CONTENT_Z_BASE`
/// so a modal panel and a popover panel render at the same z.
pub const MODAL_PANEL_Z_BASE: i32 = OVERLAY_PANEL_Z_BASE + 1;

/// Z-index used for content drawn inside a modal panel.
pub const MODAL_CONTENT_Z_BASE: i32 = OVERLAY_PANEL_Z_BASE + 3;

/// Z-offset added to a scrollbar's underlying element's z-index so the
/// thumb sits above the element it scrolls.
pub const SCROLLBAR_THUMB_Z_OFFSET: i32 = 3;

/// Hit-test order for the overlay layer.
pub const OVERLAY_HIT_TEST_ORDER: usize = usize::MAX;

/// Hit-test order for the overlay panel itself.
pub const OVERLAY_PANEL_HIT_TEST_ORDER: usize = usize::MAX - 1;

/// Hit-test order for the modal backdrop.
pub const OVERLAY_BACKDROP_HIT_TEST_ORDER: usize = usize::MAX - 2;

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression: the constants must be ordered so the overlay layer
    /// sorts above the document layer, and the modal content sorts above
    /// the modal panel.
    #[test]
    fn z_bases_are_strictly_ordered() {
        assert!(OVERLAY_PANEL_Z_BASE < OVERLAY_CONTENT_Z_BASE);
        assert!(OVERLAY_PANEL_Z_BASE < MODAL_PANEL_Z_BASE);
        assert!(MODAL_PANEL_Z_BASE < MODAL_CONTENT_Z_BASE);
    }

    /// Hit-test order must be a total order: overlay panel must come
    /// before its backdrop.
    #[test]
    fn hit_test_orders_are_strictly_ordered() {
        assert!(OVERLAY_PANEL_HIT_TEST_ORDER < OVERLAY_HIT_TEST_ORDER);
        assert!(OVERLAY_BACKDROP_HIT_TEST_ORDER < OVERLAY_PANEL_HIT_TEST_ORDER);
    }
}
