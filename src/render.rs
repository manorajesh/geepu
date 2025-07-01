use crate::{ GpuContext, RenderPipeline, TypedBuffer, Result };

/// A high-level render pass wrapper
pub struct RenderPass<'a> {
    pass: wgpu::RenderPass<'a>,
}

impl<'a> RenderPass<'a> {
    /// Create a new render pass
    pub fn new(
        encoder: &'a mut wgpu::CommandEncoder,
        color_attachments: &'a [Option<wgpu::RenderPassColorAttachment<'a>>],
        depth_stencil_attachment: Option<wgpu::RenderPassDepthStencilAttachment<'a>>,
        label: Option<&str>
    ) -> Self {
        let pass = encoder.begin_render_pass(
            &(wgpu::RenderPassDescriptor {
                label,
                color_attachments,
                depth_stencil_attachment,
                occlusion_query_set: None,
                timestamp_writes: None,
            })
        );

        Self { pass }
    }

    /// Set the render pipeline
    pub fn set_pipeline(&mut self, pipeline: &'a RenderPipeline) {
        self.pass.set_pipeline(&pipeline.pipeline);
    }

    /// Set a bind group
    pub fn set_bind_group(&mut self, index: u32, bind_group: &'a wgpu::BindGroup, offsets: &[u32]) {
        self.pass.set_bind_group(index, bind_group, offsets);
    }

    /// Set vertex buffer
    pub fn set_vertex_buffer<T>(&mut self, slot: u32, buffer: &'a TypedBuffer<T>)
        where T: bytemuck::Pod
    {
        self.pass.set_vertex_buffer(slot, buffer.buffer().slice(..));
    }

    /// Set index buffer
    pub fn set_index_buffer<T>(&mut self, buffer: &'a TypedBuffer<T>, format: wgpu::IndexFormat)
        where T: bytemuck::Pod
    {
        self.pass.set_index_buffer(buffer.buffer().slice(..), format);
    }

    /// Draw primitives
    pub fn draw(&mut self, vertices: std::ops::Range<u32>, instances: std::ops::Range<u32>) {
        self.pass.draw(vertices, instances);
    }

    /// Draw indexed primitives
    pub fn draw_indexed(
        &mut self,
        indices: std::ops::Range<u32>,
        base_vertex: i32,
        instances: std::ops::Range<u32>
    ) {
        self.pass.draw_indexed(indices, base_vertex, instances);
    }
}

/// A high-level render command builder
pub struct RenderCommands {
    encoder: wgpu::CommandEncoder,
}

impl RenderCommands {
    /// Create new render commands
    pub fn new(context: &GpuContext, label: Option<&str>) -> Self {
        let encoder = context.device.create_command_encoder(
            &(wgpu::CommandEncoderDescriptor {
                label,
            })
        );

        Self { encoder }
    }

    /// Begin a render pass
    pub fn begin_render_pass<'a>(
        &'a mut self,
        color_attachments: &'a [Option<wgpu::RenderPassColorAttachment<'a>>],
        depth_stencil_attachment: Option<wgpu::RenderPassDepthStencilAttachment<'a>>,
        label: Option<&str>
    ) -> RenderPass<'a> {
        RenderPass::new(&mut self.encoder, color_attachments, depth_stencil_attachment, label)
    }

    /// Copy buffer to buffer
    pub fn copy_buffer_to_buffer(
        &mut self,
        source: &wgpu::Buffer,
        source_offset: u64,
        destination: &wgpu::Buffer,
        destination_offset: u64,
        copy_size: u64
    ) {
        self.encoder.copy_buffer_to_buffer(
            source,
            source_offset,
            destination,
            destination_offset,
            copy_size
        );
    }

    /// Copy buffer to texture
    pub fn copy_buffer_to_texture(
        &mut self,
        source: wgpu::ImageCopyBuffer,
        destination: wgpu::ImageCopyTexture,
        copy_size: wgpu::Extent3d
    ) {
        self.encoder.copy_buffer_to_texture(source, destination, copy_size);
    }

    /// Copy texture to buffer
    pub fn copy_texture_to_buffer(
        &mut self,
        source: wgpu::ImageCopyTexture,
        destination: wgpu::ImageCopyBuffer,
        copy_size: wgpu::Extent3d
    ) {
        self.encoder.copy_texture_to_buffer(source, destination, copy_size);
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

/// Simple render target helper
pub struct RenderTarget {
    pub texture: crate::Texture,
    pub depth_texture: Option<crate::Texture>,
}

impl RenderTarget {
    /// Create a new render target
    pub fn new(
        context: &GpuContext,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        with_depth: bool,
        label: Option<&str>
    ) -> Result<Self> {
        let texture = crate::Texture::create_render_target(context, width, height, format, label)?;

        let depth_texture = if with_depth {
            Some(
                crate::Texture::create_depth_texture(
                    context,
                    width,
                    height,
                    Some(&format!("{}_depth", label.unwrap_or("render_target")))
                )?
            )
        } else {
            None
        };

        Ok(Self {
            texture,
            depth_texture,
        })
    }

    /// Get color attachment for render pass
    pub fn color_attachment(
        &self,
        clear_color: Option<wgpu::Color>
    ) -> wgpu::RenderPassColorAttachment {
        wgpu::RenderPassColorAttachment {
            view: &self.texture.view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: if let Some(color) = clear_color {
                    wgpu::LoadOp::Clear(color)
                } else {
                    wgpu::LoadOp::Load
                },
                store: wgpu::StoreOp::Store,
            },
        }
    }

    /// Get depth stencil attachment for render pass
    pub fn depth_stencil_attachment(
        &self,
        clear_depth: Option<f32>
    ) -> Option<wgpu::RenderPassDepthStencilAttachment> {
        self.depth_texture.as_ref().map(|depth| wgpu::RenderPassDepthStencilAttachment {
            view: &depth.view,
            depth_ops: Some(wgpu::Operations {
                load: if let Some(depth) = clear_depth {
                    wgpu::LoadOp::Clear(depth)
                } else {
                    wgpu::LoadOp::Load
                },
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        })
    }

    /// Get size of the render target
    pub fn size(&self) -> (u32, u32) {
        self.texture.size()
    }
}

/// Helper for creating render pass color attachments
pub fn color_attachment(
    view: &wgpu::TextureView,
    clear_color: Option<wgpu::Color>
) -> wgpu::RenderPassColorAttachment {
    wgpu::RenderPassColorAttachment {
        view,
        resolve_target: None,
        ops: wgpu::Operations {
            load: if let Some(color) = clear_color {
                wgpu::LoadOp::Clear(color)
            } else {
                wgpu::LoadOp::Load
            },
            store: wgpu::StoreOp::Store,
        },
    }
}

/// Helper for creating depth stencil attachments
pub fn depth_stencil_attachment(
    view: &wgpu::TextureView,
    clear_depth: Option<f32>,
    clear_stencil: Option<u32>
) -> wgpu::RenderPassDepthStencilAttachment {
    wgpu::RenderPassDepthStencilAttachment {
        view,
        depth_ops: clear_depth.map(|depth| wgpu::Operations {
            load: wgpu::LoadOp::Clear(depth),
            store: wgpu::StoreOp::Store,
        }),
        stencil_ops: clear_stencil.map(|stencil| wgpu::Operations {
            load: wgpu::LoadOp::Clear(stencil),
            store: wgpu::StoreOp::Store,
        }),
    }
}
