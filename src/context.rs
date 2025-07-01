use crate::{ GeepuError, Result };
use std::sync::Arc;
use winit::window::Window;
use crate::pipeline::{ PipelineBuilder, SimpleRenderPipeline };
use crate::ComputePipeline;

/// Main GPU context that wraps wgpu instance, adapter, device, and queue
pub struct GpuContext {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub surface: Option<wgpu::Surface<'static>>,
    pub surface_config: Option<wgpu::SurfaceConfiguration>,
}

impl GpuContext {
    /// Create a new GPU context without a window (for compute-only applications)
    pub async fn new() -> Result<Self> {
        Self::new_with_features(wgpu::Features::empty()).await
    }

    /// Create a new GPU context with specific features
    pub async fn new_with_features(features: wgpu::Features) -> Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(
                &(wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: None,
                    force_fallback_adapter: false,
                })
            ).await
            .ok_or(GeepuError::AdapterNotFound)?;

        let (device, queue) = adapter
            .request_device(
                &(wgpu::DeviceDescriptor {
                    label: Some("Geepu Device"),
                    required_features: features,
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                }),
                None
            ).await
            .map_err(GeepuError::DeviceCreationFailed)?;

        Ok(Self {
            instance,
            adapter,
            device: Arc::new(device),
            queue: Arc::new(queue),
            surface: None,
            surface_config: None,
        })
    }

    /// Create a new GPU context with a window for rendering
    pub async fn new_with_window(window: Arc<Window>) -> Result<Self> {
        Self::new_with_window_and_features(window, wgpu::Features::empty()).await
    }

    /// Create a new GPU context with a window and specific features
    pub async fn new_with_window_and_features(
        window: Arc<Window>,
        features: wgpu::Features
    ) -> Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance
            .create_surface(window.clone())
            .map_err(|_| GeepuError::SurfaceCreationFailed)?;

        let adapter = instance
            .request_adapter(
                &(wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
            ).await
            .ok_or(GeepuError::AdapterNotFound)?;

        let (device, queue) = adapter
            .request_device(
                &(wgpu::DeviceDescriptor {
                    label: Some("Geepu Device"),
                    required_features: features,
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                }),
                None
            ).await
            .map_err(GeepuError::DeviceCreationFailed)?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let size = window.inner_size();
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        Ok(Self {
            instance,
            adapter,
            device: Arc::new(device),
            queue: Arc::new(queue),
            surface: Some(surface),
            surface_config: Some(surface_config),
        })
    }

    /// Resize the surface (call when window is resized)
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) -> Result<()> {
        if let (Some(surface), Some(config)) = (&self.surface, &mut self.surface_config) {
            config.width = new_size.width.max(1);
            config.height = new_size.height.max(1);
            surface.configure(&self.device, config);
        }
        Ok(())
    }

    /// Get the current surface texture for rendering
    pub fn get_current_texture(&self) -> Result<wgpu::SurfaceTexture> {
        if let Some(surface) = &self.surface {
            surface
                .get_current_texture()
                .map_err(|e| GeepuError::Other(format!("Failed to acquire surface texture: {}", e)))
        } else {
            Err(
                GeepuError::Other(
                    "No surface available - context was created without window".to_string()
                )
            )
        }
    }

    /// Get surface size
    pub fn size(&self) -> (u32, u32) {
        if let Some(config) = &self.surface_config { (config.width, config.height) } else { (0, 0) }
    }

    /// Get surface format
    pub fn surface_format(&self) -> Option<wgpu::TextureFormat> {
        self.surface_config.as_ref().map(|c| c.format)
    }

    /// Create a render pipeline with a single uniform struct and optional textures, no binding groups required.
    pub fn create_simple_pipeline<U: bytemuck::Pod>(
        &self,
        vs_src: &str,
        fs_src: &str,
        vertex_layouts: &[wgpu::VertexBufferLayout<'static>],
        uniform: &U,
        textures: &[&crate::Texture],
        label: Option<&str>
    ) -> Result<SimpleRenderPipeline> {
        let format = self
            .surface_format()
            .ok_or_else(|| crate::GeepuError::Other("No surface available for pipeline".into()))?;
        // Create uniform buffer
        let data = std::slice::from_ref(uniform);
        let uni_buf = crate::TypedBuffer::uniform(self, data)?;
        // Build pipeline
        let mut builder = PipelineBuilder::new(self, vs_src, fs_src, vertex_layouts);
        builder = builder.uniform(&uni_buf);
        for tex in textures {
            builder = builder.texture(tex);
        }
        if let Some(lbl) = label {
            builder = builder.label(lbl);
        }
        builder.build(format)
    }

    /// Create a compute pipeline from a uniform struct and optional storage buffers, automatic bindings
    pub fn create_simple_compute<U: bytemuck::Pod>(
        &self,
        cs_src: &str,
        uniform: &U,
        storage: &[&wgpu::Buffer],
        label: Option<&str>
    ) -> Result<ComputePipeline> {
        // Create uniform buffer
        let data: &[U] = std::slice::from_ref(uniform);
        let uni_buf = crate::TypedBuffer::<U>::uniform(self, data)?;
        // Build bind group layout
        let mut layout = crate::pipeline::BindGroupLayoutBuilder
            ::new()
            .uniform_buffer(0, wgpu::ShaderStages::COMPUTE);
        for (i, buf) in storage.iter().enumerate() {
            layout = layout.storage_buffer((i + 1) as u32, wgpu::ShaderStages::COMPUTE, false);
        }
        let bind_layout = layout.build(self, label);
        // Build bind group
        let mut group = crate::pipeline::BindGroupBuilder
            ::new(&bind_layout)
            .buffer(0, uni_buf.buffer());
        for (i, buf) in storage.iter().enumerate() {
            group = group.buffer((i + 1) as u32, buf);
        }
        let bind_group = group.build(self, label);
        // Create compute pipeline
        ComputePipeline::new(self, cs_src, vec![bind_layout], label)
    }
}
