//! # Geepu - Ergonomic wgpu Wrapper for Rust
//!
//! Geepu is a zero-cost, ergonomic wrapper around wgpu that simplifies GPU programming in Rust
//! while maintaining performance. It provides high-level abstractions for GPU operations without
//! sacrificing the power and flexibility of wgpu.
//!
//! ## Features
//!
//! - **Easy GPU Initialization**: Simple setup for windowed or offscreen rendering
//! - **Shader Management**: Load and compile WGSL shaders with error handling
//! - **Resource Management**: Register uniforms, storage buffers, textures, and samplers
//! - **Render Passes**: High-level abstraction for drawing operations
//! - **Compute Shaders**: Support for compute pipelines and workgroup dispatch
//! - **Flexible Configuration**: Sensible defaults with advanced wgpu configuration options
//! - **Comprehensive Logging**: Built-in tracing support for debugging and profiling
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use geepu::{Renderer, WindowConfig, Size};
//! 
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // For windowed rendering
//! let mut renderer = Renderer::new(WindowConfig::default()).await?;
//! 
//! // For offscreen rendering
//! let mut renderer = Renderer::offscreen(Size::new(1920, 1080)).await?;
//! 
//! // Add resources
//! let matrix = [[1.0f32; 4]; 4]; // Identity matrix
//! renderer.add_uniform("mvp_matrix", &matrix);
//! 
//! // Load a texture (would need actual image data)
//! // renderer.add_texture("diffuse", image)?;
//! 
//! // Render
//! let mut pass = renderer.begin_pass();
//! // pass.draw_indexed(0..6, 0, 0..1)?; // Would need actual geometry
//! drop(pass);
//! renderer.submit();
//! # Ok(())
//! # }
//! ```

pub mod config;
pub mod renderer;
pub mod resources;
pub mod shaders;
pub mod error;

pub use config::{WindowConfig, Size, GpuConfig};
pub use renderer::{Renderer, RenderPassGuard};
pub use resources::{UniformBuffer, StorageBuffer, TextureResource};
pub use shaders::{ShaderManager, ComputePipeline};
pub use error::{GeepuError, Result};

/// Re-export commonly used wgpu types for convenience
pub mod wgpu {
    pub use wgpu::*;
}

/// Re-export tracing for logging
pub mod tracing {
    pub use tracing::*;
}
