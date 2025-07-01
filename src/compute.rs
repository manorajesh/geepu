use crate::{GpuContext, ComputePipeline, TypedBuffer};

/// A high-level compute pass wrapper
pub struct ComputePass<'a> {
    pass: wgpu::ComputePass<'a>,
}

impl<'a> ComputePass<'a> {
    /// Create a new compute pass
    pub fn new(encoder: &'a mut wgpu::CommandEncoder, label: Option<&str>) -> Self {
        let pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label,
            timestamp_writes: None,
        });

        Self { pass }
    }

    /// Set the compute pipeline
    pub fn set_pipeline(&mut self, pipeline: &'a ComputePipeline) {
        self.pass.set_pipeline(&pipeline.pipeline);
    }

    /// Set a bind group
    pub fn set_bind_group(&mut self, index: u32, bind_group: &'a wgpu::BindGroup, offsets: &[u32]) {
        self.pass.set_bind_group(index, bind_group, offsets);
    }

    /// Dispatch compute workgroups
    pub fn dispatch_workgroups(&mut self, workgroup_count_x: u32, workgroup_count_y: u32, workgroup_count_z: u32) {
        self.pass.dispatch_workgroups(workgroup_count_x, workgroup_count_y, workgroup_count_z);
    }

    /// Dispatch compute workgroups indirectly
    pub fn dispatch_workgroups_indirect<T>(&mut self, indirect_buffer: &'a TypedBuffer<T>, indirect_offset: u64)
    where
        T: bytemuck::Pod,
    {
        self.pass.dispatch_workgroups_indirect(indirect_buffer.buffer(), indirect_offset);
    }
}

/// A high-level compute command builder
pub struct ComputeCommands {
    encoder: wgpu::CommandEncoder,
}

impl ComputeCommands {
    /// Create new compute commands
    pub fn new(context: &GpuContext, label: Option<&str>) -> Self {
        let encoder = context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label,
        });

        Self { encoder }
    }

    /// Begin a compute pass
    pub fn begin_compute_pass<'a>(&'a mut self, label: Option<&str>) -> ComputePass<'a> {
        ComputePass::new(&mut self.encoder, label)
    }

    /// Copy buffer to buffer
    pub fn copy_buffer_to_buffer(
        &mut self,
        source: &wgpu::Buffer,
        source_offset: u64,
        destination: &wgpu::Buffer,
        destination_offset: u64,
        copy_size: u64,
    ) {
        self.encoder.copy_buffer_to_buffer(
            source,
            source_offset,
            destination,
            destination_offset,
            copy_size,
        );
    }

    /// Copy buffer to texture
    pub fn copy_buffer_to_texture(
        &mut self,
        source: wgpu::ImageCopyBuffer,
        destination: wgpu::ImageCopyTexture,
        copy_size: wgpu::Extent3d,
    ) {
        self.encoder.copy_buffer_to_texture(source, destination, copy_size);
    }

    /// Copy texture to buffer
    pub fn copy_texture_to_buffer(
        &mut self,
        source: wgpu::ImageCopyTexture,
        destination: wgpu::ImageCopyBuffer,
        copy_size: wgpu::Extent3d,
    ) {
        self.encoder.copy_texture_to_buffer(source, destination, copy_size);
    }

    /// Insert debug marker
    pub fn insert_debug_marker(&mut self, label: &str) {
        self.encoder.insert_debug_marker(label);
    }

    /// Push debug group
    pub fn push_debug_group(&mut self, label: &str) {
        self.encoder.push_debug_group(label);
    }

    /// Pop debug group
    pub fn pop_debug_group(&mut self) {
        self.encoder.pop_debug_group();
    }

    /// Finish and submit commands
    pub fn submit(self, context: &GpuContext) {
        context.queue.submit(std::iter::once(self.encoder.finish()));
    }

    /// Get the underlying encoder (for advanced usage)
    pub fn encoder(&mut self) -> &mut wgpu::CommandEncoder {
        &mut self.encoder
    }
}

/// Helper for compute workgroup size calculations
pub struct WorkgroupSize {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

impl WorkgroupSize {
    pub fn new(x: u32, y: u32, z: u32) -> Self {
        Self { x, y, z }
    }

    pub fn linear(size: u32) -> Self {
        Self { x: size, y: 1, z: 1 }
    }

    pub fn square(size: u32) -> Self {
        Self { x: size, y: size, z: 1 }
    }

    /// Calculate number of workgroups needed for given data size
    pub fn workgroups_for_size(&self, data_size_x: u32, data_size_y: u32, data_size_z: u32) -> (u32, u32, u32) {
        (
            (data_size_x + self.x - 1) / self.x,
            (data_size_y + self.y - 1) / self.y,
            (data_size_z + self.z - 1) / self.z,
        )
    }
}

/// Compute shader builder for common patterns
pub struct ComputeShaderBuilder {
    workgroup_size: WorkgroupSize,
    local_memory_size: Option<u32>,
    includes: Vec<String>,
}

impl ComputeShaderBuilder {
    pub fn new() -> Self {
        Self {
            workgroup_size: WorkgroupSize::new(64, 1, 1),
            local_memory_size: None,
            includes: Vec::new(),
        }
    }

    pub fn workgroup_size(mut self, size: WorkgroupSize) -> Self {
        self.workgroup_size = size;
        self
    }

    pub fn local_memory(mut self, size: u32) -> Self {
        self.local_memory_size = Some(size);
        self
    }

    pub fn include(mut self, code: impl Into<String>) -> Self {
        self.includes.push(code.into());
        self
    }

    /// Generate compute shader with boilerplate
    pub fn build_shader(&self, main_code: &str) -> String {
        let mut shader = String::new();
        
        // Add includes
        for include in &self.includes {
            shader.push_str(include);
            shader.push('\n');
        }

        // Add workgroup size
        shader.push_str(&format!(
            "@workgroup_size({}, {}, {})\n",
            self.workgroup_size.x, self.workgroup_size.y, self.workgroup_size.z
        ));

        // Add local memory if specified
        if let Some(size) = self.local_memory_size {
            shader.push_str(&format!("var<workgroup> local_memory: array<u32, {}>;\n\n", size));
        }

        // Add main function
        shader.push_str("@compute\n");
        shader.push_str("fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {\n");
        shader.push_str(main_code);
        shader.push_str("\n}");

        shader
    }
}

impl Default for ComputeShaderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Common compute patterns
pub mod patterns {

    /// Parallel reduction operation
    pub fn reduction_shader(
        operation: &str, // e.g., "result += data[i];" or "result = max(result, data[i]);"
        identity: &str,  // e.g., "0.0" or "-3.402823e+38"
        data_type: &str, // e.g., "f32" or "i32"
    ) -> String {
        format!(
            r#"
@group(0) @binding(0) var<storage, read> input_data: array<{}>;
@group(0) @binding(1) var<storage, read_write> output_data: array<{}>;

var<workgroup> shared_data: array<{}, 256>;

@workgroup_size(256, 1, 1)
@compute
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>,
          @builtin(local_invocation_id) local_id: vec3<u32>,
          @builtin(workgroup_id) workgroup_id: vec3<u32>) {{
    let tid = local_id.x;
    let bid = workgroup_id.x;
    let i = bid * 256u + tid;
    
    // Load data into shared memory
    if (i < arrayLength(&input_data)) {{
        shared_data[tid] = input_data[i];
    }} else {{
        shared_data[tid] = {};
    }}
    
    workgroupBarrier();
    
    // Reduction in shared memory
    var s = 128u;
    while (s > 0u) {{
        if (tid < s && (i + s) < arrayLength(&input_data)) {{
            let idx = tid + s;
            {}
        }}
        workgroupBarrier();
        s = s >> 1u;
    }}
    
    // Write result
    if (tid == 0u) {{
        output_data[bid] = shared_data[0];
    }}
}}
"#,
            data_type, data_type, data_type, identity, 
            operation.replace("result", "shared_data[tid]").replace("data[i]", "shared_data[idx]")
        )
    }

    /// Prefix sum (scan) operation
    pub fn prefix_sum_shader(data_type: &str) -> String {
        format!(
            r#"
@group(0) @binding(0) var<storage, read> input_data: array<{}>;
@group(0) @binding(1) var<storage, read_write> output_data: array<{}>;

var<workgroup> shared_data: array<{}, 256>;

@workgroup_size(256, 1, 1)
@compute
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>,
          @builtin(local_invocation_id) local_id: vec3<u32>) {{
    let tid = local_id.x;
    let i = global_id.x;
    
    // Load data
    if (i < arrayLength(&input_data)) {{
        shared_data[tid] = input_data[i];
    }} else {{
        shared_data[tid] = {};
    }}
    
    workgroupBarrier();
    
    // Up-sweep phase
    var d = 1u;
    while (d < 256u) {{
        if (tid % (2u * d) == 0u) {{
            shared_data[tid + 2u * d - 1u] = shared_data[tid + 2u * d - 1u] + shared_data[tid + d - 1u];
        }}
        workgroupBarrier();
        d = d * 2u;
    }}
    
    // Clear the last element
    if (tid == 0u) {{
        shared_data[255] = {};
    }}
    
    workgroupBarrier();
    
    // Down-sweep phase
    d = 128u;
    while (d > 0u) {{
        if (tid % (2u * d) == 0u) {{
            let temp = shared_data[tid + d - 1u];
            shared_data[tid + d - 1u] = shared_data[tid + 2u * d - 1u];
            shared_data[tid + 2u * d - 1u] = shared_data[tid + 2u * d - 1u] + temp;
        }}
        workgroupBarrier();
        d = d >> 1u;
    }}
    
    // Write result
    if (i < arrayLength(&output_data)) {{
        output_data[i] = shared_data[tid];
    }}
}}
"#,
            data_type, data_type, data_type, "0", "0"
        )
    }
}
