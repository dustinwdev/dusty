//! Platform error types.

use std::fmt;

/// Errors that can occur in the platform layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformError {
    /// Failed to create the event loop.
    EventLoopCreation(String),
    /// Failed to create a window.
    WindowCreation(String),
    /// The event loop exited with an error.
    EventLoopExit(String),
    /// A clipboard operation failed.
    ClipboardError(String),
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EventLoopCreation(msg) => write!(f, "event loop creation failed: {msg}"),
            Self::WindowCreation(msg) => write!(f, "window creation failed: {msg}"),
            Self::EventLoopExit(msg) => write!(f, "event loop exited with error: {msg}"),
            Self::ClipboardError(msg) => write!(f, "clipboard error: {msg}"),
        }
    }
}

impl std::error::Error for PlatformError {}

impl From<arboard::Error> for PlatformError {
    fn from(err: arboard::Error) -> Self {
        Self::ClipboardError(format!("{err}"))
    }
}

/// A specialized `Result` type for platform operations.
pub type Result<T> = std::result::Result<T, PlatformError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_event_loop_creation() {
        let err = PlatformError::EventLoopCreation("no display".into());
        assert_eq!(err.to_string(), "event loop creation failed: no display");
    }

    #[test]
    fn display_window_creation() {
        let err = PlatformError::WindowCreation("bad config".into());
        assert_eq!(err.to_string(), "window creation failed: bad config");
    }

    #[test]
    fn display_event_loop_exit() {
        let err = PlatformError::EventLoopExit("crash".into());
        assert_eq!(err.to_string(), "event loop exited with error: crash");
    }

    #[test]
    fn display_clipboard_error() {
        let err = PlatformError::ClipboardError("not available".into());
        assert_eq!(err.to_string(), "clipboard error: not available");
    }

    #[test]
    fn implements_std_error() {
        let err = PlatformError::EventLoopCreation("test".into());
        let std_err: &dyn std::error::Error = &err;
        assert!(std_err.source().is_none());
    }

    #[test]
    fn equality() {
        assert_eq!(
            PlatformError::EventLoopCreation("a".into()),
            PlatformError::EventLoopCreation("a".into())
        );
        assert_ne!(
            PlatformError::EventLoopCreation("a".into()),
            PlatformError::WindowCreation("a".into())
        );
    }

    #[test]
    fn from_arboard_error() {
        let arboard_err = arboard::Error::ContentNotAvailable;
        let platform_err: PlatformError = arboard_err.into();
        assert!(matches!(platform_err, PlatformError::ClipboardError(_)));
    }
}
