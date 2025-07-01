# Geepu - Ergonomic wgpu Wrapper for Rust

Geepu is a zero-cost, ergonomic wrapper around wgpu that simplifies GPU programming in Rust while maintaining performance. It provides high-level abstractions for common GPU operations without sacrificing the power and flexibility of wgpu.

## Features

- **Zero-cost abstractions**: All wrapper types compile down to their underlying wgpu equivalents
- **Type-safe buffers**: `TypedBuffer<T>` provides compile-time type safety for GPU buffers
- **Ergonomic pipeline creation**: Simplified render and compute pipeline creation with builder patterns
- **Convenient command recording**: High-level render and compute pass abstractions
- **Comprehensive texture support**: Easy texture creation and management
- **Built-in common patterns**: Pre-built compute shader patterns for reductions, prefix sums, etc.
- **Macro support**: Convenient macros for vertex layouts and bind group layouts

## Quick Start

Add geepu to your `Cargo.toml`:

```toml
[dependencies]
geepu = "0.1.0"
```

### Basic Triangle Example

```rust
use geepu::*;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

async fn render_triangle() -> Result<()> {
    // Create GPU context (without window for compute-only)
    let context = GpuContext::new().await?;

    // Or with a window for rendering
    // let context = GpuContext::new_with_window(window).await?;

    // Create vertex data
    let vertices = [
        Vertex { position: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0] },
        Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] },
        Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
    ];

    // Create typed vertex buffer
    let vertex_buffer = TypedBuffer::vertex(&context, &vertices)?;

    // Define vertex layout with macro
    let vertex_layout = vertex_layout![
        0 => VertexFormat::Float32x3, // position
        1 => VertexFormat::Float32x3, // color
    ];

    // Create render pipeline
    let pipeline = RenderPipeline::simple(
        &context,
        vertex_shader_source,
        fragment_shader_source,
        &[vertex_layout],
        surface_format,
        Some("Triangle Pipeline"),
    )?;

    Ok(())
}
```

### Compute Example

```rust
use geepu::*;

async fn compute_example() -> Result<()> {
    let context = GpuContext::new().await?;

    // Create input and output buffers
    let input_data = vec![1.0f32; 1024];
    let input_buffer = TypedBuffer::storage(&context, &input_data)?;
    let output_buffer = TypedBuffer::<f32>::empty(&context, 1024,
        BufferUsages::STORAGE | BufferUsages::COPY_SRC)?;

    // Create bind group layout
    let bind_group_layout = BindGroupLayoutBuilder::new()
        .storage_buffer(0, ShaderStages::COMPUTE, true)  // read-only input
        .storage_buffer(1, ShaderStages::COMPUTE, false) // read-write output
        .build(&context, Some("Compute Layout"));

    // Create compute pipeline
    let pipeline = ComputePipeline::new(
        &context,
        compute_shader_source,
        vec![bind_group_layout],
        Some("Compute Pipeline"),
    )?;

    Ok(())
}
```

## Core Components

### GpuContext

The main entry point that manages the GPU instance, adapter, device, and queue:

```rust
// For compute-only applications
let context = GpuContext::new().await?;

// For rendering applications
let context = GpuContext::new_with_window(window).await?;

// With specific features
let context = GpuContext::new_with_features(Features::COMPUTE_SHADER).await?;
```

### TypedBuffer<T>

Type-safe buffer wrapper that prevents common GPU programming errors:

```rust
// Create different buffer types
let vertex_buffer = TypedBuffer::vertex(&context, &vertex_data)?;
let index_buffer = TypedBuffer::index(&context, &index_data)?;
let uniform_buffer = TypedBuffer::uniform(&context, &uniform_data)?;
let storage_buffer = TypedBuffer::storage(&context, &storage_data)?;

// Write data to buffers
uniform_buffer.write(&context, &new_uniform_data)?;
```

### Texture

Simplified texture creation and management:

```rust
// Create texture from raw data
let texture = Texture::from_bytes(&context, image_bytes, width, height, format, None)?;

// Create render target
let render_target = Texture::create_render_target(&context, width, height, format, None)?;

// Create depth texture
let depth_texture = Texture::create_depth_texture(&context, width, height, None)?;

// Using the builder pattern
let texture = TextureBuilder::new(1024, 1024)
    .format(TextureFormat::Rgba8UnormSrgb)
    .usage(TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING)
    .label("My Texture")
    .build(&context)?;
```

### Pipeline Creation

Simplified pipeline creation with sensible defaults:

```rust
// Simple render pipeline
let pipeline = RenderPipeline::simple(
    &context,
    vertex_shader,
    fragment_shader,
    &vertex_layouts,
    surface_format,
    Some("My Pipeline"),
)?;

// Compute pipeline
let pipeline = ComputePipeline::new(
    &context,
    compute_shader,
    bind_group_layouts,
    Some("Compute Pipeline"),
)?;
```

### Render and Compute Commands

High-level command recording:

```rust
// Render commands
let mut commands = RenderCommands::new(&context, Some("Frame"));
let mut render_pass = commands.begin_render_pass(&color_attachments, None, Some("Main Pass"));
render_pass.set_pipeline(&pipeline);
render_pass.set_vertex_buffer(0, &vertex_buffer);
render_pass.draw(0..vertex_count, 0..1);
drop(render_pass);
commands.submit(&context);

// Compute commands
let mut commands = ComputeCommands::new(&context, Some("Compute"));
let mut compute_pass = commands.begin_compute_pass(Some("Process Data"));
compute_pass.set_pipeline(&pipeline);
compute_pass.set_bind_group(0, &bind_group, &[]);
compute_pass.dispatch_workgroups(workgroup_count_x, workgroup_count_y, workgroup_count_z);
drop(compute_pass);
commands.submit(&context);
```

## Macros

Geepu provides convenient macros for common operations:

### Vertex Layout Macro

```rust
let layout = vertex_layout![
    0 => VertexFormat::Float32x3, // position
    1 => VertexFormat::Float32x3, // normal
    2 => VertexFormat::Float32x2, // tex_coords
];
```

### Bind Group Layout Macro

```rust
let layout = bind_group_layout!(&context, Some("My Layout"), {
    0 => BindingType::UniformBuffer(ShaderStages::VERTEX),
    1 => BindingType::Texture {
        visibility: ShaderStages::FRAGMENT,
        sample_type: TextureSampleType::Float { filterable: true },
        view_dimension: TextureViewDimension::D2,
        multisampled: false
    },
    2 => BindingType::Sampler {
        visibility: ShaderStages::FRAGMENT,
        sampler_type: SamplerBindingType::Filtering
    },
});
```

## Compute Shader Patterns

Geepu includes pre-built compute shader patterns for common operations:

```rust
use geepu::compute::patterns;

// Parallel reduction
let reduction_shader = patterns::reduction_shader(
    "result += data[i];", // operation
    "0.0",               // identity value
    "f32"                // data type
);

// Prefix sum (scan)
let scan_shader = patterns::prefix_sum_shader("f32");
```

## Error Handling

Geepu uses a comprehensive error type that covers common GPU programming issues:

```rust
use geepu::{Result, GeepuError};

fn gpu_operation() -> Result<()> {
    match some_gpu_operation() {
        Ok(result) => Ok(result),
        Err(GeepuError::AdapterNotFound) => {
            eprintln!("No suitable GPU found");
            Err(GeepuError::AdapterNotFound)
        }
        Err(e) => {
            eprintln!("GPU error: {}", e);
            Err(e)
        }
    }
}
```

## Examples

The repository includes several examples:

- `triangle.rs` - Basic triangle rendering
- `compute_simple.rs` - Simple compute shader example
- `texture_rendering.rs` - Texture loading and rendering
- `instanced_rendering.rs` - Instanced rendering example
- `compute_reduction.rs` - Parallel reduction compute example

Run examples with:

```bash
cargo run --example triangle
cargo run --example compute_simple
```

## Performance

Geepu is designed to be zero-cost - all abstractions compile down to direct wgpu calls with no runtime overhead. The wrapper types are thin and the builder patterns are compile-time constructs.

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Simplified API

Geepu now provides high-level helper functions that eliminate most boilerplate, letting you create render and compute pipelines with a single call.

### Easy Render Pipeline

Define your uniform data as a Rust struct:

```rust
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Uniforms {
    model: [[f32; 4]; 4],
    view:  [[f32; 4]; 4],
    proj:  [[f32; 4]; 4],
}
```

Then build and use a pipeline in one step:

```rust
let uniforms = Uniforms { /* init matrices */ };

let simple_pipeline = context
    .create_simple_pipeline(
        vertex_shader_source,
        fragment_shader_source,
        &[vertex_layout],         // your vertex layouts
        &uniforms,                // your uniform struct
        &[&texture],              // optional textures
        Some("EasyPipeline"),    // optional label
    )?;

// Render pass usage:
let mut commands = RenderCommands::new(&context, Some("Frame"));
let mut pass = commands.begin_render_pass(
    &[Some(color_attachment(&view, None))],
    None,
    Some("MainPass"),
);
pass.set_pipeline(&simple_pipeline.pipeline);
pass.set_vertex_buffer(0, &vertex_buffer);
pass.draw(0..3, 0..1);
drop(pass);
commands.submit(&context);
```

### Easy Compute Pipeline

Similarly, for compute workloads:

```rust
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Params { value: f32; }

let params = Params { value: 1.0 };
let storage_buffers = vec![&buffer_a, &buffer_b];

let compute_pipeline = context
    .create_simple_compute(
        compute_shader_source,
        &params,                  // uniform struct
        &storage_buffers,         // list of storage buffers
        Some("EasyCompute"),     // optional label
    )?;

let mut compute_cmds = ComputeCommands::new(&context, Some("Compute"));
let mut cpass = compute_cmds.begin_compute_pass(Some("CSMain"));
cpass.set_pipeline(&compute_pipeline.pipeline);
cpass.set_bind_group(0, &compute_pipeline.bind_group, &[]);
cpass.dispatch_workgroups(64, 1, 1);
drop(cpass);
compute_cmds.submit(&context);
```
