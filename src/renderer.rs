//! Main renderer implementation

use crate::{
    config::{WindowConfig, Size, GpuConfig},
    error::{GeepuError, Result},
    resources::{ResourceManager, UniformBuffer, StorageBuffer, TextureResource},
    shaders::{ShaderManager, ComputePipeline},
};
use std::sync::Arc;
use tracing::{info, debug, warn, span, Level};
use winit::{
    event_loop::EventLoop,
    window::Window,
};

/// Main renderer struct that wraps wgpu functionality
pub struct Renderer {
    /// wgpu instance
    pub instance: wgpu::Instance,
    /// Graphics adapter
    pub adapter: wgpu::Adapter,
    /// Logical device
    pub device: wgpu::Device,
    /// Command queue
    pub queue: wgpu::Queue,
    /// Surface (for windowed rendering)
    pub surface: Option<wgpu::Surface<'static>>,
    /// Surface configuration
    pub surface_config: Option<wgpu::SurfaceConfiguration>,
    /// Window (for windowed rendering)
    pub window: Option<Arc<Window>>,
    /// Offscreen render target (for offscreen rendering)
    pub render_target: Option<TextureResource>,
    /// Current encoder for batching commands
    pub encoder: Option<wgpu::CommandEncoder>,
    /// Resource manager
    pub resources: ResourceManager,
    /// Shader manager
    pub shaders: ShaderManager,
    /// Compute pipelines
    pub compute_pipelines: std::collections::HashMap<String, ComputePipeline>,
    /// Current size
    pub size: Size,
}

impl Renderer {
    /// Create a new windowed renderer
    pub async fn new(window_config: WindowConfig) -> Result<Self> {
        Self::new_with_gpu_config(window_config, GpuConfig::default()).await
    }

    /// Create a new windowed renderer with custom GPU configuration
    pub async fn new_with_gpu_config(window_config: WindowConfig, gpu_config: GpuConfig) -> Result<Self> {
        let span = span!(Level::INFO, "create_windowed_renderer");
        let _enter = span.enter();

        info!("Creating windowed renderer with size: {:?}", window_config.size);

        // For this example, we'll create a simple window
        // In a real application, you'd handle the event loop properly
        info!("Note: Window creation simplified for this example");
        
        // Create dummy window for surface creation
        let event_loop = EventLoop::new().map_err(|e| GeepuError::Generic(format!("Failed to create event loop: {}", e)))?;
        #[allow(deprecated)]
        let window = event_loop.create_window(
            Window::default_attributes()
                .with_title(&window_config.title)
                .with_inner_size(winit::dpi::PhysicalSize::new(window_config.size.width, window_config.size.height))
                .with_resizable(window_config.resizable)
        ).map_err(|e| GeepuError::Generic(format!("Failed to create window: {}", e)))?;
        
        let window = Arc::new(window);

        // Create wgpu instance
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: gpu_config.backends,
            ..Default::default()
        });

        // Create surface
        let surface = instance.create_surface(window.clone())
            .map_err(|e| GeepuError::Generic(format!("Failed to create surface: {}", e)))?;

        // Request adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: gpu_config.power_preference,
                compatible_surface: Some(&surface),
                force_fallback_adapter: gpu_config.force_fallback_adapter,
            })
            .await
            .map_err(|e| GeepuError::Generic(format!("Failed to request adapter: {:?}", e)))?;

        info!("Found adapter: {}", adapter.get_info().name);

        // Request device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("main_device"),
                    required_features: gpu_config.features,
                    required_limits: gpu_config.limits,
                    memory_hints: Default::default(),
                    trace: Default::default(),
                },
            )
            .await?;

        info!("Created device and queue");

        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_config.size.width,
            height: window_config.size.height,
            present_mode: if window_config.vsync {
                surface_caps.present_modes[0]
            } else {
                wgpu::PresentMode::Immediate
            },
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        info!("Configured surface with format: {:?}", surface_format);

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            surface: Some(surface),
            surface_config: Some(surface_config),
            window: Some(window),
            render_target: None,
            encoder: None,
            resources: ResourceManager::new(),
            shaders: ShaderManager::new(),
            compute_pipelines: std::collections::HashMap::new(),
            size: window_config.size,
        })
    }

    /// Create a new offscreen renderer
    pub async fn offscreen(size: Size) -> Result<Self> {
        Self::offscreen_with_gpu_config(size, GpuConfig::default()).await
    }

    /// Create a new offscreen renderer with custom GPU configuration
    pub async fn offscreen_with_gpu_config(size: Size, gpu_config: GpuConfig) -> Result<Self> {
        let span = span!(Level::INFO, "create_offscreen_renderer");
        let _enter = span.enter();

        info!("Creating offscreen renderer with size: {:?}", size);

        // Create wgpu instance
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: gpu_config.backends,
            ..Default::default()
        });

        // Request adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: gpu_config.power_preference,
                compatible_surface: None,
                force_fallback_adapter: gpu_config.force_fallback_adapter,
            })
            .await
            .map_err(|e| GeepuError::Generic(format!("Failed to request adapter: {:?}", e)))?;

        info!("Found adapter: {}", adapter.get_info().name);

        // Request device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("offscreen_device"),
                    required_features: gpu_config.features,
                    required_limits: gpu_config.limits,
                    memory_hints: Default::default(),
                    trace: Default::default(),
                },
            )
            .await?;

        info!("Created device and queue");

        // Create offscreen render target
        let render_target = TextureResource::create_render_target(
            &device,
            size.width,
            size.height,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            Some("offscreen_render_target"),
        );

        info!("Created offscreen render target");

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            surface: None,
            surface_config: None,
            window: None,
            render_target: Some(render_target),
            encoder: None,
            resources: ResourceManager::new(),
            shaders: ShaderManager::new(),
            compute_pipelines: std::collections::HashMap::new(),
            size,
        })
    }

    /// Add a uniform buffer
    pub fn add_uniform<T: bytemuck::Pod + Send + Sync + 'static>(&mut self, name: &str, data: &T) {
        let span = span!(Level::DEBUG, "add_uniform", name = name);
        let _enter = span.enter();

        debug!("Adding uniform buffer: {}", name);
        let uniform = UniformBuffer::new(&self.device, data, Some(name));
        self.resources.add_uniform(name.to_string(), uniform);
    }

    /// Update a uniform buffer
    pub fn update_uniform<T: bytemuck::Pod + Send + Sync + 'static>(&self, name: &str, data: &T) -> Result<()> {
        let span = span!(Level::DEBUG, "update_uniform", name = name);
        let _enter = span.enter();

        debug!("Updating uniform buffer: {}", name);
        let uniform = self.resources.get_uniform::<T>(name)?;
        uniform.update(&self.queue, data);
        Ok(())
    }

    /// Add a storage buffer
    pub fn add_storage_buffer<T: bytemuck::Pod + Send + Sync + 'static>(&mut self, name: &str, data: &[T]) {
        self.add_storage_buffer_with_access(name, data, true)
    }

    /// Add a storage buffer with specific access mode
    pub fn add_storage_buffer_with_access<T: bytemuck::Pod + Send + Sync + 'static>(
        &mut self, 
        name: &str, 
        data: &[T], 
        read_only: bool
    ) {
        let span = span!(Level::DEBUG, "add_storage_buffer", name = name, read_only = read_only);
        let _enter = span.enter();

        debug!("Adding storage buffer: {} (read_only: {})", name, read_only);
        let buffer = StorageBuffer::new(&self.device, data, read_only, Some(name));
        self.resources.add_storage_buffer(name.to_string(), buffer);
    }

    /// Update a storage buffer
    pub fn update_storage_buffer<T: bytemuck::Pod + Send + Sync + 'static>(&self, name: &str, data: &[T]) -> Result<()> {
        let span = span!(Level::DEBUG, "update_storage_buffer", name = name);
        let _enter = span.enter();

        debug!("Updating storage buffer: {}", name);
        let buffer = self.resources.get_storage_buffer::<T>(name)?;
        buffer.update(&self.queue, data);
        Ok(())
    }

    /// Read data from a storage buffer
    pub async fn read_storage_buffer<T: bytemuck::Pod + Send + Sync + 'static>(&self, name: &str) -> Result<Vec<T>> {
        let span = span!(Level::DEBUG, "read_storage_buffer", name = name);
        let _enter = span.enter();

        debug!("Reading storage buffer: {}", name);
        let buffer = self.resources.get_storage_buffer::<T>(name)?;
        buffer.read_data(&self.device, &self.queue).await
    }

    /// Add a texture from an image
    pub fn add_texture(&mut self, name: &str, image: image::DynamicImage) -> Result<()> {
        let span = span!(Level::DEBUG, "add_texture", name = name);
        let _enter = span.enter();

        debug!("Adding texture: {}", name);
        let texture = TextureResource::from_image(&self.device, &self.queue, &image, Some(name))?;
        self.resources.add_texture(name.to_string(), texture);
        Ok(())
    }

    /// Add a compute shader
    pub fn add_compute_shader(&mut self, name: &str, source: &str) -> Result<()> {
        let span = span!(Level::DEBUG, "add_compute_shader", name = name);
        let _enter = span.enter();

        debug!("Adding compute shader: {}", name);
        self.shaders.load_compute_shader(&self.device, name, source)
    }

    /// Create a compute pipeline
    pub fn create_compute_pipeline(
        &mut self,
        name: &str,
        shader_name: &str,
        entry_point: &str,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
        workgroup_size: (u32, u32, u32),
    ) -> Result<()> {
        let span = span!(Level::DEBUG, "create_compute_pipeline", name = name, shader_name = shader_name);
        let _enter = span.enter();

        debug!("Creating compute pipeline: {}", name);
        let shader = self.shaders.get_compute_shader(shader_name)?;
        let pipeline = ComputePipeline::new(
            &self.device,
            shader,
            entry_point,
            bind_group_layouts,
            workgroup_size,
            Some(name),
        );
        self.compute_pipelines.insert(name.to_string(), pipeline);
        Ok(())
    }

    /// Dispatch a compute shader
    pub fn dispatch_compute(&mut self, name: &str, x: u32, y: u32, z: u32) -> Result<()> {
        let span = span!(Level::DEBUG, "dispatch_compute", name = name, x = x, y = y, z = z);
        let _enter = span.enter();

        debug!("Dispatching compute shader: {} with workgroups ({}, {}, {})", name, x, y, z);
        
        let pipeline = self.compute_pipelines.get(name)
            .ok_or_else(|| GeepuError::ResourceNotFound(format!("compute pipeline '{}'", name)))?;

        let encoder = self.encoder.as_mut()
            .ok_or_else(|| GeepuError::InvalidOperation("No active command encoder. Call begin_pass() first.".to_string()))?;

        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some(name),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&pipeline.pipeline);
        // Note: Bind groups would need to be set here based on the resources
        compute_pass.dispatch_workgroups(x, y, z);
        
        Ok(())
    }

    /// Begin a render pass
    pub fn begin_pass(&mut self) -> RenderPassGuard {
        let span = span!(Level::DEBUG, "begin_pass");
        let _enter = span.enter();

        debug!("Beginning render pass");
        
        if self.encoder.is_some() {
            warn!("Command encoder already exists, replacing with new one");
        }

        let encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("main_encoder"),
        });
        self.encoder = Some(encoder);

        RenderPassGuard { renderer: self }
    }

    /// Submit all pending commands
    pub fn submit(&mut self) {
        let span = span!(Level::DEBUG, "submit");
        let _enter = span.enter();

        debug!("Submitting commands");

        if let Some(encoder) = self.encoder.take() {
            self.queue.submit([encoder.finish()]);
        } else {
            warn!("No command encoder to submit");
        }
    }

    /// Get the current surface texture (for windowed rendering)
    pub fn get_surface_texture(&self) -> Result<wgpu::SurfaceTexture> {
        let surface = self.surface.as_ref()
            .ok_or_else(|| GeepuError::InvalidOperation("No surface available for offscreen renderer".to_string()))?;
        
        surface.get_current_texture().map_err(GeepuError::SurfaceError)
    }

    /// Present the current frame (for windowed rendering)
    pub fn present(&self, texture: wgpu::SurfaceTexture) {
        let span = span!(Level::DEBUG, "present");
        let _enter = span.enter();

        debug!("Presenting frame");
        texture.present();
    }

    /// Resize the renderer (for windowed rendering)
    pub fn resize(&mut self, new_size: Size) -> Result<()> {
        let span = span!(Level::INFO, "resize", width = new_size.width, height = new_size.height);
        let _enter = span.enter();

        info!("Resizing renderer to: {:?}", new_size);

        if new_size.width == 0 || new_size.height == 0 {
            return Ok(());
        }

        self.size = new_size;

        if let Some(ref mut config) = self.surface_config {
            config.width = new_size.width;
            config.height = new_size.height;
            
            if let Some(ref surface) = self.surface {
                surface.configure(&self.device, config);
            }
        }

        // Update offscreen render target if needed
        if let Some(ref mut render_target) = self.render_target {
            *render_target = TextureResource::create_render_target(
                &self.device,
                new_size.width,
                new_size.height,
                wgpu::TextureFormat::Rgba8UnormSrgb,
                Some("offscreen_render_target"),
            );
        }

        Ok(())
    }

    /// Copy the current render target to an image buffer (for offscreen rendering)
    pub async fn copy_to_buffer(&self) -> Result<Vec<u8>> {
        let span = span!(Level::DEBUG, "copy_to_buffer");
        let _enter = span.enter();

        debug!("Copying render target to buffer");

        let render_target = self.render_target.as_ref()
            .ok_or_else(|| GeepuError::InvalidOperation("No render target available for windowed renderer".to_string()))?;

        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("copy_buffer"),
            size: (self.size.width * self.size.height * 4) as u64, // RGBA8
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("copy_encoder"),
        });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &render_target.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * self.size.width),
                    rows_per_image: Some(self.size.height),
                },
            },
            wgpu::Extent3d {
                width: self.size.width,
                height: self.size.height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit([encoder.finish()]);

        let buffer_slice = buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device.poll(wgpu::MaintainBase::wait()).map_err(|e| GeepuError::Generic(format!("Poll error: {:?}", e)))?;

        let data = buffer_slice.get_mapped_range();
        let result = data.to_vec();
        drop(data);
        buffer.unmap();

        Ok(result)
    }
}

/// RAII guard for render passes
pub struct RenderPassGuard<'a> {
    renderer: &'a mut Renderer,
}

impl<'a> RenderPassGuard<'a> {
    /// Create a render pass targeting the surface or render target
    pub fn render_pass(&mut self, clear_color: Option<wgpu::Color>) -> Result<wgpu::RenderPass> {
        let encoder = self.renderer.encoder.as_mut()
            .ok_or_else(|| GeepuError::InvalidOperation("No active command encoder".to_string()))?;

        let (view, load_op) = if let Some(render_target) = &self.renderer.render_target {
            // Offscreen rendering
            (&render_target.view, if let Some(color) = clear_color {
                wgpu::LoadOp::Clear(color)
            } else {
                wgpu::LoadOp::Load
            })
        } else if let Some(surface) = &self.renderer.surface {
            // Windowed rendering
            let texture = surface.get_current_texture().map_err(GeepuError::SurfaceError)?;
            let _view = texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
            // Note: This won't compile as written - you'd need to store the texture somewhere
            return Err(GeepuError::InvalidOperation("Surface rendering not fully implemented in this guard".to_string()));
        } else {
            return Err(GeepuError::InvalidOperation("No render target or surface available".to_string()));
        };

        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("main_render_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: load_op,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        Ok(render_pass)
    }

    /// Draw a simple indexed mesh
    pub fn draw_indexed(&mut self, indices: std::ops::Range<u32>, base_vertex: i32, instances: std::ops::Range<u32>) -> Result<()> {
        // This would need a proper render pass implementation
        // For now, this is a placeholder
        debug!("Drawing indexed mesh: indices={:?}, base_vertex={}, instances={:?}", indices, base_vertex, instances);
        Ok(())
    }
}
