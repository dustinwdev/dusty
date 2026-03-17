//! A container widget that selects which arm to render based on a reactive value.
//!
//! Works like a `match` expression: a reactive value function is evaluated, and
//! the first arm whose key equals the current value produces the rendered node.
//! If no arm matches, the optional fallback is used; otherwise an empty fragment.

use dusty_core::node::{dynamic_node, ComponentNode, Node};
use dusty_core::view::View;
use dusty_reactive::Scope;

/// A reactive match container that renders the arm matching a reactive value.
///
/// `MatchView` evaluates a reactive value function each time the dynamic node
/// is resolved. The first arm whose key equals the current value is rendered.
/// If no arm matches, the fallback (if set) is rendered; otherwise an empty
/// fragment is produced.
///
/// # Example
///
/// ```
/// use dusty_reactive::{initialize_runtime, create_scope, dispose_runtime};
/// use dusty_widgets::MatchView;
/// use dusty_core::node::text;
/// use dusty_core::node::Node;
/// use dusty_core::view::View;
///
/// initialize_runtime();
/// create_scope(|cx| {
///     let node = MatchView::new(|| "a")
///         .arm("a", || Node::Text(text("Alpha")))
///         .arm("b", || Node::Text(text("Beta")))
///         .build(cx);
///     assert!(node.is_component());
/// });
/// dispose_runtime();
/// ```
pub struct MatchView<K: PartialEq + 'static> {
    value: Box<dyn Fn() -> K>,
    arms: Vec<(K, Box<dyn Fn() -> Node>)>,
    fallback: Option<Box<dyn Fn() -> Node>>,
}

impl<K: PartialEq + 'static> MatchView<K> {
    /// Creates a new `MatchView` with the given reactive value function.
    ///
    /// The value function is called each time the dynamic node is resolved
    /// to determine which arm to render.
    #[must_use]
    pub fn new(value: impl Fn() -> K + 'static) -> Self {
        Self {
            value: Box::new(value),
            arms: Vec::new(),
            fallback: None,
        }
    }

    /// Adds a match arm. When the value equals `key`, the `view` closure
    /// produces the rendered node.
    #[must_use]
    pub fn arm(mut self, key: K, view: impl Fn() -> Node + 'static) -> Self {
        self.arms.push((key, Box::new(view)));
        self
    }

    /// Sets the fallback closure, used when no arm matches the current value.
    #[must_use]
    pub fn fallback(mut self, f: impl Fn() -> Node + 'static) -> Self {
        self.fallback = Some(Box::new(f));
        self
    }
}

impl<K: PartialEq + 'static> View for MatchView<K> {
    fn build(self, _cx: Scope) -> Node {
        let value = self.value;
        let arms = self.arms;
        let fallback = self.fallback;

        let dynamic = dynamic_node(move || {
            let current = (value)();
            for (key, view_fn) in &arms {
                if *key == current {
                    return view_fn();
                }
            }
            fallback
                .as_ref()
                .map_or_else(|| Node::Fragment(vec![]), |f| f())
        });

        Node::Component(ComponentNode {
            name: "Match",
            child: Box::new(Node::Dynamic(dynamic)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dusty_core::node::{text, Node};
    use dusty_core::view::View;
    use dusty_reactive::{create_scope, create_signal, dispose_runtime, initialize_runtime, Scope};

    fn with_scope(f: impl FnOnce(Scope)) {
        initialize_runtime();
        create_scope(|cx| f(cx));
        dispose_runtime();
    }

    fn resolve_dynamic(node: &Node) -> Node {
        match node {
            Node::Component(comp) => match &*comp.child {
                Node::Dynamic(dn) => dn.current_node(),
                _ => panic!("expected Dynamic inside Component"),
            },
            _ => panic!("expected Component node"),
        }
    }

    #[test]
    fn renders_matching_arm() {
        with_scope(|cx| {
            let node = MatchView::new(|| "a")
                .arm("a", || Node::Text(text("Alpha")))
                .arm("b", || Node::Text(text("Beta")))
                .build(cx);

            let resolved = resolve_dynamic(&node);
            assert!(resolved.is_text());
            if let Node::Text(t) = resolved {
                assert_eq!(t.current_text(), "Alpha");
            }
        });
    }

    #[test]
    fn fallback_on_no_match() {
        with_scope(|cx| {
            let node = MatchView::new(|| "z")
                .arm("a", || Node::Text(text("Alpha")))
                .fallback(|| Node::Text(text("default")))
                .build(cx);

            let resolved = resolve_dynamic(&node);
            assert!(resolved.is_text());
            if let Node::Text(t) = resolved {
                assert_eq!(t.current_text(), "default");
            }
        });
    }

    #[test]
    fn no_match_no_fallback_empty() {
        with_scope(|cx| {
            let node = MatchView::new(|| "z")
                .arm("a", || Node::Text(text("Alpha")))
                .build(cx);

            let resolved = resolve_dynamic(&node);
            assert!(resolved.is_fragment());
            if let Node::Fragment(children) = resolved {
                assert!(children.is_empty());
            }
        });
    }

    #[test]
    fn reactive_value_switches_arms() {
        with_scope(|cx| {
            let sig = create_signal(1i32);
            let node = MatchView::new(move || sig.get())
                .arm(1, || Node::Text(text("One")))
                .arm(2, || Node::Text(text("Two")))
                .build(cx);

            // Initially matches arm 1
            let resolved = resolve_dynamic(&node);
            if let Node::Text(t) = resolved {
                assert_eq!(t.current_text(), "One");
            } else {
                panic!("expected Text node for arm 1");
            }

            // Switch signal to 2
            sig.set(2);

            let resolved = resolve_dynamic(&node);
            if let Node::Text(t) = resolved {
                assert_eq!(t.current_text(), "Two");
            } else {
                panic!("expected Text node for arm 2");
            }
        });
    }

    #[test]
    fn multiple_arms() {
        with_scope(|cx| {
            let node = MatchView::new(|| "b")
                .arm("a", || Node::Text(text("Alpha")))
                .arm("b", || Node::Text(text("Beta")))
                .arm("c", || Node::Text(text("Gamma")))
                .build(cx);

            let resolved = resolve_dynamic(&node);
            assert!(resolved.is_text());
            if let Node::Text(t) = resolved {
                assert_eq!(t.current_text(), "Beta");
            }
        });
    }
}
