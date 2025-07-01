//! Geepu Example - Comprehensive demonstration of the library features
//!
//! This example demonstrates:
//! - Creating windowed and offscreen renderers
//! - Loading shaders from files and strings
//! - Managing uniforms, storage buffers, and textures
//! - Compute shader dispatch
//! - Rendering operations
//! - Logging with tracing

use geepu::{Renderer, WindowConfig, Size, GpuConfig};
use tracing::{info, Level};
use tracing_subscriber;
use anyhow::Result;
use bytemuck::{Pod, Zeroable};

// Example vertex structure
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

// Example uniform structure
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct MVP {
    matrix: [[f32; 4]; 4],
}

// Example uniform for compute shader
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct Multiplier {
    value: f32,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    info!("Starting Geepu comprehensive example");

    // Run different examples
    hello_triangle_example().await?;
    compute_shader_example().await?;
    offscreen_rendering_example().await?;
    texture_example().await?;

    info!("All examples completed successfully!");
    Ok(())
}

/// Basic "Hello Triangle" example with textured quad
async fn hello_triangle_example() -> Result<()> {
    let span = tracing::span!(Level::INFO, "hello_triangle_example");
    let _enter = span.enter();

    info!("Running Hello Triangle example");

    // Create windowed renderer
    let window_config = WindowConfig::new("Geepu Hello Triangle", Size::new(800, 600))
        .resizable(true)
        .vsync(true);

    let mut renderer = Renderer::new_with_gpu_config(
        window_config,
        GpuConfig::performance()
    ).await?;

    info!("Renderer created successfully");

    // Define triangle vertices (actually a quad for texture demonstration)
    let _vertices = vec![
        Vertex { position: [-0.5, -0.5, 0.0], tex_coords: [0.0, 1.0] }, // Bottom-left
        Vertex { position: [ 0.5, -0.5, 0.0], tex_coords: [1.0, 1.0] }, // Bottom-right
        Vertex { position: [ 0.5,  0.5, 0.0], tex_coords: [1.0, 0.0] }, // Top-right
        Vertex { position: [-0.5,  0.5, 0.0], tex_coords: [0.0, 0.0] }, // Top-left
    ];

    let indices = vec![0u16, 1, 2, 2, 3, 0];

    // Create MVP matrix (identity for this simple example)
    let mvp = MVP {
        matrix: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };

    // Add resources to renderer
    renderer.add_uniform("mvp_matrix", &mvp);
    info!("Added MVP matrix uniform");

    // Create a simple procedural texture
    let texture_data = create_checkerboard_texture(256, 256);
    renderer.add_texture("checkerboard", texture_data)?;
    info!("Added checkerboard texture");

    // Load shaders (using built-in default shaders for this example)
    renderer.shaders.load_vertex_shader(
        &renderer.device,
        "quad_vertex",
        geepu::shaders::default_shaders::TEXTURED_QUAD_VERTEX,
    )?;

    renderer.shaders.load_fragment_shader(
        &renderer.device,
        "quad_fragment", 
        geepu::shaders::default_shaders::TEXTURED_QUAD_FRAGMENT,
    )?;

    info!("Loaded vertex and fragment shaders");

    // Simulate a simple render loop
    for frame in 0..5 {
        info!("Rendering frame {}", frame);

        // Update uniform each frame (simple rotation)
        let angle = frame as f32 * 0.1;
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        
        let rotated_mvp = MVP {
            matrix: [
                [cos_a, -sin_a, 0.0, 0.0],
                [sin_a,  cos_a, 0.0, 0.0],
                [0.0,    0.0,   1.0, 0.0],
                [0.0,    0.0,   0.0, 1.0],
            ],
        };

        renderer.update_uniform("mvp_matrix", &rotated_mvp)?;

        // Begin render pass
        let mut pass = renderer.begin_pass();
        
        // In a real implementation, you would:
        // 1. Create vertex/index buffers
        // 2. Create render pipeline with shaders
        // 3. Set bind groups for uniforms and textures
        // 4. Set vertex/index buffers
        // 5. Draw the geometry
        
        // For this example, we'll just simulate drawing
        pass.draw_indexed(0..indices.len() as u32, 0, 0..1)?;
        
        drop(pass);
        renderer.submit();

        // Simulate presenting (in a real app, you'd handle the event loop)
        info!("Frame {} rendered and submitted", frame);
    }

    info!("Hello Triangle example completed");
    Ok(())
}

/// Compute shader example demonstrating GPU computation
async fn compute_shader_example() -> Result<()> {
    let span = tracing::span!(Level::INFO, "compute_shader_example");
    let _enter = span.enter();

    info!("Running Compute Shader example");

    // Create offscreen renderer for compute operations
    let mut renderer = Renderer::offscreen(Size::new(1, 1)).await?;

    // Create data for computation
    let data: Vec<f32> = (0..1024).map(|i| i as f32).collect();
    let multiplier = Multiplier { value: 2.0 };

    info!("Created {} data points for computation", data.len());

    // Add resources
    renderer.add_storage_buffer("compute_data", &data);
    renderer.add_uniform("multiplier", &multiplier);

    // Load compute shader
    renderer.add_compute_shader(
        "multiply_shader",
        geepu::shaders::default_shaders::ARRAY_MULTIPLY_COMPUTE,
    )?;

    info!("Loaded compute shader");

    // For this example, we'll skip creating the full compute pipeline
    // since it requires complex bind group layout management
    info!("Would create compute pipeline here in a real implementation");

    // Simulate dispatch
    info!("Would dispatch compute shader here");

    info!("Compute shader example completed (simulated)");
    Ok(())
}

/// Offscreen rendering example
async fn offscreen_rendering_example() -> Result<()> {
    let span = tracing::span!(Level::INFO, "offscreen_rendering_example");
    let _enter = span.enter();

    info!("Running Offscreen Rendering example");

    // Create offscreen renderer
    let mut renderer = Renderer::offscreen(Size::new(512, 512)).await?;

    // Create simple scene data
    let mvp = MVP {
        matrix: [
            [0.8, 0.0, 0.0, 0.0],
            [0.0, 0.8, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };

    renderer.add_uniform("mvp_matrix", &mvp);

    // Add a colorful texture
    let gradient_texture = create_gradient_texture(256, 256);
    renderer.add_texture("gradient", gradient_texture)?;

    info!("Set up offscreen scene");

    // Render to offscreen target
    {
        let mut pass = renderer.begin_pass();
        // Simulate rendering a scene
        pass.draw_indexed(0..6, 0, 0..1)?;
        drop(pass);
        renderer.submit();
    }

    info!("Rendered scene to offscreen target");

    // Copy result to buffer
    let image_data = renderer.copy_to_buffer().await?;
    
    // Save as image (you could use the image crate to save to file)
    info!("Copied {} bytes from render target", image_data.len());
    
    // Verify we got the expected amount of data
    let expected_size = 512 * 512 * 4; // RGBA8
    assert_eq!(image_data.len(), expected_size, "Unexpected image data size");

    info!("Offscreen rendering example completed");
    Ok(())
}

/// Texture loading and management example
async fn texture_example() -> Result<()> {
    let span = tracing::span!(Level::INFO, "texture_example");
    let _enter = span.enter();

    info!("Running Texture example");

    let mut renderer = Renderer::offscreen(Size::new(256, 256)).await?;

    // Create various textures
    let red_texture = create_solid_color_texture(128, 128, [255, 0, 0, 255]);
    let green_texture = create_solid_color_texture(128, 128, [0, 255, 0, 255]);
    let blue_texture = create_solid_color_texture(128, 128, [0, 0, 255, 255]);

    renderer.add_texture("red", red_texture)?;
    renderer.add_texture("green", green_texture)?;
    renderer.add_texture("blue", blue_texture)?;

    info!("Added red, green, and blue textures");

    // Test texture retrieval
    let red_tex = renderer.resources.get_texture("red")?;
    let green_tex = renderer.resources.get_texture("green")?;
    let blue_tex = renderer.resources.get_texture("blue")?;

    info!("Successfully retrieved all textures");
    info!("Red texture size: {:?}", red_tex.texture.size());
    info!("Green texture size: {:?}", green_tex.texture.size());
    info!("Blue texture size: {:?}", blue_tex.texture.size());

    info!("Texture example completed");
    Ok(())
}

// Helper functions for creating example textures

fn create_checkerboard_texture(width: u32, height: u32) -> image::DynamicImage {
    let mut data = Vec::with_capacity((width * height * 4) as usize);
    
    for y in 0..height {
        for x in 0..width {
            let checker = ((x / 32) + (y / 32)) % 2;
            let color = if checker == 0 { 255 } else { 64 };
            data.extend_from_slice(&[color, color, color, 255]);
        }
    }
    
    image::DynamicImage::ImageRgba8(
        image::RgbaImage::from_raw(width, height, data).unwrap()
    )
}

fn create_gradient_texture(width: u32, height: u32) -> image::DynamicImage {
    let mut data = Vec::with_capacity((width * height * 4) as usize);
    
    for y in 0..height {
        for x in 0..width {
            let r = (x as f32 / width as f32 * 255.0) as u8;
            let g = (y as f32 / height as f32 * 255.0) as u8;
            let b = 128;
            data.extend_from_slice(&[r, g, b, 255]);
        }
    }
    
    image::DynamicImage::ImageRgba8(
        image::RgbaImage::from_raw(width, height, data).unwrap()
    )
}

fn create_solid_color_texture(width: u32, height: u32, color: [u8; 4]) -> image::DynamicImage {
    let data = vec![color; (width * height) as usize].concat();
    
    image::DynamicImage::ImageRgba8(
        image::RgbaImage::from_raw(width, height, data).unwrap()
    )
}
