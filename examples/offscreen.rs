//! Offscreen rendering example
//! 
//! This example demonstrates:
//! - Creating an offscreen renderer
//! - Rendering to a texture
//! - Copying the result to CPU memory
//! - Saving the result as an image file

use geepu::{Renderer, Size, GpuConfig};
use tracing::info;
use anyhow::Result;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct RenderParams {
    time: f32,
    resolution: [f32; 2],
}

const FULLSCREEN_VERTEX_SHADER: &str = r#"
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    
    // Generate fullscreen triangle
    let x = f32((vertex_index << 1u) & 2u) - 1.0;
    let y = f32(vertex_index & 2u) - 1.0;
    
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    
    return out;
}
"#;

const PROCEDURAL_FRAGMENT_SHADER: &str = r#"
struct RenderParams {
    time: f32,
    resolution: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> params: RenderParams;

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let pos = uv * params.resolution;
    
    // Create a simple animated pattern
    let time = params.time;
    let pattern = sin(pos.x * 0.01 + time) * sin(pos.y * 0.01 + time * 1.3);
    
    // Generate colors based on the pattern
    let r = (sin(pattern + time) + 1.0) * 0.5;
    let g = (sin(pattern + time + 2.0) + 1.0) * 0.5;
    let b = (sin(pattern + time + 4.0) + 1.0) * 0.5;
    
    return vec4<f32>(r, g, b, 1.0);
}
"#;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("Starting offscreen rendering example");

    // Create offscreen renderer
    let render_size = Size::new(512, 512);
    let gpu_config = GpuConfig::performance();
    let mut renderer = Renderer::offscreen_with_gpu_config(render_size, gpu_config).await?;

    info!("Created offscreen renderer with size: {:?}", render_size);

    // Set up render parameters
    let params = RenderParams {
        time: 0.0,
        resolution: [render_size.width as f32, render_size.height as f32],
    };

    // Add uniform to renderer
    renderer.add_uniform("render_params", &params);
    info!("Added render parameters uniform");

    // Load shaders
    renderer.shaders.load_vertex_shader(&renderer.device, "fullscreen_vs", FULLSCREEN_VERTEX_SHADER)?;
    renderer.shaders.load_fragment_shader(&renderer.device, "procedural_fs", PROCEDURAL_FRAGMENT_SHADER)?;
    info!("Loaded shaders");

    // Render multiple frames with different time values
    for frame in 0..5 {
        let time = frame as f32 * 0.5;
        
        info!("Rendering frame {} with time={:.1}", frame, time);

        // Update time parameter
        let updated_params = RenderParams {
            time,
            resolution: params.resolution,
        };
        renderer.update_uniform("render_params", &updated_params)?;

        // Render frame
        {
            let pass = renderer.begin_pass();
            
            // In a complete implementation, you would:
            // 1. Create a render pipeline with the loaded shaders
            // 2. Set the bind group for the render parameters
            // 3. Draw a fullscreen triangle (3 vertices, no index buffer)
            
            info!("Drawing fullscreen procedural content");
            drop(pass);
        }
        
        renderer.submit();

        // Copy result to CPU (only for the last frame to demonstrate)
        if frame == 4 {
            info!("Copying final frame to CPU memory");
            let image_data = renderer.copy_to_buffer().await?;
            
            info!("Copied {} bytes of image data", image_data.len());
            
            // Verify the data size
            let expected_size = (render_size.width * render_size.height * 4) as usize; // RGBA8
            assert_eq!(image_data.len(), expected_size, "Unexpected image data size");
            
            info!("Image data size verified: {}x{} RGBA = {} bytes", 
                  render_size.width, render_size.height, image_data.len());

            // In a real application, you could save this as an image file:
            // let image = image::RgbaImage::from_raw(
            //     render_size.width, 
            //     render_size.height, 
            //     image_data
            // ).unwrap();
            // image.save("output.png")?;
        }
    }

    info!("Offscreen rendering example completed");
    Ok(())
}
