use dusty_reactive::Scope;

use crate::element::ElementBuilder;
use crate::node::{text, Node};

/// Core trait for anything that can be rendered in the view tree.
///
/// `View::build` consumes `self` and returns a [`Node`]. Views are built
/// once — subsequent updates happen through reactive signals, not rebuilds.
pub trait View {
    /// Builds this view into a node tree.
    fn build(self, cx: Scope) -> Node;
}

/// Conversion trait for types that can become a [`Node`].
///
/// Blanket-implemented for all [`View`] types. Also implemented directly
/// for common types like `&str`, `String`, `Node`, and `Option<T>`.
pub trait IntoView {
    /// Converts this value into a node.
    fn into_view(self, cx: Scope) -> Node;
}

// Blanket: every View is IntoView
impl<V: View> IntoView for V {
    fn into_view(self, cx: Scope) -> Node {
        self.build(cx)
    }
}

// &str → static text node
impl IntoView for &str {
    fn into_view(self, _cx: Scope) -> Node {
        Node::Text(text(self))
    }
}

// String → static text node
impl IntoView for String {
    fn into_view(self, _cx: Scope) -> Node {
        Node::Text(text(self))
    }
}

// TextNode → wraps in Node::Text
impl IntoView for crate::node::TextNode {
    fn into_view(self, _cx: Scope) -> Node {
        Node::Text(self)
    }
}

// Node passes through
impl IntoView for Node {
    fn into_view(self, _cx: Scope) -> Node {
        self
    }
}

// Option<T> → the view or an empty fragment
impl<T: IntoView> IntoView for Option<T> {
    fn into_view(self, cx: Scope) -> Node {
        self.map_or_else(|| Node::Fragment(vec![]), |v| v.into_view(cx))
    }
}

// ElementBuilder → builds into a Node::Element
impl View for ElementBuilder {
    fn build(self, _cx: Scope) -> Node {
        self.build_node()
    }
}

/// Builds a fragment from a [`ViewSeq`].
///
/// # Example
///
/// ```
/// use dusty_reactive::{initialize_runtime, create_scope, dispose_runtime};
/// use dusty_core::{fragment, el};
///
/// initialize_runtime();
/// create_scope(|cx| {
///     let node = fragment(("hello", el("Spacer", cx)), cx);
///     assert!(node.is_fragment());
/// }).unwrap();
/// dispose_runtime();
/// ```
#[must_use]
pub fn fragment(seq: impl crate::view_seq::ViewSeq, cx: Scope) -> Node {
    Node::Fragment(seq.build_seq(cx))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::text;
    use dusty_reactive::{create_scope, dispose_runtime, initialize_runtime};

    fn with_scope(f: impl FnOnce(Scope)) {
        initialize_runtime();
        create_scope(|cx| f(cx)).unwrap();
        dispose_runtime();
    }

    #[test]
    fn str_into_view() {
        with_scope(|cx| {
            let node = "hello".into_view(cx);
            assert!(node.is_text());
        });
    }

    #[test]
    fn string_into_view() {
        with_scope(|cx| {
            let node = String::from("world").into_view(cx);
            assert!(node.is_text());
        });
    }

    #[test]
    fn node_into_view() {
        with_scope(|cx| {
            let original = Node::Text(text("pass-through"));
            let node = original.into_view(cx);
            assert!(node.is_text());
        });
    }

    #[test]
    fn option_some_into_view() {
        with_scope(|cx| {
            let node = Some("present").into_view(cx);
            assert!(node.is_text());
        });
    }

    #[test]
    fn option_none_into_view() {
        with_scope(|cx| {
            let node: Option<&str> = None;
            let node = node.into_view(cx);
            assert!(node.is_fragment());
            assert_eq!(node.children().len(), 0);
        });
    }

    #[test]
    fn custom_view_struct() {
        struct Greeting {
            name: String,
        }

        impl View for Greeting {
            fn build(self, _cx: Scope) -> Node {
                Node::Text(text(format!("Hello, {}!", self.name)))
            }
        }

        with_scope(|cx| {
            let node = Greeting {
                name: "Dusty".into(),
            }
            .into_view(cx);
            assert!(node.is_text());
        });
    }

    #[test]
    fn element_builder_as_view() {
        with_scope(|cx| {
            let builder = crate::el("Box", cx).attr("padding", 8i64);
            let node = builder.into_view(cx);
            assert!(node.is_element());
        });
    }

    #[test]
    fn fragment_helper() {
        with_scope(|cx| {
            let node = fragment(("a", "b", "c"), cx);
            assert!(node.is_fragment());
            assert_eq!(node.children().len(), 3);
        });
    }
}
