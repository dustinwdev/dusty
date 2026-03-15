//! Layout error types.

use std::fmt;

/// Errors that can occur during layout computation.
#[derive(Debug)]
pub enum LayoutError {
    /// The node tree is empty — nothing to lay out.
    EmptyTree,
    /// An error from the taffy layout engine.
    TaffyError(taffy::TaffyError),
    /// Failed to downcast an element's style to `dusty_style::Style`.
    StyleDowncastFailed,
}

impl fmt::Display for LayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTree => write!(f, "layout tree is empty"),
            Self::TaffyError(err) => write!(f, "taffy layout error: {err}"),
            Self::StyleDowncastFailed => {
                write!(f, "failed to downcast style to dusty_style::Style")
            }
        }
    }
}

impl std::error::Error for LayoutError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::TaffyError(err) => Some(err),
            _ => None,
        }
    }
}

impl PartialEq for LayoutError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::EmptyTree, Self::EmptyTree)
                | (Self::StyleDowncastFailed, Self::StyleDowncastFailed)
                | (Self::TaffyError(_), Self::TaffyError(_))
        )
    }
}

impl Eq for LayoutError {}

impl From<taffy::TaffyError> for LayoutError {
    fn from(err: taffy::TaffyError) -> Self {
        Self::TaffyError(err)
    }
}

/// A specialized `Result` type for layout operations.
pub type Result<T> = std::result::Result<T, LayoutError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_empty_tree() {
        let err = LayoutError::EmptyTree;
        assert_eq!(err.to_string(), "layout tree is empty");
    }

    #[test]
    fn display_taffy_error() {
        let taffy_err = taffy::TaffyError::InvalidInputNode(taffy::NodeId::new(0));
        let err = LayoutError::TaffyError(taffy_err);
        assert!(err.to_string().starts_with("taffy layout error:"));
    }

    #[test]
    fn display_style_downcast_failed() {
        let err = LayoutError::StyleDowncastFailed;
        assert_eq!(
            err.to_string(),
            "failed to downcast style to dusty_style::Style"
        );
    }

    #[test]
    fn implements_std_error() {
        let err = LayoutError::EmptyTree;
        let std_err: &dyn std::error::Error = &err;
        assert!(std_err.source().is_none());
    }

    #[test]
    fn taffy_error_chains_source() {
        let taffy_err = taffy::TaffyError::InvalidInputNode(taffy::NodeId::new(0));
        let err = LayoutError::TaffyError(taffy_err);
        let std_err: &dyn std::error::Error = &err;
        assert!(std_err.source().is_some());
    }

    #[test]
    fn equality() {
        assert_eq!(LayoutError::EmptyTree, LayoutError::EmptyTree);
        assert_ne!(LayoutError::EmptyTree, LayoutError::StyleDowncastFailed);
    }

    #[test]
    fn from_taffy_error() {
        let taffy_err = taffy::TaffyError::InvalidInputNode(taffy::NodeId::new(0));
        let layout_err: LayoutError = taffy_err.into();
        assert!(matches!(layout_err, LayoutError::TaffyError(_)));
    }
}
