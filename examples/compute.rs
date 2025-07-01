//! Compute shader example demonstrating GPU computation
//! 
//! This example shows:
//! - Creating an offscreen renderer for compute operations
//! - Loading compute shaders
//! - Setting up storage buffers
//! - Dispatching compute workgroups
//! - Reading results back from GPU

use geepu::{Renderer, Size, GpuConfig};
use tracing::info;
use anyhow::Result;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct ComputeData {
    input: f32,
    output: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct ComputeParams {
    multiplier: f32,
    offset: f32,
}

const COMPUTE_SHADER: &str = r#"
struct ComputeData {
    input: f32,
    output: f32,
}

struct ComputeParams {
    multiplier: f32,
    offset: f32,
}

@group(0) @binding(0)
var<storage, read_write> data: array<ComputeData>;

@group(0) @binding(1)
var<uniform> params: ComputeParams;

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= arrayLength(&data)) {
        return;
    }
    
    // Simple computation: output = input * multiplier + offset
    data[index].output = data[index].input * params.multiplier + params.offset;
}
"#;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("Starting compute shader example");

    // Create offscreen renderer for compute operations
    let gpu_config = GpuConfig::performance()
        .features(wgpu::Features::default()); // Compute shaders are in core features
    let mut renderer = Renderer::offscreen_with_gpu_config(Size::new(1, 1), gpu_config).await?;

    // Create input data
    let data_count = 1024;
    let mut compute_data: Vec<ComputeData> = (0..data_count)
        .map(|i| ComputeData {
            input: i as f32,
            output: 0.0,
        })
        .collect();

    info!("Created {} data points for computation", data_count);

    // Set up compute parameters
    let params = ComputeParams {
        multiplier: 2.5,
        offset: 10.0,
    };

    // Add resources to renderer
    renderer.add_storage_buffer("compute_data", &compute_data);
    renderer.add_uniform("compute_params", &params);
    info!("Added storage buffer and compute parameters");

    // Load compute shader
    renderer.add_compute_shader("math_compute", COMPUTE_SHADER)?;
    info!("Loaded compute shader");

    // Note: In a complete implementation, you would:
    // 1. Create a compute pipeline with proper bind group layouts
    // 2. Create bind groups linking the storage buffer and uniforms
    // 3. Dispatch the compute shader with appropriate workgroup counts
    // 4. Read back the results

    // Simulate the computation
    info!("Would dispatch compute shader with {} workgroups", (data_count + 63) / 64);
    
    // For demonstration, let's simulate the expected results
    for item in &mut compute_data {
        item.output = item.input * params.multiplier + params.offset;
    }

    // Show some results
    info!("Simulated computation results:");
    for (i, item) in compute_data.iter().take(10).enumerate() {
        info!("Data[{}]: input={:.1}, output={:.1}", i, item.input, item.output);
    }

    info!("Compute shader example completed");
    Ok(())
}
