//! RGBA color type for GPU-friendly color representation.

/// A color represented as four `f32` components in the range 0.0–1.0.
///
/// # Examples
///
/// ```
/// use dusty_style::Color;
///
/// let red = Color::rgb(1.0, 0.0, 0.0);
/// assert_eq!(red.a, 1.0);
///
/// let semi = Color::rgba(1.0, 1.0, 1.0, 0.5);
/// assert_eq!(semi.a, 0.5);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    /// Red component (0.0–1.0).
    pub r: f32,
    /// Green component (0.0–1.0).
    pub g: f32,
    /// Blue component (0.0–1.0).
    pub b: f32,
    /// Alpha component (0.0–1.0).
    pub a: f32,
}

impl Color {
    /// Fully opaque white.
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    /// Fully opaque black.
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    /// Fully transparent black.
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    /// Creates a color from RGBA f32 values (0.0–1.0).
    #[must_use]
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        debug_assert!((0.0..=1.0).contains(&r), "r out of range: {r}");
        debug_assert!((0.0..=1.0).contains(&g), "g out of range: {g}");
        debug_assert!((0.0..=1.0).contains(&b), "b out of range: {b}");
        debug_assert!((0.0..=1.0).contains(&a), "a out of range: {a}");
        Self { r, g, b, a }
    }

    /// Creates an opaque color from RGB f32 values (0.0–1.0).
    #[must_use]
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        debug_assert!((0.0..=1.0).contains(&r), "r out of range: {r}");
        debug_assert!((0.0..=1.0).contains(&g), "g out of range: {g}");
        debug_assert!((0.0..=1.0).contains(&b), "b out of range: {b}");
        Self { r, g, b, a: 1.0 }
    }

    /// Creates a color from RGBA u8 values (0–255).
    #[must_use]
    pub fn rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: f32::from(r) / 255.0,
            g: f32::from(g) / 255.0,
            b: f32::from(b) / 255.0,
            a: f32::from(a) / 255.0,
        }
    }

    /// Creates an opaque color from RGB u8 values (0–255).
    #[must_use]
    pub fn rgb8(r: u8, g: u8, b: u8) -> Self {
        Self::rgba8(r, g, b, 255)
    }

    /// Creates an opaque color from a 24-bit hex value (`0xRRGGBB`).
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn hex(value: u32) -> Self {
        let r = ((value >> 16) & 0xFF) as u8;
        let g = ((value >> 8) & 0xFF) as u8;
        let b = (value & 0xFF) as u8;
        Self::rgb8(r, g, b)
    }

    /// Creates a color from a 32-bit hex value (`0xRRGGBBAA`).
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn hexa(value: u32) -> Self {
        let r = ((value >> 24) & 0xFF) as u8;
        let g = ((value >> 16) & 0xFF) as u8;
        let b = ((value >> 8) & 0xFF) as u8;
        let a = (value & 0xFF) as u8;
        Self::rgba8(r, g, b, a)
    }
}

impl std::fmt::Display for Color {
    /// Formats the color as `#RRGGBB` when fully opaque, or `#RRGGBBAA` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use dusty_style::Color;
    ///
    /// assert_eq!(Color::rgb8(255, 128, 0).to_string(), "#FF8000");
    /// assert_eq!(Color::rgba8(255, 128, 0, 128).to_string(), "#FF800080");
    /// ```
    #[allow(clippy::many_single_char_names)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let [red, green, blue, alpha] = [
            (self.r * 255.0).round() as u8,
            (self.g * 255.0).round() as u8,
            (self.b * 255.0).round() as u8,
            (self.a * 255.0).round() as u8,
        ];

        if alpha == 255 {
            write!(f, "#{red:02X}{green:02X}{blue:02X}")
        } else {
            write!(f, "#{red:02X}{green:02X}{blue:02X}{alpha:02X}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgba_constructor() {
        let c = Color::rgba(0.1, 0.2, 0.3, 0.4);
        assert_eq!(c.r, 0.1);
        assert_eq!(c.g, 0.2);
        assert_eq!(c.b, 0.3);
        assert_eq!(c.a, 0.4);
    }

    #[test]
    fn rgb_constructor_sets_alpha_to_one() {
        let c = Color::rgb(0.5, 0.6, 0.7);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn rgba8_converts_from_u8() {
        let c = Color::rgba8(255, 0, 128, 255);
        assert!((c.r - 1.0).abs() < f32::EPSILON);
        assert!(c.g.abs() < f32::EPSILON);
        assert!((c.b - 128.0 / 255.0).abs() < f32::EPSILON);
        assert!((c.a - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn rgb8_converts_with_full_alpha() {
        let c = Color::rgb8(0, 255, 0);
        assert!((c.g - 1.0).abs() < f32::EPSILON);
        assert!((c.a - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn hex_constructor() {
        let c = Color::hex(0xFF8000);
        assert!((c.r - 1.0).abs() < f32::EPSILON);
        assert!((c.g - 128.0 / 255.0).abs() < f32::EPSILON);
        assert!(c.b.abs() < f32::EPSILON);
        assert!((c.a - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn hexa_constructor() {
        let c = Color::hexa(0xFF00_0080);
        assert!((c.r - 1.0).abs() < f32::EPSILON);
        assert!(c.g.abs() < f32::EPSILON);
        assert!(c.b.abs() < f32::EPSILON);
        assert!((c.a - 128.0 / 255.0).abs() < f32::EPSILON);
    }

    #[test]
    fn constants() {
        assert_eq!(Color::WHITE, Color::rgb(1.0, 1.0, 1.0));
        assert_eq!(Color::BLACK, Color::rgb(0.0, 0.0, 0.0));
        assert_eq!(Color::TRANSPARENT, Color::rgba(0.0, 0.0, 0.0, 0.0));
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "out of range")]
    fn rgba_rejects_out_of_range_r() {
        let _ = Color::rgba(2.0, 0.0, 0.0, 1.0);
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "out of range")]
    fn rgba_rejects_negative_g() {
        let _ = Color::rgba(0.0, -0.5, 0.0, 1.0);
    }

    #[test]
    fn color_is_copy() {
        let a = Color::WHITE;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn display_opaque_color() {
        assert_eq!(Color::WHITE.to_string(), "#FFFFFF");
        assert_eq!(Color::BLACK.to_string(), "#000000");
        assert_eq!(Color::rgb8(255, 128, 0).to_string(), "#FF8000");
    }

    #[test]
    fn display_transparent_color() {
        assert_eq!(Color::TRANSPARENT.to_string(), "#00000000");
        assert_eq!(Color::rgba8(255, 128, 0, 128).to_string(), "#FF800080");
    }

    #[test]
    fn display_half_alpha() {
        let c = Color::rgba(1.0, 0.0, 0.0, 0.5);
        let s = c.to_string();
        assert!(s.starts_with("#FF0000"));
        assert!(s.len() == 9); // #RRGGBBAA
    }
}
