//! Scene graph and GPU rendering for Dusty.
//!
//! Provides a draw-command abstraction over wgpu, using a single SDF
//! uber-shader for filled rects, rounded rects, bordered rects, shadows,
//! gradients, and clipping, plus a textured pipeline for text and images.
//!
//! # Architecture
//!
//! The rendering pipeline has three layers:
//!
//! 1. **Command encoding** ([`CommandEncoder`]) — converts `Style` + `Rect`
//!    into [`DrawCommand`]s (plain data, testable without a GPU).
//! 2. **GPU context** ([`GpuContext`]) — wgpu device, queue, surface management.
//! 3. **Renderer** ([`Renderer`]) — uploads primitives and records render passes.

pub mod atlas;
mod clip;
mod command;
mod error;
pub mod glyph_cache;
mod gpu;
pub mod image_cache;
mod pipeline;
pub(crate) mod primitive;
mod renderer;
mod shader;
pub mod text_pipeline;
pub mod text_shader;
pub mod tree;

pub use clip::ClipStack;
pub use command::CommandEncoder;
pub use error::{RenderError, Result};
pub use gpu::GpuContext;
pub use primitive::{
    ClipRegion, DrawCommand, DrawPrimitive, GradientData, ImageId, ImagePrimitive, PrimitiveFlags,
    Rect, ShadowPrimitive, TextGlyph, TextPrimitive, MAX_GRADIENT_STOPS,
};
pub use renderer::Renderer;
pub use shader::SHADER_SOURCE;
