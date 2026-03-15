//! Proc macros for Dusty components and widgets.

mod component;
mod error;
mod prop;

use proc_macro::TokenStream;

/// Transforms a function into a Dusty component with an auto-generated props
/// struct, builder pattern, and `View` implementation.
///
/// # Example
///
/// ```ignore
/// use dusty_macros::component;
/// use dusty_reactive::Scope;
/// use dusty_core::node::Node;
///
/// #[component]
/// fn Greeting(cx: Scope, name: String) -> Node {
///     dusty_core::el("Greeting", cx)
///         .child(dusty_core::text(format!("Hello, {}!", name)))
///         .build_node()
/// }
///
/// // Generates:
/// // pub struct Greeting { name: String }
/// // impl Greeting { pub fn new(name: impl Into<String>) -> Self { ... } }
/// // impl View for Greeting { fn build(self, cx: Scope) -> Node { ... } }
/// ```
#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    component::expand(item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
