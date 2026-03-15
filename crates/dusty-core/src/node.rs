use std::borrow::Cow;
use std::fmt;

use crate::element::Element;

/// A node in the view tree.
pub enum Node {
    /// An element with attributes, styles, event handlers, and children.
    Element(Element),
    /// A text node.
    Text(TextNode),
    /// A fragment containing multiple sibling nodes.
    Fragment(Vec<Self>),
    /// A component wrapper with a debug name and its rendered child.
    Component(ComponentNode),
    /// A lazily-resolved subtree driven by reactive state.
    Dynamic(DynamicNode),
}

impl Node {
    /// Returns `true` if this is an `Element` node.
    #[must_use]
    pub const fn is_element(&self) -> bool {
        matches!(self, Self::Element(_))
    }

    /// Returns `true` if this is a `Text` node.
    #[must_use]
    pub const fn is_text(&self) -> bool {
        matches!(self, Self::Text(_))
    }

    /// Returns `true` if this is a `Fragment` node.
    #[must_use]
    pub const fn is_fragment(&self) -> bool {
        matches!(self, Self::Fragment(_))
    }

    /// Returns `true` if this is a `Component` node.
    #[must_use]
    pub const fn is_component(&self) -> bool {
        matches!(self, Self::Component(_))
    }

    /// Returns `true` if this is a `Dynamic` node.
    #[must_use]
    pub const fn is_dynamic(&self) -> bool {
        matches!(self, Self::Dynamic(_))
    }

    /// Returns the direct children of this node.
    ///
    /// - `Element` returns its children.
    /// - `Fragment` returns its items.
    /// - `Component` returns a slice containing the single child.
    /// - `Text` and `Dynamic` return an empty slice.
    #[must_use]
    pub fn children(&self) -> &[Self] {
        match self {
            Self::Element(el) => el.children(),
            Self::Fragment(nodes) => nodes,
            Self::Component(comp) => std::slice::from_ref(&comp.child),
            Self::Text(_) | Self::Dynamic(_) => &[],
        }
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Text(a), Self::Text(b)) => a == b,
            (Self::Fragment(a), Self::Fragment(b)) => a == b,
            (Self::Component(a), Self::Component(b)) => a.name == b.name && a.child == b.child,
            // Element contains Box<dyn Any> and closures; Dynamic contains a closure.
            // These cannot be compared structurally.
            _ => false,
        }
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Element(el) => f.debug_tuple("Element").field(el).finish(),
            Self::Text(text) => f.debug_tuple("Text").field(text).finish(),
            Self::Fragment(nodes) => f.debug_tuple("Fragment").field(nodes).finish(),
            Self::Component(comp) => f.debug_tuple("Component").field(comp).finish(),
            Self::Dynamic(_) => f.debug_tuple("Dynamic").field(&"<resolver>").finish(),
        }
    }
}

/// A text node in the view tree.
pub struct TextNode {
    /// The text content — static or dynamically computed.
    pub content: TextContent,
}

impl TextNode {
    /// Returns the current text value.
    ///
    /// For static text, returns a borrowed reference (no allocation).
    /// For dynamic text, calls the closure and returns an owned value.
    #[must_use]
    pub fn current_text(&self) -> Cow<'_, str> {
        match &self.content {
            TextContent::Static(s) => Cow::Borrowed(s),
            TextContent::Dynamic(f) => Cow::Owned(f()),
        }
    }
}

impl PartialEq for TextNode {
    fn eq(&self, other: &Self) -> bool {
        match (&self.content, &other.content) {
            (TextContent::Static(a), TextContent::Static(b)) => a == b,
            // Dynamic closures cannot be compared.
            _ => false,
        }
    }
}

impl fmt::Debug for TextNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextNode")
            .field("content", &self.content)
            .finish()
    }
}

/// The content of a text node.
pub enum TextContent {
    /// A fixed string.
    Static(String),
    /// A closure that computes the current text value.
    ///
    /// The renderer wraps this in an effect for automatic dependency tracking.
    Dynamic(Box<dyn Fn() -> String>),
}

impl fmt::Debug for TextContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Static(s) => f.debug_tuple("Static").field(s).finish(),
            Self::Dynamic(_) => f.debug_tuple("Dynamic").field(&"<closure>").finish(),
        }
    }
}

/// A component node — wraps a rendered child with a debug name.
pub struct ComponentNode {
    /// The component's name, for debugging and devtools.
    pub name: &'static str,
    /// The rendered output of this component.
    pub child: Box<Node>,
}

impl fmt::Debug for ComponentNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ComponentNode")
            .field("name", &self.name)
            .field("child", &self.child)
            .finish()
    }
}

/// Creates a static text node.
///
/// # Example
///
/// ```
/// use dusty_core::text;
///
/// let node = text("hello");
/// assert_eq!(node.current_text(), "hello");
/// ```
pub fn text(s: impl Into<String>) -> TextNode {
    TextNode {
        content: TextContent::Static(s.into()),
    }
}

/// Creates a dynamic text node from a closure.
///
/// The closure is called to compute the current text value. When used with
/// signals, the renderer wraps this in an effect for automatic updates.
///
/// # Example
///
/// ```
/// use dusty_core::text_dynamic;
///
/// let node = text_dynamic(|| "computed".to_string());
/// assert_eq!(node.current_text(), "computed");
/// ```
pub fn text_dynamic(f: impl Fn() -> String + 'static) -> TextNode {
    TextNode {
        content: TextContent::Dynamic(Box::new(f)),
    }
}

/// A lazily-resolved node whose subtree is determined at resolution time.
///
/// Used by container widgets like `Show`, `Match`, `For`, and `Suspense`
/// to produce subtrees that change structurally based on reactive state.
///
/// # Example
///
/// ```
/// use dusty_core::node::{dynamic_node, Node, text};
///
/// let dn = dynamic_node(|| Node::Text(text("resolved")));
/// let node = dn.current_node();
/// assert!(node.is_text());
/// ```
pub struct DynamicNode {
    pub(crate) resolver: Box<dyn Fn() -> Node>,
}

impl DynamicNode {
    /// Resolves the current subtree by calling the resolver closure.
    #[must_use]
    pub fn current_node(&self) -> Node {
        (self.resolver)()
    }
}

/// Creates a [`DynamicNode`] from a closure that produces a [`Node`].
///
/// # Example
///
/// ```
/// use dusty_core::node::{dynamic_node, Node, text};
///
/// let dn = dynamic_node(|| Node::Text(text("hello")));
/// assert!(dn.current_node().is_text());
/// ```
pub fn dynamic_node(f: impl Fn() -> Node + 'static) -> DynamicNode {
    DynamicNode {
        resolver: Box::new(f),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element::Element;

    #[test]
    fn text_node_static() {
        let node = text("hello");
        assert_eq!(node.current_text(), "hello");
    }

    #[test]
    fn text_node_dynamic() {
        let counter = std::cell::Cell::new(0);
        let node = text_dynamic(move || {
            counter.set(counter.get() + 1);
            format!("call #{}", counter.get())
        });
        assert_eq!(node.current_text(), "call #1");
        assert_eq!(node.current_text(), "call #2");
    }

    #[test]
    fn text_content_debug() {
        let static_content = TextContent::Static("hi".into());
        assert!(format!("{static_content:?}").contains("Static"));

        let dynamic_content = TextContent::Dynamic(Box::new(|| "x".into()));
        let debug = format!("{dynamic_content:?}");
        assert!(debug.contains("Dynamic"));
        assert!(debug.contains("<closure>"));
    }

    #[test]
    fn node_type_checks() {
        let el_node = Node::Element(Element::new("Div"));
        assert!(el_node.is_element());
        assert!(!el_node.is_text());
        assert!(!el_node.is_fragment());
        assert!(!el_node.is_component());

        let text_node = Node::Text(text("hi"));
        assert!(text_node.is_text());

        let frag = Node::Fragment(vec![]);
        assert!(frag.is_fragment());

        let comp = Node::Component(ComponentNode {
            name: "App",
            child: Box::new(Node::Fragment(vec![])),
        });
        assert!(comp.is_component());
    }

    #[test]
    fn node_children() {
        let mut el = Element::new("Row");
        el.push_child(Node::Text(text("a")));
        el.push_child(Node::Text(text("b")));
        let node = Node::Element(el);
        assert_eq!(node.children().len(), 2);

        let frag = Node::Fragment(vec![Node::Text(text("x"))]);
        assert_eq!(frag.children().len(), 1);

        let comp = Node::Component(ComponentNode {
            name: "Wrapper",
            child: Box::new(Node::Text(text("inner"))),
        });
        assert_eq!(comp.children().len(), 1);

        let text_node = Node::Text(text("leaf"));
        assert_eq!(text_node.children().len(), 0);
    }

    #[test]
    fn node_debug() {
        let node = Node::Text(text("hello"));
        let debug = format!("{node:?}");
        assert!(debug.contains("Text"));
        assert!(debug.contains("hello"));
    }

    #[test]
    fn dynamic_node_resolves_current() {
        let dn = dynamic_node(|| Node::Text(text("resolved")));
        let node = dn.current_node();
        assert!(node.is_text());
        if let Node::Text(t) = node {
            assert_eq!(t.current_text(), "resolved");
        }
    }

    #[test]
    fn dynamic_node_is_dynamic() {
        let node = Node::Dynamic(dynamic_node(|| Node::Fragment(vec![])));
        assert!(node.is_dynamic());
        assert!(!node.is_element());
        assert!(!node.is_text());
        assert!(!node.is_fragment());
        assert!(!node.is_component());
    }

    #[test]
    fn dynamic_node_children_empty() {
        let node = Node::Dynamic(dynamic_node(|| Node::Text(text("x"))));
        assert_eq!(node.children().len(), 0);
    }

    #[test]
    fn dynamic_node_debug() {
        let node = Node::Dynamic(dynamic_node(|| Node::Fragment(vec![])));
        let debug = format!("{node:?}");
        assert!(debug.contains("Dynamic"));
        assert!(debug.contains("<resolver>"));
    }

    #[test]
    fn current_text_static_returns_borrowed() {
        let node = text("hello");
        let cow = node.current_text();
        assert!(matches!(cow, std::borrow::Cow::Borrowed(_)));
        assert_eq!(&*cow, "hello");
    }

    #[test]
    fn current_text_dynamic_returns_owned() {
        let node = text_dynamic(|| "dynamic".to_string());
        let cow = node.current_text();
        assert!(matches!(cow, std::borrow::Cow::Owned(_)));
        assert_eq!(&*cow, "dynamic");
    }

    // --- PartialEq tests ---

    #[test]
    fn text_node_eq_static_same() {
        assert_eq!(text("hello"), text("hello"));
    }

    #[test]
    fn text_node_ne_static_different() {
        assert_ne!(text("hello"), text("world"));
    }

    #[test]
    fn text_node_ne_dynamic_always() {
        let a = TextNode {
            content: TextContent::Dynamic(Box::new(|| "same".to_string())),
        };
        let b = TextNode {
            content: TextContent::Dynamic(Box::new(|| "same".to_string())),
        };
        // Dynamic closures are never considered equal.
        assert_ne!(a, b);
    }

    #[test]
    fn text_node_ne_static_vs_dynamic() {
        let static_node = text("hello");
        let dynamic_node_val = TextNode {
            content: TextContent::Dynamic(Box::new(|| "hello".to_string())),
        };
        assert_ne!(static_node, dynamic_node_val);
    }

    #[test]
    fn node_eq_text_same() {
        assert_eq!(Node::Text(text("a")), Node::Text(text("a")));
    }

    #[test]
    fn node_ne_text_different() {
        assert_ne!(Node::Text(text("a")), Node::Text(text("b")));
    }

    #[test]
    fn node_eq_empty_fragment() {
        assert_eq!(Node::Fragment(vec![]), Node::Fragment(vec![]));
    }

    #[test]
    fn node_eq_fragment_with_children() {
        let a = Node::Fragment(vec![Node::Text(text("x")), Node::Text(text("y"))]);
        let b = Node::Fragment(vec![Node::Text(text("x")), Node::Text(text("y"))]);
        assert_eq!(a, b);
    }

    #[test]
    fn node_ne_fragment_different_children() {
        let a = Node::Fragment(vec![Node::Text(text("x"))]);
        let b = Node::Fragment(vec![Node::Text(text("y"))]);
        assert_ne!(a, b);
    }

    #[test]
    fn node_eq_component_same_name_and_child() {
        let a = Node::Component(ComponentNode {
            name: "App",
            child: Box::new(Node::Text(text("inner"))),
        });
        let b = Node::Component(ComponentNode {
            name: "App",
            child: Box::new(Node::Text(text("inner"))),
        });
        assert_eq!(a, b);
    }

    #[test]
    fn node_ne_component_different_name() {
        let a = Node::Component(ComponentNode {
            name: "App",
            child: Box::new(Node::Fragment(vec![])),
        });
        let b = Node::Component(ComponentNode {
            name: "Root",
            child: Box::new(Node::Fragment(vec![])),
        });
        assert_ne!(a, b);
    }

    #[test]
    fn node_ne_element_always() {
        let a = Node::Element(Element::new("Div"));
        let b = Node::Element(Element::new("Div"));
        // Element contains Box<dyn Any>, cannot be compared structurally.
        assert_ne!(a, b);
    }

    #[test]
    fn node_ne_dynamic_always() {
        let a = Node::Dynamic(dynamic_node(|| Node::Fragment(vec![])));
        let b = Node::Dynamic(dynamic_node(|| Node::Fragment(vec![])));
        assert_ne!(a, b);
    }

    #[test]
    fn node_ne_different_variants() {
        assert_ne!(Node::Text(text("a")), Node::Fragment(vec![]));
    }
}
