//! Box shadow type.

use crate::Color;

/// A box shadow with offset, blur, spread, color, and inset flag.
///
/// # Examples
///
/// ```
/// use dusty_style::{BoxShadow, Color};
///
/// let shadow = BoxShadow {
///     offset_x: 0.0,
///     offset_y: 4.0,
///     blur_radius: 6.0,
///     spread_radius: -1.0,
///     color: Color::rgba(0.0, 0.0, 0.0, 0.1),
///     inset: false,
/// };
/// assert_eq!(shadow.blur_radius, 6.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoxShadow {
    /// Horizontal offset in pixels.
    pub offset_x: f32,
    /// Vertical offset in pixels.
    pub offset_y: f32,
    /// Blur radius in pixels.
    pub blur_radius: f32,
    /// Spread radius in pixels (negative shrinks).
    pub spread_radius: f32,
    /// Shadow color.
    pub color: Color,
    /// Whether the shadow is inset.
    pub inset: bool,
}

impl BoxShadow {
    /// Creates a new box shadow, validating `blur_radius >= 0.0` in debug builds.
    #[must_use]
    pub fn new(
        offset_x: f32,
        offset_y: f32,
        blur_radius: f32,
        spread_radius: f32,
        color: Color,
        inset: bool,
    ) -> Self {
        debug_assert!(
            blur_radius >= 0.0,
            "blur_radius must be non-negative: {blur_radius}"
        );
        Self {
            offset_x,
            offset_y,
            blur_radius: blur_radius.max(0.0),
            spread_radius,
            color,
            inset,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_access() {
        let s = BoxShadow {
            offset_x: 1.0,
            offset_y: 2.0,
            blur_radius: 3.0,
            spread_radius: 4.0,
            color: Color::BLACK,
            inset: true,
        };
        assert_eq!(s.offset_x, 1.0);
        assert_eq!(s.offset_y, 2.0);
        assert_eq!(s.blur_radius, 3.0);
        assert_eq!(s.spread_radius, 4.0);
        assert_eq!(s.color, Color::BLACK);
        assert!(s.inset);
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "blur_radius")]
    fn box_shadow_new_rejects_negative_blur() {
        let _ = BoxShadow::new(0.0, 4.0, -1.0, 0.0, Color::BLACK, false);
    }

    #[test]
    fn box_shadow_new_constructs_correctly() {
        let s = BoxShadow::new(1.0, 2.0, 3.0, 4.0, Color::BLACK, false);
        assert_eq!(s.offset_x, 1.0);
        assert_eq!(s.offset_y, 2.0);
        assert_eq!(s.blur_radius, 3.0);
        assert_eq!(s.spread_radius, 4.0);
        assert_eq!(s.color, Color::BLACK);
        assert!(!s.inset);
    }

    #[test]
    fn box_shadow_new_inset() {
        let s = BoxShadow::new(0.0, 0.0, 5.0, 0.0, Color::WHITE, true);
        assert!(s.inset);
    }

    #[test]
    fn box_shadow_new_negative_spread() {
        let s = BoxShadow::new(0.0, 4.0, 6.0, -1.0, Color::rgba(0.0, 0.0, 0.0, 0.1), false);
        assert_eq!(s.spread_radius, -1.0);
    }

    #[test]
    fn box_shadow_new_zero_blur() {
        let s = BoxShadow::new(2.0, 2.0, 0.0, 0.0, Color::BLACK, false);
        assert_eq!(s.blur_radius, 0.0);
    }

    #[cfg(not(debug_assertions))]
    #[test]
    fn negative_blur_radius_clamped_to_zero() {
        let s = BoxShadow::new(0.0, 4.0, -5.0, 0.0, Color::BLACK, false);
        assert_eq!(s.blur_radius, 0.0);
    }

    #[test]
    fn shadow_is_copy() {
        let a = BoxShadow {
            offset_x: 0.0,
            offset_y: 0.0,
            blur_radius: 0.0,
            spread_radius: 0.0,
            color: Color::TRANSPARENT,
            inset: false,
        };
        let b = a;
        assert_eq!(a, b);
    }
}
