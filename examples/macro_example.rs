use geepu::*;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    tex_coords: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct UniformData {
    model_matrix: [[f32; 4]; 4],
    view_matrix: [[f32; 4]; 4],
    projection_matrix: [[f32; 4]; 4],
}

async fn macro_example() -> Result<()> {
    // Create GPU context
    let context = GpuContext::new().await?;

    println!("GPU Context created successfully!");

    // Create vertex data
    let vertices = [
        Vertex {
            position: [0.0, 0.5, 0.0],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [0.5, 0.0],
        },
        Vertex {
            position: [-0.5, -0.5, 0.0],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [0.0, 1.0],
        },
        Vertex {
            position: [0.5, -0.5, 0.0],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [1.0, 1.0],
        },
    ];

    // Create buffers
    let vertex_buffer = TypedBuffer::vertex(&context, &vertices)?;

    let uniform_data = UniformData {
        model_matrix: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
        view_matrix: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
        projection_matrix: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };
    let uniform_buffer = TypedBuffer::uniform(&context, &[uniform_data])?;

    println!("Buffers created successfully!");
    println!("Vertex buffer size: {} vertices", vertex_buffer.len());
    println!("Uniform buffer size: {} bytes", uniform_buffer.size_bytes());

    // Demonstrate vertex layout macro
    let vertex_layout =
        vertex_layout![
        0 => VertexFormat::Float32x3, // position
        1 => VertexFormat::Float32x3, // normal
        2 => VertexFormat::Float32x2, // tex_coords
    ];

    println!("Vertex layout created with macro: {} attributes", vertex_layout.attributes.len());
    println!("Vertex stride: {} bytes", vertex_layout.array_stride);

    // Create a texture for demonstration
    let texture_data: Vec<u8> = (0..64).flat_map(|_| [255u8, 255, 255, 255]).collect(); // 8x8 white texture
    let texture = Texture::from_bytes(
        &context,
        &texture_data,
        8,
        8,
        TextureFormat::Rgba8UnormSrgb,
        Some("Demo Texture")
    )?;

    println!("Texture created successfully!");

    // Demonstrate bind group layout using builder (macro version would be complex)
    let bind_group_layout = BindGroupLayoutBuilder::new()
        .uniform_buffer(0, ShaderStages::VERTEX)
        .texture(
            1,
            ShaderStages::FRAGMENT,
            TextureSampleType::Float { filterable: true },
            TextureViewDimension::D2,
            false
        )
        .sampler(2, ShaderStages::FRAGMENT, SamplerBindingType::Filtering)
        .build(&context, Some("Demo Layout"));

    println!("Bind group layout created with builder!");

    // Create bind group using builder
    let _bind_group = BindGroupBuilder::new(&bind_group_layout)
        .buffer(0, uniform_buffer.buffer())
        .texture_view(1, &texture.view)
        .sampler(2, &texture.sampler)
        .build(&context, Some("Demo Bind Group"));

    println!("Bind group created successfully!");
    // Note: wgpu::BindGroupLayout doesn't expose entry count, but we know we added 3 bindings

    // Test the VertexBufferBuilder
    let another_vertex_layout = VertexBufferBuilder::new()
        .attribute(VertexFormat::Float32x3, 0) // position
        .attribute(VertexFormat::Float32x3, 1) // normal
        .attribute(VertexFormat::Float32x2, 2) // tex_coords
        .step_mode(wgpu::VertexStepMode::Vertex)
        .build();

    println!("Alternative vertex layout created with builder!");
    println!("Alternative layout stride: {} bytes", another_vertex_layout.array_stride);

    // Test compute workgroup size utilities
    let workgroup_size = WorkgroupSize::new(16, 16, 1);
    let data_size = (1024, 1024, 1);
    let (wx, wy, wz) = workgroup_size.workgroups_for_size(data_size.0, data_size.1, data_size.2);

    println!("Workgroup calculation:");
    println!("  Data size: {:?}", data_size);
    println!(
        "  Workgroup size: ({}, {}, {})",
        workgroup_size.x,
        workgroup_size.y,
        workgroup_size.z
    );
    println!("  Required workgroups: ({}, {}, {})", wx, wy, wz);

    println!("✅ All macro and builder examples completed successfully!");

    Ok(())
}

fn main() {
    env_logger::init();

    match pollster::block_on(macro_example()) {
        Ok(()) => println!("Macro example completed successfully!"),
        Err(e) => eprintln!("Error: {}", e),
    }
}
