//! Basic triangle rendering example
//! 
//! This example demonstrates:
//! - Creating a windowed renderer
//! - Loading shaders
//! - Setting up uniforms
//! - Basic rendering

use geepu::{Renderer, WindowConfig, Size, GpuConfig};
use tracing::info;
use anyhow::Result;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct MVP {
    matrix: [[f32; 4]; 4],
}

const TRIANGLE_VERTEX_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> mvp: mat4x4<f32>;

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = mvp * vec4<f32>(model.position, 1.0);
    return out;
}
"#;

const TRIANGLE_FRAGMENT_SHADER: &str = r#"
@fragment
fn fs_main(@location(0) color: vec3<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(color, 1.0);
}
"#;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("Starting triangle example");

    // Create renderer
    let window_config = WindowConfig::new("Geepu Triangle", Size::new(800, 600));
    let gpu_config = GpuConfig::performance();
    let mut renderer = Renderer::new_with_gpu_config(window_config, gpu_config).await?;

    // Define triangle vertices
    let vertices = vec![
        Vertex { position: [ 0.0,  0.5, 0.0], color: [1.0, 0.0, 0.0] }, // Top (red)
        Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] }, // Bottom-left (green)
        Vertex { position: [ 0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] }, // Bottom-right (blue)
    ];

    // Create MVP matrix (identity for this simple example)
    let mvp = MVP {
        matrix: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };

    // Add resources
    renderer.add_uniform("mvp", &mvp);
    info!("Added MVP uniform");

    // Load shaders
    renderer.shaders.load_vertex_shader(&renderer.device, "triangle_vs", TRIANGLE_VERTEX_SHADER)?;
    renderer.shaders.load_fragment_shader(&renderer.device, "triangle_fs", TRIANGLE_FRAGMENT_SHADER)?;
    info!("Loaded shaders");

    // Simulate rendering loop
    for frame in 0..10 {
        info!("Rendering frame {}", frame);

        // Rotate the triangle slightly each frame
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

        renderer.update_uniform("mvp", &rotated_mvp)?;

        // Begin render pass
        let mut pass = renderer.begin_pass();
        
        // In a complete implementation, you would:
        // 1. Create vertex buffer for the triangle
        // 2. Create a render pipeline with the loaded shaders
        // 3. Set the bind group for the MVP uniform
        // 4. Set the vertex buffer
        // 5. Draw the triangle
        
        // For this example, we simulate the draw call
        info!("Drawing triangle with {} vertices", vertices.len());
        
        drop(pass);
        renderer.submit();
    }

    info!("Triangle example completed");
    Ok(())
}
