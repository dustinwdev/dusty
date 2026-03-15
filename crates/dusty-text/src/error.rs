//! Text error types.

use std::fmt;

/// Errors that can occur during text operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextError {
    /// No fonts matched the requested family.
    FontNotFound(String),
    /// The internal `FontSystem` is already borrowed (re-entrant access).
    BorrowConflict,
    /// Invalid font metrics (e.g. non-positive font size or line height).
    InvalidMetrics(String),
    /// Text shaping failed for the given input.
    ShapingFailed(String),
}

impl fmt::Display for TextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FontNotFound(family) => write!(f, "font not found: {family}"),
            Self::BorrowConflict => write!(f, "font system already borrowed"),
            Self::InvalidMetrics(detail) => write!(f, "invalid metrics: {detail}"),
            Self::ShapingFailed(detail) => write!(f, "shaping failed: {detail}"),
        }
    }
}

impl std::error::Error for TextError {}

/// A specialized `Result` type for text operations.
pub type Result<T> = std::result::Result<T, TextError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_font_not_found() {
        let err = TextError::FontNotFound("Inter".into());
        assert_eq!(err.to_string(), "font not found: Inter");
    }

    #[test]
    fn implements_std_error() {
        let err = TextError::FontNotFound("Arial".into());
        let std_err: &dyn std::error::Error = &err;
        assert!(std_err.source().is_none());
    }

    #[test]
    fn equality() {
        assert_eq!(
            TextError::FontNotFound("A".into()),
            TextError::FontNotFound("A".into())
        );
        assert_ne!(
            TextError::FontNotFound("A".into()),
            TextError::FontNotFound("B".into())
        );
    }

    #[test]
    fn debug_format() {
        let err = TextError::FontNotFound("Roboto".into());
        let debug = format!("{err:?}");
        assert!(debug.contains("FontNotFound"));
        assert!(debug.contains("Roboto"));
    }

    #[test]
    fn display_invalid_metrics() {
        let err = TextError::InvalidMetrics("font size must be positive".into());
        assert_eq!(
            err.to_string(),
            "invalid metrics: font size must be positive"
        );
    }

    #[test]
    fn display_shaping_failed() {
        let err = TextError::ShapingFailed("no glyphs for input".into());
        assert_eq!(err.to_string(), "shaping failed: no glyphs for input");
    }

    #[test]
    fn display_borrow_conflict() {
        let err = TextError::BorrowConflict;
        assert_eq!(err.to_string(), "font system already borrowed");
    }

    #[test]
    fn invalid_metrics_equality() {
        assert_eq!(
            TextError::InvalidMetrics("a".into()),
            TextError::InvalidMetrics("a".into())
        );
        assert_ne!(
            TextError::InvalidMetrics("a".into()),
            TextError::InvalidMetrics("b".into())
        );
    }

    #[test]
    fn shaping_failed_equality() {
        assert_eq!(
            TextError::ShapingFailed("x".into()),
            TextError::ShapingFailed("x".into())
        );
        assert_ne!(
            TextError::ShapingFailed("x".into()),
            TextError::ShapingFailed("y".into())
        );
    }

    #[test]
    fn different_variants_not_equal() {
        assert_ne!(
            TextError::FontNotFound("Inter".into()),
            TextError::InvalidMetrics("Inter".into())
        );
        assert_ne!(
            TextError::FontNotFound("Inter".into()),
            TextError::ShapingFailed("Inter".into())
        );
        assert_ne!(
            TextError::InvalidMetrics("x".into()),
            TextError::ShapingFailed("x".into())
        );
    }

    #[test]
    fn new_variants_implement_std_error() {
        let err1: &dyn std::error::Error = &TextError::InvalidMetrics("bad".into());
        assert!(err1.source().is_none());

        let err2: &dyn std::error::Error = &TextError::ShapingFailed("bad".into());
        assert!(err2.source().is_none());
    }

    #[test]
    fn debug_format_new_variants() {
        let err = TextError::InvalidMetrics("zero size".into());
        let debug = format!("{err:?}");
        assert!(debug.contains("InvalidMetrics"));
        assert!(debug.contains("zero size"));

        let err = TextError::ShapingFailed("no glyphs".into());
        let debug = format!("{err:?}");
        assert!(debug.contains("ShapingFailed"));
        assert!(debug.contains("no glyphs"));
    }
}
