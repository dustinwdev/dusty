//! The `For` container widget — renders a list of items from a reactive source.
//!
//! `For` maps each item in a reactive `Vec<T>` to a view using a view function,
//! with an optional key function for future reconciliation optimization.
//!
//! # Example
//!
//! ```
//! use dusty_reactive::{initialize_runtime, create_scope, dispose_runtime};
//! use dusty_core::node::{text, Node};
//! use dusty_core::view::View;
//! use dusty_widgets::For;
//!
//! initialize_runtime();
//! create_scope(|cx| {
//!     let node = For::<i32, i32, Node>::new(|| vec![1, 2, 3])
//!         .key(|x: &i32| *x)
//!         .view(|x: i32| Node::Text(text(format!("{x}"))))
//!         .build(cx);
//!     assert!(node.is_component());
//! });
//! dispose_runtime();
//! ```

use dusty_core::node::{dynamic_node, ComponentNode, Node};
use dusty_core::view::{IntoView, View};
use dusty_reactive::Scope;

type KeyFn<T, K> = Box<dyn Fn(&T) -> K>;

/// A container widget that renders a list of items from a reactive source.
///
/// Each item is mapped to a view via the view function. An optional key
/// function enables future reconciliation optimization when the list changes.
///
/// # Example
///
/// ```
/// use dusty_reactive::{initialize_runtime, create_scope, dispose_runtime};
/// use dusty_core::node::{text, Node};
/// use dusty_core::view::View;
/// use dusty_widgets::For;
///
/// initialize_runtime();
/// create_scope(|cx| {
///     let node = For::<i32, i32, Node>::new(|| vec![1, 2, 3])
///         .key(|x: &i32| *x)
///         .view(|x: i32| Node::Text(text(format!("{x}"))))
///         .build(cx);
///     assert!(node.is_component());
/// });
/// dispose_runtime();
/// ```
pub struct For<T, K, V>
where
    T: Clone + 'static,
    K: PartialEq + 'static,
    V: IntoView + 'static,
{
    each: Box<dyn Fn() -> Vec<T>>,
    key_fn: Option<KeyFn<T, K>>,
    view_fn: Option<Box<dyn Fn(T) -> V>>,
}

impl<T, K, V> For<T, K, V>
where
    T: Clone + 'static,
    K: PartialEq + 'static,
    V: IntoView + 'static,
{
    /// Creates a new `For` with the given reactive list source.
    ///
    /// The `each` closure is called to obtain the current list of items.
    /// Use [`.key()`](Self::key) and [`.view()`](Self::view) to configure
    /// the key extraction and view mapping functions.
    ///
    /// # Example
    ///
    /// ```
    /// use dusty_core::node::Node;
    /// use dusty_widgets::For;
    ///
    /// let _for_widget = For::<i32, i32, Node>::new(|| vec![1, 2, 3]);
    /// ```
    #[must_use]
    pub fn new(each: impl Fn() -> Vec<T> + 'static) -> Self {
        Self {
            each: Box::new(each),
            key_fn: None,
            view_fn: None,
        }
    }

    /// Sets the key extraction function for future reconciliation.
    ///
    /// The key function maps each item to a comparable key value. When the
    /// list changes, keys will be used to match old and new items for efficient
    /// updates. **Note:** key-based reconciliation is not yet implemented;
    /// currently the list is fully rebuilt on each change.
    #[must_use]
    pub fn key(mut self, f: impl Fn(&T) -> K + 'static) -> Self {
        self.key_fn = Some(Box::new(f));
        self
    }

    /// Sets the view function that maps each item to a view.
    ///
    /// This function is called for each item in the list to produce the
    /// rendered output.
    #[must_use]
    pub fn view(mut self, f: impl Fn(T) -> V + 'static) -> Self {
        self.view_fn = Some(Box::new(f));
        self
    }
}

impl<T, K, V> View for For<T, K, V>
where
    T: Clone + 'static,
    K: PartialEq + 'static,
    V: IntoView + 'static,
{
    fn build(self, cx: Scope) -> Node {
        let each = self.each;
        let key_fn = self.key_fn;
        let view_fn = self.view_fn;

        let dynamic = dynamic_node(move || {
            let items = (each)();
            view_fn.as_ref().map_or_else(
                || Node::Fragment(vec![]),
                |vf| {
                    // TODO: key_fn stored for future reconciliation — not yet used for diffing
                    let _key_fn = &key_fn;
                    let nodes: Vec<Node> = items
                        .into_iter()
                        .map(|item| vf(item).into_view(cx))
                        .collect();
                    Node::Fragment(nodes)
                },
            )
        });

        Node::Component(ComponentNode {
            name: "For",
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
    use std::cell::Cell;
    use std::rc::Rc;

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

    fn fragment_len(node: &Node) -> usize {
        match node {
            Node::Fragment(nodes) => nodes.len(),
            _ => panic!("expected Fragment node"),
        }
    }

    fn fragment_text_at(node: &Node, index: usize) -> String {
        match node {
            Node::Fragment(nodes) => match &nodes[index] {
                Node::Text(t) => t.current_text().into_owned(),
                _ => panic!("expected Text node at index {index}"),
            },
            _ => panic!("expected Fragment node"),
        }
    }

    #[test]
    fn renders_items() {
        with_scope(|cx| {
            let node = For::<i32, i32, Node>::new(|| vec![1, 2, 3])
                .key(|x: &i32| *x)
                .view(|x: i32| Node::Text(text(format!("{x}"))))
                .build(cx);

            let resolved = resolve_dynamic(&node);
            assert_eq!(fragment_len(&resolved), 3);
            assert_eq!(fragment_text_at(&resolved, 0), "1");
            assert_eq!(fragment_text_at(&resolved, 1), "2");
            assert_eq!(fragment_text_at(&resolved, 2), "3");
        });
    }

    #[test]
    fn empty_list() {
        with_scope(|cx| {
            let node = For::<i32, i32, Node>::new(|| vec![])
                .key(|x: &i32| *x)
                .view(|x: i32| Node::Text(text(format!("{x}"))))
                .build(cx);

            let resolved = resolve_dynamic(&node);
            assert_eq!(fragment_len(&resolved), 0);
        });
    }

    #[test]
    fn reactive_list_updates() {
        with_scope(|cx| {
            let items = create_signal(vec![1, 2]);

            let node = For::<i32, i32, Node>::new(move || items.get())
                .key(|x: &i32| *x)
                .view(|x: i32| Node::Text(text(format!("{x}"))))
                .build(cx);

            let resolved = resolve_dynamic(&node);
            assert_eq!(fragment_len(&resolved), 2);

            items.set(vec![1, 2, 3]);

            let resolved = resolve_dynamic(&node);
            assert_eq!(fragment_len(&resolved), 3);
        });
    }

    #[test]
    fn add_item() {
        with_scope(|cx| {
            let items = create_signal(vec![10, 20]);

            let node = For::<i32, i32, Node>::new(move || items.get())
                .key(|x: &i32| *x)
                .view(|x: i32| Node::Text(text(format!("{x}"))))
                .build(cx);

            let resolved = resolve_dynamic(&node);
            assert_eq!(fragment_len(&resolved), 2);

            items.update(|v| v.push(30));

            let resolved = resolve_dynamic(&node);
            assert_eq!(fragment_len(&resolved), 3);
            assert_eq!(fragment_text_at(&resolved, 2), "30");
        });
    }

    #[test]
    fn remove_item() {
        with_scope(|cx| {
            let items = create_signal(vec![1, 2, 3]);

            let node = For::<i32, i32, Node>::new(move || items.get())
                .key(|x: &i32| *x)
                .view(|x: i32| Node::Text(text(format!("{x}"))))
                .build(cx);

            let resolved = resolve_dynamic(&node);
            assert_eq!(fragment_len(&resolved), 3);

            items.set(vec![1, 3]);

            let resolved = resolve_dynamic(&node);
            assert_eq!(fragment_len(&resolved), 2);
            assert_eq!(fragment_text_at(&resolved, 0), "1");
            assert_eq!(fragment_text_at(&resolved, 1), "3");
        });
    }

    #[test]
    fn reorder_items() {
        with_scope(|cx| {
            let node =
                For::<String, String, Node>::new(|| vec!["b".into(), "a".into(), "c".into()])
                    .key(|s: &String| s.clone())
                    .view(|s: String| Node::Text(text(s)))
                    .build(cx);

            let resolved = resolve_dynamic(&node);
            assert_eq!(fragment_len(&resolved), 3);
            assert_eq!(fragment_text_at(&resolved, 0), "b");
            assert_eq!(fragment_text_at(&resolved, 1), "a");
            assert_eq!(fragment_text_at(&resolved, 2), "c");
        });
    }

    #[test]
    fn key_function_not_called_without_reconciliation() {
        with_scope(|cx| {
            let counter = Rc::new(Cell::new(0u32));
            let counter_clone = Rc::clone(&counter);

            let node = For::<i32, i32, Node>::new(|| vec![1, 2, 3])
                .key(move |x: &i32| {
                    counter_clone.set(counter_clone.get() + 1);
                    *x
                })
                .view(|x: i32| Node::Text(text(format!("{x}"))))
                .build(cx);

            let _resolved = resolve_dynamic(&node);
            // Key function is stored but not called — reconciliation not yet implemented
            assert_eq!(counter.get(), 0);
        });
    }

    #[test]
    fn builds_component() {
        with_scope(|cx| {
            let node = For::<i32, i32, Node>::new(|| vec![1])
                .view(|x: i32| Node::Text(text(format!("{x}"))))
                .build(cx);

            assert!(node.is_component());
            if let Node::Component(comp) = &node {
                assert_eq!(comp.name, "For");
            }
        });
    }

    #[test]
    fn no_view_fn_produces_empty_fragment() {
        with_scope(|cx| {
            let node = For::<i32, i32, Node>::new(|| vec![1, 2, 3]).build(cx);

            let resolved = resolve_dynamic(&node);
            assert_eq!(fragment_len(&resolved), 0);
        });
    }

    #[test]
    fn no_key_fn_still_renders() {
        with_scope(|cx| {
            let node = For::<i32, i32, Node>::new(|| vec![1, 2])
                .view(|x: i32| Node::Text(text(format!("{x}"))))
                .build(cx);

            let resolved = resolve_dynamic(&node);
            assert_eq!(fragment_len(&resolved), 2);
            assert_eq!(fragment_text_at(&resolved, 0), "1");
            assert_eq!(fragment_text_at(&resolved, 1), "2");
        });
    }
}
