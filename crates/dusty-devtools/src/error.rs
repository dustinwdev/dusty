//! Error types for devtools operations.

use std::fmt;

use dusty_reactive::ReactiveError;

/// Errors that can occur during devtools operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DevtoolsError {
    /// The node tree is empty — nothing to inspect or audit.
    EmptyTree,
    /// An error from the reactive runtime.
    Reactive(ReactiveError),
    /// The layout result does not match the node tree.
    LayoutMismatch,
}

impl fmt::Display for DevtoolsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTree => write!(f, "node tree is empty"),
            Self::Reactive(e) => write!(f, "reactive error: {e}"),
            Self::LayoutMismatch => write!(f, "layout result does not match node tree"),
        }
    }
}

impl std::error::Error for DevtoolsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Reactive(e) => Some(e),
            Self::EmptyTree | Self::LayoutMismatch => None,
        }
    }
}

impl From<ReactiveError> for DevtoolsError {
    fn from(err: ReactiveError) -> Self {
        Self::Reactive(err)
    }
}

/// A specialized `Result` type for devtools operations.
pub type Result<T> = std::result::Result<T, DevtoolsError>;

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn display_empty_tree() {
        let err = DevtoolsError::EmptyTree;
        assert_eq!(err.to_string(), "node tree is empty");
    }

    #[test]
    fn display_reactive() {
        let err = DevtoolsError::Reactive(ReactiveError::NoRuntime);
        assert!(err.to_string().contains("reactive error"));
        assert!(err.to_string().contains("no reactive runtime"));
    }

    #[test]
    fn display_layout_mismatch() {
        let err = DevtoolsError::LayoutMismatch;
        assert_eq!(err.to_string(), "layout result does not match node tree");
    }

    #[test]
    fn from_reactive_error() {
        let reactive_err = ReactiveError::NoRuntime;
        let devtools_err: DevtoolsError = reactive_err.into();
        assert_eq!(
            devtools_err,
            DevtoolsError::Reactive(ReactiveError::NoRuntime)
        );
    }

    #[test]
    fn source_returns_reactive_error() {
        let err = DevtoolsError::Reactive(ReactiveError::NoRuntime);
        let source = std::error::Error::source(&err);
        assert!(source.is_some());
    }

    #[test]
    fn source_returns_none_for_empty_tree() {
        let err = DevtoolsError::EmptyTree;
        let source = std::error::Error::source(&err);
        assert!(source.is_none());
    }

    #[test]
    fn source_returns_none_for_layout_mismatch() {
        let err = DevtoolsError::LayoutMismatch;
        let source = std::error::Error::source(&err);
        assert!(source.is_none());
    }

    #[test]
    fn result_alias_works() {
        let ok: Result<i32> = Ok(42);
        assert_eq!(ok.unwrap(), 42);

        let err: Result<i32> = Err(DevtoolsError::EmptyTree);
        assert!(err.is_err());
    }

    #[test]
    fn debug_output() {
        let err = DevtoolsError::EmptyTree;
        let debug = format!("{err:?}");
        assert!(debug.contains("EmptyTree"));
    }

    #[test]
    fn equality() {
        assert_eq!(DevtoolsError::EmptyTree, DevtoolsError::EmptyTree);
        assert_eq!(DevtoolsError::LayoutMismatch, DevtoolsError::LayoutMismatch);
        assert_ne!(DevtoolsError::EmptyTree, DevtoolsError::LayoutMismatch);
    }
}
