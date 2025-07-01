use crate::{ GpuContext, Result, TypedBuffer };
use wgpu::{ ShaderStages, TextureSampleType, TextureViewDimension, SamplerBindingType };

/// A wrapper around render pipeline with convenient creation methods
pub struct RenderPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layouts: Vec<wgpu::BindGroupLayout>,
}

impl RenderPipeline {
    /// Create a render pipeline from shader source
    pub fn new(
        context: &GpuContext,
        vertex_shader: &str,
        fragment_shader: Option<&str>,
        vertex_layouts: &[wgpu::VertexBufferLayout],
        color_targets: &[Option<wgpu::ColorTargetState>],
        depth_stencil: Option<wgpu::DepthStencilState>,
        bind_group_layouts: Vec<wgpu::BindGroupLayout>,
        label: Option<&str>
    ) -> Result<Self> {
        let vertex_module = context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Vertex Shader"),
            source: wgpu::ShaderSource::Wgsl(vertex_shader.into()),
        });

        let fragment_module = if let Some(fragment_shader) = fragment_shader {
            Some(
                context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Fragment Shader"),
                    source: wgpu::ShaderSource::Wgsl(fragment_shader.into()),
                })
            )
        } else {
            None
        };

        let bind_group_layout_refs: Vec<&wgpu::BindGroupLayout> = bind_group_layouts
            .iter()
            .collect();

        let pipeline_layout = context.device.create_pipeline_layout(
            &(wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &bind_group_layout_refs,
                push_constant_ranges: &[],
            })
        );

        let pipeline = context.device.create_render_pipeline(
            &(wgpu::RenderPipelineDescriptor {
                label,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &vertex_module,
                    entry_point: "vs_main",
                    buffers: vertex_layouts,
                    compilation_options: Default::default(),
                },
                fragment: fragment_module.as_ref().map(|module| wgpu::FragmentState {
                    module,
                    entry_point: "fs_main",
                    targets: color_targets,
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            })
        );

        Ok(Self {
            pipeline,
            bind_group_layouts,
        })
    }

    /// Create a simple render pipeline with common defaults
    pub fn simple(
        context: &GpuContext,
        vertex_shader: &str,
        fragment_shader: &str,
        vertex_layouts: &[wgpu::VertexBufferLayout],
        surface_format: wgpu::TextureFormat,
        label: Option<&str>
    ) -> Result<Self> {
        let color_targets = &[
            Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            }),
        ];

        Self::new(
            context,
            vertex_shader,
            Some(fragment_shader),
            vertex_layouts,
            color_targets,
            None,
            vec![],
            label
        )
    }
}

/// A wrapper around compute pipeline
pub struct ComputePipeline {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group_layouts: Vec<wgpu::BindGroupLayout>,
}

impl ComputePipeline {
    /// Create a compute pipeline from shader source
    pub fn new(
        context: &GpuContext,
        shader_source: &str,
        bind_group_layouts: Vec<wgpu::BindGroupLayout>,
        label: Option<&str>
    ) -> Result<Self> {
        let shader_module = context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let bind_group_layout_refs: Vec<&wgpu::BindGroupLayout> = bind_group_layouts
            .iter()
            .collect();

        let pipeline_layout = context.device.create_pipeline_layout(
            &(wgpu::PipelineLayoutDescriptor {
                label: Some("Compute Pipeline Layout"),
                bind_group_layouts: &bind_group_layout_refs,
                push_constant_ranges: &[],
            })
        );

        let pipeline = context.device.create_compute_pipeline(
            &(wgpu::ComputePipelineDescriptor {
                label,
                layout: Some(&pipeline_layout),
                module: &shader_module,
                entry_point: "cs_main",
                compilation_options: Default::default(),
                cache: None,
            })
        );

        Ok(Self {
            pipeline,
            bind_group_layouts,
        })
    }
}

/// Builder for creating bind group layouts
pub struct BindGroupLayoutBuilder {
    entries: Vec<wgpu::BindGroupLayoutEntry>,
}

impl BindGroupLayoutBuilder {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Add a uniform buffer binding
    pub fn uniform_buffer(mut self, binding: u32, visibility: wgpu::ShaderStages) -> Self {
        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });
        self
    }

    /// Add a storage buffer binding
    pub fn storage_buffer(
        mut self,
        binding: u32,
        visibility: wgpu::ShaderStages,
        read_only: bool
    ) -> Self {
        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });
        self
    }

    /// Add a texture binding
    pub fn texture(
        mut self,
        binding: u32,
        visibility: wgpu::ShaderStages,
        sample_type: wgpu::TextureSampleType,
        view_dimension: wgpu::TextureViewDimension,
        multisampled: bool
    ) -> Self {
        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Texture {
                sample_type,
                view_dimension,
                multisampled,
            },
            count: None,
        });
        self
    }

    /// Add a sampler binding
    pub fn sampler(
        mut self,
        binding: u32,
        visibility: wgpu::ShaderStages,
        sampler_type: wgpu::SamplerBindingType
    ) -> Self {
        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Sampler(sampler_type),
            count: None,
        });
        self
    }

    /// Build the bind group layout
    pub fn build(self, context: &GpuContext, label: Option<&str>) -> wgpu::BindGroupLayout {
        context.device.create_bind_group_layout(
            &(wgpu::BindGroupLayoutDescriptor {
                label,
                entries: &self.entries,
            })
        )
    }
}

impl Default for BindGroupLayoutBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating bind groups
pub struct BindGroupBuilder<'a> {
    layout: &'a wgpu::BindGroupLayout,
    entries: Vec<wgpu::BindGroupEntry<'a>>,
}

impl<'a> BindGroupBuilder<'a> {
    pub fn new(layout: &'a wgpu::BindGroupLayout) -> Self {
        Self {
            layout,
            entries: Vec::new(),
        }
    }

    /// Add a buffer binding
    pub fn buffer(mut self, binding: u32, buffer: &'a wgpu::Buffer) -> Self {
        self.entries.push(wgpu::BindGroupEntry {
            binding,
            resource: buffer.as_entire_binding(),
        });
        self
    }

    /// Add a buffer binding with range
    pub fn buffer_range(
        mut self,
        binding: u32,
        buffer: &'a wgpu::Buffer,
        offset: u64,
        size: Option<u64>
    ) -> Self {
        self.entries.push(wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer,
                offset,
                size: size.map(wgpu::BufferSize::new).flatten(),
            }),
        });
        self
    }

    /// Add a texture view binding
    pub fn texture_view(mut self, binding: u32, view: &'a wgpu::TextureView) -> Self {
        self.entries.push(wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::TextureView(view),
        });
        self
    }

    /// Add a sampler binding
    pub fn sampler(mut self, binding: u32, sampler: &'a wgpu::Sampler) -> Self {
        self.entries.push(wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::Sampler(sampler),
        });
        self
    }

    /// Build the bind group
    pub fn build(self, context: &GpuContext, label: Option<&str>) -> wgpu::BindGroup {
        context.device.create_bind_group(
            &(wgpu::BindGroupDescriptor {
                label,
                layout: self.layout,
                entries: &self.entries,
            })
        )
    }
}

/// Convenience macro for creating bind group layouts
#[macro_export]
macro_rules! bind_group_layout {
    ($context:expr, $label:expr, { $($binding:expr => $entry_type:expr),* $(,)? }) => {
        {
            let mut builder = $crate::BindGroupLayoutBuilder::new();
            $(
                builder = match $entry_type {
                    $crate::BindingType::UniformBuffer(visibility) => {
                        builder.uniform_buffer($binding, visibility)
                    },
                    $crate::BindingType::StorageBuffer { visibility, read_only } => {
                        builder.storage_buffer($binding, visibility, read_only)
                    },
                    $crate::BindingType::Texture { visibility, sample_type, view_dimension, multisampled } => {
                        builder.texture($binding, visibility, sample_type, view_dimension, multisampled)
                    },
                    $crate::BindingType::Sampler { visibility, sampler_type } => {
                        builder.sampler($binding, visibility, sampler_type)
                    },
                };
            )*
            builder.build($context, $label)
        }
    };
}

/// Helper enum for the macro
#[derive(Debug, Clone)]
pub enum BindingType {
    UniformBuffer(wgpu::ShaderStages),
    StorageBuffer {
        visibility: wgpu::ShaderStages,
        read_only: bool,
    },
    Texture {
        visibility: wgpu::ShaderStages,
        sample_type: wgpu::TextureSampleType,
        view_dimension: wgpu::TextureViewDimension,
        multisampled: bool,
    },
    Sampler {
        visibility: wgpu::ShaderStages,
        sampler_type: wgpu::SamplerBindingType,
    },
}

/// A simple wrapper that combines a render pipeline and its default bind group
pub struct SimpleRenderPipeline {
    pub pipeline: RenderPipeline,
    pub bind_group: wgpu::BindGroup,
}

/// Builder for creating a render pipeline with automatic resource bindings
pub struct PipelineBuilder<'a> {
    context: &'a GpuContext,
    vs_src: &'a str,
    fs_src: Option<&'a str>,
    layouts: &'a [wgpu::VertexBufferLayout<'static>],
    uniforms: Vec<&'a wgpu::Buffer>,
    textures: Vec<&'a crate::texture::Texture>,
    label: Option<&'a str>,
}

impl<'a> PipelineBuilder<'a> {
    /// Create a new pipeline builder
    pub fn new(
        context: &'a GpuContext,
        vs_src: &'a str,
        fs_src: &'a str,
        layouts: &'a [wgpu::VertexBufferLayout<'static>]
    ) -> Self {
        Self {
            context,
            vs_src,
            fs_src: Some(fs_src),
            layouts,
            uniforms: Vec::new(),
            textures: Vec::new(),
            label: None,
        }
    }

    /// Add a uniform buffer (binding index assigned automatically)
    pub fn uniform<T: bytemuck::Pod>(mut self, buffer: &'a TypedBuffer<T>) -> Self {
        self.uniforms.push(buffer.buffer());
        self
    }

    /// Add a texture (binding index assigned automatically)
    pub fn texture(mut self, texture: &'a crate::texture::Texture) -> Self {
        self.textures.push(texture);
        self
    }

    /// Set an optional label for pipeline and resources
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Build a simple pipeline with automatic bind group
    pub fn build(self, surface_format: wgpu::TextureFormat) -> Result<SimpleRenderPipeline> {
        // Build bind group layout
        let mut layout_builder = BindGroupLayoutBuilder::new();
        // Uniform bindings (both vertex and fragment)
        let uni_vis = ShaderStages::VERTEX | ShaderStages::FRAGMENT;
        for (i, _) in self.uniforms.iter().enumerate() {
            layout_builder = layout_builder.uniform_buffer(i as u32, uni_vis);
        }
        // Texture and sampler bindings (fragment only)
        let tex_vis = ShaderStages::FRAGMENT;
        let ucount = self.uniforms.len() as u32;
        let tcount = self.textures.len() as u32;
        for (i, _) in self.textures.iter().enumerate() {
            let idx = ucount + (i as u32);
            layout_builder = layout_builder.texture(
                idx,
                tex_vis,
                TextureSampleType::Float { filterable: true },
                TextureViewDimension::D2,
                false
            );
            layout_builder = layout_builder.sampler(
                ucount + tcount + (i as u32),
                tex_vis,
                SamplerBindingType::Filtering
            );
        }
        let bind_layout = layout_builder.build(self.context, self.label);

        // Create bind group
        let mut group_builder = BindGroupBuilder::new(&bind_layout);
        for (i, buf) in self.uniforms.iter().enumerate() {
            group_builder = group_builder.buffer(i as u32, buf);
        }
        for (i, tex) in self.textures.iter().enumerate() {
            let idx = ucount + (i as u32);
            group_builder = group_builder.texture_view(idx, &tex.view);
            group_builder = group_builder.sampler(ucount + tcount + (i as u32), &tex.sampler);
        }
        let bind_group = group_builder.build(self.context, self.label);

        // Create the render pipeline
        let pipeline = RenderPipeline::simple(
            self.context,
            self.vs_src,
            self.fs_src.unwrap(),
            self.layouts,
            surface_format,
            self.label
        )?;

        Ok(SimpleRenderPipeline { pipeline, bind_group })
    }
}
