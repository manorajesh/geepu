use std::fmt;

/// Error types for Geepu operations
#[derive(Debug)]
pub enum GeepuError {
    /// Failed to create wgpu adapter
    AdapterNotFound,
    /// Failed to create wgpu device
    DeviceCreationFailed(wgpu::RequestDeviceError),
    /// Failed to create surface
    SurfaceCreationFailed,
    /// Shader compilation error
    ShaderError(String),
    /// Buffer creation error
    BufferError(String),
    /// Texture creation error
    TextureError(String),
    /// Pipeline creation error
    PipelineError(String),
    /// Generic error with message
    Other(String),
}

impl fmt::Display for GeepuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GeepuError::AdapterNotFound => write!(f, "No suitable GPU adapter found"),
            GeepuError::DeviceCreationFailed(e) => write!(f, "Failed to create GPU device: {}", e),
            GeepuError::SurfaceCreationFailed => write!(f, "Failed to create rendering surface"),
            GeepuError::ShaderError(msg) => write!(f, "Shader error: {}", msg),
            GeepuError::BufferError(msg) => write!(f, "Buffer error: {}", msg),
            GeepuError::TextureError(msg) => write!(f, "Texture error: {}", msg),
            GeepuError::PipelineError(msg) => write!(f, "Pipeline error: {}", msg),
            GeepuError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for GeepuError {}

pub type Result<T> = std::result::Result<T, GeepuError>;
