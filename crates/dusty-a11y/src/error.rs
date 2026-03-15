//! Accessibility error types.

use std::fmt;

/// Errors that can occur during accessibility tree generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum A11yError {
    /// The node tree is empty — nothing to build an accessibility tree from.
    EmptyTree,
}

impl fmt::Display for A11yError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTree => write!(f, "accessibility tree is empty"),
        }
    }
}

impl std::error::Error for A11yError {}

/// A specialized `Result` type for accessibility operations.
pub type Result<T> = std::result::Result<T, A11yError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_empty_tree() {
        let err = A11yError::EmptyTree;
        assert_eq!(err.to_string(), "accessibility tree is empty");
    }

    #[test]
    fn error_trait_implemented() {
        let err = A11yError::EmptyTree;
        let std_err: &dyn std::error::Error = &err;
        assert!(std_err.source().is_none());
    }

    #[test]
    fn debug_output() {
        let err = A11yError::EmptyTree;
        let debug = format!("{err:?}");
        assert!(debug.contains("EmptyTree"));
    }

    #[test]
    fn equality() {
        assert_eq!(A11yError::EmptyTree, A11yError::EmptyTree);
    }
}
