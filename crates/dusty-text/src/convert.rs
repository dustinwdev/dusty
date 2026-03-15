//! Conversions from dusty-style types to cosmic-text types.

use cosmic_text::{Attrs, Family, Metrics, Style, Weight};
use dusty_style::{Color, FontSlant, FontStyle, FontWeight};

use crate::rich::TextSpan;

const DEFAULT_FONT_SIZE: f32 = 16.0;
const DEFAULT_LINE_HEIGHT_FACTOR: f32 = 1.2;

/// Converts a `FontWeight` to a cosmic-text `Weight`.
const fn to_cosmic_weight(weight: FontWeight) -> Weight {
    Weight(weight.0)
}

/// Converts a `FontSlant` to a cosmic-text `Style`.
const fn to_cosmic_style(slant: FontSlant) -> Style {
    match slant {
        FontSlant::Normal => Style::Normal,
        FontSlant::Italic => Style::Italic,
        FontSlant::Oblique => Style::Oblique,
    }
}

/// Converts a dusty `Color` to a cosmic-text `Color`.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_lossless
)]
fn to_cosmic_color(color: Color) -> cosmic_text::Color {
    let r = (color.r.clamp(0.0, 1.0) * 255.0).round() as u8;
    let g = (color.g.clamp(0.0, 1.0) * 255.0).round() as u8;
    let b = (color.b.clamp(0.0, 1.0) * 255.0).round() as u8;
    let a = (color.a.clamp(0.0, 1.0) * 255.0).round() as u8;
    cosmic_text::Color::rgba(r, g, b, a)
}

/// Builds cosmic-text `Attrs` from a `FontStyle`.
///
/// Note: `FontStyle.letter_spacing` is not applied here — cosmic-text does
/// not support per-span letter spacing. A custom shaping pass would be needed.
pub fn font_style_to_attrs(font: &FontStyle) -> Attrs<'_> {
    let mut attrs = Attrs::new();

    if let Some(ref family) = font.family {
        attrs = attrs.family(Family::Name(family.as_ref()));
    } else {
        attrs = attrs.family(Family::SansSerif);
    }

    if let Some(weight) = font.weight {
        attrs = attrs.weight(to_cosmic_weight(weight));
    }

    if let Some(slant) = font.slant {
        attrs = attrs.style(to_cosmic_style(slant));
    }

    attrs
}

/// Builds cosmic-text `Metrics` from a `FontStyle`.
pub fn font_style_to_metrics(font: &FontStyle) -> Metrics {
    let font_size = font.size.unwrap_or(DEFAULT_FONT_SIZE);
    let line_height = font_size * font.line_height.unwrap_or(DEFAULT_LINE_HEIGHT_FACTOR);
    Metrics::new(font_size, line_height)
}

/// Layers span overrides onto base attrs.
pub fn span_to_cosmic<'a>(span: &'a TextSpan<'a>, base: &Attrs<'a>) -> Attrs<'a> {
    let mut attrs = base.clone();

    if let Some(family) = span.family {
        attrs = attrs.family(Family::Name(family));
    }

    if let Some(weight) = span.weight {
        attrs = attrs.weight(to_cosmic_weight(weight));
    }

    if let Some(slant) = span.slant {
        attrs = attrs.style(to_cosmic_style(slant));
    }

    if let Some(color) = span.color {
        attrs = attrs.color(to_cosmic_color(color));
    }

    attrs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weight_conversion_preserves_value() {
        let weight = to_cosmic_weight(FontWeight::BOLD);
        assert_eq!(weight.0, 700);
    }

    #[test]
    fn weight_conversion_thin() {
        let weight = to_cosmic_weight(FontWeight::THIN);
        assert_eq!(weight.0, 100);
    }

    #[test]
    fn style_conversion_normal() {
        assert_eq!(to_cosmic_style(FontSlant::Normal), Style::Normal);
    }

    #[test]
    fn style_conversion_italic() {
        assert_eq!(to_cosmic_style(FontSlant::Italic), Style::Italic);
    }

    #[test]
    fn style_conversion_oblique() {
        assert_eq!(to_cosmic_style(FontSlant::Oblique), Style::Oblique);
    }

    #[test]
    fn color_conversion_white() {
        let c = to_cosmic_color(Color::WHITE);
        assert_eq!(c.r(), 255);
        assert_eq!(c.g(), 255);
        assert_eq!(c.b(), 255);
        assert_eq!(c.a(), 255);
    }

    #[test]
    fn color_conversion_black() {
        let c = to_cosmic_color(Color::BLACK);
        assert_eq!(c.r(), 0);
        assert_eq!(c.g(), 0);
        assert_eq!(c.b(), 0);
        assert_eq!(c.a(), 255);
    }

    #[test]
    fn color_conversion_transparent() {
        let c = to_cosmic_color(Color::TRANSPARENT);
        assert_eq!(c.a(), 0);
    }

    #[test]
    fn color_conversion_mid_values() {
        let c = to_cosmic_color(Color::rgba(0.5, 0.5, 0.5, 0.5));
        // 0.5 * 255 = 127.5 → rounds to 128
        assert_eq!(c.r(), 128);
        assert_eq!(c.g(), 128);
        assert_eq!(c.b(), 128);
        assert_eq!(c.a(), 128);
    }

    #[test]
    fn metrics_defaults_when_none() {
        let font = FontStyle::default();
        let metrics = font_style_to_metrics(&font);
        assert_eq!(metrics.font_size, 16.0);
        assert_eq!(metrics.line_height, 19.2); // 16 * 1.2
    }

    #[test]
    fn metrics_uses_specified_size() {
        let font = FontStyle {
            size: Some(24.0),
            ..FontStyle::default()
        };
        let metrics = font_style_to_metrics(&font);
        assert_eq!(metrics.font_size, 24.0);
        assert!((metrics.line_height - 28.8).abs() < 0.01); // 24 * 1.2
    }

    #[test]
    fn metrics_uses_explicit_line_height() {
        let font = FontStyle {
            size: Some(16.0),
            line_height: Some(1.5),
            ..FontStyle::default()
        };
        let metrics = font_style_to_metrics(&font);
        assert_eq!(metrics.font_size, 16.0);
        assert!((metrics.line_height - 24.0).abs() < 0.01); // 16 * 1.5
    }

    #[test]
    fn attrs_defaults_to_sans_serif() {
        let font = FontStyle::default();
        let attrs = font_style_to_attrs(&font);
        assert_eq!(attrs.family, Family::SansSerif);
    }

    #[test]
    fn attrs_uses_specified_family() {
        let font = FontStyle {
            family: Some("Inter".into()),
            ..FontStyle::default()
        };
        let attrs = font_style_to_attrs(&font);
        assert_eq!(attrs.family, Family::Name("Inter"));
    }

    #[test]
    fn attrs_uses_specified_weight() {
        let font = FontStyle {
            weight: Some(FontWeight::BOLD),
            ..FontStyle::default()
        };
        let attrs = font_style_to_attrs(&font);
        assert_eq!(attrs.weight, Weight(700));
    }

    #[test]
    fn attrs_uses_specified_slant() {
        let font = FontStyle {
            slant: Some(FontSlant::Italic),
            ..FontStyle::default()
        };
        let attrs = font_style_to_attrs(&font);
        assert_eq!(attrs.style, Style::Italic);
    }

    #[test]
    fn span_override_weight() {
        let base = Attrs::new();
        let span = TextSpan::new("bold").weight(FontWeight::BOLD);
        let attrs = span_to_cosmic(&span, &base);
        assert_eq!(attrs.weight, Weight(700));
    }

    #[test]
    fn span_override_family() {
        let base = Attrs::new();
        let span = TextSpan::new("mono").family("Fira Code");
        let attrs = span_to_cosmic(&span, &base);
        assert_eq!(attrs.family, Family::Name("Fira Code"));
    }

    #[test]
    fn span_preserves_base_when_no_override() {
        let base = Attrs::new().weight(Weight(700));
        let span = TextSpan::new("plain");
        let attrs = span_to_cosmic(&span, &base);
        assert_eq!(attrs.weight, Weight(700));
    }

    #[test]
    fn metrics_line_height_is_multiplier() {
        let font = FontStyle {
            size: Some(16.0),
            line_height: Some(1.5),
            ..FontStyle::default()
        };
        let metrics = font_style_to_metrics(&font);
        assert_eq!(metrics.font_size, 16.0);
        // 1.5 is a multiplier: 16.0 * 1.5 = 24.0
        assert!((metrics.line_height - 24.0).abs() < 0.01);
    }

    #[test]
    fn color_conversion_clamps_out_of_range() {
        // Construct directly to bypass Color::rgba debug_assert
        let color = Color {
            r: 1.5,
            g: -0.5,
            b: 2.0,
            a: 0.5,
        };
        let c = to_cosmic_color(color);
        assert_eq!(c.r(), 255);
        assert_eq!(c.g(), 0);
        assert_eq!(c.b(), 255);
        assert_eq!(c.a(), 128);
    }

    #[test]
    fn span_override_color() {
        let base = Attrs::new();
        let span = TextSpan::new("red").color(Color::rgb(1.0, 0.0, 0.0));
        let attrs = span_to_cosmic(&span, &base);
        assert_eq!(
            attrs.color_opt,
            Some(cosmic_text::Color::rgba(255, 0, 0, 255))
        );
    }
}
