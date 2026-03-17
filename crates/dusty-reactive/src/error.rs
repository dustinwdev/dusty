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

/// Unwrap a reactive `Result`, panicking with a diagnostic message on `Err`.
///
/// The panic message includes the operation name, error variant, and a hint
/// to use the `try_*` fallible variant for error handling.
///
/// `#[track_caller]` ensures the panic reports the user's call site, not this
/// function.
///
/// # Panics
///
/// Panics if `result` is `Err`.
#[track_caller]
#[inline]
pub fn unwrap_reactive<T>(result: Result<T>, context: &str) -> T {
    match result {
        Ok(val) => val,
        Err(err) => panic!(
            "dusty reactive error in `{context}`: {err}\n\
             \n\
             This is a programming bug — common causes:\n\
             - No runtime: call `initialize_runtime()` before using reactive primitives\n\
             - Use-after-dispose: a signal/memo/effect/scope was used after its scope was disposed\n\
             - Re-entrant borrow: signal read/write from inside a signal write\n\
             \n\
             Hint: use the `try_{fn_name}` variant if you need a Result instead of a panic",
            fn_name = context
                .rsplit("::")
                .next()
                .unwrap_or(context)
                .trim_start_matches("try_"),
        ),
    }
}

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

    #[test]
    fn unwrap_reactive_returns_ok_value() {
        let result: Result<i32> = Ok(42);
        assert_eq!(unwrap_reactive(result, "Signal::get"), 42);
    }

    #[test]
    #[should_panic(expected = "dusty reactive error in `Signal::get`")]
    fn unwrap_reactive_panics_on_err() {
        let result: Result<i32> = Err(ReactiveError::SignalDisposed);
        unwrap_reactive(result, "Signal::get");
    }

    #[test]
    #[should_panic(expected = "try_get")]
    fn unwrap_reactive_panic_message_includes_try_hint() {
        let result: Result<i32> = Err(ReactiveError::NoRuntime);
        unwrap_reactive(result, "Signal::get");
    }
}
