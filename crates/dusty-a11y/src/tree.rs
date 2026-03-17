//! Builds an accesskit [`TreeUpdate`] from a Dusty node tree and layout result.

use accesskit::{NodeId, Rect, TreeUpdate};
use dusty_core::element::AttributeValue;
use dusty_core::Node;
use dusty_layout::LayoutResult;

use crate::error::{A11yError, Result};
use crate::role::element_role;

/// Builds an accessibility tree from a Dusty node tree and its computed layout.
///
/// Walks the node tree in the same order as `dusty-layout` and `dusty-render`,
/// producing an accesskit [`TreeUpdate`] that maps each element/text node to
/// an accessibility node with appropriate roles, labels, and bounds.
///
/// # ID Mapping
///
/// `NodeId(0)` is reserved for a synthetic root container. Layout node IDs
/// map to `NodeId(layout_id + 1)`.
///
/// # Errors
///
/// Returns [`A11yError::EmptyTree`] if the tree contains no layout nodes
/// (e.g. an empty fragment).
///
/// # Examples
///
/// ```
/// use dusty_core::{el, text};
/// use dusty_style::{Style, FontStyle};
/// use dusty_layout::{compute_layout, TextMeasure};
/// use dusty_a11y::build_accessibility_tree;
/// use dusty_reactive::{initialize_runtime, create_scope, dispose_runtime};
///
/// struct Mock;
/// impl TextMeasure for Mock {
///     fn measure(&self, _: &str, _: Option<f32>, _: &FontStyle) -> (f32, f32) {
///         (50.0, 16.0)
///     }
/// }
///
/// initialize_runtime();
/// create_scope(|cx| {
///     let node = el("Button", cx)
///         .attr("label", "Submit")
///         .style(Style { width: Some(100.0), height: Some(40.0), ..Style::default() })
///         .build_node();
///     let layout = compute_layout(&node, 400.0, 300.0, &Mock).unwrap();
///     let update = build_accessibility_tree(&node, &layout, None).unwrap();
///     assert!(!update.nodes.is_empty());
/// }).unwrap();
/// dispose_runtime();
/// ```
pub fn build_accessibility_tree(
    root: &Node,
    layout: &LayoutResult,
    focus: Option<usize>,
) -> Result<TreeUpdate> {
    let mut walker = TreeWalker {
        next_id: 0,
        nodes: Vec::new(),
    };

    let child_ids = walker.walk_node(root, layout);

    if child_ids.is_empty() {
        return Err(A11yError::EmptyTree);
    }

    // Create synthetic root at NodeId(0)
    let root_ak_id = NodeId(0);
    let mut root_node = accesskit::Node::new(accesskit::Role::GenericContainer);
    root_node.set_children(child_ids);

    walker.nodes.push((root_ak_id, root_node));

    let focus_id = focus.map_or(root_ak_id, |id| {
        #[allow(clippy::cast_possible_truncation)]
        NodeId(id as u64 + 1)
    });

    Ok(TreeUpdate {
        nodes: walker.nodes,
        tree: Some(accesskit::Tree::new(root_ak_id)),
        focus: focus_id,
    })
}

struct TreeWalker {
    next_id: usize,
    nodes: Vec<(NodeId, accesskit::Node)>,
}

impl TreeWalker {
    /// Allocates a layout node ID and returns both the layout ID and the
    /// corresponding accesskit `NodeId` (`layout_id` + 1, since 0 is the
    /// synthetic root).
    fn alloc_id(&mut self) -> (usize, NodeId) {
        let layout_id = self.next_id;
        self.next_id += 1;
        #[allow(clippy::cast_possible_truncation)]
        let ak_id = NodeId(layout_id as u64 + 1);
        (layout_id, ak_id)
    }

    fn walk_node(&mut self, node: &Node, layout: &LayoutResult) -> Vec<NodeId> {
        match node {
            Node::Element(el) => self.walk_element(el, layout),
            Node::Text(text_node) => self.walk_text(text_node, layout),
            Node::Fragment(children) => self.walk_children(children, layout),
            Node::Component(comp) => self.walk_node(&comp.child, layout),
            Node::Dynamic(dn) => {
                let resolved = dn.current_node();
                self.walk_node(&resolved, layout)
            }
        }
    }

    #[allow(clippy::cast_lossless)]
    fn walk_element(&mut self, el: &dusty_core::Element, layout: &LayoutResult) -> Vec<NodeId> {
        let (layout_id, ak_id) = self.alloc_id();

        let role = element_role(el.name());
        let mut ak_node = accesskit::Node::new(role);

        // Set bounds from layout
        if let Some(rect) = layout.get(dusty_layout::LayoutNodeId(layout_id)) {
            ak_node.set_bounds(Rect::new(
                rect.x as f64,
                rect.y as f64,
                (rect.x + rect.width) as f64,
                (rect.y + rect.height) as f64,
            ));
        }

        // Extract label: prefer "label" or "aria-label", fall back to "placeholder"
        let label = el.attr("label").or_else(|| el.attr("aria-label"));
        let placeholder = el.attr("placeholder");

        if let Some(label_val) = label {
            if let Some(s) = attr_as_str(label_val) {
                ak_node.set_label(s);
            }
        } else if let Some(ph_val) = placeholder {
            if let Some(s) = attr_as_str(ph_val) {
                ak_node.set_label(s);
            }
        }

        // Fallback: compute accessible name from child text nodes
        if label.is_none() && placeholder.is_none() {
            if let Some(child_text) = collect_text_from_children(el.children()) {
                ak_node.set_label(child_text);
            }
        }

        // Description
        if let Some(desc_val) = el
            .attr("description")
            .or_else(|| el.attr("aria-description"))
        {
            if let Some(s) = attr_as_str(desc_val) {
                ak_node.set_description(s);
            }
        }

        // Value
        if let Some(val) = el.attr("value") {
            if let Some(s) = attr_as_str(val) {
                ak_node.set_value(s);
            }
        }

        // Disabled
        if matches!(el.attr("disabled"), Some(AttributeValue::Bool(true))) {
            ak_node.set_disabled();
        }

        // Toggled / checked
        match el.attr("checked") {
            Some(AttributeValue::Bool(true)) => {
                ak_node.set_toggled(accesskit::Toggled::True);
            }
            Some(AttributeValue::Bool(false)) => {
                ak_node.set_toggled(accesskit::Toggled::False);
            }
            _ => {}
        }

        // Live region
        if let Some(live_val) = el.attr("aria-live") {
            if let Some(s) = attr_as_str(live_val) {
                match s.as_ref() {
                    "polite" => ak_node.set_live(accesskit::Live::Polite),
                    "assertive" => ak_node.set_live(accesskit::Live::Assertive),
                    _ => {}
                }
            }
        }

        // Walk children
        let child_ids = self.walk_children(el.children(), layout);
        if !child_ids.is_empty() {
            ak_node.set_children(child_ids);
        }

        self.nodes.push((ak_id, ak_node));
        vec![ak_id]
    }

    #[allow(clippy::cast_lossless)]
    fn walk_text(
        &mut self,
        text_node: &dusty_core::TextNode,
        layout: &LayoutResult,
    ) -> Vec<NodeId> {
        let (layout_id, ak_id) = self.alloc_id();

        let mut ak_node = accesskit::Node::new(accesskit::Role::Label);

        let text = text_node.current_text().into_owned();
        ak_node.set_value(text);

        // Set bounds from layout
        if let Some(rect) = layout.get(dusty_layout::LayoutNodeId(layout_id)) {
            ak_node.set_bounds(Rect::new(
                rect.x as f64,
                rect.y as f64,
                (rect.x + rect.width) as f64,
                (rect.y + rect.height) as f64,
            ));
        }

        self.nodes.push((ak_id, ak_node));
        vec![ak_id]
    }

    fn walk_children(&mut self, children: &[Node], layout: &LayoutResult) -> Vec<NodeId> {
        let mut ids = Vec::new();
        for child in children {
            ids.extend(self.walk_node(child, layout));
        }
        ids
    }
}

/// Collects text content from child text nodes to compute an accessible name
/// when no explicit label attribute is set.
fn collect_text_from_children(children: &[Node]) -> Option<String> {
    let mut parts = Vec::new();
    for child in children {
        match child {
            Node::Text(text_node) => {
                let t = text_node.current_text();
                let s = t.trim();
                if !s.is_empty() {
                    parts.push(s.to_string());
                }
            }
            Node::Component(comp) => {
                if let Some(text) = collect_text_from_children(std::slice::from_ref(&comp.child)) {
                    parts.push(text);
                }
            }
            Node::Fragment(children) => {
                if let Some(text) = collect_text_from_children(children) {
                    parts.push(text);
                }
            }
            _ => {}
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" "))
    }
}

fn attr_as_str(val: &AttributeValue) -> Option<Box<str>> {
    match val {
        AttributeValue::String(s) => Some(s.clone().into_boxed_str()),
        _ => None,
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::float_cmp,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
mod tests {
    use super::*;
    use dusty_core::{el, text, text_dynamic, ComponentNode};
    use dusty_layout::{compute_layout, TextMeasure};
    use dusty_reactive::{create_scope, dispose_runtime, initialize_runtime};
    use dusty_style::{FontStyle, Style};

    struct MockMeasure;
    impl TextMeasure for MockMeasure {
        fn measure(&self, text: &str, max_width: Option<f32>, _font: &FontStyle) -> (f32, f32) {
            let char_width = 8.0;
            let line_height = 16.0;
            let text_width = text.len() as f32 * char_width;
            if let Some(max) = max_width {
                if text_width > max {
                    let chars_per_line = (max / char_width).floor() as usize;
                    if chars_per_line == 0 {
                        return (char_width, line_height);
                    }
                    let lines = text.len().div_ceil(chars_per_line);
                    return (max, lines as f32 * line_height);
                }
            }
            (text_width, line_height)
        }
    }

    fn with_scope(f: impl FnOnce(dusty_reactive::Scope)) {
        initialize_runtime();
        create_scope(f).unwrap();
        dispose_runtime();
    }

    #[test]
    fn single_element_produces_tree_update() {
        with_scope(|cx| {
            let node = el("Button", cx)
                .style(Style {
                    width: Some(100.0),
                    height: Some(40.0),
                    ..Style::default()
                })
                .build_node();
            let layout = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
            let update = build_accessibility_tree(&node, &layout, None).unwrap();

            // Should have synthetic root + 1 element = 2 nodes
            assert_eq!(update.nodes.len(), 2);
        });
    }

    #[test]
    fn tree_update_has_root_and_tree() {
        with_scope(|cx| {
            let node = el("Box", cx)
                .style(Style {
                    width: Some(100.0),
                    height: Some(50.0),
                    ..Style::default()
                })
                .build_node();
            let layout = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
            let update = build_accessibility_tree(&node, &layout, None).unwrap();

            assert!(update.tree.is_some());
            assert_eq!(update.tree.unwrap().root, NodeId(0));
        });
    }

    #[test]
    fn element_role_is_set_correctly() {
        with_scope(|cx| {
            let node = el("Button", cx)
                .style(Style {
                    width: Some(100.0),
                    height: Some(40.0),
                    ..Style::default()
                })
                .build_node();
            let layout = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
            let update = build_accessibility_tree(&node, &layout, None).unwrap();

            // Find the Button node (NodeId(1))
            let button = update
                .nodes
                .iter()
                .find(|(id, _)| *id == NodeId(1))
                .map(|(_, n)| n)
                .unwrap();
            assert_eq!(button.role(), accesskit::Role::Button);
        });
    }

    #[test]
    fn text_node_maps_to_label() {
        let node = Node::Text(text("Hello world"));
        let layout = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
        let update = build_accessibility_tree(&node, &layout, None).unwrap();

        let text_ak = update
            .nodes
            .iter()
            .find(|(id, _)| *id == NodeId(1))
            .map(|(_, n)| n)
            .unwrap();
        assert_eq!(text_ak.role(), accesskit::Role::Label);
        assert_eq!(text_ak.value(), Some("Hello world"));
    }

    #[test]
    fn dynamic_text_reads_current_value() {
        let node = Node::Text(text_dynamic(|| "dynamic content".to_string()));
        let layout = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
        let update = build_accessibility_tree(&node, &layout, None).unwrap();

        let text_ak = update
            .nodes
            .iter()
            .find(|(id, _)| *id == NodeId(1))
            .map(|(_, n)| n)
            .unwrap();
        assert_eq!(text_ak.value(), Some("dynamic content"));
    }

    #[test]
    fn element_with_children_has_child_ids() {
        with_scope(|cx| {
            let node = el("Row", cx)
                .style(Style {
                    width: Some(200.0),
                    height: Some(100.0),
                    ..Style::default()
                })
                .child(
                    el("A", cx)
                        .style(Style {
                            width: Some(50.0),
                            height: Some(50.0),
                            ..Style::default()
                        })
                        .build_node(),
                )
                .child(
                    el("B", cx)
                        .style(Style {
                            width: Some(50.0),
                            height: Some(50.0),
                            ..Style::default()
                        })
                        .build_node(),
                )
                .build_node();
            let layout = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
            let update = build_accessibility_tree(&node, &layout, None).unwrap();

            // Row is NodeId(1), A is NodeId(2), B is NodeId(3)
            let row = update
                .nodes
                .iter()
                .find(|(id, _)| *id == NodeId(1))
                .map(|(_, n)| n)
                .unwrap();
            let children = row.children();
            assert_eq!(children.len(), 2);
            assert_eq!(children[0], NodeId(2));
            assert_eq!(children[1], NodeId(3));
        });
    }

    #[test]
    fn nested_elements_correct_hierarchy() {
        with_scope(|cx| {
            let node = el("Outer", cx)
                .style(Style {
                    width: Some(300.0),
                    height: Some(300.0),
                    ..Style::default()
                })
                .child(
                    el("Middle", cx)
                        .style(Style {
                            width: Some(200.0),
                            height: Some(200.0),
                            ..Style::default()
                        })
                        .child(
                            el("Inner", cx)
                                .style(Style {
                                    width: Some(100.0),
                                    height: Some(100.0),
                                    ..Style::default()
                                })
                                .build_node(),
                        )
                        .build_node(),
                )
                .build_node();
            let layout = compute_layout(&node, 400.0, 400.0, &MockMeasure).unwrap();
            let update = build_accessibility_tree(&node, &layout, None).unwrap();

            // 4 nodes: root(0), Outer(1), Middle(2), Inner(3)
            assert_eq!(update.nodes.len(), 4);

            let outer = update
                .nodes
                .iter()
                .find(|(id, _)| *id == NodeId(1))
                .map(|(_, n)| n)
                .unwrap();
            assert_eq!(outer.children(), &[NodeId(2)]);

            let middle = update
                .nodes
                .iter()
                .find(|(id, _)| *id == NodeId(2))
                .map(|(_, n)| n)
                .unwrap();
            assert_eq!(middle.children(), &[NodeId(3)]);

            let inner = update
                .nodes
                .iter()
                .find(|(id, _)| *id == NodeId(3))
                .map(|(_, n)| n)
                .unwrap();
            assert!(inner.children().is_empty());
        });
    }

    #[test]
    fn fragment_children_flattened() {
        with_scope(|cx| {
            let frag = Node::Fragment(vec![
                el("A", cx)
                    .style(Style {
                        width: Some(50.0),
                        height: Some(50.0),
                        ..Style::default()
                    })
                    .build_node(),
                el("B", cx)
                    .style(Style {
                        width: Some(50.0),
                        height: Some(50.0),
                        ..Style::default()
                    })
                    .build_node(),
            ]);

            let parent = el("Parent", cx)
                .style(Style {
                    width: Some(200.0),
                    height: Some(100.0),
                    ..Style::default()
                })
                .child_node(frag)
                .build_node();

            let layout = compute_layout(&parent, 400.0, 300.0, &MockMeasure).unwrap();
            let update = build_accessibility_tree(&parent, &layout, None).unwrap();

            // Parent(1) should have A(2) and B(3) as direct children
            let parent_ak = update
                .nodes
                .iter()
                .find(|(id, _)| *id == NodeId(1))
                .map(|(_, n)| n)
                .unwrap();
            assert_eq!(parent_ak.children(), &[NodeId(2), NodeId(3)]);
        });
    }

    #[test]
    fn fragment_consumes_no_id() {
        with_scope(|cx| {
            let frag = Node::Fragment(vec![
                el("A", cx)
                    .style(Style {
                        width: Some(50.0),
                        height: Some(50.0),
                        ..Style::default()
                    })
                    .build_node(),
                el("B", cx)
                    .style(Style {
                        width: Some(50.0),
                        height: Some(50.0),
                        ..Style::default()
                    })
                    .build_node(),
            ]);

            let parent = el("Parent", cx)
                .style(Style {
                    width: Some(200.0),
                    height: Some(100.0),
                    ..Style::default()
                })
                .child_node(frag)
                .build_node();

            let layout = compute_layout(&parent, 400.0, 300.0, &MockMeasure).unwrap();
            let update = build_accessibility_tree(&parent, &layout, None).unwrap();

            // Parent + A + B + root = 4 nodes (Fragment is transparent)
            assert_eq!(update.nodes.len(), 4);
        });
    }

    #[test]
    fn component_node_transparent() {
        with_scope(|cx| {
            let inner = el("Inner", cx)
                .style(Style {
                    width: Some(80.0),
                    height: Some(40.0),
                    ..Style::default()
                })
                .build_node();

            let comp = Node::Component(ComponentNode {
                name: "MyComponent",
                child: Box::new(inner),
            });

            let parent = el("Parent", cx)
                .style(Style {
                    width: Some(200.0),
                    height: Some(100.0),
                    ..Style::default()
                })
                .child_node(comp)
                .build_node();

            let layout = compute_layout(&parent, 400.0, 300.0, &MockMeasure).unwrap();
            let update = build_accessibility_tree(&parent, &layout, None).unwrap();

            // Parent + Inner + root = 3 nodes (Component is transparent)
            assert_eq!(update.nodes.len(), 3);
        });
    }

    #[test]
    fn label_attribute_sets_name() {
        with_scope(|cx| {
            let node = el("Button", cx)
                .attr("label", "Submit")
                .style(Style {
                    width: Some(100.0),
                    height: Some(40.0),
                    ..Style::default()
                })
                .build_node();
            let layout = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
            let update = build_accessibility_tree(&node, &layout, None).unwrap();

            let button = update
                .nodes
                .iter()
                .find(|(id, _)| *id == NodeId(1))
                .map(|(_, n)| n)
                .unwrap();
            assert_eq!(button.label(), Some("Submit"));
        });
    }

    #[test]
    fn aria_label_attribute_sets_name() {
        with_scope(|cx| {
            let node = el("Button", cx)
                .attr("aria-label", "Close")
                .style(Style {
                    width: Some(100.0),
                    height: Some(40.0),
                    ..Style::default()
                })
                .build_node();
            let layout = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
            let update = build_accessibility_tree(&node, &layout, None).unwrap();

            let button = update
                .nodes
                .iter()
                .find(|(id, _)| *id == NodeId(1))
                .map(|(_, n)| n)
                .unwrap();
            assert_eq!(button.label(), Some("Close"));
        });
    }

    #[test]
    fn description_attribute_sets_description() {
        with_scope(|cx| {
            let node = el("Button", cx)
                .attr("description", "Click to submit the form")
                .style(Style {
                    width: Some(100.0),
                    height: Some(40.0),
                    ..Style::default()
                })
                .build_node();
            let layout = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
            let update = build_accessibility_tree(&node, &layout, None).unwrap();

            let button = update
                .nodes
                .iter()
                .find(|(id, _)| *id == NodeId(1))
                .map(|(_, n)| n)
                .unwrap();
            assert_eq!(button.description(), Some("Click to submit the form"));
        });
    }

    #[test]
    fn placeholder_as_fallback_name() {
        with_scope(|cx| {
            let node = el("TextInput", cx)
                .attr("placeholder", "Enter name")
                .style(Style {
                    width: Some(200.0),
                    height: Some(30.0),
                    ..Style::default()
                })
                .build_node();
            let layout = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
            let update = build_accessibility_tree(&node, &layout, None).unwrap();

            let input = update
                .nodes
                .iter()
                .find(|(id, _)| *id == NodeId(1))
                .map(|(_, n)| n)
                .unwrap();
            assert_eq!(input.label(), Some("Enter name"));
        });
    }

    #[test]
    fn label_takes_precedence_over_placeholder() {
        with_scope(|cx| {
            let node = el("TextInput", cx)
                .attr("label", "Full Name")
                .attr("placeholder", "Enter name")
                .style(Style {
                    width: Some(200.0),
                    height: Some(30.0),
                    ..Style::default()
                })
                .build_node();
            let layout = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
            let update = build_accessibility_tree(&node, &layout, None).unwrap();

            let input = update
                .nodes
                .iter()
                .find(|(id, _)| *id == NodeId(1))
                .map(|(_, n)| n)
                .unwrap();
            assert_eq!(input.label(), Some("Full Name"));
        });
    }

    #[test]
    fn value_attribute_sets_value() {
        with_scope(|cx| {
            let node = el("TextInput", cx)
                .attr("value", "hello")
                .style(Style {
                    width: Some(200.0),
                    height: Some(30.0),
                    ..Style::default()
                })
                .build_node();
            let layout = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
            let update = build_accessibility_tree(&node, &layout, None).unwrap();

            let input = update
                .nodes
                .iter()
                .find(|(id, _)| *id == NodeId(1))
                .map(|(_, n)| n)
                .unwrap();
            assert_eq!(input.value(), Some("hello"));
        });
    }

    #[test]
    fn disabled_attribute_sets_disabled() {
        with_scope(|cx| {
            let node = el("Button", cx)
                .attr("disabled", true)
                .style(Style {
                    width: Some(100.0),
                    height: Some(40.0),
                    ..Style::default()
                })
                .build_node();
            let layout = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
            let update = build_accessibility_tree(&node, &layout, None).unwrap();

            let button = update
                .nodes
                .iter()
                .find(|(id, _)| *id == NodeId(1))
                .map(|(_, n)| n)
                .unwrap();
            assert!(button.is_disabled());
        });
    }

    #[test]
    fn checked_true_sets_toggled() {
        with_scope(|cx| {
            let node = el("Checkbox", cx)
                .attr("checked", true)
                .style(Style {
                    width: Some(20.0),
                    height: Some(20.0),
                    ..Style::default()
                })
                .build_node();
            let layout = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
            let update = build_accessibility_tree(&node, &layout, None).unwrap();

            let cb = update
                .nodes
                .iter()
                .find(|(id, _)| *id == NodeId(1))
                .map(|(_, n)| n)
                .unwrap();
            assert_eq!(cb.toggled(), Some(accesskit::Toggled::True));
        });
    }

    #[test]
    fn checked_false_sets_toggled_false() {
        with_scope(|cx| {
            let node = el("Checkbox", cx)
                .attr("checked", false)
                .style(Style {
                    width: Some(20.0),
                    height: Some(20.0),
                    ..Style::default()
                })
                .build_node();
            let layout = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
            let update = build_accessibility_tree(&node, &layout, None).unwrap();

            let cb = update
                .nodes
                .iter()
                .find(|(id, _)| *id == NodeId(1))
                .map(|(_, n)| n)
                .unwrap();
            assert_eq!(cb.toggled(), Some(accesskit::Toggled::False));
        });
    }

    #[test]
    fn focus_targets_correct_node() {
        with_scope(|cx| {
            let node = el("Row", cx)
                .style(Style {
                    width: Some(200.0),
                    height: Some(100.0),
                    ..Style::default()
                })
                .child(
                    el("Button", cx)
                        .style(Style {
                            width: Some(80.0),
                            height: Some(30.0),
                            ..Style::default()
                        })
                        .build_node(),
                )
                .build_node();
            let layout = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
            let update = build_accessibility_tree(&node, &layout, Some(1)).unwrap();

            assert_eq!(update.focus, NodeId(2));
        });
    }

    #[test]
    fn no_focus_defaults_to_root() {
        with_scope(|cx| {
            let node = el("Box", cx)
                .style(Style {
                    width: Some(100.0),
                    height: Some(50.0),
                    ..Style::default()
                })
                .build_node();
            let layout = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
            let update = build_accessibility_tree(&node, &layout, None).unwrap();

            assert_eq!(update.focus, NodeId(0));
        });
    }

    #[test]
    fn element_has_correct_bounds() {
        with_scope(|cx| {
            let node = el("Box", cx)
                .style(Style {
                    width: Some(200.0),
                    height: Some(100.0),
                    ..Style::default()
                })
                .build_node();
            let layout = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
            let update = build_accessibility_tree(&node, &layout, None).unwrap();

            let box_node = update
                .nodes
                .iter()
                .find(|(id, _)| *id == NodeId(1))
                .map(|(_, n)| n)
                .unwrap();
            let bounds = box_node.bounds().unwrap();
            assert_eq!(bounds.x0, 0.0);
            assert_eq!(bounds.y0, 0.0);
            assert_eq!(bounds.x1, 200.0);
            assert_eq!(bounds.y1, 100.0);
        });
    }

    #[test]
    fn aria_live_polite_sets_live() {
        with_scope(|cx| {
            let node = el("Row", cx)
                .attr("aria-live", "polite")
                .style(Style {
                    width: Some(200.0),
                    height: Some(100.0),
                    ..Style::default()
                })
                .build_node();
            let layout = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
            let update = build_accessibility_tree(&node, &layout, None).unwrap();

            let row = update
                .nodes
                .iter()
                .find(|(id, _)| *id == NodeId(1))
                .map(|(_, n)| n)
                .unwrap();
            assert_eq!(row.live(), Some(accesskit::Live::Polite));
        });
    }

    #[test]
    fn aria_live_assertive_sets_live() {
        with_scope(|cx| {
            let node = el("Row", cx)
                .attr("aria-live", "assertive")
                .style(Style {
                    width: Some(200.0),
                    height: Some(100.0),
                    ..Style::default()
                })
                .build_node();
            let layout = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
            let update = build_accessibility_tree(&node, &layout, None).unwrap();

            let row = update
                .nodes
                .iter()
                .find(|(id, _)| *id == NodeId(1))
                .map(|(_, n)| n)
                .unwrap();
            assert_eq!(row.live(), Some(accesskit::Live::Assertive));
        });
    }

    #[test]
    fn a11y_tree_computes_name_from_child_text() {
        with_scope(|cx| {
            let node = el("Button", cx)
                .style(Style {
                    width: Some(100.0),
                    height: Some(40.0),
                    ..Style::default()
                })
                .child(text("Submit"))
                .build_node();
            let layout = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
            let update = build_accessibility_tree(&node, &layout, None).unwrap();

            let button = update
                .nodes
                .iter()
                .find(|(id, _)| *id == NodeId(1))
                .map(|(_, n)| n)
                .unwrap();
            // Label should be computed from child text
            assert_eq!(button.label(), Some("Submit"));
        });
    }

    #[test]
    fn no_aria_live_no_live_region() {
        with_scope(|cx| {
            let node = el("Row", cx)
                .style(Style {
                    width: Some(200.0),
                    height: Some(100.0),
                    ..Style::default()
                })
                .build_node();
            let layout = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
            let update = build_accessibility_tree(&node, &layout, None).unwrap();

            let row = update
                .nodes
                .iter()
                .find(|(id, _)| *id == NodeId(1))
                .map(|(_, n)| n)
                .unwrap();
            assert_eq!(row.live(), None);
        });
    }

    #[test]
    fn empty_fragment_returns_error() {
        let node = Node::Fragment(vec![]);
        // Empty fragment produces EmptyTree from compute_layout, but we
        // also verify build_accessibility_tree returns EmptyTree for empty input.
        // Use a single-node layout as a stand-in (the walker sees no nodes from
        // an empty Fragment, so it returns EmptyTree).
        with_scope(|cx| {
            let helper = el("X", cx)
                .style(Style {
                    width: Some(10.0),
                    height: Some(10.0),
                    ..Style::default()
                })
                .build_node();
            let layout = compute_layout(&helper, 100.0, 100.0, &MockMeasure).unwrap();
            // Walk an empty fragment against any layout → no IDs consumed → EmptyTree
            let result = build_accessibility_tree(&node, &layout, None);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), A11yError::EmptyTree);
        });
    }

    #[test]
    fn node_id_assignment_matches_layout() {
        with_scope(|cx| {
            let node = el("Root", cx)
                .style(Style {
                    width: Some(200.0),
                    height: Some(100.0),
                    ..Style::default()
                })
                .child(text("hello"))
                .build_node();

            let layout = compute_layout(&node, 200.0, 100.0, &MockMeasure).unwrap();

            // Layout assigns ID 0 to Root, ID 1 to text
            assert_eq!(layout.len(), 2);

            // A11y should assign NodeId(1) to Root, NodeId(2) to text
            // (matching layout IDs + 1)
            let update = build_accessibility_tree(&node, &layout, None).unwrap();

            // Root element at NodeId(1) should have text at NodeId(2)
            let root_el = update
                .nodes
                .iter()
                .find(|(id, _)| *id == NodeId(1))
                .map(|(_, n)| n)
                .unwrap();
            assert_eq!(root_el.children(), &[NodeId(2)]);

            // Text at NodeId(2) should have the text content
            let text_node = update
                .nodes
                .iter()
                .find(|(id, _)| *id == NodeId(2))
                .map(|(_, n)| n)
                .unwrap();
            assert_eq!(text_node.value(), Some("hello"));
        });
    }
}
