//! Node tree inspector — walks a Dusty node tree and produces a flat
//! snapshot of nodes with their styles, bounds, attributes, and event handlers.

use dusty_core::element::AttributeValue;
use dusty_core::{DynamicNode, Element, Node, TextNode};
use dusty_layout::{LayoutNodeId, LayoutResult, Rect};
use dusty_style::Style;

use crate::error::{DevtoolsError, Result};

/// The kind of an inspected node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InspectorNodeKind {
    /// An element with a name.
    Element { name: &'static str },
    /// A text node with its current content.
    Text { content: String },
    /// A fragment with a count of direct children.
    Fragment { child_count: usize },
    /// A component wrapper with its debug name.
    Component { name: &'static str },
    /// A lazily-resolved dynamic node.
    Dynamic,
}

/// A snapshot of a single node in the inspector tree.
#[derive(Debug, Clone)]
pub struct InspectorNode {
    /// What kind of node this is.
    pub kind: InspectorNodeKind,
    /// Depth in the tree (0 = root).
    pub depth: usize,
    /// Attributes as `(name, value_string)` pairs.
    pub attributes: Vec<(String, String)>,
    /// Cloned style data, if the element has a `Style` set.
    pub style: Option<StyleSnapshot>,
    /// Computed layout bounds, if layout was provided.
    pub bounds: Option<NodeBounds>,
    /// Names of attached event handlers.
    pub event_handlers: Vec<&'static str>,
    /// Indices into [`InspectorTree::nodes`] for this node's children.
    pub child_indices: Vec<usize>,
}

/// An axis-aligned rectangle representing a node's computed bounds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeBounds {
    /// Absolute X position (left edge).
    pub x: f32,
    /// Absolute Y position (top edge).
    pub y: f32,
    /// Width in pixels.
    pub width: f32,
    /// Height in pixels.
    pub height: f32,
}

impl From<&Rect> for NodeBounds {
    fn from(r: &Rect) -> Self {
        Self {
            x: r.x,
            y: r.y,
            width: r.width,
            height: r.height,
        }
    }
}

/// A cloned snapshot of an element's [`Style`].
#[derive(Debug, Clone, PartialEq)]
pub struct StyleSnapshot(pub Style);

/// A flat representation of the inspected node tree.
#[derive(Debug, Clone)]
pub struct InspectorTree {
    /// All nodes in depth-first order.
    pub nodes: Vec<InspectorNode>,
    /// Indices of the root-level nodes.
    pub root_indices: Vec<usize>,
}

/// Inspects a node tree and produces an [`InspectorTree`] snapshot.
///
/// Walks the tree in the same order as `dusty-layout` and `dusty-a11y`:
/// Element and Text nodes consume layout IDs (incrementing counter);
/// Fragment and Component nodes are transparent (no layout ID consumed).
///
/// Dynamic nodes are resolved via [`dusty_reactive::untrack`] to avoid
/// accidental subscriptions.
///
/// # Errors
///
/// Returns [`DevtoolsError::EmptyTree`] if the tree contains no concrete
/// nodes (e.g. an empty fragment).
///
/// # Examples
///
/// ```
/// use dusty_reactive::{initialize_runtime, create_scope, dispose_runtime};
/// use dusty_core::{el, text};
/// use dusty_devtools::inspector::{inspect, InspectorNodeKind};
///
/// initialize_runtime();
/// create_scope(|cx| {
///     let node = el("Button", cx).attr("label", "OK").build_node();
///     let tree = inspect(&node, None).unwrap();
///     assert_eq!(tree.nodes.len(), 1);
///     assert!(matches!(tree.nodes[0].kind, InspectorNodeKind::Element { name: "Button" }));
/// });
/// dispose_runtime();
/// ```
pub fn inspect(root: &Node, layout: Option<&LayoutResult>) -> Result<InspectorTree> {
    let mut walker = InspectorWalker {
        nodes: Vec::new(),
        next_layout_id: 0,
        layout,
    };

    let root_indices = walker.walk_node(root, 0);

    if root_indices.is_empty() {
        return Err(DevtoolsError::EmptyTree);
    }

    Ok(InspectorTree {
        nodes: walker.nodes,
        root_indices,
    })
}

struct InspectorWalker<'a> {
    nodes: Vec<InspectorNode>,
    next_layout_id: usize,
    layout: Option<&'a LayoutResult>,
}

impl InspectorWalker<'_> {
    fn walk_node(&mut self, node: &Node, depth: usize) -> Vec<usize> {
        match node {
            Node::Element(el) => self.walk_element(el, depth),
            Node::Text(text_node) => self.walk_text(text_node, depth),
            // Fragment: transparent — children promoted to parent level
            Node::Fragment(children) => self.walk_children(children, depth),
            // Component: transparent — child promoted to parent level
            Node::Component(comp) => self.walk_node(&comp.child, depth),
            Node::Dynamic(dn) => self.walk_dynamic(dn, depth),
        }
    }

    fn walk_element(&mut self, el: &Element, depth: usize) -> Vec<usize> {
        let layout_id = self.alloc_layout_id();
        let node_index = self.nodes.len();

        let style = el
            .style()
            .downcast_ref::<Style>()
            .map(|s| StyleSnapshot(s.clone()));

        let bounds = self.layout_bounds(layout_id);

        let attributes = el
            .attributes()
            .iter()
            .map(|(name, val)| ((*name).to_string(), attr_value_to_string(val)))
            .collect();

        let event_handlers = el
            .event_handlers()
            .iter()
            .map(dusty_core::EventHandler::name)
            .collect();

        self.nodes.push(InspectorNode {
            kind: InspectorNodeKind::Element { name: el.name() },
            depth,
            attributes,
            style,
            bounds,
            event_handlers,
            child_indices: Vec::new(),
        });

        let child_indices = self.walk_children(el.children(), depth + 1);
        self.nodes[node_index].child_indices = child_indices;

        vec![node_index]
    }

    fn walk_text(&mut self, text_node: &TextNode, depth: usize) -> Vec<usize> {
        let layout_id = self.alloc_layout_id();
        let bounds = self.layout_bounds(layout_id);
        let content = dusty_reactive::untrack(|| text_node.current_text().into_owned());

        let node_index = self.nodes.len();
        self.nodes.push(InspectorNode {
            kind: InspectorNodeKind::Text { content },
            depth,
            attributes: Vec::new(),
            style: None,
            bounds,
            event_handlers: Vec::new(),
            child_indices: Vec::new(),
        });

        vec![node_index]
    }

    fn walk_dynamic(&mut self, dn: &DynamicNode, depth: usize) -> Vec<usize> {
        let resolved = dusty_reactive::untrack(|| dn.current_node());
        let node_index = self.nodes.len();

        self.nodes.push(InspectorNode {
            kind: InspectorNodeKind::Dynamic,
            depth,
            attributes: Vec::new(),
            style: None,
            bounds: None,
            event_handlers: Vec::new(),
            child_indices: Vec::new(),
        });

        let child_indices = self.walk_node(&resolved, depth + 1);
        self.nodes[node_index].child_indices = child_indices;

        vec![node_index]
    }

    fn walk_children(&mut self, children: &[Node], depth: usize) -> Vec<usize> {
        let mut indices = Vec::new();
        for child in children {
            indices.extend(self.walk_node(child, depth));
        }
        indices
    }

    fn alloc_layout_id(&mut self) -> usize {
        let id = self.next_layout_id;
        self.next_layout_id += 1;
        id
    }

    fn layout_bounds(&self, layout_id: usize) -> Option<NodeBounds> {
        self.layout
            .and_then(|l| l.get(LayoutNodeId(layout_id)).map(NodeBounds::from))
    }
}

fn attr_value_to_string(val: &AttributeValue) -> String {
    match val {
        AttributeValue::String(s) => s.clone(),
        AttributeValue::Int(i) => i.to_string(),
        AttributeValue::Float(f) => f.to_string(),
        AttributeValue::Bool(b) => b.to_string(),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use dusty_core::event::{ClickEvent, HoverEvent};
    use dusty_core::{el, text, ComponentNode};
    use dusty_reactive::{create_scope, dispose_runtime, initialize_runtime};

    fn with_scope(f: impl FnOnce(dusty_reactive::Scope)) {
        initialize_runtime();
        create_scope(|cx| f(cx));
        dispose_runtime();
    }

    #[test]
    fn single_element() {
        with_scope(|cx| {
            let node = el("Button", cx).build_node();
            let tree = inspect(&node, None).unwrap();
            assert_eq!(tree.nodes.len(), 1);
            assert_eq!(tree.root_indices, vec![0]);
            assert!(matches!(
                tree.nodes[0].kind,
                InspectorNodeKind::Element { name: "Button" }
            ));
        });
    }

    #[test]
    fn text_node() {
        let node = Node::Text(text("hello"));
        let tree = inspect(&node, None).unwrap();
        assert_eq!(tree.nodes.len(), 1);
        assert!(matches!(
            &tree.nodes[0].kind,
            InspectorNodeKind::Text { content } if content == "hello"
        ));
    }

    #[test]
    fn fragment_flattened() {
        with_scope(|cx| {
            let frag = Node::Fragment(vec![el("A", cx).build_node(), el("B", cx).build_node()]);
            let tree = inspect(&frag, None).unwrap();
            // Fragment is transparent — two root nodes, not one
            assert_eq!(tree.nodes.len(), 2);
            assert_eq!(tree.root_indices.len(), 2);
        });
    }

    #[test]
    fn component_transparent() {
        with_scope(|cx| {
            let inner = el("Inner", cx).build_node();
            let comp = Node::Component(ComponentNode {
                name: "MyComponent",
                child: Box::new(inner),
            });
            let tree = inspect(&comp, None).unwrap();
            // Component is transparent — only the inner element appears
            assert_eq!(tree.nodes.len(), 1);
            assert!(matches!(
                tree.nodes[0].kind,
                InspectorNodeKind::Element { name: "Inner" }
            ));
        });
    }

    #[test]
    fn empty_tree_error() {
        let empty = Node::Fragment(vec![]);
        let result = inspect(&empty, None);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DevtoolsError::EmptyTree);
    }

    #[test]
    fn attributes_extracted() {
        with_scope(|cx| {
            let node = el("Button", cx)
                .attr("label", "Submit")
                .attr("disabled", true)
                .build_node();
            let tree = inspect(&node, None).unwrap();
            let attrs = &tree.nodes[0].attributes;
            assert_eq!(attrs.len(), 2);
            assert!(attrs.contains(&("label".to_string(), "Submit".to_string())));
            assert!(attrs.contains(&("disabled".to_string(), "true".to_string())));
        });
    }

    #[test]
    fn event_handlers_listed() {
        with_scope(|cx| {
            let node = el("Button", cx)
                .on_click(|_e: &ClickEvent| {})
                .on_hover(|_e: &HoverEvent| {})
                .build_node();
            let tree = inspect(&node, None).unwrap();
            let handlers = &tree.nodes[0].event_handlers;
            assert_eq!(handlers.len(), 2);
            assert!(handlers.contains(&"click"));
            assert!(handlers.contains(&"hover"));
        });
    }

    #[test]
    fn style_extraction() {
        with_scope(|cx| {
            let style = Style {
                width: Some(dusty_style::Length::Px(100.0)),
                height: Some(dusty_style::Length::Px(50.0)),
                ..Style::default()
            };
            let node = el("Box", cx).style(style.clone()).build_node();
            let tree = inspect(&node, None).unwrap();
            let snapshot = tree.nodes[0].style.as_ref().unwrap();
            assert_eq!(snapshot.0.width, Some(dusty_style::Length::Px(100.0)));
            assert_eq!(snapshot.0.height, Some(dusty_style::Length::Px(50.0)));
        });
    }

    #[test]
    fn no_style_returns_none() {
        with_scope(|cx| {
            let node = el("Box", cx).build_node();
            let tree = inspect(&node, None).unwrap();
            assert!(tree.nodes[0].style.is_none());
        });
    }

    #[test]
    fn nested_children_indices() {
        with_scope(|cx| {
            let node = el("Row", cx)
                .child(el("A", cx).build_node())
                .child(el("B", cx).build_node())
                .build_node();
            let tree = inspect(&node, None).unwrap();
            assert_eq!(tree.nodes.len(), 3); // Row, A, B
            assert_eq!(tree.root_indices, vec![0]); // Row is root
            assert_eq!(tree.nodes[0].child_indices, vec![1, 2]); // A and B
            assert!(tree.nodes[1].child_indices.is_empty());
            assert!(tree.nodes[2].child_indices.is_empty());
        });
    }

    #[test]
    fn depth_tracking() {
        with_scope(|cx| {
            let node = el("Outer", cx)
                .child(el("Inner", cx).child(text("leaf")).build_node())
                .build_node();
            let tree = inspect(&node, None).unwrap();
            assert_eq!(tree.nodes[0].depth, 0); // Outer
            assert_eq!(tree.nodes[1].depth, 1); // Inner
            assert_eq!(tree.nodes[2].depth, 2); // leaf text
        });
    }

    #[test]
    fn attr_value_string_conversion() {
        assert_eq!(
            attr_value_to_string(&AttributeValue::String("hi".into())),
            "hi"
        );
        assert_eq!(attr_value_to_string(&AttributeValue::Int(42)), "42");
        assert_eq!(attr_value_to_string(&AttributeValue::Float(3.14)), "3.14");
        assert_eq!(attr_value_to_string(&AttributeValue::Bool(true)), "true");
    }
}
