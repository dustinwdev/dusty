use proc_macro2::Span;
use syn::Error;

/// Creates a compile error at the given span.
pub fn err(span: Span, msg: impl std::fmt::Display) -> Error {
    Error::new(span, msg)
}
