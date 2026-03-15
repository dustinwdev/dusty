//! Text/image pipeline — GPU pipeline for textured quads.
//!
//! Separate from the SDF pipeline, this handles text glyphs (alpha-masked)
//! and images (RGBA) using a shared bind group layout with a texture sampler.

use crate::gpu::GpuContext;
use crate::text_shader::TEXT_SHADER_SOURCE;

/// GPU data for a single textured quad instance.
///
/// Must match the `TexturedQuad` struct in the WGSL shader exactly.
/// Total size: 80 bytes (5 × vec4<f32>).
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck_derive::Pod, bytemuck_derive::Zeroable)]
pub struct TexturedQuad {
    /// Destination rect `[x, y, width, height]`.
    pub rect: [f32; 4],
    /// UV coordinates `[u_min, v_min, u_max, v_max]`.
    pub uv: [f32; 4],
    /// Color (RGBA). For text: text color; for images: tint.
    pub color: [f32; 4],
    /// Clip rect `[x, y, width, height]`. All zeros = no clip.
    pub clip_rect: [f32; 4],
    /// Extra params: `[opacity, mode (0=text, 1=image), 0, 0]`.
    pub params: [f32; 4],
}

/// Compiled GPU pipeline for rendering textured quads.
///
/// Fields are `pub(crate)` — not yet read because the text/image render pass
/// integration is pending (Phase 22). The struct and constructor are ready so
/// downstream code can instantiate it when the pass is wired up.
#[allow(dead_code)]
pub struct TextPipeline {
    pub(crate) pipeline: wgpu::RenderPipeline,
    pub(crate) bind_group_layout: wgpu::BindGroupLayout,
}

impl TextPipeline {
    /// Creates the text/image render pipeline.
    pub fn new(ctx: &GpuContext) -> Self {
        let shader_module = ctx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("dusty_text_shader"),
                source: wgpu::ShaderSource::Wgsl(TEXT_SHADER_SOURCE.into()),
            });

        let bind_group_layout =
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("dusty_text_bind_group_layout"),
                    entries: &[
                        // Storage buffer for quads
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        // Uniform buffer (viewport)
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        // Texture
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        // Sampler
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

        let pipeline_layout = ctx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("dusty_text_pipeline_layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let pipeline = ctx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("dusty_text_render_pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_module,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader_module,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: ctx.surface_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        Self {
            pipeline,
            bind_group_layout,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn textured_quad_is_80_bytes() {
        assert_eq!(
            std::mem::size_of::<TexturedQuad>(),
            80,
            "TexturedQuad must be 80 bytes to match WGSL layout"
        );
    }

    #[test]
    fn textured_quad_zeroed() {
        let quad: TexturedQuad = bytemuck::Zeroable::zeroed();
        assert_eq!(quad.rect, [0.0; 4]);
        assert_eq!(quad.params, [0.0; 4]);
    }
}
