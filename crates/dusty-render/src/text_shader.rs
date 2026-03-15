//! WGSL shader for textured quads (text glyphs and images).
//!
//! Uses the same instanced-quad approach as the SDF shader but samples
//! from a texture instead of evaluating SDFs.

/// WGSL source for the textured quad shader.
///
/// Supports two modes controlled by `params.y`:
/// - `0.0` = text (alpha mask): samples the texture's red channel as alpha,
///   multiplied by the glyph color.
/// - `1.0` = image (RGBA): samples the texture directly, tinted by color.
pub const TEXT_SHADER_SOURCE: &str = r"
// ── Quad data ──────────────────────────────────────────────────────

struct TexturedQuad {
    // rect: x, y, w, h
    rect: vec4<f32>,
    // uv: u_min, v_min, u_max, v_max
    uv: vec4<f32>,
    // color: RGBA
    color: vec4<f32>,
    // clip_rect: x, y, w, h  (all zeros = no clip)
    clip_rect: vec4<f32>,
    // params: opacity, mode (0=text, 1=image), reserved, reserved
    params: vec4<f32>,
};

@group(0) @binding(0) var<storage, read> quads: array<TexturedQuad>;

struct Uniforms {
    viewport_size: vec2<f32>,
};
@group(0) @binding(1) var<uniform> uniforms: Uniforms;

@group(0) @binding(2) var atlas_texture: texture_2d<f32>;
@group(0) @binding(3) var atlas_sampler: sampler;

// ── Vertex shader ──────────────────────────────────────────────────

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) instance_id: u32,
    @location(2) world_pos: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    let quad = quads[instance_index];

    let quad_uv = array<vec2<f32>, 4>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 1.0),
    );
    let indices = array<u32, 6>(0u, 2u, 1u, 1u, 2u, 3u);
    let idx = indices[vertex_index];
    let local_uv = quad_uv[idx];

    // World position from rect
    let world_pos = vec2<f32>(
        quad.rect.x + local_uv.x * quad.rect.z,
        quad.rect.y + local_uv.y * quad.rect.w,
    );

    // Interpolate UV from atlas coordinates
    let tex_uv = vec2<f32>(
        mix(quad.uv.x, quad.uv.z, local_uv.x),
        mix(quad.uv.y, quad.uv.w, local_uv.y),
    );

    // Convert to NDC
    let ndc = vec2<f32>(
        world_pos.x / uniforms.viewport_size.x * 2.0 - 1.0,
        1.0 - world_pos.y / uniforms.viewport_size.y * 2.0,
    );

    var out: VertexOutput;
    out.position = vec4<f32>(ndc, 0.0, 1.0);
    out.uv = tex_uv;
    out.instance_id = instance_index;
    out.world_pos = world_pos;
    return out;
}

// ── Fragment shader ────────────────────────────────────────────────

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let quad = quads[in.instance_id];
    let opacity = quad.params.x;
    let mode = quad.params.y;

    let tex_color = textureSample(atlas_texture, atlas_sampler, in.uv);

    var color: vec4<f32>;

    if mode < 0.5 {
        // Text mode: texture red channel is alpha coverage
        let alpha = tex_color.r * quad.color.a * opacity;
        if alpha <= 0.001 {
            discard;
        }
        color = vec4<f32>(quad.color.rgb, alpha);
    } else {
        // Image mode: use texture RGBA, tinted by color
        color = tex_color * quad.color * vec4<f32>(1.0, 1.0, 1.0, opacity);
        if color.a <= 0.001 {
            discard;
        }
    }

    // Apply clip rect
    let clip = quad.clip_rect;
    if clip.z > 0.0 && clip.w > 0.0 {
        let inside_x = step(clip.x, in.world_pos.x) * step(in.world_pos.x, clip.x + clip.z);
        let inside_y = step(clip.y, in.world_pos.y) * step(in.world_pos.y, clip.y + clip.w);
        color = vec4<f32>(color.rgb, color.a * inside_x * inside_y);
        if color.a <= 0.0 {
            discard;
        }
    }

    return color;
}
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shader_source_is_nonempty() {
        assert!(!TEXT_SHADER_SOURCE.is_empty());
    }

    #[test]
    fn shader_contains_entry_points() {
        assert!(TEXT_SHADER_SOURCE.contains("fn vs_main"));
        assert!(TEXT_SHADER_SOURCE.contains("fn fs_main"));
    }

    #[test]
    fn shader_contains_textured_quad_struct() {
        assert!(TEXT_SHADER_SOURCE.contains("struct TexturedQuad"));
    }

    #[test]
    fn shader_contains_texture_bindings() {
        assert!(TEXT_SHADER_SOURCE.contains("atlas_texture"));
        assert!(TEXT_SHADER_SOURCE.contains("atlas_sampler"));
    }

    #[test]
    fn shader_handles_text_and_image_modes() {
        assert!(TEXT_SHADER_SOURCE.contains("mode < 0.5"));
    }
}
