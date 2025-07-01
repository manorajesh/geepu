# Geepu - Ergonomic wgpu Wrapper for Rust

Geepu is a zero-cost, ergonomic wrapper around wgpu that simplifies GPU programming in Rust while maintaining performance. It provides high-level abstractions for GPU operations without sacrificing the power and flexibility of wgpu.

## Features

- **🚀 Easy GPU Initialization**: Simple setup for windowed or offscreen rendering
- **🔧 Shader Management**: Load and compile WGSL shaders with comprehensive error handling
- **💾 Resource Management**: Register uniforms, storage buffers, textures, and samplers with type safety
- **🎨 Render Passes**: High-level abstraction for drawing operations
- **⚡ Compute Shaders**: Full support for compute pipelines and workgroup dispatch
- **🔍 Flexible Configuration**: Sensible defaults with advanced wgpu configuration options
- **📊 Comprehensive Logging**: Built-in tracing support for debugging and profiling
- **🔒 Memory Safety**: Leverages Rust's type system and bytemuck for safe GPU memory operations

## Quick Start

Add Geepu to your `Cargo.toml`:

```toml
[dependencies]
geepu = "0.2.0"
anyhow = "1.0"
bytemuck = { version = "1.23", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = "0.3"
tokio = { version = "1.0", features = ["rt", "rt-multi-thread", "macros"] }
```

### Basic Example

```rust
use geepu::{Renderer, WindowConfig, Size};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create windowed renderer
    let window_config = WindowConfig::new("My App", Size::new(800, 600));
    let mut renderer = Renderer::new(window_config).await?;

    // Add resources
    let mvp_matrix = [[1.0f32; 4]; 4]; // Identity matrix
    renderer.add_uniform("mvp", &mvp_matrix);

    // Load a texture
    let image = image::open("texture.png")?;
    renderer.add_texture("diffuse", image)?;

    // Render loop
    for frame in 0..60 {
        // Update uniforms
        renderer.update_uniform("mvp", &updated_matrix)?;

        // Render
        let mut pass = renderer.begin_pass();
        pass.draw_indexed(0..indices.len() as u32, 0, 0..1)?;
        drop(pass);
        
        renderer.submit();
    }

    Ok(())
}
```

## API Overview

### Renderer Creation

```rust
// Windowed rendering
let renderer = Renderer::new(WindowConfig::default()).await?;

// Offscreen rendering
let renderer = Renderer::offscreen(Size::new(1920, 1080)).await?;

// With custom GPU configuration
let gpu_config = GpuConfig::performance()
    .features(wgpu::Features::COMPUTE_SHADERS)
    .backends(wgpu::Backends::VULKAN | wgpu::Backends::METAL);
let renderer = Renderer::new_with_gpu_config(window_config, gpu_config).await?;
```

### Resource Management

```rust
// Uniforms (constant data)
#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
struct MVP {
    matrix: [[f32; 4]; 4],
}

let mvp = MVP { matrix: identity_matrix() };
renderer.add_uniform("mvp", &mvp);
renderer.update_uniform("mvp", &updated_mvp)?;

// Storage buffers (read/write data)
let data: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
renderer.add_storage_buffer("data", &data);
renderer.update_storage_buffer("data", &new_data)?;

// Read back from GPU
let results = renderer.read_storage_buffer::<f32>("data").await?;

// Textures
let image = image::open("texture.png")?;
renderer.add_texture("diffuse", image)?;
```

### Shader Management

```rust
// Load shaders from strings
renderer.shaders.load_vertex_shader(&device, "main_vs", vertex_source)?;
renderer.shaders.load_fragment_shader(&device, "main_fs", fragment_source)?;

// Load from files
renderer.shaders.load_shader_from_file(
    &device, 
    "my_shader", 
    "shaders/compute.wgsl", 
    ShaderType::Compute
)?;

// Add compute shaders
renderer.add_compute_shader("multiply", compute_source)?;
```

### Compute Shaders

```rust
// Load compute shader
let compute_source = r#"
@group(0) @binding(0)
var<storage, read_write> data: array<f32>;

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= arrayLength(&data)) { return; }
    data[index] = data[index] * 2.0;
}
"#;

renderer.add_compute_shader("multiply", compute_source)?;

// Dispatch computation
let workgroups = (data.len() + 63) / 64;
renderer.dispatch_compute("multiply", workgroups, 1, 1)?;
renderer.submit();
```

### Render Passes

```rust
// Begin a render pass
let mut pass = renderer.begin_pass();

// In a complete implementation, you would:
// - Create render pipelines
// - Set bind groups for resources
// - Set vertex/index buffers
// - Issue draw calls

pass.draw_indexed(indices_range, base_vertex, instances_range)?;
drop(pass);

// Submit all commands
renderer.submit();
```

### Offscreen Rendering

```rust
// Create offscreen renderer
let mut renderer = Renderer::offscreen(Size::new(1024, 1024)).await?;

// Render scene
{
    let mut pass = renderer.begin_pass();
    // ... render operations ...
    drop(pass);
    renderer.submit();
}

// Copy result to CPU
let image_data = renderer.copy_to_buffer().await?;

// Save to file
let image = image::RgbaImage::from_raw(1024, 1024, image_data).unwrap();
image.save("output.png")?;
```

## Advanced Configuration

### GPU Configuration

```rust
let gpu_config = GpuConfig::default()
    .backends(wgpu::Backends::VULKAN | wgpu::Backends::METAL)
    .features(wgpu::Features::COMPUTE_SHADERS | wgpu::Features::TEXTURE_BINDING_ARRAY)
    .limits(wgpu::Limits {
        max_compute_workgroup_size_x: 1024,
        max_compute_workgroup_size_y: 1024,
        max_compute_workgroup_size_z: 64,
        ..Default::default()
    })
    .force_fallback_adapter(false);
```

### Window Configuration

```rust
let window_config = WindowConfig::new("My Application", Size::new(1280, 720))
    .resizable(true)
    .vsync(false);
```

### Logging Configuration

```rust
use tracing_subscriber::fmt;
use tracing::Level;

// Basic logging
tracing_subscriber::fmt::init();

// Detailed logging with custom format
fmt()
    .with_max_level(Level::DEBUG)
    .with_target(false)
    .with_thread_ids(true)
    .with_file(true)
    .with_line_number(true)
    .init();
```

## Shader Examples

### Vertex Shader (WGSL)

```wgsl
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> mvp_matrix: mat4x4<f32>;

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = mvp_matrix * vec4<f32>(model.position, 1.0);
    return out;
}
```

### Fragment Shader (WGSL)

```wgsl
@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(@location(0) tex_coords: vec2<f32>) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, tex_coords);
}
```

### Compute Shader (WGSL)

```wgsl
@group(0) @binding(0)
var<storage, read_write> data: array<f32>;

@group(0) @binding(1)
var<uniform> params: ComputeParams;

struct ComputeParams {
    multiplier: f32,
    offset: f32,
}

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= arrayLength(&data)) {
        return;
    }
    
    data[index] = data[index] * params.multiplier + params.offset;
}
```

## Performance Tips

1. **Batch Operations**: Group multiple draw calls or compute dispatches together
2. **Resource Reuse**: Reuse buffers and textures when possible
3. **Optimal Workgroup Sizes**: Use workgroup sizes that are multiples of the GPU's warp/wavefront size
4. **Memory Layout**: Use `#[repr(C)]` and `bytemuck` for predictable memory layouts
5. **Pipeline Caching**: Reuse render and compute pipelines across frames

## Error Handling

Geepu provides comprehensive error handling through the `GeepuError` type:

```rust
use geepu::{GeepuError, Result};

match renderer.add_texture("missing", image) {
    Ok(_) => println!("Texture added successfully"),
    Err(GeepuError::Image(e)) => println!("Image error: {}", e),
    Err(GeepuError::ResourceNotFound(name)) => println!("Resource {} not found", name),
    Err(e) => println!("Other error: {}", e),
}
```

## Examples

Run the included examples:

```bash
# Basic hello triangle example
cargo run --example hello_triangle

# Compute shader example
cargo run --example compute

# Offscreen rendering example
cargo run --example offscreen

# Comprehensive example (included in main.rs)
cargo run
```

## Architecture

Geepu is built on top of wgpu and provides:

- **Zero-cost abstractions**: No runtime overhead compared to raw wgpu
- **Type safety**: Leverages Rust's type system to prevent common GPU programming errors
- **Resource management**: Automatic cleanup and efficient resource tracking
- **Extensibility**: Easy to extend with custom pipelines and operations

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built on the excellent [wgpu](https://github.com/gfx-rs/wgpu) project
- Inspired by modern graphics API best practices
- Thanks to the Rust graphics community for feedback and contributions
