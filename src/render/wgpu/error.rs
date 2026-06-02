use thiserror::Error;

use crate::core::DisplayListError;

#[derive(Debug, Error)]
pub enum RendererError {
    #[error("no compatible wgpu adapter found")]
    NoAdapter,

    #[error("failed to request wgpu device: {0}")]
    RequestDevice(#[from] wgpu::RequestDeviceError),

    #[error("surface error: {0}")]
    Surface(String),

    #[error("buffer mapping failed: {0}")]
    BufferMap(#[from] wgpu::BufferAsyncError),

    #[error("texture readback failed: {0}")]
    Readback(String),

    /// Display-list validation failed. The wrapped
    /// [`DisplayListError`] is structured; match on its
    /// variants to handle specific cases. Bug fix 5.7: the
    /// previous payload was a free-form `String`, which
    /// callers couldn't match on. The `From<DisplayListError>`
    /// impl makes `?` work transparently from
    /// `DisplayList::validate`.
    #[error("display list validation failed: {0}")]
    DisplayListValidation(#[from] DisplayListError),

    /// Lowering / atlas / glyphon errors that don't map to a
    /// `DisplayListError`. Kept as a free-form `String` for
    /// now; each call site owns its message format.
    #[error("display list invalid: {0}")]
    InvalidDisplayList(String),
}

pub type RendererResult<T> = Result<T, RendererError>;
