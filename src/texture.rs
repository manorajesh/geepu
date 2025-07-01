use crate::{ GpuContext, GeepuError, Result };
use wgpu::util::DeviceExt;

/// A wrapper around wgpu::Texture with convenient methods
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    /// Create a new texture from raw data
    pub fn from_bytes(
        context: &GpuContext,
        bytes: &[u8],
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        label: Option<&str>
    ) -> Result<Self> {
        let texture = context.device.create_texture_with_data(
            &context.queue,
            &(wgpu::TextureDescriptor {
                label,
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            }),
            wgpu::util::TextureDataOrder::LayerMajor,
            bytes
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = context.device.create_sampler(
            &(wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            })
        );

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }

    /// Create an empty texture for rendering
    pub fn create_empty(
        context: &GpuContext,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
        label: Option<&str>
    ) -> Result<Self> {
        let texture = context.device.create_texture(
            &(wgpu::TextureDescriptor {
                label,
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage,
                view_formats: &[],
            })
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = context.device.create_sampler(
            &(wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            })
        );

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }

    /// Create a depth texture
    pub fn create_depth_texture(
        context: &GpuContext,
        width: u32,
        height: u32,
        label: Option<&str>
    ) -> Result<Self> {
        let format = wgpu::TextureFormat::Depth32Float;
        let texture = context.device.create_texture(
            &(wgpu::TextureDescriptor {
                label,
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT |
                wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = context.device.create_sampler(
            &(wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                compare: Some(wgpu::CompareFunction::LessEqual),
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            })
        );

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }

    /// Create a render target texture
    pub fn create_render_target(
        context: &GpuContext,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        label: Option<&str>
    ) -> Result<Self> {
        Self::create_empty(
            context,
            width,
            height,
            format,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            label
        )
    }

    /// Get size of the texture
    pub fn size(&self) -> (u32, u32) {
        let size = self.texture.size();
        (size.width, size.height)
    }

    /// Get format of the texture
    pub fn format(&self) -> wgpu::TextureFormat {
        self.texture.format()
    }
}

/// Builder for creating textures with custom settings
pub struct TextureBuilder {
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
    label: Option<String>,
    mip_level_count: u32,
    sample_count: u32,
    sampler_descriptor: wgpu::SamplerDescriptor<'static>,
}

impl TextureBuilder {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: None,
            mip_level_count: 1,
            sample_count: 1,
            sampler_descriptor: wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            },
        }
    }

    pub fn format(mut self, format: wgpu::TextureFormat) -> Self {
        self.format = format;
        self
    }

    pub fn usage(mut self, usage: wgpu::TextureUsages) -> Self {
        self.usage = usage;
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn mip_levels(mut self, count: u32) -> Self {
        self.mip_level_count = count;
        self
    }

    pub fn sample_count(mut self, count: u32) -> Self {
        self.sample_count = count;
        self
    }

    pub fn sampler(mut self, sampler_descriptor: wgpu::SamplerDescriptor<'static>) -> Self {
        self.sampler_descriptor = sampler_descriptor;
        self
    }

    pub fn build(self, context: &GpuContext) -> Result<Texture> {
        let texture = context.device.create_texture(
            &(wgpu::TextureDescriptor {
                label: self.label.as_deref(),
                size: wgpu::Extent3d {
                    width: self.width,
                    height: self.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: self.mip_level_count,
                sample_count: self.sample_count,
                dimension: wgpu::TextureDimension::D2,
                format: self.format,
                usage: self.usage,
                view_formats: &[],
            })
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = context.device.create_sampler(&self.sampler_descriptor);

        Ok(Texture {
            texture,
            view,
            sampler,
        })
    }
}

/// Convenience functions for common texture operations
impl Texture {
    /// Write data to texture
    pub fn write_data(
        &self,
        context: &GpuContext,
        data: &[u8],
        width: u32,
        height: u32
    ) -> Result<()> {
        let bytes_per_pixel = match self.format() {
            | wgpu::TextureFormat::Rgba8Unorm
            | wgpu::TextureFormat::Rgba8UnormSrgb
            | wgpu::TextureFormat::Bgra8Unorm
            | wgpu::TextureFormat::Bgra8UnormSrgb => 4,
            wgpu::TextureFormat::Rgb9e5Ufloat => 4,
            wgpu::TextureFormat::Rg8Unorm => 2,
            wgpu::TextureFormat::R8Unorm => 1,
            _ => {
                return Err(
                    GeepuError::TextureError(
                        "Unsupported texture format for write_data".to_string()
                    )
                );
            }
        };

        context.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_pixel * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            }
        );

        Ok(())
    }
}
