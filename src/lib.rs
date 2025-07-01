//! Geepu - An ergonomic, zero-cost wrapper around wgpu
//! 
//! This library provides a simplified interface to wgpu while maintaining
//! performance and zero-cost abstractions.

pub mod context;
pub mod buffer;
pub mod texture;
pub mod pipeline;
pub mod render;
pub mod compute;
pub mod error;

pub use context::*;
pub use buffer::*;
pub use texture::*;
pub use pipeline::*;
pub use render::*;
pub use compute::*;
pub use error::*;

// Re-export commonly used wgpu types
pub use wgpu::{
    Color, BufferUsages, TextureFormat, TextureUsages, Features, Limits,
    ShaderStages, BindingType, SamplerBindingType, TextureSampleType,
    TextureViewDimension, PrimitiveTopology, FrontFace, PolygonMode,
    BlendState, ColorTargetState, MultisampleState, DepthStencilState,
    CompareFunction, StencilState, DepthBiasState,
};

// Re-export bytemuck for vertex data
pub use bytemuck::{Pod, Zeroable};
