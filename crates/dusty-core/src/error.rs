use std::fmt;

use dusty_reactive::ReactiveError;

/// Errors that can occur in the core view/node layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreError {
    /// An error from the reactive layer.
    Reactive(ReactiveError),
    /// The target path used for event dispatch does not exist in the node tree.
    InvalidTargetPath,
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reactive(err) => write!(f, "reactive error: {err}"),
            Self::InvalidTargetPath => write!(f, "invalid target path for event dispatch"),
        }
    }
}

impl std::error::Error for CoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Reactive(err) => Some(err),
            Self::InvalidTargetPath => None,
        }
    }
}

impl From<ReactiveError> for CoreError {
    fn from(err: ReactiveError) -> Self {
        Self::Reactive(err)
    }
}

/// A specialized `Result` type for core operations.
pub type Result<T> = std::result::Result<T, CoreError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_reactive_variant() {
        let err = CoreError::Reactive(ReactiveError::NoRuntime);
        assert_eq!(
            err.to_string(),
            "reactive error: no reactive runtime initialized on this thread"
        );
    }

    #[test]
    fn from_reactive_error() {
        let reactive_err = ReactiveError::SignalDisposed;
        let core_err: CoreError = reactive_err.into();
        assert_eq!(core_err, CoreError::Reactive(ReactiveError::SignalDisposed));
    }

    #[test]
    fn implements_std_error() {
        let err = CoreError::Reactive(ReactiveError::NoRuntime);
        let std_err: &dyn std::error::Error = &err;
        assert!(std_err.source().is_some());
    }

    #[test]
    fn display_invalid_target_path() {
        let err = CoreError::InvalidTargetPath;
        assert_eq!(err.to_string(), "invalid target path for event dispatch");
    }

    #[test]
    fn invalid_target_path_has_no_source() {
        let err = CoreError::InvalidTargetPath;
        let std_err: &dyn std::error::Error = &err;
        assert!(std_err.source().is_none());
    }
}
