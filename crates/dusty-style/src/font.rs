//! Font style types for typography properties.

use std::sync::Arc;

/// Font weight as a numeric value (100–900).
///
/// # Examples
///
/// ```
/// use dusty_style::FontWeight;
///
/// assert_eq!(FontWeight::BOLD.0, 700);
/// assert!(FontWeight::THIN < FontWeight::BLACK);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FontWeight(pub u16);

impl FontWeight {
    /// Thin (100).
    pub const THIN: Self = Self(100);
    /// Extra-light (200).
    pub const EXTRA_LIGHT: Self = Self(200);
    /// Light (300).
    pub const LIGHT: Self = Self(300);
    /// Normal (400).
    pub const NORMAL: Self = Self(400);
    /// Medium (500).
    pub const MEDIUM: Self = Self(500);
    /// Semi-bold (600).
    pub const SEMI_BOLD: Self = Self(600);
    /// Bold (700).
    pub const BOLD: Self = Self(700);
    /// Extra-bold (800).
    pub const EXTRA_BOLD: Self = Self(800);
    /// Black (900).
    pub const BLACK: Self = Self(900);
}

impl Default for FontWeight {
    fn default() -> Self {
        Self::NORMAL
    }
}

/// Font slant/style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FontSlant {
    /// Upright glyphs.
    #[default]
    Normal,
    /// Italic glyphs (true italic design).
    Italic,
    /// Oblique glyphs (slanted version of upright).
    Oblique,
}

/// Typography style properties. `None` fields are not set (inherit/default).
///
/// # Examples
///
/// ```
/// use dusty_style::{FontStyle, FontWeight};
///
/// let heading = FontStyle {
///     size: Some(24.0),
///     weight: Some(FontWeight::BOLD),
///     ..FontStyle::default()
/// };
/// assert_eq!(heading.size, Some(24.0));
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FontStyle {
    /// Font family name.
    ///
    /// Uses `Arc<str>` for cheap cloning during style merges and to enable
    /// string interning in the future. Convert from `&str` or `String` via
    /// `.into()`.
    pub family: Option<Arc<str>>,
    /// Font size in pixels.
    pub size: Option<f32>,
    /// Font weight (100–900).
    pub weight: Option<FontWeight>,
    /// Font slant (normal, italic, oblique).
    pub slant: Option<FontSlant>,
    /// Line height multiplier.
    pub line_height: Option<f32>,
    /// Letter spacing in pixels.
    pub letter_spacing: Option<f32>,
}

impl FontStyle {
    /// Merges `other` on top of `self`. Other's `Some` values win.
    #[must_use]
    pub fn merge(&self, other: &Self) -> Self {
        Self {
            family: other.family.clone().or_else(|| self.family.clone()),
            size: other.size.or(self.size),
            weight: other.weight.or(self.weight),
            slant: other.slant.or(self.slant),
            line_height: other.line_height.or(self.line_height),
            letter_spacing: other.letter_spacing.or(self.letter_spacing),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_weight_constants() {
        assert_eq!(FontWeight::THIN.0, 100);
        assert_eq!(FontWeight::EXTRA_LIGHT.0, 200);
        assert_eq!(FontWeight::LIGHT.0, 300);
        assert_eq!(FontWeight::NORMAL.0, 400);
        assert_eq!(FontWeight::MEDIUM.0, 500);
        assert_eq!(FontWeight::SEMI_BOLD.0, 600);
        assert_eq!(FontWeight::BOLD.0, 700);
        assert_eq!(FontWeight::EXTRA_BOLD.0, 800);
        assert_eq!(FontWeight::BLACK.0, 900);
    }

    #[test]
    fn font_weight_default_is_normal() {
        assert_eq!(FontWeight::default(), FontWeight::NORMAL);
    }

    #[test]
    fn font_weight_ordering() {
        assert!(FontWeight::THIN < FontWeight::BOLD);
        assert!(FontWeight::BLACK > FontWeight::MEDIUM);
    }

    #[test]
    fn font_slant_default_is_normal() {
        assert_eq!(FontSlant::default(), FontSlant::Normal);
    }

    #[test]
    fn font_style_default_is_all_none() {
        let fs = FontStyle::default();
        assert_eq!(fs.family, None);
        assert_eq!(fs.size, None);
        assert_eq!(fs.weight, None);
        assert_eq!(fs.slant, None);
        assert_eq!(fs.line_height, None);
        assert_eq!(fs.letter_spacing, None);
    }

    #[test]
    fn font_style_merge_other_wins() {
        let base = FontStyle {
            size: Some(16.0),
            weight: Some(FontWeight::NORMAL),
            family: Some("Inter".into()),
            ..FontStyle::default()
        };
        let over = FontStyle {
            size: Some(24.0),
            weight: None,
            family: Some("Roboto".into()),
            ..FontStyle::default()
        };
        let merged = base.merge(&over);
        assert_eq!(merged.size, Some(24.0));
        assert_eq!(merged.weight, Some(FontWeight::NORMAL));
        assert_eq!(merged.family, Some("Roboto".into()));
    }

    #[test]
    fn font_style_merge_preserves_base() {
        let base = FontStyle {
            size: Some(16.0),
            line_height: Some(1.5),
            ..FontStyle::default()
        };
        let over = FontStyle::default();
        let merged = base.merge(&over);
        assert_eq!(merged.size, Some(16.0));
        assert_eq!(merged.line_height, Some(1.5));
    }
}
