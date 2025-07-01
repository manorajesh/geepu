//! Error types for Geepu

use std::fmt;

/// Main error type for Geepu operations
#[derive(Debug)]
pub enum GeepuError {
    /// wgpu-related errors
    Wgpu(wgpu::Error),
    /// Surface configuration errors
    SurfaceError(wgpu::SurfaceError),
    /// Request adapter failed
    AdapterNotFound,
    /// Request device failed
    DeviceRequestFailed(wgpu::RequestDeviceError),
    /// Shader compilation error
    ShaderCompilation(String),
    /// Resource not found
    ResourceNotFound(String),
    /// Invalid operation
    InvalidOperation(String),
    /// IO errors
    Io(std::io::Error),
    /// Image processing errors
    Image(image::ImageError),
    /// Generic error with message
    Generic(String),
}

impl fmt::Display for GeepuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GeepuError::Wgpu(err) => write!(f, "wgpu error: {}", err),
            GeepuError::SurfaceError(err) => write!(f, "surface error: {}", err),
            GeepuError::AdapterNotFound => write!(f, "no suitable GPU adapter found"),
            GeepuError::DeviceRequestFailed(err) => write!(f, "failed to request device: {}", err),
            GeepuError::ShaderCompilation(msg) => write!(f, "shader compilation error: {}", msg),
            GeepuError::ResourceNotFound(name) => write!(f, "resource '{}' not found", name),
            GeepuError::InvalidOperation(msg) => write!(f, "invalid operation: {}", msg),
            GeepuError::Io(err) => write!(f, "IO error: {}", err),
            GeepuError::Image(err) => write!(f, "image error: {}", err),
            GeepuError::Generic(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for GeepuError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            GeepuError::Wgpu(err) => Some(err),
            GeepuError::SurfaceError(err) => Some(err),
            GeepuError::DeviceRequestFailed(err) => Some(err),
            GeepuError::Io(err) => Some(err),
            GeepuError::Image(err) => Some(err),
            _ => None,
        }
    }
}

impl From<wgpu::Error> for GeepuError {
    fn from(err: wgpu::Error) -> Self {
        GeepuError::Wgpu(err)
    }
}

impl From<wgpu::SurfaceError> for GeepuError {
    fn from(err: wgpu::SurfaceError) -> Self {
        GeepuError::SurfaceError(err)
    }
}

impl From<wgpu::RequestDeviceError> for GeepuError {
    fn from(err: wgpu::RequestDeviceError) -> Self {
        GeepuError::DeviceRequestFailed(err)
    }
}

impl From<std::io::Error> for GeepuError {
    fn from(err: std::io::Error) -> Self {
        GeepuError::Io(err)
    }
}

impl From<image::ImageError> for GeepuError {
    fn from(err: image::ImageError) -> Self {
        GeepuError::Image(err)
    }
}

/// Result type alias for Geepu operations
pub type Result<T> = std::result::Result<T, GeepuError>;
