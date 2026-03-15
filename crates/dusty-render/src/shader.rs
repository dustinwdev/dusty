//! WGSL shader source for the SDF uber-shader.
//!
//! A single vertex/fragment shader pair handles all primitives using
//! signed distance functions. The vertex shader generates full-screen
//! quads from instance data; the fragment shader evaluates SDFs for
//! rounded rects, borders, shadows, gradients, and clipping.

/// WGSL source for the SDF uber-shader.
///
/// The shader uses storage buffers for primitive data and renders
/// instanced quads. Each primitive is a struct containing all the
/// data needed to render filled, rounded, bordered, shadowed, and
/// gradient rects with anti-aliasing via smoothstep on the SDF.
pub const SHADER_SOURCE: &str = r"
// ── Primitive data ──────────────────────────────────────────────────

struct Primitive {
    // rect: x, y, w, h
    rect: vec4<f32>,
    // corner radii: tl, tr, br, bl
    radii: vec4<f32>,
    // fill color (RGBA, premultiplied)
    fill_color: vec4<f32>,
    // border color (RGBA)
    border_color: vec4<f32>,
    // border widths: top, right, bottom, left
    border_widths: vec4<f32>,
    // clip rect: x, y, w, h  (all zeros = no clip)
    clip_rect: vec4<f32>,
    // clip radii: tl, tr, br, bl
    clip_radii: vec4<f32>,
    // x: opacity, y: flags, z: shadow_blur, w: is_shadow (0 or 1)
    params: vec4<f32>,
    // gradient: x = angle (radians), y = stop_count, z/w = reserved
    gradient_params: vec4<f32>,
    // gradient stops: up to 8 (color + position packed)
    gradient_stop_colors_01: mat4x4<f32>,  // stops 0-3 colors (each row = rgba)
    gradient_stop_colors_45: mat4x4<f32>,  // stops 4-7 colors
    gradient_stop_positions: vec4<f32>,     // positions 0-3
    gradient_stop_positions_hi: vec4<f32>,  // positions 4-7
};

// Flags (matches PrimitiveFlags in Rust)
const FLAG_ROUNDED: u32      = 1u;
const FLAG_BORDERED: u32     = 2u;
const FLAG_GRADIENT: u32     = 4u;
const FLAG_CLIP_ROUNDED: u32 = 8u;

@group(0) @binding(0) var<storage, read> primitives: array<Primitive>;

struct Uniforms {
    viewport_size: vec2<f32>,
};
@group(0) @binding(1) var<uniform> uniforms: Uniforms;

// ── Vertex shader ───────────────────────────────────────────────────

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
    let prim = primitives[instance_index];

    // Expand rect for shadows (blur radius extends beyond the rect)
    let blur_expand = prim.params.z * 3.0;
    let rect_x = prim.rect.x - blur_expand;
    let rect_y = prim.rect.y - blur_expand;
    let rect_w = prim.rect.z + blur_expand * 2.0;
    let rect_h = prim.rect.w + blur_expand * 2.0;

    // Quad vertices: 0=TL, 1=TR, 2=BL, 3=BR (triangle strip: 0,2,1,3)
    let quad_uv = array<vec2<f32>, 4>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 1.0),
    );
    // Triangle strip order: 0, 2, 1, 3
    let indices = array<u32, 6>(0u, 2u, 1u, 1u, 2u, 3u);
    let idx = indices[vertex_index];
    let uv = quad_uv[idx];

    let world_pos = vec2<f32>(
        rect_x + uv.x * rect_w,
        rect_y + uv.y * rect_h,
    );

    // Convert to NDC: (0,0) top-left, (w,h) bottom-right → (-1,1) to (1,-1)
    let ndc = vec2<f32>(
        world_pos.x / uniforms.viewport_size.x * 2.0 - 1.0,
        1.0 - world_pos.y / uniforms.viewport_size.y * 2.0,
    );

    var out: VertexOutput;
    out.position = vec4<f32>(ndc, 0.0, 1.0);
    out.uv = uv;
    out.instance_id = instance_index;
    out.world_pos = world_pos;
    return out;
}

// ── SDF helpers ─────────────────────────────────────────────────────

// SDF for an axis-aligned rounded box centered at the origin.
fn sdf_rounded_box(p: vec2<f32>, half_size: vec2<f32>, radii: vec4<f32>) -> f32 {
    // Select radius based on quadrant
    var r: f32;
    if p.x > 0.0 {
        if p.y > 0.0 {
            r = radii.z;  // bottom-right
        } else {
            r = radii.y;  // top-right
        }
    } else {
        if p.y > 0.0 {
            r = radii.w;  // bottom-left
        } else {
            r = radii.x;  // top-left
        }
    }
    r = min(r, min(half_size.x, half_size.y));

    let q = abs(p) - half_size + vec2<f32>(r, r);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - r;
}

// Approximate Gaussian blur for shadow via SDF
fn shadow_alpha(dist: f32, blur_radius: f32) -> f32 {
    if blur_radius <= 0.0 {
        return select(0.0, 1.0, dist <= 0.0);
    }
    let sigma = blur_radius / 2.0;
    return 1.0 - smoothstep(-sigma * 1.5, sigma * 1.5, dist);
}

// ── Fragment shader ─────────────────────────────────────────────────

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let prim = primitives[in.instance_id];

    let rect_pos = prim.rect.xy;
    let rect_size = prim.rect.zw;
    let half_size = rect_size * 0.5;
    let center = rect_pos + half_size;
    let local_p = in.world_pos - center;

    let opacity = prim.params.x;
    let flags = u32(prim.params.y);
    let blur_radius = prim.params.z;
    let is_shadow = prim.params.w > 0.5;

    // Determine radii
    var radii = vec4<f32>(0.0);
    if (flags & FLAG_ROUNDED) != 0u {
        radii = prim.radii;
    }

    // Compute SDF distance
    let dist = sdf_rounded_box(local_p, half_size, radii);

    // ── Shadow path ─────────────────────────────────────────────
    if is_shadow {
        let alpha = shadow_alpha(dist, blur_radius) * prim.fill_color.a * opacity;
        if alpha <= 0.0 {
            discard;
        }

        var final_color = vec4<f32>(prim.fill_color.rgb, alpha);

        // Apply clip
        final_color = apply_clip(final_color, in.world_pos, prim, flags);

        return final_color;
    }

    // ── Normal rect path ────────────────────────────────────────

    // Anti-aliased edge
    let aa = fwidth(dist);
    let shape_alpha = 1.0 - smoothstep(-aa, aa, dist);

    if shape_alpha <= 0.0 {
        discard;
    }

    // Determine fill color
    var color: vec4<f32>;
    if (flags & FLAG_GRADIENT) != 0u {
        color = sample_gradient(in.world_pos, prim);
    } else {
        color = prim.fill_color;
    }

    // Border
    if (flags & FLAG_BORDERED) != 0u {
        // Inset distance for border
        let border_inset = vec2<f32>(
            select(prim.border_widths.w, prim.border_widths.y, local_p.x > 0.0),
            select(prim.border_widths.x, prim.border_widths.z, local_p.y > 0.0),
        );
        let inner_half = half_size - border_inset;
        let inner_radii = max(radii - vec4<f32>(border_inset.y, border_inset.x, border_inset.y, border_inset.x), vec4<f32>(0.0));
        let inner_dist = sdf_rounded_box(local_p, inner_half, inner_radii);
        let inner_alpha = 1.0 - smoothstep(-aa, aa, inner_dist);

        // Mix: border where inner_alpha < 1, fill where inner_alpha = 1
        let border_factor = 1.0 - inner_alpha;
        color = mix(color, prim.border_color, border_factor);
    }

    color = vec4<f32>(color.rgb, color.a * shape_alpha * opacity);

    // Apply clip
    color = apply_clip(color, in.world_pos, prim, flags);

    if color.a <= 0.0 {
        discard;
    }

    return color;
}

// ── Gradient sampling ───────────────────────────────────────────────

fn sample_gradient(world_pos: vec2<f32>, prim: Primitive) -> vec4<f32> {
    let rect_pos = prim.rect.xy;
    let rect_size = prim.rect.zw;

    let angle = prim.gradient_params.x;
    let stop_count = u32(prim.gradient_params.y);

    // Compute gradient coordinate (0..1 along the gradient axis)
    let center = rect_pos + rect_size * 0.5;
    let dir = vec2<f32>(sin(angle), -cos(angle));

    // Project point onto gradient axis
    let relative = world_pos - center;
    let proj_len = dot(relative, dir);
    let half_len = abs(dot(rect_size * 0.5, abs(dir)));
    var t: f32;
    if half_len > 0.0 {
        t = (proj_len / half_len) * 0.5 + 0.5;
    } else {
        t = 0.5;
    }
    t = clamp(t, 0.0, 1.0);

    // Sample color stops
    return sample_stops(t, prim, stop_count);
}

fn get_stop_color(prim: Primitive, index: u32) -> vec4<f32> {
    switch index {
        case 0u: { return prim.gradient_stop_colors_01[0]; }
        case 1u: { return prim.gradient_stop_colors_01[1]; }
        case 2u: { return prim.gradient_stop_colors_01[2]; }
        case 3u: { return prim.gradient_stop_colors_01[3]; }
        case 4u: { return prim.gradient_stop_colors_45[0]; }
        case 5u: { return prim.gradient_stop_colors_45[1]; }
        case 6u: { return prim.gradient_stop_colors_45[2]; }
        case 7u: { return prim.gradient_stop_colors_45[3]; }
        default: { return vec4<f32>(0.0); }
    }
}

fn get_stop_position(prim: Primitive, index: u32) -> f32 {
    switch index {
        case 0u: { return prim.gradient_stop_positions.x; }
        case 1u: { return prim.gradient_stop_positions.y; }
        case 2u: { return prim.gradient_stop_positions.z; }
        case 3u: { return prim.gradient_stop_positions.w; }
        case 4u: { return prim.gradient_stop_positions_hi.x; }
        case 5u: { return prim.gradient_stop_positions_hi.y; }
        case 6u: { return prim.gradient_stop_positions_hi.z; }
        case 7u: { return prim.gradient_stop_positions_hi.w; }
        default: { return 0.0; }
    }
}

fn sample_stops(t: f32, prim: Primitive, stop_count: u32) -> vec4<f32> {
    if stop_count == 0u {
        return prim.fill_color;
    }
    if stop_count == 1u {
        return get_stop_color(prim, 0u);
    }

    // Find the two stops to interpolate between
    if t <= get_stop_position(prim, 0u) {
        return get_stop_color(prim, 0u);
    }
    let last = stop_count - 1u;
    if t >= get_stop_position(prim, last) {
        return get_stop_color(prim, last);
    }

    for (var i: u32 = 0u; i < last; i = i + 1u) {
        let p0 = get_stop_position(prim, i);
        let p1 = get_stop_position(prim, i + 1u);
        if t >= p0 && t <= p1 {
            let frac = (t - p0) / max(p1 - p0, 0.0001);
            return mix(get_stop_color(prim, i), get_stop_color(prim, i + 1u), frac);
        }
    }
    return get_stop_color(prim, last);
}

// ── Clipping ────────────────────────────────────────────────────────

fn apply_clip(color: vec4<f32>, world_pos: vec2<f32>, prim: Primitive, flags: u32) -> vec4<f32> {
    let clip_rect = prim.clip_rect;
    // If clip_rect size is zero, no clipping
    if clip_rect.z <= 0.0 || clip_rect.w <= 0.0 {
        return color;
    }

    if (flags & FLAG_CLIP_ROUNDED) != 0u {
        // Rounded clip via SDF
        let clip_half = clip_rect.zw * 0.5;
        let clip_center = clip_rect.xy + clip_half;
        let clip_p = world_pos - clip_center;
        let clip_dist = sdf_rounded_box(clip_p, clip_half, prim.clip_radii);
        let clip_aa = fwidth(clip_dist);
        let clip_alpha = 1.0 - smoothstep(-clip_aa, clip_aa, clip_dist);
        return vec4<f32>(color.rgb, color.a * clip_alpha);
    } else {
        // Axis-aligned clip (sharp)
        let inside_x = step(clip_rect.x, world_pos.x) * step(world_pos.x, clip_rect.x + clip_rect.z);
        let inside_y = step(clip_rect.y, world_pos.y) * step(world_pos.y, clip_rect.y + clip_rect.w);
        return vec4<f32>(color.rgb, color.a * inside_x * inside_y);
    }
}
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shader_source_is_nonempty() {
        assert!(!SHADER_SOURCE.is_empty());
    }

    #[test]
    fn shader_contains_entry_points() {
        assert!(SHADER_SOURCE.contains("fn vs_main"));
        assert!(SHADER_SOURCE.contains("fn fs_main"));
    }

    #[test]
    fn shader_contains_sdf_function() {
        assert!(SHADER_SOURCE.contains("fn sdf_rounded_box"));
    }

    #[test]
    fn shader_contains_gradient_sampling() {
        assert!(SHADER_SOURCE.contains("fn sample_gradient"));
        assert!(SHADER_SOURCE.contains("fn sample_stops"));
    }

    #[test]
    fn shader_contains_clip_function() {
        assert!(SHADER_SOURCE.contains("fn apply_clip"));
    }

    #[test]
    fn shader_contains_flag_constants() {
        assert!(SHADER_SOURCE.contains("FLAG_ROUNDED"));
        assert!(SHADER_SOURCE.contains("FLAG_BORDERED"));
        assert!(SHADER_SOURCE.contains("FLAG_GRADIENT"));
        assert!(SHADER_SOURCE.contains("FLAG_CLIP_ROUNDED"));
    }
}
