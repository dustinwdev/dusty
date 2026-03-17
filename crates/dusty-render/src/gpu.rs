//! GPU context — wgpu instance, adapter, device, queue, and surface management.

use crate::error::{RenderError, Result};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

/// Wraps wgpu instance, adapter, device, queue, and surface.
///
/// Accepts any window type implementing the raw-window-handle traits,
/// keeping the render crate independent of winit.
pub struct GpuContext {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) surface_config: wgpu::SurfaceConfiguration,
    pub(crate) surface_format: wgpu::TextureFormat,
}

impl GpuContext {
    /// Creates a new GPU context from a window handle.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError::NoAdapter`] if no suitable GPU adapter is found.
    /// Returns [`RenderError::DeviceCreation`] if the device cannot be created.
    /// Returns [`RenderError::SurfaceConfig`] if the surface cannot be configured.
    pub async fn new(
        window: impl HasWindowHandle + HasDisplayHandle + Send + Sync + 'static,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance
            .create_surface(window)
            .map_err(|e| RenderError::SurfaceConfig(e.to_string()))?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(RenderError::NoAdapter)?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("dusty_device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    ..Default::default()
                },
                None,
            )
            .await
            .map_err(|e| RenderError::DeviceCreation(e.to_string()))?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .or_else(|| surface_caps.formats.first().copied())
            .ok_or_else(|| {
                RenderError::SurfaceConfig("no supported surface formats".to_string())
            })?;

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: width.max(1),
            height: height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes.first().copied().ok_or_else(|| {
                RenderError::SurfaceConfig("no supported alpha modes".to_string())
            })?,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        Ok(Self {
            device,
            queue,
            surface,
            surface_config,
            surface_format,
        })
    }

    /// Resizes the surface. Must be called when the window size changes.
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.surface_config.width = width;
            self.surface_config.height = height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }

    /// Returns the current surface texture for rendering.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError::SurfaceLost`] if the surface is lost and must be recreated.
    pub fn current_texture(&self) -> Result<wgpu::SurfaceTexture> {
        self.surface.get_current_texture().map_err(|e| match e {
            wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated => RenderError::SurfaceLost,
            other => RenderError::DrawError(other.to_string()),
        })
    }

    /// Returns the surface dimensions `(width, height)`.
    #[must_use]
    pub const fn size(&self) -> (u32, u32) {
        (self.surface_config.width, self.surface_config.height)
    }

    /// Returns the surface texture format.
    #[must_use]
    pub const fn format(&self) -> wgpu::TextureFormat {
        self.surface_format
    }

    /// Returns a reference to the wgpu device.
    #[must_use]
    pub const fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Returns a reference to the wgpu queue.
    #[must_use]
    pub const fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}

// No unit tests here — GPU tests are #[ignore] integration tests in tests/gpu_integration.rs
