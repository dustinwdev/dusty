use std::fmt;

use dusty_core::CoreError;

/// Errors that can occur in the widget layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WidgetError {
    /// An error from the core layer.
    Core(CoreError),
}

impl fmt::Display for WidgetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Core(err) => write!(f, "core error: {err}"),
        }
    }
}

impl std::error::Error for WidgetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Core(err) => Some(err),
        }
    }
}

impl From<CoreError> for WidgetError {
    fn from(err: CoreError) -> Self {
        Self::Core(err)
    }
}

/// A specialized `Result` type for widget operations.
pub type Result<T> = std::result::Result<T, WidgetError>;

#[cfg(test)]
mod tests {
    use super::*;
    use dusty_core::CoreError;
    use dusty_reactive::ReactiveError;

    #[test]
    fn display_core_variant() {
        let err = WidgetError::Core(CoreError::InvalidTargetPath);
        assert!(err.to_string().contains("core error"));
    }

    #[test]
    fn from_core_error() {
        let core_err = CoreError::Reactive(ReactiveError::NoRuntime);
        let widget_err: WidgetError = core_err.into();
        assert!(matches!(widget_err, WidgetError::Core(_)));
    }

    #[test]
    fn implements_std_error() {
        let err = WidgetError::Core(CoreError::InvalidTargetPath);
        let std_err: &dyn std::error::Error = &err;
        assert!(std_err.source().is_some());
    }
}
