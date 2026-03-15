//! Render pipeline — shader compilation, bind group layouts, and pipeline creation.

use crate::gpu::GpuContext;
use crate::shader::SHADER_SOURCE;

/// GPU data for a single primitive, laid out for the storage buffer.
///
/// Must match the `Primitive` struct in the WGSL shader exactly.
/// Total size: 304 bytes (9 × vec4 + 2 × mat4x4 + 2 × vec4 = 19 × 16).
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck_derive::Pod, bytemuck_derive::Zeroable)]
pub struct GpuPrimitive {
    pub rect: [f32; 4],
    pub radii: [f32; 4],
    pub fill_color: [f32; 4],
    pub border_color: [f32; 4],
    pub border_widths: [f32; 4],
    pub clip_rect: [f32; 4],
    pub clip_radii: [f32; 4],
    pub params: [f32; 4],
    pub gradient_params: [f32; 4],
    pub gradient_stop_colors_01: [[f32; 4]; 4],
    pub gradient_stop_colors_45: [[f32; 4]; 4],
    pub gradient_stop_positions: [f32; 4],
    pub gradient_stop_positions_hi: [f32; 4],
}

/// Uniform data passed to the shader.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck_derive::Pod, bytemuck_derive::Zeroable)]
pub struct Uniforms {
    pub viewport_size: [f32; 2],
    pub _padding: [f32; 2],
}

/// The compiled GPU render pipeline and associated resources.
pub struct RenderPipeline {
    pub(crate) pipeline: wgpu::RenderPipeline,
    pub(crate) bind_group_layout: wgpu::BindGroupLayout,
}

impl RenderPipeline {
    /// Creates the render pipeline from the uber-shader.
    pub fn new(ctx: &GpuContext) -> Self {
        let shader_module = ctx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("dusty_sdf_shader"),
                source: wgpu::ShaderSource::Wgsl(SHADER_SOURCE.into()),
            });

        let bind_group_layout =
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("dusty_bind_group_layout"),
                    entries: &[
                        // Storage buffer for primitives
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
                        // Uniform buffer
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        let pipeline_layout = ctx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("dusty_pipeline_layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let pipeline = ctx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("dusty_render_pipeline"),
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
