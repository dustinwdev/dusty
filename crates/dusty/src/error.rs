//! Top-level error type for the Dusty framework.

use std::fmt;

use dusty_platform::PlatformError;
use dusty_reactive::ReactiveError;

/// Errors that can occur when running a Dusty application.
///
/// # Example
///
/// ```
/// use dusty::DustyError;
///
/// let err = DustyError::NoRoot;
/// assert_eq!(err.to_string(), "no root component provided");
/// ```
#[derive(Debug)]
pub enum DustyError {
    /// No root component was provided to the app builder.
    NoRoot,
    /// An error from the reactive runtime.
    Reactive(ReactiveError),
    /// An error from the platform layer.
    Platform(PlatformError),
}

impl fmt::Display for DustyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoRoot => write!(f, "no root component provided"),
            Self::Reactive(err) => write!(f, "reactive error: {err}"),
            Self::Platform(err) => write!(f, "platform error: {err}"),
        }
    }
}

impl std::error::Error for DustyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NoRoot => None,
            Self::Reactive(err) => Some(err),
            Self::Platform(err) => Some(err),
        }
    }
}

impl From<ReactiveError> for DustyError {
    fn from(err: ReactiveError) -> Self {
        Self::Reactive(err)
    }
}

impl From<PlatformError> for DustyError {
    fn from(err: PlatformError) -> Self {
        Self::Platform(err)
    }
}

/// A specialized `Result` type for Dusty operations.
pub type Result<T> = std::result::Result<T, DustyError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_no_root() {
        let err = DustyError::NoRoot;
        assert_eq!(err.to_string(), "no root component provided");
    }

    #[test]
    fn display_reactive() {
        let err = DustyError::Reactive(ReactiveError::NoRuntime);
        assert_eq!(
            err.to_string(),
            "reactive error: no reactive runtime initialized on this thread"
        );
    }

    #[test]
    fn display_platform() {
        let err = DustyError::Platform(PlatformError::EventLoopCreation("test".into()));
        assert_eq!(
            err.to_string(),
            "platform error: event loop creation failed: test"
        );
    }

    #[test]
    fn source_no_root_is_none() {
        let err = DustyError::NoRoot;
        assert!(std::error::Error::source(&err).is_none());
    }

    #[test]
    fn source_reactive_delegates() {
        let err = DustyError::Reactive(ReactiveError::NoRuntime);
        let source = std::error::Error::source(&err);
        assert!(source.is_some());
        assert_eq!(
            source.map(ToString::to_string),
            Some("no reactive runtime initialized on this thread".to_string())
        );
    }

    #[test]
    fn source_platform_delegates() {
        let err = DustyError::Platform(PlatformError::EventLoopCreation("fail".into()));
        let source = std::error::Error::source(&err);
        assert!(source.is_some());
        assert_eq!(
            source.map(ToString::to_string),
            Some("event loop creation failed: fail".to_string())
        );
    }

    #[test]
    fn from_reactive_error() {
        let err: DustyError = ReactiveError::NoRuntime.into();
        assert!(matches!(
            err,
            DustyError::Reactive(ReactiveError::NoRuntime)
        ));
    }

    #[test]
    fn from_platform_error() {
        let err: DustyError = PlatformError::EventLoopCreation("x".into()).into();
        assert!(matches!(err, DustyError::Platform(_)));
    }

    #[test]
    fn result_alias_works() {
        let ok: Result<i32> = Ok(42);
        assert!(ok.is_ok());

        let err: Result<i32> = Err(DustyError::NoRoot);
        assert!(err.is_err());
    }
}
