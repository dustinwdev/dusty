//! Error types for the reactive runtime.

use core::fmt;

/// Errors that can occur when interacting with the reactive runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReactiveError {
    /// No runtime has been initialized on this thread.
    NoRuntime,
    /// The signal has been disposed and is no longer valid.
    SignalDisposed,
    /// The memo has been disposed and is no longer valid.
    MemoDisposed,
    /// The effect has been disposed and is no longer valid.
    EffectDisposed,
    /// The resource has been disposed and is no longer valid.
    ResourceDisposed,
    /// The scope has been disposed and is no longer valid.
    ScopeDisposed,
    /// A cyclic dependency was detected among memos.
    CyclicDependency,
    /// A downcast failed due to a type mismatch.
    ///
    /// This should be unreachable through the safe public API — it indicates
    /// an internal bug where a signal slot was accessed with the wrong type.
    TypeMismatch,
    /// The runtime is already borrowed (re-entrancy conflict).
    RuntimeBorrowError,
}

impl fmt::Display for ReactiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoRuntime => write!(f, "no reactive runtime initialized on this thread"),
            Self::SignalDisposed => write!(f, "signal has been disposed"),
            Self::MemoDisposed => write!(f, "memo has been disposed"),
            Self::EffectDisposed => write!(f, "effect has been disposed"),
            Self::ResourceDisposed => write!(f, "resource has been disposed"),
            Self::ScopeDisposed => write!(f, "scope has been disposed"),
            Self::CyclicDependency => write!(f, "cyclic dependency detected among memos"),
            Self::TypeMismatch => write!(f, "type mismatch on signal slot downcast"),
            Self::RuntimeBorrowError => {
                write!(f, "runtime is already borrowed (re-entrancy conflict)")
            }
        }
    }
}

impl std::error::Error for ReactiveError {}

/// A specialized `Result` type for reactive operations.
pub type Result<T> = std::result::Result<T, ReactiveError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_no_runtime() {
        let err = ReactiveError::NoRuntime;
        assert_eq!(
            err.to_string(),
            "no reactive runtime initialized on this thread"
        );
    }

    #[test]
    fn display_signal_disposed() {
        let err = ReactiveError::SignalDisposed;
        assert_eq!(err.to_string(), "signal has been disposed");
    }

    #[test]
    fn display_memo_disposed() {
        let err = ReactiveError::MemoDisposed;
        assert_eq!(err.to_string(), "memo has been disposed");
    }

    #[test]
    fn display_effect_disposed() {
        let err = ReactiveError::EffectDisposed;
        assert_eq!(err.to_string(), "effect has been disposed");
    }

    #[test]
    fn display_resource_disposed() {
        let err = ReactiveError::ResourceDisposed;
        assert_eq!(err.to_string(), "resource has been disposed");
    }

    #[test]
    fn display_type_mismatch() {
        let err = ReactiveError::TypeMismatch;
        assert_eq!(err.to_string(), "type mismatch on signal slot downcast");
    }

    #[test]
    fn display_runtime_borrow_error() {
        let err = ReactiveError::RuntimeBorrowError;
        assert_eq!(
            err.to_string(),
            "runtime is already borrowed (re-entrancy conflict)"
        );
    }

    #[test]
    fn implements_std_error() {
        let err: &dyn std::error::Error = &ReactiveError::NoRuntime;
        // source() returns None for leaf errors
        assert!(err.source().is_none());
    }

    #[test]
    fn errors_are_copy() {
        let err = ReactiveError::SignalDisposed;
        let err2 = err;
        assert_eq!(err, err2);
    }

    #[test]
    fn result_alias_works() {
        let ok: Result<i32> = Ok(42);
        assert_eq!(ok.unwrap(), 42);

        let err: Result<i32> = Err(ReactiveError::NoRuntime);
        assert!(err.is_err());
    }
}
