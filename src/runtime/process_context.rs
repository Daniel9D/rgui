//! Phase 4 / Plan 04-01: `ProcessContext` placeholder.
//!
//! In 04-01 this is a zero-sized stub that exists only to give
//! `UiRuntime::for_window` a stable signature. Plan 04-02 expands
//! the struct to the full D-13 shape: `node_ids: NodeIdAllocator`
//! and `a11y: Option<SharedAccessibility>`.
//!
//! Hosts construct one `ProcessContext` per process and pass it to
//! every `UiRuntime::for_window(id, &ctx)` call. The `_private`
//! field keeps the type unnameable to outside code so future
//! refactors stay source-compatible.

/// Per-process shared state passed to every `UiRuntime::for_window`.
#[derive(Copy, Clone, Debug, Default)]
pub struct ProcessContext {
    _private: (),
}

impl ProcessContext {
    /// Construct a new `ProcessContext`. The 04-02 expansion will
    /// take constructor parameters (`SharedWgpuDevice`, optional
    /// `SharedAccessibility`); the 04-01 stub takes none.
    pub const fn new() -> Self {
        Self { _private: () }
    }
}
