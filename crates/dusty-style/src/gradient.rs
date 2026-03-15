//! Linear gradient types for background fills.

use crate::Color;

/// Direction of a linear gradient.
///
/// # Examples
///
/// ```
/// use dusty_style::GradientDirection;
///
/// let angle = GradientDirection::Angle(45.0);
/// assert_eq!(GradientDirection::ToRight.to_angle(), 90.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum GradientDirection {
    /// Gradient flows from bottom to top (0°).
    ToTop,
    /// Gradient flows from left to right (90°).
    ToRight,
    /// Gradient flows from top to bottom (180°).
    #[default]
    ToBottom,
    /// Gradient flows from right to left (270°).
    ToLeft,
    /// Gradient at an arbitrary angle in degrees.
    Angle(f32),
}

impl GradientDirection {
    /// Converts the direction to an angle in degrees.
    ///
    /// CSS convention: 0° = to top, 90° = to right, 180° = to bottom.
    #[must_use]
    pub const fn to_angle(self) -> f32 {
        match self {
            Self::ToTop => 0.0,
            Self::ToRight => 90.0,
            Self::ToBottom => 180.0,
            Self::ToLeft => 270.0,
            Self::Angle(deg) => deg,
        }
    }
}

/// A color stop in a gradient.
///
/// # Examples
///
/// ```
/// use dusty_style::{Color, ColorStop};
///
/// let stop = ColorStop { color: Color::WHITE, position: 0.0 };
/// assert_eq!(stop.position, 0.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorStop {
    /// The color at this stop.
    pub color: Color,
    /// Position along the gradient (0.0–1.0).
    pub position: f32,
}

/// A linear gradient with direction and color stops.
///
/// # Examples
///
/// ```
/// use dusty_style::{Color, ColorStop, GradientDirection, LinearGradient};
///
/// let gradient = LinearGradient {
///     direction: GradientDirection::ToRight,
///     stops: vec![
///         ColorStop { color: Color::WHITE, position: 0.0 },
///         ColorStop { color: Color::BLACK, position: 1.0 },
///     ],
/// };
/// assert_eq!(gradient.stops.len(), 2);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct LinearGradient {
    /// Direction of the gradient.
    pub direction: GradientDirection,
    /// Color stops along the gradient axis.
    pub stops: Vec<ColorStop>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direction_to_angle() {
        assert_eq!(GradientDirection::ToTop.to_angle(), 0.0);
        assert_eq!(GradientDirection::ToRight.to_angle(), 90.0);
        assert_eq!(GradientDirection::ToBottom.to_angle(), 180.0);
        assert_eq!(GradientDirection::ToLeft.to_angle(), 270.0);
        assert_eq!(GradientDirection::Angle(45.0).to_angle(), 45.0);
    }

    #[test]
    fn direction_default_is_to_bottom() {
        assert_eq!(GradientDirection::default(), GradientDirection::ToBottom);
    }

    #[test]
    fn color_stop_construction() {
        let stop = ColorStop {
            color: Color::WHITE,
            position: 0.5,
        };
        assert_eq!(stop.color, Color::WHITE);
        assert_eq!(stop.position, 0.5);
    }

    #[test]
    fn linear_gradient_construction() {
        let gradient = LinearGradient {
            direction: GradientDirection::ToRight,
            stops: vec![
                ColorStop {
                    color: Color::WHITE,
                    position: 0.0,
                },
                ColorStop {
                    color: Color::BLACK,
                    position: 1.0,
                },
            ],
        };
        assert_eq!(gradient.direction, GradientDirection::ToRight);
        assert_eq!(gradient.stops.len(), 2);
        assert_eq!(gradient.stops[0].position, 0.0);
        assert_eq!(gradient.stops[1].position, 1.0);
    }

    #[test]
    fn gradient_direction_is_copy() {
        let a = GradientDirection::ToRight;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn color_stop_is_copy() {
        let a = ColorStop {
            color: Color::WHITE,
            position: 0.0,
        };
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn gradient_clone() {
        let gradient = LinearGradient {
            direction: GradientDirection::Angle(135.0),
            stops: vec![
                ColorStop {
                    color: Color::rgb(1.0, 0.0, 0.0),
                    position: 0.0,
                },
                ColorStop {
                    color: Color::rgb(0.0, 0.0, 1.0),
                    position: 1.0,
                },
            ],
        };
        let cloned = gradient.clone();
        assert_eq!(gradient, cloned);
    }
}
