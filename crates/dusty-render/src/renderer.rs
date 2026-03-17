//! Top-level renderer — owns GPU context and pipeline, drives frame rendering.

use crate::error::Result;
use crate::gpu::GpuContext;
use crate::pipeline::{GpuPrimitive, RenderPipeline, Uniforms};
use crate::primitive::{DrawCommand, PrimitiveFlags};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

/// Top-level renderer that owns the GPU context and render pipeline.
///
/// Converts [`DrawCommand`] slices into GPU-rendered frames.
pub struct Renderer {
    ctx: GpuContext,
    pipeline: RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    storage_buffer: wgpu::Buffer,
    storage_capacity: usize,
    bind_group: wgpu::BindGroup,
    bind_group_valid: bool,
}

impl Renderer {
    /// Creates a new renderer attached to the given window.
    ///
    /// # Errors
    ///
    /// Returns a [`RenderError`] if GPU initialization fails.
    pub async fn new(
        window: impl HasWindowHandle + HasDisplayHandle + Send + Sync + 'static,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        let ctx = GpuContext::new(window, width, height).await?;
        let pipeline = RenderPipeline::new(&ctx);

        let uniform_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("dusty_uniform_buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let initial_storage_capacity = 256;
        let storage_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("dusty_primitive_buffer"),
            size: (initial_storage_capacity * std::mem::size_of::<GpuPrimitive>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("dusty_bind_group"),
            layout: &pipeline.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: storage_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
        });

        Ok(Self {
            ctx,
            pipeline,
            uniform_buffer,
            storage_buffer,
            storage_capacity: initial_storage_capacity,
            bind_group,
            bind_group_valid: true,
        })
    }

    /// Renders a frame from the given draw commands.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError::SurfaceLost`] if the surface needs recreation.
    /// Returns [`RenderError::DrawError`] on other rendering failures.
    pub fn render(&mut self, commands: &[DrawCommand]) -> Result<()> {
        let output = self.ctx.current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let gpu_primitives = encode_commands(commands);

        if gpu_primitives.is_empty() {
            self.render_clear_pass(&view);
            self.ctx.queue.submit([]); // flush
            output.present();
            return Ok(());
        }

        self.render_primitives_pass(&view, &gpu_primitives);
        output.present();

        Ok(())
    }

    /// Resizes the rendering surface. Call when the window size changes.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.ctx.resize(width, height);
    }

    /// Returns the surface dimensions `(width, height)`.
    #[must_use]
    pub const fn size(&self) -> (u32, u32) {
        self.ctx.size()
    }

    /// Returns a reference to the underlying GPU context.
    #[must_use]
    pub const fn gpu_context(&self) -> &GpuContext {
        &self.ctx
    }

    fn render_clear_pass(&self, view: &wgpu::TextureView) {
        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("dusty_clear_encoder"),
            });
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("dusty_clear_pass"),
                color_attachments: &[Some(Self::clear_attachment(view))],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }
        self.ctx.queue.submit(std::iter::once(encoder.finish()));
    }

    fn render_primitives_pass(
        &mut self,
        view: &wgpu::TextureView,
        gpu_primitives: &[GpuPrimitive],
    ) {
        let (width, height) = self.ctx.size();

        #[allow(clippy::cast_precision_loss)]
        let uniforms = Uniforms {
            viewport_size: [width as f32, height as f32],
            _padding: [0.0; 2],
        };
        self.ctx
            .queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        let primitive_data: &[u8] = bytemuck::cast_slice(gpu_primitives);

        if gpu_primitives.len() > self.storage_capacity {
            // Geometric growth for storage buffer
            self.storage_capacity = gpu_primitives.len().next_power_of_two();
            self.storage_buffer = self.ctx.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("dusty_primitive_buffer"),
                size: (self.storage_capacity * std::mem::size_of::<GpuPrimitive>()) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.bind_group_valid = false;
        }

        self.ctx
            .queue
            .write_buffer(&self.storage_buffer, 0, primitive_data);

        if !self.bind_group_valid {
            self.bind_group = self
                .ctx
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("dusty_bind_group"),
                    layout: &self.pipeline.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: self.storage_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: self.uniform_buffer.as_entire_binding(),
                        },
                    ],
                });
            self.bind_group_valid = true;
        }

        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("dusty_render_encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("dusty_render_pass"),
                color_attachments: &[Some(Self::clear_attachment(view))],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&self.pipeline.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);

            #[allow(clippy::cast_possible_truncation)]
            let instance_count = gpu_primitives.len() as u32;
            pass.draw(0..6, 0..instance_count);
        }

        self.ctx.queue.submit(std::iter::once(encoder.finish()));
    }

    const fn clear_attachment(view: &wgpu::TextureView) -> wgpu::RenderPassColorAttachment<'_> {
        wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                }),
                store: wgpu::StoreOp::Store,
            },
        }
    }
}

/// Converts draw commands into GPU primitive data.
fn encode_commands(commands: &[DrawCommand]) -> Vec<GpuPrimitive> {
    let mut primitives = Vec::new();

    for cmd in commands {
        match cmd {
            DrawCommand::Rect(prim) => {
                primitives.push(encode_rect(prim));
            }
            DrawCommand::Shadow(shadow) => {
                primitives.push(encode_shadow(shadow));
            }
            // Text/Image: handled by the textured pipeline, not the SDF pipeline.
            // PushClip/PopClip: handled by the CommandEncoder's clip stack.
            DrawCommand::Text(_)
            | DrawCommand::Image(_)
            | DrawCommand::PushClip(_)
            | DrawCommand::PopClip => {}
        }
    }

    primitives
}

fn encode_rect(prim: &crate::primitive::DrawPrimitive) -> GpuPrimitive {
    let mut gpu = GpuPrimitive::zeroed();
    gpu.rect = [prim.rect.x, prim.rect.y, prim.rect.width, prim.rect.height];
    gpu.radii = prim.radii;
    gpu.fill_color = [
        prim.fill_color.r,
        prim.fill_color.g,
        prim.fill_color.b,
        prim.fill_color.a,
    ];
    gpu.border_color = [
        prim.border_color.r,
        prim.border_color.g,
        prim.border_color.b,
        prim.border_color.a,
    ];
    gpu.border_widths = prim.border_widths;
    if let Some(clip) = &prim.clip_rect {
        gpu.clip_rect = [clip.x, clip.y, clip.width, clip.height];
    }
    gpu.clip_radii = prim.clip_radii;

    #[allow(clippy::cast_precision_loss)]
    let flags_f32 = prim.flags.bits() as f32;
    gpu.params = [prim.opacity, flags_f32, 0.0, 0.0];

    if let Some(gd) = &prim.gradient {
        #[allow(clippy::cast_precision_loss)]
        let stop_count = gd.stops.len() as f32;
        gpu.gradient_params = [gd.angle_radians, stop_count, 0.0, 0.0];

        for (i, (color, pos)) in gd.stops.iter().enumerate().take(8) {
            let color_arr = [color.r, color.g, color.b, color.a];
            if i < 4 {
                gpu.gradient_stop_colors_01[i] = color_arr;
                gpu.gradient_stop_positions[i] = *pos;
            } else {
                gpu.gradient_stop_colors_45[i - 4] = color_arr;
                gpu.gradient_stop_positions_hi[i - 4] = *pos;
            }
        }
    }

    gpu
}

fn encode_shadow(shadow: &crate::primitive::ShadowPrimitive) -> GpuPrimitive {
    let mut gpu = GpuPrimitive::zeroed();
    gpu.rect = [
        shadow.rect.x,
        shadow.rect.y,
        shadow.rect.width,
        shadow.rect.height,
    ];
    gpu.radii = shadow.radii;
    gpu.fill_color = [
        shadow.color.r,
        shadow.color.g,
        shadow.color.b,
        shadow.color.a,
    ];
    if let Some(clip) = &shadow.clip_rect {
        gpu.clip_rect = [clip.x, clip.y, clip.width, clip.height];
    }

    let mut flags = PrimitiveFlags::empty();
    if shadow.radii.iter().any(|&r| r > 0.0) {
        flags |= PrimitiveFlags::ROUNDED;
    }
    #[allow(clippy::cast_precision_loss)]
    let flags_f32 = flags.bits() as f32;
    gpu.params = [shadow.opacity, flags_f32, shadow.blur_radius, 1.0];

    gpu
}

impl GpuPrimitive {
    fn zeroed() -> Self {
        bytemuck::Zeroable::zeroed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::{DrawPrimitive, Rect, ShadowPrimitive};
    use dusty_style::Color;

    #[test]
    fn encode_empty_commands() {
        let gpu_prims = encode_commands(&[]);
        assert!(gpu_prims.is_empty());
    }

    #[test]
    fn encode_rect_command() {
        let cmd = DrawCommand::Rect(DrawPrimitive {
            rect: Rect {
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
            },
            radii: [4.0; 4],
            fill_color: Color::WHITE,
            border_color: Color::TRANSPARENT,
            border_widths: [0.0; 4],
            opacity: 0.8,
            clip_rect: None,
            clip_radii: [0.0; 4],
            flags: PrimitiveFlags::ROUNDED,
            gradient: None,
        });
        let gpu_prims = encode_commands(&[cmd]);
        assert_eq!(gpu_prims.len(), 1);
        assert_eq!(gpu_prims[0].rect, [10.0, 20.0, 100.0, 50.0]);
        assert_eq!(gpu_prims[0].radii, [4.0; 4]);
        assert_eq!(gpu_prims[0].params[0], 0.8); // opacity
    }

    #[test]
    fn encode_shadow_command() {
        let cmd = DrawCommand::Shadow(ShadowPrimitive {
            rect: Rect {
                x: 5.0,
                y: 5.0,
                width: 90.0,
                height: 40.0,
            },
            radii: [8.0; 4],
            color: Color::rgba(0.0, 0.0, 0.0, 0.5),
            blur_radius: 10.0,
            inset: false,
            opacity: 1.0,
            clip_rect: None,
        });
        let gpu_prims = encode_commands(&[cmd]);
        assert_eq!(gpu_prims.len(), 1);
        assert_eq!(gpu_prims[0].params[2], 10.0); // blur_radius
        assert_eq!(gpu_prims[0].params[3], 1.0); // is_shadow
    }

    #[test]
    fn encode_push_pop_clip_no_gpu_prims() {
        let cmds = vec![
            DrawCommand::PushClip(crate::primitive::ClipRegion {
                rect: Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 100.0,
                    height: 100.0,
                },
                radii: [0.0; 4],
            }),
            DrawCommand::PopClip,
        ];
        let gpu_prims = encode_commands(&cmds);
        assert!(gpu_prims.is_empty());
    }

    #[test]
    fn gpu_primitive_is_304_bytes() {
        assert_eq!(
            std::mem::size_of::<GpuPrimitive>(),
            304,
            "GpuPrimitive must be 304 bytes to match WGSL layout"
        );
    }

    #[test]
    fn uniforms_is_16_bytes() {
        assert_eq!(
            std::mem::size_of::<Uniforms>(),
            16,
            "Uniforms must be 16 bytes (vec2 + padding)"
        );
    }
}
