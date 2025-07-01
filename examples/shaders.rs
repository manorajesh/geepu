//! File-based shader loading example
//! 
//! This example demonstrates:
//! - Loading shaders from external WGSL files
//! - Using the built-in shader examples
//! - Error handling for missing files

use geepu::{Renderer, Size, GpuConfig, shaders::ShaderType};
use tracing::{info, warn};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("Starting shader loading example");

    // Create offscreen renderer
    let mut renderer = Renderer::offscreen_with_gpu_config(
        Size::new(256, 256),
        GpuConfig::performance()
    ).await?;

    info!("Created renderer");

    // Try to load shaders from files (these files exist in the project)
    match renderer.shaders.load_shader_from_file(
        &renderer.device,
        "quad_vertex_file",
        "shaders/quad.vert.wgsl",
        ShaderType::Vertex,
    ) {
        Ok(_) => info!("Successfully loaded vertex shader from file"),
        Err(e) => warn!("Failed to load vertex shader from file: {}", e),
    }

    match renderer.shaders.load_shader_from_file(
        &renderer.device,
        "quad_fragment_file",
        "shaders/quad.frag.wgsl",
        ShaderType::Fragment,
    ) {
        Ok(_) => info!("Successfully loaded fragment shader from file"),
        Err(e) => warn!("Failed to load fragment shader from file: {}", e),
    }

    match renderer.shaders.load_shader_from_file(
        &renderer.device,
        "compute_file",
        "shaders/multiply.comp.wgsl",
        ShaderType::Compute,
    ) {
        Ok(_) => info!("Successfully loaded compute shader from file"),
        Err(e) => warn!("Failed to load compute shader from file: {}", e),
    }

    // Load built-in shaders for comparison
    info!("Loading built-in shaders");

    renderer.shaders.load_vertex_shader(
        &renderer.device,
        "builtin_vertex",
        geepu::shaders::default_shaders::TEXTURED_QUAD_VERTEX,
    )?;

    renderer.shaders.load_fragment_shader(
        &renderer.device,
        "builtin_fragment", 
        geepu::shaders::default_shaders::TEXTURED_QUAD_FRAGMENT,
    )?;

    renderer.add_compute_shader(
        "builtin_compute",
        geepu::shaders::default_shaders::ARRAY_MULTIPLY_COMPUTE,
    )?;

    info!("All built-in shaders loaded successfully");

    // Test shader retrieval
    match renderer.shaders.get_vertex_shader("builtin_vertex") {
        Ok(_) => info!("Successfully retrieved vertex shader"),
        Err(e) => warn!("Failed to retrieve vertex shader: {}", e),
    }

    match renderer.shaders.get_fragment_shader("builtin_fragment") {
        Ok(_) => info!("Successfully retrieved fragment shader"),
        Err(e) => warn!("Failed to retrieve fragment shader: {}", e),
    }

    match renderer.shaders.get_compute_shader("builtin_compute") {
        Ok(_) => info!("Successfully retrieved compute shader"),
        Err(e) => warn!("Failed to retrieve compute shader: {}", e),
    }

    // Try to get a non-existent shader (should fail)
    match renderer.shaders.get_vertex_shader("nonexistent") {
        Ok(_) => warn!("Unexpectedly found non-existent shader"),
        Err(e) => info!("Expected error for non-existent shader: {}", e),
    }

    info!("Shader loading example completed");
    Ok(())
}
