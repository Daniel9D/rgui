//! Phase 4 / Plan 04-01: `WindowId` newtype.
//!
//! The lib is host-agnostic; this newtype gives every `UiRuntime` a
//! first-class window identity without depending on any specific
//! windowing library. Hosts convert their own window-id type into
//! `WindowId` via `From` impls (e.g. `From<winit::window::WindowId>`
//! behind the `winit` feature).
//!
//! `WindowId::unknown()` (sentinel `WindowId(0)`) is the default
//! used by `UiRuntime::default()` for backward-compatible
//! single-window use. Multi-window hosts construct runtimes via
//! `UiRuntime::for_window(id, &ctx)`.

use std::fmt;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct WindowId(u64);

impl WindowId {
    /// Wrap a raw `u64` window id. Use this for hosts that have a
    /// `u64` window-id native to the platform (the common case).
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// The raw `u64` underlying the newtype.
    pub const fn raw(&self) -> u64 {
        self.0
    }

    /// Sentinel "no window" id. `UiRuntime::default()` and tests use
    /// this. `WindowId::unknown() == WindowId::default()`.
    pub const fn unknown() -> Self {
        Self(0)
    }
}

impl Default for WindowId {
    fn default() -> Self {
        Self::unknown()
    }
}

impl fmt::Debug for WindowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == Self::unknown() {
            write!(f, "WindowId(unknown)")
        } else {
            write!(f, "WindowId({})", self.0)
        }
    }
}

impl fmt::Display for WindowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == Self::unknown() {
            write!(f, "unknown")
        } else {
            write!(f, "{}", self.0)
        }
    }
}

impl From<winit::window::WindowId> for WindowId {
    /// Best-effort conversion from winit's `WindowId`.
    ///
    /// winit's `WindowId` is opaque (the inner `u64` field is
    /// `pub(crate)` and not accessible to dependents). We hash the
    /// id to produce a stable `u64` key. Hash collisions across the
    /// handful of windows a single process owns are essentially
    /// impossible; the resulting id is for runtime bookkeeping
    /// (focus routing, snapshot identity), not platform identity.
    /// Other host libraries (SDL, native platforms) should write
    /// their own `From` impls in their own crates.
    ///
    /// Unconditional: `winit` is an unconditional dep of the lib
    /// (the lib has no default features that would gate it out).
    fn from(id: winit::window::WindowId) -> Self {
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash(&id, &mut hasher);
        Self::new(hasher.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn unknown_equals_default() {
        assert_eq!(WindowId::unknown(), WindowId::default());
    }

    #[test]
    fn new_wraps_raw() {
        assert_eq!(WindowId::new(7).raw(), 7);
    }

    #[test]
    fn unknown_is_zero() {
        assert_eq!(WindowId::unknown().raw(), 0);
    }

    #[test]
    fn two_equal_ids_hash_the_same() {
        let mut map: HashMap<WindowId, &'static str> = HashMap::new();
        map.insert(WindowId::new(1), "alpha");
        map.insert(WindowId::new(2), "beta");
        assert_eq!(map.get(&WindowId::new(1)).copied(), Some("alpha"));
        assert_eq!(map.get(&WindowId::new(2)).copied(), Some("beta"));
        assert_eq!(map.get(&WindowId::new(3)), None);
        assert_eq!(map.len(), 2, "no overwrites on equal keys");
    }

    #[test]
    fn zero_and_one_are_distinct() {
        assert_ne!(WindowId::new(0), WindowId::new(1));
        assert_eq!(WindowId::new(0), WindowId::unknown());
    }

    #[test]
    fn ord_orders_by_raw() {
        assert!(WindowId::new(1) < WindowId::new(2));
        assert!(WindowId::new(2) > WindowId::new(1));
    }

    #[test]
    fn debug_formatting() {
        assert_eq!(format!("{:?}", WindowId::unknown()), "WindowId(unknown)");
        assert_eq!(format!("{:?}", WindowId::new(42)), "WindowId(42)");
    }
}
