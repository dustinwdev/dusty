//! `TextSystem` — wraps `FontSystem` and implements `TextMeasure`.

use std::cell::RefCell;
use std::marker::PhantomData;

use cosmic_text::{Buffer, FontSystem, Shaping, Wrap};
use dusty_layout::TextMeasure;
use dusty_style::FontStyle;

use crate::convert::{font_style_to_attrs, font_style_to_metrics};
use crate::layout::TextLayout;
use crate::rich::TextSpan;
use crate::truncate::{TruncatedText, Truncation};

/// Central text system wrapping a cosmic-text `FontSystem`.
///
/// **Thread safety:** `TextSystem` is intentionally `!Send + !Sync` because
/// `FontSystem` uses interior mutability via `RefCell`. Create one per thread.
///
/// Provides text measurement for the layout engine via [`TextMeasure`],
/// plus richer APIs for styled text via [`TextLayout`].
///
/// # Examples
///
/// ```
/// use dusty_text::TextSystem;
/// use dusty_layout::TextMeasure;
/// use dusty_style::FontStyle;
///
/// let system = TextSystem::new();
/// let (w, h) = system.measure("hello", None, &FontStyle::default());
/// assert!(w > 0.0);
/// assert!(h > 0.0);
/// ```
///
/// `TextSystem` cannot be sent across threads:
///
/// ```compile_fail
/// use dusty_text::TextSystem;
/// fn requires_send<T: Send>() {}
/// requires_send::<TextSystem>();
/// ```
///
/// `TextSystem` cannot be shared across threads:
///
/// ```compile_fail
/// use dusty_text::TextSystem;
/// fn requires_sync<T: Sync>() {}
/// requires_sync::<TextSystem>();
/// ```
pub struct TextSystem {
    font_system: RefCell<FontSystem>,
    /// Marker to ensure `TextSystem` is `!Send + !Sync`.
    /// `RefCell` already provides `!Sync`, and this marker adds `!Send`.
    _not_send: PhantomData<*const ()>,
}

impl TextSystem {
    /// Creates a new `TextSystem`, loading system fonts.
    #[must_use]
    pub fn new() -> Self {
        Self {
            font_system: RefCell::new(FontSystem::new()),
            _not_send: PhantomData,
        }
    }

    /// Measures rich (multi-span) text, returning `(width, height)`.
    ///
    /// # Errors
    ///
    /// Returns [`TextError::BorrowConflict`](crate::error::TextError::BorrowConflict)
    /// if the `FontSystem` is already borrowed.
    pub fn measure_rich(
        &self,
        spans: &[TextSpan<'_>],
        max_width: Option<f32>,
        font: &FontStyle,
    ) -> crate::error::Result<(f32, f32)> {
        let layout = TextLayout::new_rich(self, spans, font, max_width)?;
        Ok(layout.size())
    }

    /// Truncates text to fit within `max_width`, using the given strategy.
    ///
    /// With [`Truncation::None`], returns the text as-is with its measured size.
    /// With [`Truncation::Ellipsis`], appends "…" if the text overflows.
    ///
    /// # Examples
    ///
    /// ```
    /// use dusty_text::{TextSystem, Truncation};
    /// use dusty_style::FontStyle;
    ///
    /// let system = TextSystem::new();
    /// let result = system.truncate("hello", 1000.0, &FontStyle::default(), Truncation::Ellipsis);
    /// assert!(!result.was_truncated);
    /// assert_eq!(result.text, "hello");
    /// ```
    pub fn truncate(
        &self,
        text: &str,
        max_width: f32,
        font: &FontStyle,
        strategy: Truncation,
    ) -> TruncatedText {
        let (full_w, full_h) = self.measure(text, None, font);

        // If no truncation requested, or text already fits, return as-is.
        if matches!(strategy, Truncation::None) || full_w <= max_width {
            return TruncatedText {
                text: text.to_string(),
                was_truncated: false,
                size: (full_w, full_h),
            };
        }

        // Empty text can't be truncated further.
        if text.is_empty() {
            return TruncatedText {
                text: String::new(),
                was_truncated: false,
                size: (0.0, full_h),
            };
        }

        // Binary search for the longest prefix where "prefix…" fits.
        let ellipsis = '…';
        let char_indices: Vec<usize> = text.char_indices().map(|(i, _)| i).collect();

        let mut lo: usize = 0;
        let mut hi: usize = char_indices.len();
        let mut best: usize = 0; // byte index of best cut point

        while lo < hi {
            let mid = (lo + hi) / 2;
            let byte_idx = char_indices[mid];
            let candidate = format!("{}{ellipsis}", &text[..byte_idx]);
            let (w, _) = self.measure(&candidate, None, font);

            if w <= max_width {
                best = byte_idx;
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }

        let truncated = format!("{}{ellipsis}", &text[..best]);
        let (w, h) = self.measure(&truncated, None, font);

        TruncatedText {
            text: truncated,
            was_truncated: true,
            size: (w, h),
        }
    }

    /// Returns a mutable reference to the inner `FontSystem`.
    ///
    /// Used internally by [`TextLayout`] and by `dusty-render` for
    /// glyph rasterization via `SwashCache`.
    ///
    /// # Errors
    ///
    /// Returns [`BorrowConflict`](crate::error::TextError::BorrowConflict) if
    /// the `FontSystem` is already borrowed.
    pub fn font_system_mut(&self) -> crate::error::Result<std::cell::RefMut<'_, FontSystem>> {
        self.font_system
            .try_borrow_mut()
            .map_err(|_| crate::error::TextError::BorrowConflict)
    }
}

impl Default for TextSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl TextMeasure for TextSystem {
    fn measure(&self, text: &str, max_width: Option<f32>, font: &FontStyle) -> (f32, f32) {
        let metrics = font_style_to_metrics(font);
        let attrs = font_style_to_attrs(font);

        let Ok(mut font_system) = self.font_system.try_borrow_mut() else {
            eprintln!("dusty-text: BUG — FontSystem borrow conflict in measure");
            return (0.0, 0.0);
        };
        let mut buffer = Buffer::new(&mut font_system, metrics);

        let wrap = if max_width.is_some() {
            Wrap::Word
        } else {
            Wrap::None
        };

        buffer.set_wrap(&mut font_system, wrap);
        buffer.set_size(&mut font_system, max_width, None);
        buffer.set_text(&mut font_system, text, &attrs, Shaping::Advanced, None);

        compute_buffer_size(&buffer)
    }
}

/// Computes the bounding box of a shaped buffer from its layout runs.
pub(crate) fn compute_buffer_size(buffer: &Buffer) -> (f32, f32) {
    let mut width: f32 = 0.0;
    let mut height: f32 = 0.0;

    for run in buffer.layout_runs() {
        width = width.max(run.line_w);
        height = height.max(run.line_top + run.line_height);
    }

    (width, height)
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;
    use dusty_style::FontWeight;

    #[test]
    fn text_system_creates_successfully() {
        let _system = TextSystem::new();
    }

    #[test]
    fn text_system_default_works() {
        let _system = TextSystem::default();
    }

    #[test]
    fn measure_empty_string_returns_zero_width() {
        let system = TextSystem::new();
        let (w, _h) = system.measure("", None, &FontStyle::default());
        assert_eq!(w, 0.0);
    }

    #[test]
    fn measure_nonempty_returns_positive_dimensions() {
        let system = TextSystem::new();
        let (w, h) = system.measure("hello world", None, &FontStyle::default());
        assert!(w > 0.0, "width should be positive, got {w}");
        assert!(h > 0.0, "height should be positive, got {h}");
    }

    #[test]
    fn longer_text_is_wider() {
        let system = TextSystem::new();
        let font = FontStyle::default();
        let (short_w, _) = system.measure("hi", None, &font);
        let (long_w, _) = system.measure("hello world this is longer", None, &font);
        assert!(
            long_w > short_w,
            "longer text should be wider: {long_w} > {short_w}"
        );
    }

    #[test]
    fn larger_font_is_taller() {
        let system = TextSystem::new();
        let small = FontStyle {
            size: Some(12.0),
            ..FontStyle::default()
        };
        let large = FontStyle {
            size: Some(48.0),
            ..FontStyle::default()
        };
        let (_, small_h) = system.measure("hello", None, &small);
        let (_, large_h) = system.measure("hello", None, &large);
        assert!(
            large_h > small_h,
            "larger font should be taller: {large_h} > {small_h}"
        );
    }

    #[test]
    fn max_width_constrains_wrapping() {
        let system = TextSystem::new();
        let font = FontStyle::default();
        let text = "hello world this is a longer piece of text that should wrap";

        let (unconstrained_w, _) = system.measure(text, None, &font);
        let (constrained_w, constrained_h) = system.measure(text, Some(100.0), &font);
        let (_, unconstrained_h) = system.measure(text, None, &font);

        // Constrained width should be <= max_width
        assert!(
            constrained_w <= 100.5, // small tolerance for glyph overflow
            "constrained width should be near max_width: {constrained_w}"
        );

        // Wrapping should make text taller
        if unconstrained_w > 100.0 {
            assert!(
                constrained_h > unconstrained_h,
                "wrapped text should be taller: {constrained_h} > {unconstrained_h}"
            );
        }
    }

    #[test]
    fn measure_with_bold_weight() {
        let system = TextSystem::new();
        let font = FontStyle {
            weight: Some(FontWeight::BOLD),
            ..FontStyle::default()
        };
        let (w, h) = system.measure("hello", None, &font);
        assert!(w > 0.0);
        assert!(h > 0.0);
    }

    #[test]
    fn text_system_is_not_send_or_sync() {
        // TextSystem wraps RefCell<FontSystem> and must remain !Send + !Sync.
        // Compile-time verification: the functions below require Send/Sync bounds.
        // If TextSystem ever became Send or Sync, uncommenting the calls
        // would compile — but they must NOT compile.
        fn _require_send<T: Send>() {}
        fn _require_sync<T: Sync>() {}
        // _require_send::<TextSystem>(); // must not compile
        // _require_sync::<TextSystem>(); // must not compile

        // We verify this invariant via compile_fail doctests on the struct.
    }

    // --- truncation tests ---

    #[test]
    fn truncate_none_returns_original_text() {
        let system = TextSystem::new();
        let font = FontStyle::default();
        let result = system.truncate("hello world", 1000.0, &font, Truncation::None);
        assert_eq!(result.text, "hello world");
        assert!(!result.was_truncated);
        assert!(result.size.0 > 0.0);
        assert!(result.size.1 > 0.0);
    }

    #[test]
    fn truncate_ellipsis_text_fits_returns_original() {
        let system = TextSystem::new();
        let font = FontStyle::default();
        let result = system.truncate("hi", 1000.0, &font, Truncation::Ellipsis);
        assert_eq!(result.text, "hi");
        assert!(!result.was_truncated);
    }

    #[test]
    fn truncate_ellipsis_long_text_gets_truncated() {
        let system = TextSystem::new();
        let font = FontStyle::default();
        let text = "hello world this is a long string that should be truncated";
        let result = system.truncate(text, 80.0, &font, Truncation::Ellipsis);
        assert!(result.was_truncated, "should be truncated");
        assert!(
            result.text.ends_with('…'),
            "should end with ellipsis: {:?}",
            result.text
        );
        assert!(
            result.text.len() < text.len(),
            "truncated should be shorter"
        );
    }

    #[test]
    fn truncate_ellipsis_width_within_budget() {
        let system = TextSystem::new();
        let font = FontStyle::default();
        let text = "hello world this is a long string that should be truncated";
        let max_width = 80.0;
        let result = system.truncate(text, max_width, &font, Truncation::Ellipsis);
        assert!(
            result.size.0 <= max_width + 1.0, // small tolerance for glyph overflow
            "truncated width {} should be <= max_width {}",
            result.size.0,
            max_width
        );
    }

    #[test]
    fn truncate_empty_string_returns_empty() {
        let system = TextSystem::new();
        let font = FontStyle::default();
        let result = system.truncate("", 80.0, &font, Truncation::Ellipsis);
        assert_eq!(result.text, "");
        assert!(!result.was_truncated);
    }

    #[test]
    fn truncate_ellipsis_preserves_char_boundaries() {
        let system = TextSystem::new();
        let font = FontStyle::default();
        // Multi-byte UTF-8 characters
        let text = "héllo wörld thïs ïs lõng ënoügh tö trûncáte";
        let result = system.truncate(text, 80.0, &font, Truncation::Ellipsis);
        // Should not panic from slicing mid-character
        assert!(result.text.ends_with('…') || !result.was_truncated);
    }

    #[test]
    fn truncate_none_even_when_overflowing() {
        let system = TextSystem::new();
        let font = FontStyle::default();
        let text = "this is very long text";
        let result = system.truncate(text, 10.0, &font, Truncation::None);
        assert_eq!(result.text, text);
        assert!(!result.was_truncated);
    }

    #[test]
    fn text_measure_trait_object_works() {
        let system = TextSystem::new();
        let measure: &dyn TextMeasure = &system;
        let (w, h) = measure.measure("hello", None, &FontStyle::default());
        assert!(w > 0.0);
        assert!(h > 0.0);
    }

    #[test]
    fn borrow_conflict_returns_zero_without_panic() {
        let system = TextSystem::new();
        // Hold a mutable borrow to simulate a conflict
        let _guard = system.font_system_mut().unwrap();
        // In release mode, this should return (0.0, 0.0) without panicking
        let (w, h) = system.measure("hello", None, &FontStyle::default());
        assert_eq!(w, 0.0);
        assert_eq!(h, 0.0);
    }

    #[test]
    fn measure_rich_plain_matches_measure() {
        let system = TextSystem::new();
        let font = FontStyle::default();
        let text = "hello";

        let (plain_w, plain_h) = system.measure(text, None, &font);
        let spans = [TextSpan::new(text)];
        let (rich_w, rich_h) = system.measure_rich(&spans, None, &font).unwrap();

        // Should be very close (same text, same style)
        assert!(
            (plain_w - rich_w).abs() < 1.0,
            "plain and rich width should match: {plain_w} vs {rich_w}"
        );
        assert!(
            (plain_h - rich_h).abs() < 1.0,
            "plain and rich height should match: {plain_h} vs {rich_h}"
        );
    }
}
