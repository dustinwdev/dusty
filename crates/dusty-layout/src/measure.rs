//! Text measurement trait.

use dusty_style::FontStyle;

/// Callback for measuring text during layout.
///
/// Decouples the layout engine from text rendering. The actual implementation
/// lives in `dusty-text`; tests use a mock.
///
/// # Examples
///
/// ```
/// use dusty_layout::TextMeasure;
/// use dusty_style::FontStyle;
///
/// struct FixedMeasure;
/// impl TextMeasure for FixedMeasure {
///     fn measure(&self, _text: &str, _max_width: Option<f32>, _font: &FontStyle) -> (f32, f32) {
///         (100.0, 16.0)
///     }
/// }
/// ```
pub trait TextMeasure {
    /// Measures the given text, returning `(width, height)`.
    ///
    /// - `max_width`: if `Some`, text should wrap at this width.
    /// - `font`: font style properties for sizing.
    ///
    /// Implementations must not panic. If measurement fails (e.g. due to a
    /// borrow conflict), return `(0.0, 0.0)` and log the error.
    fn measure(&self, text: &str, max_width: Option<f32>, font: &FontStyle) -> (f32, f32);
}

#[cfg(test)]
#[allow(
    clippy::float_cmp,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
mod tests {
    use super::*;

    struct MockMeasure;
    impl TextMeasure for MockMeasure {
        fn measure(&self, text: &str, max_width: Option<f32>, _font: &FontStyle) -> (f32, f32) {
            let char_width = 8.0;
            let line_height = 16.0;
            let text_width = text.len() as f32 * char_width;
            if let Some(max) = max_width {
                if text_width > max {
                    let chars_per_line = (max / char_width).floor() as usize;
                    if chars_per_line == 0 {
                        return (char_width, line_height);
                    }
                    let lines = text.len().div_ceil(chars_per_line);
                    return (max, lines as f32 * line_height);
                }
            }
            (text_width, line_height)
        }
    }

    #[test]
    fn mock_measure_simple() {
        let m = MockMeasure;
        let (w, h) = m.measure("hello", None, &FontStyle::default());
        assert_eq!(w, 40.0); // 5 * 8
        assert_eq!(h, 16.0);
    }

    #[test]
    fn mock_measure_with_max_width() {
        let m = MockMeasure;
        let (w, h) = m.measure("hello world", Some(48.0), &FontStyle::default());
        // 11 chars, 48px max → 6 chars/line → 2 lines
        assert_eq!(w, 48.0);
        assert_eq!(h, 32.0);
    }

    #[test]
    fn trait_object_works() {
        let m: &dyn TextMeasure = &MockMeasure;
        let (w, _h) = m.measure("ab", None, &FontStyle::default());
        assert_eq!(w, 16.0);
    }
}
