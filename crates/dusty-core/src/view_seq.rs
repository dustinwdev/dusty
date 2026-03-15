use dusty_reactive::Scope;

use crate::node::Node;
use crate::view::IntoView;

/// A sequence of views that can be flattened into a `Vec<Node>`.
///
/// Implemented for tuples of [`IntoView`] (up to arity 12),
/// `Vec<impl IntoView>`, and `Option<impl IntoView>`.
pub trait ViewSeq {
    /// Builds each item in the sequence and collects the resulting nodes.
    fn build_seq(self, cx: Scope) -> Vec<Node>;
}

// Empty tuple — no children
impl ViewSeq for () {
    fn build_seq(self, _cx: Scope) -> Vec<Node> {
        Vec::new()
    }
}

macro_rules! impl_view_seq {
    ($($idx:tt : $T:ident),+) => {
        impl<$($T: IntoView),+> ViewSeq for ($($T,)+) {
            fn build_seq(self, cx: Scope) -> Vec<Node> {
                vec![$(self.$idx.into_view(cx)),+]
            }
        }
    };
}

impl_view_seq!(0: A);
impl_view_seq!(0: A, 1: B);
impl_view_seq!(0: A, 1: B, 2: C);
impl_view_seq!(0: A, 1: B, 2: C, 3: D);
impl_view_seq!(0: A, 1: B, 2: C, 3: D, 4: E);
impl_view_seq!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F);
impl_view_seq!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G);
impl_view_seq!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H);
impl_view_seq!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I);
impl_view_seq!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J);
impl_view_seq!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J, 10: K);
impl_view_seq!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J, 10: K, 11: L);

impl<V: IntoView> ViewSeq for Vec<V> {
    fn build_seq(self, cx: Scope) -> Vec<Node> {
        self.into_iter().map(|v| v.into_view(cx)).collect()
    }
}

impl<V: IntoView> ViewSeq for Option<V> {
    fn build_seq(self, cx: Scope) -> Vec<Node> {
        self.map_or_else(Vec::new, |v| vec![v.into_view(cx)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::el;
    use crate::node::text;
    use dusty_reactive::{create_scope, dispose_runtime, initialize_runtime};

    fn with_scope(f: impl FnOnce(Scope)) {
        initialize_runtime();
        create_scope(|cx| f(cx)).unwrap();
        dispose_runtime();
    }

    #[test]
    fn empty_tuple() {
        with_scope(|cx| {
            let nodes = ().build_seq(cx);
            assert!(nodes.is_empty());
        });
    }

    #[test]
    fn single_element_tuple() {
        with_scope(|cx| {
            let nodes = ("hello",).build_seq(cx);
            assert_eq!(nodes.len(), 1);
            assert!(nodes[0].is_text());
        });
    }

    #[test]
    fn mixed_type_tuple() {
        with_scope(|cx| {
            let nodes = ("text", String::from("string"), el("Box", cx)).build_seq(cx);
            assert_eq!(nodes.len(), 3);
            assert!(nodes[0].is_text());
            assert!(nodes[1].is_text());
            assert!(nodes[2].is_element());
        });
    }

    #[test]
    fn vec_of_views() {
        with_scope(|cx| {
            let items = vec!["a", "b", "c"];
            let nodes = items.build_seq(cx);
            assert_eq!(nodes.len(), 3);
        });
    }

    #[test]
    fn option_some() {
        with_scope(|cx| {
            let nodes = Some("present").build_seq(cx);
            assert_eq!(nodes.len(), 1);
        });
    }

    #[test]
    fn option_none() {
        with_scope(|cx| {
            let nodes: Vec<Node> = Option::<&str>::None.build_seq(cx);
            assert!(nodes.is_empty());
        });
    }

    #[test]
    fn twelve_arity_tuple_compiles() {
        with_scope(|cx| {
            let nodes = ("a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l").build_seq(cx);
            assert_eq!(nodes.len(), 12);
        });
    }

    #[test]
    fn vec_of_nodes() {
        with_scope(|cx| {
            let items: Vec<Node> = vec![Node::Text(text("first")), Node::Text(text("second"))];
            let nodes = items.build_seq(cx);
            assert_eq!(nodes.len(), 2);
        });
    }
}
