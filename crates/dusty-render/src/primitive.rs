//! Draw primitives — the data types consumed by the GPU pipeline.

use bitflags::bitflags;
use dusty_style::Color;

/// A rectangle in physical pixel coordinates.
///
/// # Examples
///
/// ```
/// use dusty_render::Rect;
///
/// let r = Rect { x: 10.0, y: 20.0, width: 100.0, height: 50.0 };
/// assert_eq!(r.right(), 110.0);
/// assert_eq!(r.bottom(), 70.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    /// X position of the left edge in physical pixels.
    pub x: f32,
    /// Y position of the top edge in physical pixels.
    pub y: f32,
    /// Width in physical pixels.
    pub width: f32,
    /// Height in physical pixels.
    pub height: f32,
}

impl Rect {
    /// The right edge (`x + width`).
    #[must_use]
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    /// The bottom edge (`y + height`).
    #[must_use]
    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    /// Intersect two rectangles, returning the overlap or `None` if disjoint.
    #[must_use]
    pub fn intersect(&self, other: &Self) -> Option<Self> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());

        if right > x && bottom > y {
            Some(Self {
                x,
                y,
                width: right - x,
                height: bottom - y,
            })
        } else {
            None
        }
    }

    /// Expand or shrink the rect by `amount` on all sides.
    #[must_use]
    pub fn inflate(&self, amount: f32) -> Self {
        Self {
            x: self.x - amount,
            y: self.y - amount,
            width: amount.mul_add(2.0, self.width),
            height: amount.mul_add(2.0, self.height),
        }
    }

    /// Offset the rect by `(dx, dy)`.
    #[must_use]
    pub fn offset(&self, dx: f32, dy: f32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
            width: self.width,
            height: self.height,
        }
    }
}

bitflags! {
    /// Flags describing which features a draw primitive uses.
    ///
    /// The fragment shader uses these to skip unused code paths.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PrimitiveFlags: u32 {
        /// The primitive has rounded corners.
        const ROUNDED       = 0b0000_0001;
        /// The primitive has a border.
        const BORDERED      = 0b0000_0010;
        /// The primitive has a gradient fill.
        const GRADIENT      = 0b0000_0100;
        /// The primitive has a clip region with rounded corners.
        const CLIP_ROUNDED  = 0b0000_1000;
    }
}

/// Gradient data encoded for the GPU.
#[derive(Debug, Clone, PartialEq)]
pub struct GradientData {
    /// Angle in radians.
    pub angle_radians: f32,
    /// Color stops (color + position), up to [`MAX_GRADIENT_STOPS`].
    pub stops: Vec<(Color, f32)>,
}

/// Maximum number of gradient stops supported by the shader.
pub const MAX_GRADIENT_STOPS: usize = 8;

/// A drawable rectangle primitive with all visual properties.
///
/// All coordinates are in physical pixels.
#[derive(Debug, Clone, PartialEq)]
pub struct DrawPrimitive {
    /// Bounding rectangle.
    pub rect: Rect,
    /// Corner radii `[top_left, top_right, bottom_right, bottom_left]`.
    pub radii: [f32; 4],
    /// Fill color.
    pub fill_color: Color,
    /// Border color.
    pub border_color: Color,
    /// Border widths `[top, right, bottom, left]`.
    pub border_widths: [f32; 4],
    /// Element opacity (0.0–1.0).
    pub opacity: f32,
    /// Optional axis-aligned clip rectangle.
    pub clip_rect: Option<Rect>,
    /// Clip corner radii (used when `flags` includes `CLIP_ROUNDED`).
    pub clip_radii: [f32; 4],
    /// Feature flags.
    pub flags: PrimitiveFlags,
    /// Optional gradient data.
    pub gradient: Option<GradientData>,
}

/// A shadow primitive rendered behind (or inside) an element.
#[derive(Debug, Clone, PartialEq)]
pub struct ShadowPrimitive {
    /// The shadow rect (element rect adjusted for offset + spread).
    pub rect: Rect,
    /// Corner radii inherited from the element.
    pub radii: [f32; 4],
    /// Shadow color.
    pub color: Color,
    /// Blur radius in physical pixels.
    pub blur_radius: f32,
    /// Whether this is an inset shadow.
    pub inset: bool,
    /// Element opacity.
    pub opacity: f32,
    /// Optional axis-aligned clip rectangle.
    pub clip_rect: Option<Rect>,
}

/// Describes a clip region for nested clipping.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClipRegion {
    /// The clip rectangle.
    pub rect: Rect,
    /// Corner radii for rounded clipping.
    pub radii: [f32; 4],
}

/// A single glyph positioned for rendering.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextGlyph {
    /// X position in physical pixels.
    pub x: f32,
    /// Y position in physical pixels.
    pub y: f32,
    /// Glyph width in physical pixels.
    pub width: f32,
    /// Glyph height in physical pixels.
    pub height: f32,
    /// UV coordinates in the glyph atlas `[u_min, v_min, u_max, v_max]`.
    pub uv: [f32; 4],
    /// Text color (RGBA).
    pub color: [f32; 4],
    /// Element opacity.
    pub opacity: f32,
    /// Optional clip rectangle.
    pub clip_rect: Option<Rect>,
}

/// A text draw primitive containing positioned glyphs.
#[derive(Debug, Clone, PartialEq)]
pub struct TextPrimitive {
    /// The glyphs to render.
    pub glyphs: Vec<TextGlyph>,
}

/// Opaque identifier for a loaded image texture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ImageId(pub u64);

/// An image draw primitive.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImagePrimitive {
    /// Destination rectangle in physical pixels.
    pub rect: Rect,
    /// The image texture to sample.
    pub texture_id: ImageId,
    /// UV coordinates `[u_min, v_min, u_max, v_max]`.
    pub uv: [f32; 4],
    /// Element opacity.
    pub opacity: f32,
    /// Optional clip rectangle.
    pub clip_rect: Option<Rect>,
}

/// A draw command consumed by the renderer.
///
/// # Examples
///
/// ```
/// use dusty_render::{DrawCommand, ClipRegion, Rect};
///
/// let clip = DrawCommand::PushClip(ClipRegion {
///     rect: Rect { x: 0.0, y: 0.0, width: 100.0, height: 100.0 },
///     radii: [0.0; 4],
/// });
/// let pop = DrawCommand::PopClip;
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum DrawCommand {
    /// Draw a rectangle (filled, rounded, bordered, gradient).
    Rect(DrawPrimitive),
    /// Draw a shadow.
    Shadow(ShadowPrimitive),
    /// Draw text glyphs from a texture atlas.
    Text(TextPrimitive),
    /// Draw an image.
    Image(ImagePrimitive),
    /// Push a clip region onto the clip stack.
    PushClip(ClipRegion),
    /// Pop the most recent clip region.
    PopClip,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_right_and_bottom() {
        let r = Rect {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
        };
        assert_eq!(r.right(), 110.0);
        assert_eq!(r.bottom(), 70.0);
    }

    #[test]
    fn rect_intersect_overlapping() {
        let a = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let b = Rect {
            x: 50.0,
            y: 50.0,
            width: 100.0,
            height: 100.0,
        };
        let result = a.intersect(&b);
        assert_eq!(
            result,
            Some(Rect {
                x: 50.0,
                y: 50.0,
                width: 50.0,
                height: 50.0,
            })
        );
    }

    #[test]
    fn rect_intersect_disjoint() {
        let a = Rect {
            x: 0.0,
            y: 0.0,
            width: 50.0,
            height: 50.0,
        };
        let b = Rect {
            x: 100.0,
            y: 100.0,
            width: 50.0,
            height: 50.0,
        };
        assert_eq!(a.intersect(&b), None);
    }

    #[test]
    fn rect_intersect_contained() {
        let outer = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let inner = Rect {
            x: 25.0,
            y: 25.0,
            width: 50.0,
            height: 50.0,
        };
        assert_eq!(outer.intersect(&inner), Some(inner));
    }

    #[test]
    fn rect_inflate() {
        let r = Rect {
            x: 10.0,
            y: 10.0,
            width: 80.0,
            height: 60.0,
        };
        let inflated = r.inflate(5.0);
        assert_eq!(inflated.x, 5.0);
        assert_eq!(inflated.y, 5.0);
        assert_eq!(inflated.width, 90.0);
        assert_eq!(inflated.height, 70.0);
    }

    #[test]
    fn rect_offset() {
        let r = Rect {
            x: 10.0,
            y: 20.0,
            width: 30.0,
            height: 40.0,
        };
        let moved = r.offset(5.0, -10.0);
        assert_eq!(moved.x, 15.0);
        assert_eq!(moved.y, 10.0);
        assert_eq!(moved.width, 30.0);
        assert_eq!(moved.height, 40.0);
    }

    #[test]
    fn primitive_flags_combinations() {
        let flags = PrimitiveFlags::ROUNDED | PrimitiveFlags::BORDERED;
        assert!(flags.contains(PrimitiveFlags::ROUNDED));
        assert!(flags.contains(PrimitiveFlags::BORDERED));
        assert!(!flags.contains(PrimitiveFlags::GRADIENT));
    }

    #[test]
    fn primitive_flags_empty() {
        let flags = PrimitiveFlags::empty();
        assert!(!flags.contains(PrimitiveFlags::ROUNDED));
        assert!(!flags.contains(PrimitiveFlags::BORDERED));
    }

    #[test]
    fn draw_primitive_construction() {
        let prim = DrawPrimitive {
            rect: Rect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 50.0,
            },
            radii: [0.0; 4],
            fill_color: Color::WHITE,
            border_color: Color::TRANSPARENT,
            border_widths: [0.0; 4],
            opacity: 1.0,
            clip_rect: None,
            clip_radii: [0.0; 4],
            flags: PrimitiveFlags::empty(),
            gradient: None,
        };
        assert_eq!(prim.rect.width, 100.0);
        assert_eq!(prim.opacity, 1.0);
    }

    #[test]
    fn shadow_primitive_construction() {
        let shadow = ShadowPrimitive {
            rect: Rect {
                x: 0.0,
                y: 4.0,
                width: 100.0,
                height: 50.0,
            },
            radii: [8.0; 4],
            color: Color::rgba(0.0, 0.0, 0.0, 0.25),
            blur_radius: 10.0,
            inset: false,
            opacity: 1.0,
            clip_rect: None,
        };
        assert_eq!(shadow.blur_radius, 10.0);
        assert!(!shadow.inset);
    }

    #[test]
    fn clip_region_construction() {
        let clip = ClipRegion {
            rect: Rect {
                x: 10.0,
                y: 10.0,
                width: 80.0,
                height: 80.0,
            },
            radii: [4.0; 4],
        };
        assert_eq!(clip.rect.x, 10.0);
        assert_eq!(clip.radii[0], 4.0);
    }

    #[test]
    fn draw_command_variants() {
        let rect_cmd = DrawCommand::Rect(DrawPrimitive {
            rect: Rect {
                x: 0.0,
                y: 0.0,
                width: 50.0,
                height: 50.0,
            },
            radii: [0.0; 4],
            fill_color: Color::WHITE,
            border_color: Color::TRANSPARENT,
            border_widths: [0.0; 4],
            opacity: 1.0,
            clip_rect: None,
            clip_radii: [0.0; 4],
            flags: PrimitiveFlags::empty(),
            gradient: None,
        });
        assert!(matches!(rect_cmd, DrawCommand::Rect(_)));

        let shadow_cmd = DrawCommand::Shadow(ShadowPrimitive {
            rect: Rect {
                x: 0.0,
                y: 0.0,
                width: 50.0,
                height: 50.0,
            },
            radii: [0.0; 4],
            color: Color::BLACK,
            blur_radius: 5.0,
            inset: false,
            opacity: 1.0,
            clip_rect: None,
        });
        assert!(matches!(shadow_cmd, DrawCommand::Shadow(_)));

        let push = DrawCommand::PushClip(ClipRegion {
            rect: Rect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            },
            radii: [0.0; 4],
        });
        assert!(matches!(push, DrawCommand::PushClip(_)));

        assert!(matches!(DrawCommand::PopClip, DrawCommand::PopClip));
    }

    #[test]
    fn gradient_data_construction() {
        let gd = GradientData {
            angle_radians: std::f32::consts::FRAC_PI_2,
            stops: vec![(Color::WHITE, 0.0), (Color::BLACK, 1.0)],
        };
        assert_eq!(gd.stops.len(), 2);
        assert_eq!(gd.angle_radians, std::f32::consts::FRAC_PI_2);
    }

    #[test]
    fn max_gradient_stops_is_8() {
        assert_eq!(MAX_GRADIENT_STOPS, 8);
    }

    #[test]
    fn text_glyph_construction() {
        let glyph = TextGlyph {
            x: 10.0,
            y: 20.0,
            width: 8.0,
            height: 16.0,
            uv: [0.0, 0.0, 0.5, 0.5],
            color: [1.0, 1.0, 1.0, 1.0],
            opacity: 1.0,
            clip_rect: None,
        };
        assert_eq!(glyph.x, 10.0);
        assert_eq!(glyph.width, 8.0);
        assert_eq!(glyph.uv[2], 0.5);
    }

    #[test]
    fn text_primitive_construction() {
        let prim = TextPrimitive {
            glyphs: vec![TextGlyph {
                x: 0.0,
                y: 0.0,
                width: 8.0,
                height: 16.0,
                uv: [0.0; 4],
                color: [0.0, 0.0, 0.0, 1.0],
                opacity: 1.0,
                clip_rect: None,
            }],
        };
        assert_eq!(prim.glyphs.len(), 1);
    }

    #[test]
    fn image_id_equality() {
        let a = ImageId(1);
        let b = ImageId(1);
        let c = ImageId(2);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn image_primitive_construction() {
        let prim = ImagePrimitive {
            rect: Rect {
                x: 0.0,
                y: 0.0,
                width: 200.0,
                height: 100.0,
            },
            texture_id: ImageId(42),
            uv: [0.0, 0.0, 1.0, 1.0],
            opacity: 0.9,
            clip_rect: None,
        };
        assert_eq!(prim.texture_id, ImageId(42));
        assert_eq!(prim.opacity, 0.9);
    }

    #[test]
    fn draw_command_text_variant() {
        let cmd = DrawCommand::Text(TextPrimitive { glyphs: vec![] });
        assert!(matches!(cmd, DrawCommand::Text(_)));
    }

    #[test]
    fn draw_command_image_variant() {
        let cmd = DrawCommand::Image(ImagePrimitive {
            rect: Rect {
                x: 0.0,
                y: 0.0,
                width: 50.0,
                height: 50.0,
            },
            texture_id: ImageId(1),
            uv: [0.0, 0.0, 1.0, 1.0],
            opacity: 1.0,
            clip_rect: None,
        });
        assert!(matches!(cmd, DrawCommand::Image(_)));
    }
}
