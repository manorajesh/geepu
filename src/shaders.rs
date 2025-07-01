//! Shader management and compute pipeline support

use crate::error::{GeepuError, Result};
use std::collections::HashMap;
use tracing::{info, error};

/// Shader manager for loading and compiling WGSL shaders
pub struct ShaderManager {
    vertex_shaders: HashMap<String, wgpu::ShaderModule>,
    fragment_shaders: HashMap<String, wgpu::ShaderModule>,
    compute_shaders: HashMap<String, wgpu::ShaderModule>,
}

impl ShaderManager {
    pub fn new() -> Self {
        Self {
            vertex_shaders: HashMap::new(),
            fragment_shaders: HashMap::new(),
            compute_shaders: HashMap::new(),
        }
    }

    /// Load a vertex shader from WGSL source
    pub fn load_vertex_shader(&mut self, device: &wgpu::Device, name: &str, source: &str) -> Result<()> {
        let span = tracing::span!(tracing::Level::INFO, "load_vertex_shader", name = name);
        let _enter = span.enter();

        info!("Loading vertex shader: {}", name);
        
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(name),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });

        self.vertex_shaders.insert(name.to_string(), shader);
        info!("Successfully loaded vertex shader: {}", name);
        Ok(())
    }

    /// Load a fragment shader from WGSL source
    pub fn load_fragment_shader(&mut self, device: &wgpu::Device, name: &str, source: &str) -> Result<()> {
        let span = tracing::span!(tracing::Level::INFO, "load_fragment_shader", name = name);
        let _enter = span.enter();

        info!("Loading fragment shader: {}", name);
        
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(name),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });

        self.fragment_shaders.insert(name.to_string(), shader);
        info!("Successfully loaded fragment shader: {}", name);
        Ok(())
    }

    /// Load a compute shader from WGSL source
    pub fn load_compute_shader(&mut self, device: &wgpu::Device, name: &str, source: &str) -> Result<()> {
        let span = tracing::span!(tracing::Level::INFO, "load_compute_shader", name = name);
        let _enter = span.enter();

        info!("Loading compute shader: {}", name);
        
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(name),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });

        self.compute_shaders.insert(name.to_string(), shader);
        info!("Successfully loaded compute shader: {}", name);
        Ok(())
    }

    /// Load shader from file
    pub fn load_shader_from_file(&mut self, device: &wgpu::Device, name: &str, path: &str, shader_type: ShaderType) -> Result<()> {
        let span = tracing::span!(tracing::Level::INFO, "load_shader_from_file", name = name, path = path);
        let _enter = span.enter();

        info!("Loading shader from file: {} -> {}", path, name);
        
        let source = std::fs::read_to_string(path)
            .map_err(|e| {
                error!("Failed to read shader file {}: {}", path, e);
                GeepuError::Io(e)
            })?;

        match shader_type {
            ShaderType::Vertex => self.load_vertex_shader(device, name, &source),
            ShaderType::Fragment => self.load_fragment_shader(device, name, &source),
            ShaderType::Compute => self.load_compute_shader(device, name, &source),
        }
    }

    pub fn get_vertex_shader(&self, name: &str) -> Result<&wgpu::ShaderModule> {
        self.vertex_shaders
            .get(name)
            .ok_or_else(|| GeepuError::ResourceNotFound(format!("vertex shader '{}'", name)))
    }

    pub fn get_fragment_shader(&self, name: &str) -> Result<&wgpu::ShaderModule> {
        self.fragment_shaders
            .get(name)
            .ok_or_else(|| GeepuError::ResourceNotFound(format!("fragment shader '{}'", name)))
    }

    pub fn get_compute_shader(&self, name: &str) -> Result<&wgpu::ShaderModule> {
        self.compute_shaders
            .get(name)
            .ok_or_else(|| GeepuError::ResourceNotFound(format!("compute shader '{}'", name)))
    }
}

/// Shader type enumeration
pub enum ShaderType {
    Vertex,
    Fragment,
    Compute,
}

/// Compute pipeline wrapper
pub struct ComputePipeline {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group_layouts: Vec<wgpu::BindGroupLayout>,
    workgroup_size: (u32, u32, u32),
}

impl ComputePipeline {
    /// Create a new compute pipeline
    pub fn new(
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        entry_point: &str,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
        workgroup_size: (u32, u32, u32),
        label: Option<&str>,
    ) -> Self {
        let span = tracing::span!(tracing::Level::INFO, "create_compute_pipeline", 
            entry_point = entry_point, label = label);
        let _enter = span.enter();

        info!("Creating compute pipeline: {:?}", label);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label,
            bind_group_layouts,
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label,
            layout: Some(&pipeline_layout),
            module: shader,
            entry_point: Some(entry_point),
            compilation_options: Default::default(),
            cache: None,
        });

        info!("Successfully created compute pipeline: {:?}", label);

        Self {
            pipeline,
            bind_group_layouts: Vec::new(), // Simplified for now
            workgroup_size,
        }
    }

    /// Get the optimal dispatch size for a given problem size
    pub fn optimal_dispatch_size(&self, problem_size: (u32, u32, u32)) -> (u32, u32, u32) {
        let (px, py, pz) = problem_size;
        let (wx, wy, wz) = self.workgroup_size;
        
        (
            (px + wx - 1) / wx,
            (py + wy - 1) / wy,
            (pz + wz - 1) / wz,
        )
    }
}

/// Vertex attribute helper
#[derive(Debug, Clone)]
pub struct VertexAttribute {
    pub offset: u64,
    pub shader_location: u32,
    pub format: wgpu::VertexFormat,
}

/// Vertex buffer layout builder
pub struct VertexBufferLayoutBuilder {
    attributes: Vec<VertexAttribute>,
    stride: u64,
}

impl VertexBufferLayoutBuilder {
    pub fn new() -> Self {
        Self {
            attributes: Vec::new(),
            stride: 0,
        }
    }

    pub fn add_attribute(mut self, format: wgpu::VertexFormat, shader_location: u32) -> Self {
        self.attributes.push(VertexAttribute {
            offset: self.stride,
            shader_location,
            format,
        });
        
        self.stride += format.size();
        self
    }

    pub fn build(self) -> wgpu::VertexBufferLayout<'static> {
        let attributes: Vec<wgpu::VertexAttribute> = self.attributes
            .into_iter()
            .map(|attr| wgpu::VertexAttribute {
                offset: attr.offset,
                shader_location: attr.shader_location,
                format: attr.format,
            })
            .collect();

        wgpu::VertexBufferLayout {
            array_stride: self.stride,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: attributes.leak(), // Note: This leaks memory, use with caution
        }
    }
}

/// Default shader sources for common operations
pub mod default_shaders {
    /// Basic vertex shader for textured quads
    pub const TEXTURED_QUAD_VERTEX: &str = r#"
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

struct Uniforms {
    mvp_matrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = uniforms.mvp_matrix * vec4<f32>(model.position, 1.0);
    return out;
}
"#;

    /// Basic fragment shader for textured quads
    pub const TEXTURED_QUAD_FRAGMENT: &str = r#"
@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(@location(0) tex_coords: vec2<f32>) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, tex_coords);
}
"#;

    /// Simple compute shader for array processing
    pub const ARRAY_MULTIPLY_COMPUTE: &str = r#"
@group(0) @binding(0)
var<storage, read_write> data: array<f32>;

@group(0) @binding(1)
var<uniform> multiplier: f32;

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= arrayLength(&data)) {
        return;
    }
    
    data[index] = data[index] * multiplier;
}
"#;
}
