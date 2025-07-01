use crate::{ GpuContext, GeepuError, Result };
use std::marker::PhantomData;
use wgpu::util::DeviceExt;

/// A typed buffer wrapper that provides zero-cost abstractions
pub struct TypedBuffer<T> {
    buffer: wgpu::Buffer,
    len: usize,
    _phantom: PhantomData<T>,
}

impl<T> TypedBuffer<T> where T: bytemuck::Pod {
    /// Create a new buffer with data
    pub fn new(context: &GpuContext, data: &[T], usage: wgpu::BufferUsages) -> Result<Self> {
        let buffer = context.device.create_buffer_init(
            &(wgpu::util::BufferInitDescriptor {
                label: Some(&format!("TypedBuffer<{}>", std::any::type_name::<T>())),
                contents: bytemuck::cast_slice(data),
                usage,
            })
        );

        Ok(Self {
            buffer,
            len: data.len(),
            _phantom: PhantomData,
        })
    }

    /// Create an empty buffer with a specific size
    pub fn empty(context: &GpuContext, len: usize, usage: wgpu::BufferUsages) -> Result<Self> {
        let buffer = context.device.create_buffer(
            &(wgpu::BufferDescriptor {
                label: Some(&format!("TypedBuffer<{}>", std::any::type_name::<T>())),
                size: (len * std::mem::size_of::<T>()) as u64,
                usage,
                mapped_at_creation: false,
            })
        );

        Ok(Self {
            buffer,
            len,
            _phantom: PhantomData,
        })
    }

    /// Write data to the buffer
    pub fn write(&self, context: &GpuContext, data: &[T]) -> Result<()> {
        if data.len() > self.len {
            return Err(GeepuError::BufferError("Data size exceeds buffer capacity".to_string()));
        }

        context.queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(data));
        Ok(())
    }

    /// Get the underlying wgpu buffer
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    /// Get the number of elements in the buffer
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the size in bytes
    pub fn size_bytes(&self) -> u64 {
        (self.len * std::mem::size_of::<T>()) as u64
    }
}

/// A builder for creating vertex buffers with ergonomic attribute specification
pub struct VertexBufferBuilder {
    attributes: Vec<wgpu::VertexAttribute>,
    stride: u64,
    step_mode: wgpu::VertexStepMode,
}

impl VertexBufferBuilder {
    pub fn new() -> Self {
        Self {
            attributes: Vec::new(),
            stride: 0,
            step_mode: wgpu::VertexStepMode::Vertex,
        }
    }

    /// Add a vertex attribute
    pub fn attribute(mut self, format: wgpu::VertexFormat, shader_location: u32) -> Self {
        let offset = self.stride;
        self.attributes.push(wgpu::VertexAttribute {
            offset,
            shader_location,
            format,
        });
        self.stride += format.size();
        self
    }

    /// Set step mode (vertex or instance)
    pub fn step_mode(mut self, step_mode: wgpu::VertexStepMode) -> Self {
        self.step_mode = step_mode;
        self
    }

    /// Build the vertex buffer layout
    pub fn build(self) -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: self.stride,
            step_mode: self.step_mode,
            attributes: self.attributes.leak(), // Safe for static layouts
        }
    }
}

impl Default for VertexBufferBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience functions for common buffer types
impl<T> TypedBuffer<T> where T: bytemuck::Pod {
    /// Create a vertex buffer
    pub fn vertex(context: &GpuContext, data: &[T]) -> Result<Self> {
        Self::new(context, data, wgpu::BufferUsages::VERTEX)
    }

    /// Create an index buffer
    pub fn index(context: &GpuContext, data: &[T]) -> Result<Self> {
        Self::new(context, data, wgpu::BufferUsages::INDEX)
    }

    /// Create a uniform buffer
    pub fn uniform(context: &GpuContext, data: &[T]) -> Result<Self> {
        Self::new(context, data, wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST)
    }

    /// Create a storage buffer
    pub fn storage(context: &GpuContext, data: &[T]) -> Result<Self> {
        Self::new(context, data, wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST)
    }
}

/// A staging buffer for CPU-GPU data transfers
pub struct StagingBuffer {
    buffer: wgpu::Buffer,
    size: u64,
}

impl StagingBuffer {
    /// Create a new staging buffer
    pub fn new(context: &GpuContext, size: u64) -> Result<Self> {
        let buffer = context.device.create_buffer(
            &(wgpu::BufferDescriptor {
                label: Some("Staging Buffer"),
                size,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        );

        Ok(Self { buffer, size })
    }

    /// Copy data from a GPU buffer to this staging buffer
    pub fn copy_from_buffer(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::Buffer,
        size: Option<u64>
    ) {
        let copy_size = size.unwrap_or(self.size);
        encoder.copy_buffer_to_buffer(source, 0, &self.buffer, 0, copy_size);
    }

    /// Map the buffer and read data
    pub async fn read_data<T>(&self, context: &GpuContext) -> Result<Vec<T>> where T: bytemuck::Pod {
        let buffer_slice = self.buffer.slice(..);

        // Use a simple future with shared state
        let (sender, receiver) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });

        context.device.poll(wgpu::Maintain::Wait);

        receiver
            .recv()
            .unwrap()
            .map_err(|e| { GeepuError::BufferError(format!("Failed to map buffer: {:?}", e)) })?;

        let data = buffer_slice.get_mapped_range();
        let result = bytemuck::cast_slice(&data).to_vec();
        drop(data);
        self.buffer.unmap();

        Ok(result)
    }

    /// Get the underlying buffer
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}

/// Convenience macro for creating vertex buffer layouts
#[macro_export]
macro_rules! vertex_layout {
    ($($location:expr => $format:expr),* $(,)?) => {
        {
            let mut builder = $crate::VertexBufferBuilder::new();
            $(
                builder = builder.attribute($format, $location);
            )*
            builder.build()
        }
    };
}

// Re-export for convenience
pub use wgpu::VertexFormat;
