//! Configuration types for Geepu

use winit::dpi::PhysicalSize;

/// Size configuration for textures and surfaces
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    /// Create a new size
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Get the aspect ratio (width / height)
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }
}

impl From<PhysicalSize<u32>> for Size {
    fn from(size: PhysicalSize<u32>) -> Self {
        Self::new(size.width, size.height)
    }
}

impl From<Size> for PhysicalSize<u32> {
    fn from(size: Size) -> Self {
        PhysicalSize::new(size.width, size.height)
    }
}

/// Window configuration for windowed rendering
#[derive(Debug, Clone)]
pub struct WindowConfig {
    pub title: String,
    pub size: Size,
    pub resizable: bool,
    pub vsync: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Geepu Application".to_string(),
            size: Size::new(1280, 720),
            resizable: true,
            vsync: true,
        }
    }
}

impl WindowConfig {
    /// Create a new window configuration
    pub fn new(title: impl Into<String>, size: Size) -> Self {
        Self {
            title: title.into(),
            size,
            resizable: true,
            vsync: true,
        }
    }

    /// Set whether the window is resizable
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Set whether vsync is enabled
    pub fn vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;
        self
    }
}

/// Advanced GPU configuration
#[derive(Debug, Clone)]
pub struct GpuConfig {
    /// wgpu backends to use
    pub backends: wgpu::Backends,
    /// Required device features
    pub features: wgpu::Features,
    /// Device limits
    pub limits: wgpu::Limits,
    /// Power preference
    pub power_preference: wgpu::PowerPreference,
    /// Force fallback adapter
    pub force_fallback_adapter: bool,
}

impl Default for GpuConfig {
    fn default() -> Self {
        Self {
            backends: wgpu::Backends::all(),
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::default(),
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
        }
    }
}

impl GpuConfig {
    /// Create a new GPU configuration with defaults optimized for performance
    pub fn performance() -> Self {
        Self {
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        }
    }

    /// Create a new GPU configuration with defaults optimized for power saving
    pub fn low_power() -> Self {
        Self {
            power_preference: wgpu::PowerPreference::LowPower,
            ..Default::default()
        }
    }

    /// Set the backends to use
    pub fn backends(mut self, backends: wgpu::Backends) -> Self {
        self.backends = backends;
        self
    }

    /// Set required features
    pub fn features(mut self, features: wgpu::Features) -> Self {
        self.features = features;
        self
    }

    /// Set device limits
    pub fn limits(mut self, limits: wgpu::Limits) -> Self {
        self.limits = limits;
        self
    }

    /// Force fallback adapter
    pub fn force_fallback_adapter(mut self, force: bool) -> Self {
        self.force_fallback_adapter = force;
        self
    }
}
