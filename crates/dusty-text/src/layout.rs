//! `TextLayout` — a shaped text buffer with size/line queries.

use cosmic_text::{Attrs, Buffer, Shaping, Wrap};
use dusty_style::FontStyle;

use crate::convert::{font_style_to_attrs, font_style_to_metrics, span_to_cosmic};
use crate::error::Result;
use crate::rich::TextSpan;
use crate::system::{compute_buffer_size, TextSystem};

/// A laid-out text buffer with computed size and line information.
///
/// Wraps a cosmic-text `Buffer` after shaping and layout.
///
/// # Examples
///
/// ```
/// use dusty_text::{TextLayout, TextSystem};
/// use dusty_style::FontStyle;
///
/// let system = TextSystem::new();
/// let layout = TextLayout::new(&system, "hello world", &FontStyle::default(), None).unwrap();
/// let (w, h) = layout.size();
/// assert!(w > 0.0);
/// assert!(h > 0.0);
/// ```
pub struct TextLayout {
    buffer: Buffer,
    size: (f32, f32),
}

impl TextLayout {
    /// Creates a `TextLayout` from plain text.
    ///
    /// # Errors
    ///
    /// Returns [`TextError::BorrowConflict`] if the `FontSystem` is already borrowed.
    pub fn new(
        system: &TextSystem,
        text: &str,
        font: &FontStyle,
        max_width: Option<f32>,
    ) -> Result<Self> {
        let metrics = font_style_to_metrics(font);
        let attrs = font_style_to_attrs(font);

        let mut font_system = system.font_system_mut()?;
        let mut buffer = Buffer::new(&mut font_system, metrics);

        let wrap = if max_width.is_some() {
            Wrap::Word
        } else {
            Wrap::None
        };

        buffer.set_wrap(&mut font_system, wrap);
        buffer.set_size(&mut font_system, max_width, None);
        buffer.set_text(&mut font_system, text, &attrs, Shaping::Advanced, None);

        let size = compute_buffer_size(&buffer);
        Ok(Self { buffer, size })
    }

    /// Creates a `TextLayout` from rich (multi-span) text.
    ///
    /// # Errors
    ///
    /// Returns [`TextError::BorrowConflict`] if the `FontSystem` is already borrowed.
    pub fn new_rich(
        system: &TextSystem,
        spans: &[TextSpan<'_>],
        font: &FontStyle,
        max_width: Option<f32>,
    ) -> Result<Self> {
        let metrics = font_style_to_metrics(font);
        let base_attrs = font_style_to_attrs(font);

        let mut font_system = system.font_system_mut()?;
        let mut buffer = Buffer::new(&mut font_system, metrics);

        let wrap = if max_width.is_some() {
            Wrap::Word
        } else {
            Wrap::None
        };

        buffer.set_wrap(&mut font_system, wrap);
        buffer.set_size(&mut font_system, max_width, None);

        let rich_spans: Vec<(&str, Attrs<'_>)> = spans
            .iter()
            .map(|span| {
                let attrs = span_to_cosmic(span, &base_attrs);
                (span.text, attrs)
            })
            .collect();

        buffer.set_rich_text(
            &mut font_system,
            rich_spans,
            &base_attrs,
            Shaping::Advanced,
            None,
        );

        let size = compute_buffer_size(&buffer);
        Ok(Self { buffer, size })
    }

    /// Returns the computed bounding box `(width, height)`.
    #[must_use]
    pub const fn size(&self) -> (f32, f32) {
        self.size
    }

    /// Returns the number of visual lines after wrapping.
    #[must_use]
    pub fn line_count(&self) -> usize {
        self.buffer.layout_runs().count()
    }

    /// Access the underlying cosmic-text `Buffer` for the render pipeline.
    #[must_use]
    pub const fn buffer(&self) -> &Buffer {
        &self.buffer
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;
    use dusty_style::FontWeight;

    #[test]
    fn plain_text_has_positive_size() {
        let system = TextSystem::new();
        let layout = TextLayout::new(&system, "hello", &FontStyle::default(), None).unwrap();
        let (w, h) = layout.size();
        assert!(w > 0.0, "width should be positive: {w}");
        assert!(h > 0.0, "height should be positive: {h}");
    }

    #[test]
    fn empty_text_has_zero_width() {
        let system = TextSystem::new();
        let layout = TextLayout::new(&system, "", &FontStyle::default(), None).unwrap();
        let (w, _h) = layout.size();
        assert_eq!(w, 0.0);
    }

    #[test]
    fn single_line_has_one_run() {
        let system = TextSystem::new();
        let layout = TextLayout::new(&system, "hello", &FontStyle::default(), None).unwrap();
        assert_eq!(layout.line_count(), 1);
    }

    #[test]
    fn wrapped_text_has_multiple_lines() {
        let system = TextSystem::new();
        let long_text = "hello world this is a very long piece of text that should wrap";
        let layout =
            TextLayout::new(&system, long_text, &FontStyle::default(), Some(50.0)).unwrap();
        assert!(
            layout.line_count() > 1,
            "should have multiple lines: {}",
            layout.line_count()
        );
    }

    #[test]
    fn buffer_is_accessible() {
        let system = TextSystem::new();
        let layout = TextLayout::new(&system, "hello", &FontStyle::default(), None).unwrap();
        let _buffer = layout.buffer();
    }

    #[test]
    fn rich_text_has_positive_size() {
        let system = TextSystem::new();
        let spans = [
            TextSpan::new("hello "),
            TextSpan::new("bold").weight(FontWeight::BOLD),
        ];
        let layout = TextLayout::new_rich(&system, &spans, &FontStyle::default(), None).unwrap();
        let (w, h) = layout.size();
        assert!(w > 0.0, "width should be positive: {w}");
        assert!(h > 0.0, "height should be positive: {h}");
    }

    #[test]
    fn rich_text_wraps_with_max_width() {
        let system = TextSystem::new();
        let spans = [
            TextSpan::new("hello "),
            TextSpan::new("world "),
            TextSpan::new("this is long text"),
        ];
        let layout =
            TextLayout::new_rich(&system, &spans, &FontStyle::default(), Some(50.0)).unwrap();
        assert!(
            layout.line_count() > 1,
            "rich text should wrap: {}",
            layout.line_count()
        );
    }

    #[test]
    fn larger_font_produces_larger_layout() {
        let system = TextSystem::new();
        let small = FontStyle {
            size: Some(12.0),
            ..FontStyle::default()
        };
        let large = FontStyle {
            size: Some(48.0),
            ..FontStyle::default()
        };
        let small_layout = TextLayout::new(&system, "hello", &small, None).unwrap();
        let large_layout = TextLayout::new(&system, "hello", &large, None).unwrap();

        assert!(
            large_layout.size().1 > small_layout.size().1,
            "larger font should be taller"
        );
    }
}
