//! Ellipsis truncation for text that overflows its container.

/// Text truncation strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Truncation {
    /// No truncation — text may overflow.
    #[default]
    None,
    /// Truncate with an ellipsis ("…") when text overflows.
    Ellipsis,
}

/// Result of a truncation operation.
#[derive(Debug, Clone, PartialEq)]
pub struct TruncatedText {
    /// The (possibly truncated) text content.
    pub text: String,
    /// Whether the text was actually truncated.
    pub was_truncated: bool,
    /// The measured size of the truncated text.
    pub size: (f32, f32),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncation_default_is_none() {
        assert_eq!(Truncation::default(), Truncation::None);
    }

    #[test]
    fn truncated_text_fields() {
        let result = TruncatedText {
            text: "hello…".into(),
            was_truncated: true,
            size: (50.0, 16.0),
        };
        assert_eq!(result.text, "hello…");
        assert!(result.was_truncated);
        assert_eq!(result.size, (50.0, 16.0));
    }

    #[test]
    fn truncated_text_not_truncated() {
        let result = TruncatedText {
            text: "short".into(),
            was_truncated: false,
            size: (40.0, 16.0),
        };
        assert!(!result.was_truncated);
    }
}
