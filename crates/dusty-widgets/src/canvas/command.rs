//! Canvas drawing commands and supporting types.

use dusty_style::Color;

/// A 2D point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    /// X coordinate.
    pub x: f32,
    /// Y coordinate.
    pub y: f32,
}

impl Point {
    /// Creates a new point.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Fill style for painting shapes.
#[derive(Debug, Clone, PartialEq)]
pub enum FillStyle {
    /// Solid color fill.
    Solid(Color),
}

/// Stroke style for outlining shapes.
#[derive(Debug, Clone, PartialEq)]
pub struct StrokeStyle {
    /// Stroke color.
    pub color: Color,
    /// Stroke width in pixels.
    pub width: f32,
}

/// A 2D affine transform.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    /// The 3x2 matrix values `[a, b, c, d, e, f]`.
    ///
    /// Represents:
    /// ```text
    /// | a c e |
    /// | b d f |
    /// | 0 0 1 |
    /// ```
    pub matrix: [f32; 6],
}

impl Transform {
    /// Identity transform.
    pub const IDENTITY: Self = Self {
        matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    };

    /// Creates a translation transform.
    #[must_use]
    pub const fn translate(x: f32, y: f32) -> Self {
        Self {
            matrix: [1.0, 0.0, 0.0, 1.0, x, y],
        }
    }

    /// Creates a uniform scale transform.
    #[must_use]
    pub const fn scale(sx: f32, sy: f32) -> Self {
        Self {
            matrix: [sx, 0.0, 0.0, sy, 0.0, 0.0],
        }
    }

    /// Creates a rotation transform (angle in radians).
    #[must_use]
    pub fn rotate(angle: f32) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();
        Self {
            matrix: [cos, sin, -sin, cos, 0.0, 0.0],
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

/// A recorded canvas drawing command.
#[derive(Debug, Clone, PartialEq)]
pub enum CanvasCommand {
    /// Move the pen to a point.
    MoveTo(Point),
    /// Draw a line to a point.
    LineTo(Point),
    /// Draw a quadratic bezier curve.
    QuadTo {
        /// Control point.
        control: Point,
        /// End point.
        to: Point,
    },
    /// Draw a cubic bezier curve.
    CubicTo {
        /// First control point.
        c1: Point,
        /// Second control point.
        c2: Point,
        /// End point.
        to: Point,
    },
    /// Close the current path.
    ClosePath,
    /// Fill the current path.
    Fill(FillStyle),
    /// Stroke the current path.
    Stroke(StrokeStyle),
    /// Draw a filled/stroked rectangle.
    Rect {
        /// X position.
        x: f32,
        /// Y position.
        y: f32,
        /// Width.
        width: f32,
        /// Height.
        height: f32,
        /// Fill style.
        fill: Option<FillStyle>,
        /// Stroke style.
        stroke: Option<StrokeStyle>,
    },
    /// Draw a filled/stroked rounded rectangle.
    RoundRect {
        /// X position.
        x: f32,
        /// Y position.
        y: f32,
        /// Width.
        width: f32,
        /// Height.
        height: f32,
        /// Corner radius.
        radius: f32,
        /// Fill style.
        fill: Option<FillStyle>,
        /// Stroke style.
        stroke: Option<StrokeStyle>,
    },
    /// Draw a filled/stroked circle.
    Circle {
        /// Center X.
        cx: f32,
        /// Center Y.
        cy: f32,
        /// Radius.
        radius: f32,
        /// Fill style.
        fill: Option<FillStyle>,
        /// Stroke style.
        stroke: Option<StrokeStyle>,
    },
    /// Draw text at a position.
    Text {
        /// Text content.
        content: String,
        /// X position.
        x: f32,
        /// Y position.
        y: f32,
        /// Fill style.
        fill: FillStyle,
    },
    /// Draw an image at a position.
    Image {
        /// Image source path.
        src: String,
        /// X position.
        x: f32,
        /// Y position.
        y: f32,
        /// Width.
        width: f32,
        /// Height.
        height: f32,
    },
    /// Push a transform onto the stack.
    PushTransform(Transform),
    /// Pop the most recent transform.
    PopTransform,
    /// Push a clip rectangle.
    PushClip {
        /// X position.
        x: f32,
        /// Y position.
        y: f32,
        /// Width.
        width: f32,
        /// Height.
        height: f32,
    },
    /// Pop the most recent clip.
    PopClip,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_construction() {
        let p = Point::new(10.0, 20.0);
        assert_eq!(p.x, 10.0);
        assert_eq!(p.y, 20.0);
    }

    #[test]
    fn transform_identity() {
        let t = Transform::IDENTITY;
        assert_eq!(t.matrix, [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
    }

    #[test]
    fn transform_translate() {
        let t = Transform::translate(5.0, 10.0);
        assert_eq!(t.matrix[4], 5.0);
        assert_eq!(t.matrix[5], 10.0);
    }

    #[test]
    fn transform_scale() {
        let t = Transform::scale(2.0, 3.0);
        assert_eq!(t.matrix[0], 2.0);
        assert_eq!(t.matrix[3], 3.0);
    }

    #[test]
    fn transform_rotate() {
        let t = Transform::rotate(std::f32::consts::FRAC_PI_2);
        // cos(pi/2) ≈ 0, sin(pi/2) ≈ 1
        assert!((t.matrix[0]).abs() < 1e-6);
        assert!((t.matrix[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn transform_default_is_identity() {
        assert_eq!(Transform::default(), Transform::IDENTITY);
    }

    #[test]
    fn fill_style_solid() {
        let fill = FillStyle::Solid(Color::WHITE);
        assert_eq!(fill, FillStyle::Solid(Color::WHITE));
    }

    #[test]
    fn stroke_style_construction() {
        let stroke = StrokeStyle {
            color: Color::BLACK,
            width: 2.0,
        };
        assert_eq!(stroke.color, Color::BLACK);
        assert_eq!(stroke.width, 2.0);
    }
}
