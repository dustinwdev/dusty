//! Integration tests for the node tree inspector.

#![allow(
    clippy::unwrap_used,
    clippy::float_cmp,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation
)]

use dusty_core::event::{ClickEvent, FocusEvent, HoverEvent};
use dusty_core::{el, text, text_dynamic, ComponentNode, Node};
use dusty_devtools::inspector::{inspect, InspectorNodeKind, NodeBounds};
use dusty_devtools::DevtoolsError;
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
    create_scope(|cx| f(cx));
    dispose_runtime();
}

#[test]
fn single_element_produces_one_node() {
    with_scope(|cx| {
        let node = el("Button", cx).attr("label", "OK").build_node();
        let tree = inspect(&node, None).unwrap();

        assert_eq!(tree.nodes.len(), 1);
        assert_eq!(tree.root_indices, vec![0]);
        assert!(matches!(
            tree.nodes[0].kind,
            InspectorNodeKind::Element { name: "Button" }
        ));
        assert_eq!(tree.nodes[0].depth, 0);
    });
}

#[test]
fn text_node_captures_content() {
    let node = Node::Text(text("hello world"));
    let tree = inspect(&node, None).unwrap();

    assert_eq!(tree.nodes.len(), 1);
    assert!(matches!(
        &tree.nodes[0].kind,
        InspectorNodeKind::Text { content } if content == "hello world"
    ));
}

#[test]
fn dynamic_text_captured_without_tracking() {
    let node = Node::Text(text_dynamic(|| "computed".to_string()));
    let tree = inspect(&node, None).unwrap();

    assert!(matches!(
        &tree.nodes[0].kind,
        InspectorNodeKind::Text { content } if content == "computed"
    ));
}

#[test]
fn fragment_children_flattened_to_root() {
    with_scope(|cx| {
        let frag = Node::Fragment(vec![el("A", cx).build_node(), el("B", cx).build_node()]);
        let tree = inspect(&frag, None).unwrap();

        // Fragment is transparent — two root nodes
        assert_eq!(tree.nodes.len(), 2);
        assert_eq!(tree.root_indices, vec![0, 1]);
        assert!(matches!(
            tree.nodes[0].kind,
            InspectorNodeKind::Element { name: "A" }
        ));
        assert!(matches!(
            tree.nodes[1].kind,
            InspectorNodeKind::Element { name: "B" }
        ));
    });
}

#[test]
fn fragment_inside_element_flattened() {
    with_scope(|cx| {
        let frag = Node::Fragment(vec![el("X", cx).build_node(), el("Y", cx).build_node()]);
        let parent = el("Row", cx).child_node(frag).build_node();
        let tree = inspect(&parent, None).unwrap();

        // Row + X + Y = 3 nodes
        assert_eq!(tree.nodes.len(), 3);
        assert_eq!(tree.root_indices, vec![0]);
        // Row's children are X and Y (fragment is transparent)
        assert_eq!(tree.nodes[0].child_indices, vec![1, 2]);
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

        // Component wrapper is transparent
        assert_eq!(tree.nodes.len(), 1);
        assert!(matches!(
            tree.nodes[0].kind,
            InspectorNodeKind::Element { name: "Inner" }
        ));
    });
}

#[test]
fn dynamic_node_resolved() {
    let dn = dusty_core::dynamic_node(|| Node::Text(text("resolved")));
    let node = Node::Dynamic(dn);
    let tree = inspect(&node, None).unwrap();

    // Dynamic node + resolved text = 2 nodes
    assert_eq!(tree.nodes.len(), 2);
    assert!(matches!(tree.nodes[0].kind, InspectorNodeKind::Dynamic));
    assert!(matches!(
        &tree.nodes[1].kind,
        InspectorNodeKind::Text { content } if content == "resolved"
    ));
    assert_eq!(tree.nodes[0].child_indices, vec![1]);
}

#[test]
fn with_layout_bounds() {
    with_scope(|cx| {
        let node = el("Box", cx)
            .style(Style {
                width: Some(200.0),
                height: Some(100.0),
                ..Style::default()
            })
            .build_node();
        let layout = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
        let tree = inspect(&node, Some(&layout)).unwrap();

        let bounds = tree.nodes[0].bounds.unwrap();
        assert_eq!(bounds.x, 0.0);
        assert_eq!(bounds.y, 0.0);
        assert_eq!(bounds.width, 200.0);
        assert_eq!(bounds.height, 100.0);
    });
}

#[test]
fn without_layout_bounds_are_none() {
    with_scope(|cx| {
        let node = el("Box", cx).build_node();
        let tree = inspect(&node, None).unwrap();
        assert!(tree.nodes[0].bounds.is_none());
    });
}

#[test]
fn style_extraction_from_element() {
    with_scope(|cx| {
        let style = Style {
            width: Some(100.0),
            height: Some(50.0),
            ..Style::default()
        };
        let node = el("Box", cx).style(style).build_node();
        let tree = inspect(&node, None).unwrap();

        let snapshot = tree.nodes[0].style.as_ref().unwrap();
        assert_eq!(snapshot.0.width, Some(100.0));
        assert_eq!(snapshot.0.height, Some(50.0));
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
fn event_handlers_listed_by_name() {
    with_scope(|cx| {
        let node = el("Button", cx)
            .on_click(|_e: &ClickEvent| {})
            .on_hover(|_e: &HoverEvent| {})
            .on_focus(|_e: &FocusEvent| {})
            .build_node();
        let tree = inspect(&node, None).unwrap();

        let handlers = &tree.nodes[0].event_handlers;
        assert_eq!(handlers.len(), 3);
        assert!(handlers.contains(&"click"));
        assert!(handlers.contains(&"hover"));
        assert!(handlers.contains(&"focus"));
    });
}

#[test]
fn attributes_extracted_as_strings() {
    with_scope(|cx| {
        let node = el("Input", cx)
            .attr("placeholder", "type here")
            .attr("max_length", 100i64)
            .attr("opacity", 0.5f64)
            .attr("disabled", false)
            .build_node();
        let tree = inspect(&node, None).unwrap();

        let attrs = &tree.nodes[0].attributes;
        assert_eq!(attrs.len(), 4);
        assert!(attrs.contains(&("placeholder".to_string(), "type here".to_string())));
        assert!(attrs.contains(&("max_length".to_string(), "100".to_string())));
        assert!(attrs.contains(&("opacity".to_string(), "0.5".to_string())));
        assert!(attrs.contains(&("disabled".to_string(), "false".to_string())));
    });
}

#[test]
fn empty_tree_returns_error() {
    let empty = Node::Fragment(vec![]);
    let result = inspect(&empty, None);
    assert_eq!(result.unwrap_err(), DevtoolsError::EmptyTree);
}

#[test]
fn nested_tree_correct_indices() {
    with_scope(|cx| {
        let node = el("Outer", cx)
            .child(
                el("Middle", cx)
                    .child(el("Inner", cx).build_node())
                    .build_node(),
            )
            .build_node();
        let tree = inspect(&node, None).unwrap();

        assert_eq!(tree.nodes.len(), 3);
        assert_eq!(tree.root_indices, vec![0]);
        assert_eq!(tree.nodes[0].child_indices, vec![1]); // Outer -> Middle
        assert_eq!(tree.nodes[1].child_indices, vec![2]); // Middle -> Inner
        assert!(tree.nodes[2].child_indices.is_empty()); // Inner has no children
    });
}

#[test]
fn layout_id_assignment_matches_layout() {
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
        let tree = inspect(&node, Some(&layout)).unwrap();

        // Root element gets layout ID 0, text gets layout ID 1
        // Both should have bounds
        assert!(tree.nodes[0].bounds.is_some());
        assert!(tree.nodes[1].bounds.is_some());

        let root_bounds = tree.nodes[0].bounds.unwrap();
        assert_eq!(
            root_bounds,
            NodeBounds {
                x: 0.0,
                y: 0.0,
                width: 200.0,
                height: 100.0,
            }
        );
    });
}

#[test]
fn fragment_does_not_consume_layout_id() {
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
        let tree = inspect(&parent, Some(&layout)).unwrap();

        // Parent gets layout ID 0, A gets 1, B gets 2
        // Fragment doesn't consume an ID
        assert!(tree.nodes[0].bounds.is_some()); // Parent
        assert!(tree.nodes[1].bounds.is_some()); // A
        assert!(tree.nodes[2].bounds.is_some()); // B
    });
}
