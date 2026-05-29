use thiserror::Error;

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

    #[error("display list validation failed: {0}")]
    InvalidDisplayList(String),
}

pub type RendererResult<T> = Result<T, RendererError>;
