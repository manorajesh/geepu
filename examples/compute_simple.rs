use geepu::*;

async fn compute_example() -> Result<()> {
    // Create GPU context
    let context = GpuContext::new().await?;
    
    println!("GPU Context created successfully!");
    println!("Adapter: {:?}", context.adapter.get_info());

    // Create input data (sum of 1..=1024 should be 524800)
    let input_data: Vec<f32> = (1..=1024).map(|x| x as f32).collect();
    let expected_sum: f32 = input_data.iter().sum();
    println!("Input data length: {}", input_data.len());
    println!("Expected sum: {}", expected_sum);

    // Create buffers
    let input_buffer = TypedBuffer::storage(&context, &input_data)?;
    let output_buffer = TypedBuffer::<i32>::empty(&context, 1, 
        BufferUsages::STORAGE | BufferUsages::COPY_SRC)?;

    // Create staging buffer for reading results
    let staging_buffer = StagingBuffer::new(&context, 4)?; // 4 bytes for i32

    // Create bind group layout
    let bind_group_layout = BindGroupLayoutBuilder::new()
        .storage_buffer(0, ShaderStages::COMPUTE, true)  // read-only input
        .storage_buffer(1, ShaderStages::COMPUTE, false) // read-write output
        .build(&context, Some("Compute Layout"));

    // Create bind group
    let bind_group = BindGroupBuilder::new(&bind_group_layout)
        .buffer(0, input_buffer.buffer())
        .buffer(1, output_buffer.buffer())
        .build(&context, Some("Compute Bind Group"));

    // Simple compute shader that sums all elements
    let compute_shader = r#"
        @group(0) @binding(0) var<storage, read> input_data: array<f32>;
        @group(0) @binding(1) var<storage, read_write> output_data: array<atomic<i32>>;

        @workgroup_size(64, 1, 1)
        @compute
        fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
            let index = global_id.x;
            
            // Simple parallel reduction - each thread adds its element to output[0]
            if (index < arrayLength(&input_data)) {
                atomicAdd(&output_data[0], i32(input_data[index]));
            }
        }
    "#;

    // Create compute pipeline
    let pipeline = ComputePipeline::new(
        &context,
        compute_shader,
        vec![bind_group_layout],
        Some("Sum Compute Pipeline"),
    )?;

    // Execute compute shader
    let mut commands = ComputeCommands::new(&context, Some("Compute Commands"));
    
    {
        let mut compute_pass = commands.begin_compute_pass(Some("Sum Pass"));
        compute_pass.set_pipeline(&pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        
        // Dispatch workgroups (1024 elements / 64 threads per workgroup = 16 workgroups)
        let workgroup_count = (input_data.len() as u32 + 63) / 64;
        compute_pass.dispatch_workgroups(workgroup_count, 1, 1);
    }

    // Copy result to staging buffer
    staging_buffer.copy_from_buffer(commands.encoder(), output_buffer.buffer(), Some(4));

    // Submit commands and wait
    commands.submit(&context);

    // Read back the result
    let result_data: Vec<i32> = staging_buffer.read_data(&context).await?;
    let computed_sum = result_data[0] as f32;

    println!("Computed sum: {}", computed_sum);
    println!("Expected sum: {}", expected_sum);
    println!("Difference: {}", (computed_sum - expected_sum).abs());
    
    if (computed_sum - expected_sum).abs() < 0.001 {
        println!("✅ Compute shader executed successfully!");
    } else {
        println!("❌ Compute shader result doesn't match expected value");
    }

    Ok(())
}

fn main() {
    env_logger::init();
    
    match pollster::block_on(compute_example()) {
        Ok(()) => println!("Example completed successfully!"),
        Err(e) => eprintln!("Error: {}", e),
    }
}
