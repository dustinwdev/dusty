//! Rich text spans for styled text ranges.

use dusty_style::{Color, FontSlant, FontWeight};

/// A span of text with optional style overrides.
///
/// Used with [`TextLayout::new_rich`](crate::TextLayout::new_rich) and
/// [`TextSystem::measure_rich`](crate::TextSystem::measure_rich) to render
/// mixed-style text.
///
/// **Note:** Per-span font size is not supported by the underlying text
/// shaping engine (cosmic-text uses per-buffer `Metrics` for sizing).
/// To use different font sizes, create separate [`TextLayout`](crate::TextLayout) instances.
///
/// # Examples
///
/// ```
/// use dusty_text::TextSpan;
/// use dusty_style::FontWeight;
///
/// let span = TextSpan::new("bold text")
///     .weight(FontWeight::BOLD);
/// assert_eq!(span.text, "bold text");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct TextSpan<'a> {
    /// The text content.
    pub text: &'a str,
    /// Optional font family override.
    pub family: Option<&'a str>,
    /// Optional font weight override.
    pub weight: Option<FontWeight>,
    /// Optional font slant override.
    pub slant: Option<FontSlant>,
    /// Optional text color override.
    pub color: Option<Color>,
}

impl<'a> TextSpan<'a> {
    /// Creates a new span with the given text and no style overrides.
    #[must_use]
    pub const fn new(text: &'a str) -> Self {
        Self {
            text,
            family: None,
            weight: None,
            slant: None,
            color: None,
        }
    }

    /// Sets the font family.
    #[must_use]
    pub const fn family(mut self, family: &'a str) -> Self {
        self.family = Some(family);
        self
    }

    /// Sets the font weight.
    #[must_use]
    pub const fn weight(mut self, weight: FontWeight) -> Self {
        self.weight = Some(weight);
        self
    }

    /// Sets the font slant.
    #[must_use]
    pub const fn slant(mut self, slant: FontSlant) -> Self {
        self.slant = Some(slant);
        self
    }

    /// Sets the text color.
    #[must_use]
    pub const fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_plain_span() {
        let span = TextSpan::new("hello");
        assert_eq!(span.text, "hello");
        assert_eq!(span.family, None);
        assert_eq!(span.weight, None);
        assert_eq!(span.slant, None);
        assert_eq!(span.color, None);
    }

    #[test]
    fn builder_sets_weight() {
        let span = TextSpan::new("bold").weight(FontWeight::BOLD);
        assert_eq!(span.weight, Some(FontWeight::BOLD));
    }

    #[test]
    fn builder_sets_slant() {
        let span = TextSpan::new("italic").slant(FontSlant::Italic);
        assert_eq!(span.slant, Some(FontSlant::Italic));
    }

    #[test]
    fn builder_sets_family() {
        let span = TextSpan::new("mono").family("Fira Code");
        assert_eq!(span.family, Some("Fira Code"));
    }

    #[test]
    fn builder_sets_color() {
        let red = Color::rgb(1.0, 0.0, 0.0);
        let span = TextSpan::new("red").color(red);
        assert_eq!(span.color, Some(red));
    }

    #[test]
    fn builder_chains() {
        let span = TextSpan::new("styled")
            .family("Inter")
            .weight(FontWeight::SEMI_BOLD)
            .slant(FontSlant::Italic)
            .color(Color::hex(0xFF0000));

        assert_eq!(span.text, "styled");
        assert_eq!(span.family, Some("Inter"));
        assert_eq!(span.weight, Some(FontWeight::SEMI_BOLD));
        assert_eq!(span.slant, Some(FontSlant::Italic));
        assert!(span.color.is_some());
    }

    #[test]
    fn span_is_clone() {
        let span = TextSpan::new("clone me").weight(FontWeight::BOLD);
        let cloned = span.clone();
        assert_eq!(span, cloned);
    }
}
