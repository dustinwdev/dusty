#![allow(
    clippy::unwrap_used,
    clippy::float_cmp,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use dusty_core::{el, text, ComponentNode, Node};
use dusty_layout::{compute_layout, LayoutError, LayoutNodeId, TextMeasure};
use dusty_reactive::{create_scope, dispose_runtime, initialize_runtime, Scope};
use dusty_style::{Edges, FlexDirection, FlexWrap, FontStyle, Length, LengthPercent, Style};

/// Mock text measure: 8px per character wide, 16px line height.
/// Wraps at `max_width` if provided.
struct MockTextMeasure;

impl TextMeasure for MockTextMeasure {
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

fn with_scope(f: impl FnOnce(Scope)) {
    initialize_runtime();
    create_scope(f);
    dispose_runtime();
}

// --- Basic layout ---

#[test]
fn single_element_fills_available() {
    with_scope(|cx| {
        let node = el("Root", cx).style(Style::default()).build_node();

        let result = compute_layout(&node, 800.0, 600.0, &MockTextMeasure).unwrap();
        assert_eq!(result.len(), 1);
        let root = result.root_rect().unwrap();
        // Default flex element with no fixed size fills available width, height 0 (no content)
        assert_eq!(root.x, 0.0);
        assert_eq!(root.y, 0.0);
    });
}

#[test]
fn element_with_fixed_size() {
    with_scope(|cx| {
        let node = el("Box", cx)
            .style(Style {
                width: Some(Length::Px(200.0)),
                height: Some(Length::Px(150.0)),
                ..Style::default()
            })
            .build_node();

        let result = compute_layout(&node, 800.0, 600.0, &MockTextMeasure).unwrap();
        let rect = result.root_rect().unwrap();
        assert_eq!(rect.width, 200.0);
        assert_eq!(rect.height, 150.0);
    });
}

// --- Row/Column layout ---

#[test]
fn row_layout_distributes_children() {
    with_scope(|cx| {
        let node = el("Row", cx)
            .style(Style {
                width: Some(Length::Px(300.0)),
                height: Some(Length::Px(100.0)),
                flex_direction: Some(FlexDirection::Row),
                ..Style::default()
            })
            .child(
                el("A", cx)
                    .style(Style {
                        flex_grow: Some(1.0),
                        ..Style::default()
                    })
                    .build_node(),
            )
            .child(
                el("B", cx)
                    .style(Style {
                        flex_grow: Some(1.0),
                        ..Style::default()
                    })
                    .build_node(),
            )
            .build_node();

        let result = compute_layout(&node, 300.0, 100.0, &MockTextMeasure).unwrap();
        let a = result.get(LayoutNodeId(1)).unwrap();
        let b = result.get(LayoutNodeId(2)).unwrap();

        assert_eq!(a.width, 150.0);
        assert_eq!(b.width, 150.0);
        assert_eq!(a.x, 0.0);
        assert_eq!(b.x, 150.0);
    });
}

#[test]
fn column_layout_stacks_vertically() {
    with_scope(|cx| {
        let node = el("Col", cx)
            .style(Style {
                width: Some(Length::Px(200.0)),
                height: Some(Length::Px(300.0)),
                flex_direction: Some(FlexDirection::Column),
                ..Style::default()
            })
            .child(
                el("A", cx)
                    .style(Style {
                        height: Some(Length::Px(80.0)),
                        ..Style::default()
                    })
                    .build_node(),
            )
            .child(
                el("B", cx)
                    .style(Style {
                        height: Some(Length::Px(60.0)),
                        ..Style::default()
                    })
                    .build_node(),
            )
            .build_node();

        let result = compute_layout(&node, 200.0, 300.0, &MockTextMeasure).unwrap();
        let a = result.get(LayoutNodeId(1)).unwrap();
        let b = result.get(LayoutNodeId(2)).unwrap();

        assert_eq!(a.y, 0.0);
        assert_eq!(a.height, 80.0);
        assert_eq!(b.y, 80.0);
        assert_eq!(b.height, 60.0);
    });
}

// --- Nested containers ---

#[test]
fn nested_flex_containers() {
    with_scope(|cx| {
        let node = el("Outer", cx)
            .style(Style {
                width: Some(Length::Px(400.0)),
                height: Some(Length::Px(200.0)),
                flex_direction: Some(FlexDirection::Row),
                ..Style::default()
            })
            .child(
                el("Left", cx)
                    .style(Style {
                        width: Some(Length::Px(200.0)),
                        flex_direction: Some(FlexDirection::Column),
                        ..Style::default()
                    })
                    .child(
                        el("TopLeft", cx)
                            .style(Style {
                                height: Some(Length::Px(100.0)),
                                ..Style::default()
                            })
                            .build_node(),
                    )
                    .child(
                        el("BottomLeft", cx)
                            .style(Style {
                                height: Some(Length::Px(100.0)),
                                ..Style::default()
                            })
                            .build_node(),
                    )
                    .build_node(),
            )
            .child(
                el("Right", cx)
                    .style(Style {
                        width: Some(Length::Px(200.0)),
                        ..Style::default()
                    })
                    .build_node(),
            )
            .build_node();

        let result = compute_layout(&node, 400.0, 200.0, &MockTextMeasure).unwrap();

        let left = result.get(LayoutNodeId(1)).unwrap();
        assert_eq!(left.x, 0.0);
        assert_eq!(left.width, 200.0);

        let top_left = result.get(LayoutNodeId(2)).unwrap();
        assert_eq!(top_left.x, 0.0);
        assert_eq!(top_left.y, 0.0);
        assert_eq!(top_left.height, 100.0);

        let bottom_left = result.get(LayoutNodeId(3)).unwrap();
        assert_eq!(bottom_left.x, 0.0);
        assert_eq!(bottom_left.y, 100.0);

        let right = result.get(LayoutNodeId(4)).unwrap();
        assert_eq!(right.x, 200.0);
    });
}

// --- Gap ---

#[test]
fn gap_between_children() {
    with_scope(|cx| {
        let node = el("Row", cx)
            .style(Style {
                width: Some(Length::Px(400.0)),
                height: Some(Length::Px(100.0)),
                flex_direction: Some(FlexDirection::Row),
                gap: Some(LengthPercent::Px(20.0)),
                ..Style::default()
            })
            .child(
                el("A", cx)
                    .style(Style {
                        width: Some(Length::Px(100.0)),
                        ..Style::default()
                    })
                    .build_node(),
            )
            .child(
                el("B", cx)
                    .style(Style {
                        width: Some(Length::Px(100.0)),
                        ..Style::default()
                    })
                    .build_node(),
            )
            .child(
                el("C", cx)
                    .style(Style {
                        width: Some(Length::Px(100.0)),
                        ..Style::default()
                    })
                    .build_node(),
            )
            .build_node();

        let result = compute_layout(&node, 400.0, 100.0, &MockTextMeasure).unwrap();
        let a = result.get(LayoutNodeId(1)).unwrap();
        let b = result.get(LayoutNodeId(2)).unwrap();
        let c = result.get(LayoutNodeId(3)).unwrap();

        assert_eq!(a.x, 0.0);
        assert_eq!(b.x, 120.0); // 100 + 20 gap
        assert_eq!(c.x, 240.0); // 200 + 40 gap
    });
}

// --- Padding ---

#[test]
fn padding_affects_content_area() {
    with_scope(|cx| {
        let node = el("Container", cx)
            .style(Style {
                width: Some(Length::Px(200.0)),
                height: Some(Length::Px(200.0)),
                padding: Edges::all(LengthPercent::Px(20.0)),
                flex_direction: Some(FlexDirection::Column),
                ..Style::default()
            })
            .child(
                el("Child", cx)
                    .style(Style {
                        width: Some(Length::Px(100.0)),
                        height: Some(Length::Px(50.0)),
                        ..Style::default()
                    })
                    .build_node(),
            )
            .build_node();

        let result = compute_layout(&node, 200.0, 200.0, &MockTextMeasure).unwrap();
        let child = result.get(LayoutNodeId(1)).unwrap();

        // Child is positioned inside padding
        assert_eq!(child.x, 20.0);
        assert_eq!(child.y, 20.0);
    });
}

// --- Margin ---

#[test]
fn margin_offsets_position() {
    with_scope(|cx| {
        let node = el("Container", cx)
            .style(Style {
                width: Some(Length::Px(300.0)),
                height: Some(Length::Px(300.0)),
                flex_direction: Some(FlexDirection::Column),
                ..Style::default()
            })
            .child(
                el("Child", cx)
                    .style(Style {
                        width: Some(Length::Px(100.0)),
                        height: Some(Length::Px(50.0)),
                        margin: Edges::new(
                            Length::Px(15.0),
                            Length::Px(0.0),
                            Length::Px(0.0),
                            Length::Px(25.0),
                        ),
                        ..Style::default()
                    })
                    .build_node(),
            )
            .build_node();

        let result = compute_layout(&node, 300.0, 300.0, &MockTextMeasure).unwrap();
        let child = result.get(LayoutNodeId(1)).unwrap();

        assert_eq!(child.x, 25.0); // left margin
        assert_eq!(child.y, 15.0); // top margin
    });
}

// --- Alignment ---

#[test]
fn align_items_center() {
    with_scope(|cx| {
        let node = el("Row", cx)
            .style(Style {
                width: Some(Length::Px(300.0)),
                height: Some(Length::Px(100.0)),
                flex_direction: Some(FlexDirection::Row),
                align_items: Some(dusty_style::AlignItems::Center),
                ..Style::default()
            })
            .child(
                el("Short", cx)
                    .style(Style {
                        width: Some(Length::Px(50.0)),
                        height: Some(Length::Px(20.0)),
                        ..Style::default()
                    })
                    .build_node(),
            )
            .build_node();

        let result = compute_layout(&node, 300.0, 100.0, &MockTextMeasure).unwrap();
        let child = result.get(LayoutNodeId(1)).unwrap();

        // Centered: (100 - 20) / 2 = 40
        assert_eq!(child.y, 40.0);
        assert_eq!(child.height, 20.0);
    });
}

#[test]
fn justify_content_center() {
    with_scope(|cx| {
        let node = el("Row", cx)
            .style(Style {
                width: Some(Length::Px(300.0)),
                height: Some(Length::Px(100.0)),
                flex_direction: Some(FlexDirection::Row),
                justify_content: Some(dusty_style::JustifyContent::Center),
                ..Style::default()
            })
            .child(
                el("A", cx)
                    .style(Style {
                        width: Some(Length::Px(100.0)),
                        height: Some(Length::Px(50.0)),
                        ..Style::default()
                    })
                    .build_node(),
            )
            .build_node();

        let result = compute_layout(&node, 300.0, 100.0, &MockTextMeasure).unwrap();
        let child = result.get(LayoutNodeId(1)).unwrap();

        // Centered: (300 - 100) / 2 = 100
        assert_eq!(child.x, 100.0);
    });
}

#[test]
fn justify_content_space_between() {
    with_scope(|cx| {
        let node = el("Row", cx)
            .style(Style {
                width: Some(Length::Px(300.0)),
                height: Some(Length::Px(100.0)),
                flex_direction: Some(FlexDirection::Row),
                justify_content: Some(dusty_style::JustifyContent::SpaceBetween),
                ..Style::default()
            })
            .child(
                el("A", cx)
                    .style(Style {
                        width: Some(Length::Px(50.0)),
                        ..Style::default()
                    })
                    .build_node(),
            )
            .child(
                el("B", cx)
                    .style(Style {
                        width: Some(Length::Px(50.0)),
                        ..Style::default()
                    })
                    .build_node(),
            )
            .build_node();

        let result = compute_layout(&node, 300.0, 100.0, &MockTextMeasure).unwrap();
        let a = result.get(LayoutNodeId(1)).unwrap();
        let b = result.get(LayoutNodeId(2)).unwrap();

        assert_eq!(a.x, 0.0);
        assert_eq!(b.x, 250.0); // 300 - 50
    });
}

// --- Flex grow/shrink/wrap ---

#[test]
fn flex_grow_proportional() {
    with_scope(|cx| {
        let node = el("Row", cx)
            .style(Style {
                width: Some(Length::Px(300.0)),
                height: Some(Length::Px(100.0)),
                flex_direction: Some(FlexDirection::Row),
                ..Style::default()
            })
            .child(
                el("A", cx)
                    .style(Style {
                        flex_grow: Some(1.0),
                        ..Style::default()
                    })
                    .build_node(),
            )
            .child(
                el("B", cx)
                    .style(Style {
                        flex_grow: Some(2.0),
                        ..Style::default()
                    })
                    .build_node(),
            )
            .build_node();

        let result = compute_layout(&node, 300.0, 100.0, &MockTextMeasure).unwrap();
        let a = result.get(LayoutNodeId(1)).unwrap();
        let b = result.get(LayoutNodeId(2)).unwrap();

        assert_eq!(a.width, 100.0); // 1/3
        assert_eq!(b.width, 200.0); // 2/3
    });
}

#[test]
fn flex_wrap_wraps_to_next_line() {
    with_scope(|cx| {
        // No fixed height — container sizes to content so wrap lines aren't stretched.
        let node = el("Row", cx)
            .style(Style {
                width: Some(Length::Px(200.0)),
                flex_direction: Some(FlexDirection::Row),
                flex_wrap: Some(FlexWrap::Wrap),
                ..Style::default()
            })
            .child(
                el("A", cx)
                    .style(Style {
                        width: Some(Length::Px(120.0)),
                        height: Some(Length::Px(50.0)),
                        ..Style::default()
                    })
                    .build_node(),
            )
            .child(
                el("B", cx)
                    .style(Style {
                        width: Some(Length::Px(120.0)),
                        height: Some(Length::Px(50.0)),
                        ..Style::default()
                    })
                    .build_node(),
            )
            .build_node();

        let result = compute_layout(&node, 200.0, 200.0, &MockTextMeasure).unwrap();
        let a = result.get(LayoutNodeId(1)).unwrap();
        let b = result.get(LayoutNodeId(2)).unwrap();

        // A fits on first line, B wraps to second line
        assert_eq!(a.y, 0.0);
        assert_eq!(b.y, 50.0);
    });
}

// --- Min/Max constraints ---

#[test]
fn min_max_constraints() {
    with_scope(|cx| {
        let node = el("Container", cx)
            .style(Style {
                width: Some(Length::Px(400.0)),
                height: Some(Length::Px(200.0)),
                flex_direction: Some(FlexDirection::Row),
                ..Style::default()
            })
            .child(
                el("Constrained", cx)
                    .style(Style {
                        flex_grow: Some(1.0),
                        min_width: Some(Length::Px(100.0)),
                        max_width: Some(Length::Px(250.0)),
                        height: Some(Length::Px(50.0)),
                        ..Style::default()
                    })
                    .build_node(),
            )
            .build_node();

        let result = compute_layout(&node, 400.0, 200.0, &MockTextMeasure).unwrap();
        let child = result.get(LayoutNodeId(1)).unwrap();

        // flex_grow=1 would give it 400px, but max_width caps at 250
        assert_eq!(child.width, 250.0);
    });
}

// --- Text measurement ---

#[test]
fn text_node_uses_text_measure() {
    let node = Node::Text(text("hello world"));
    let result = compute_layout(&node, 800.0, 600.0, &MockTextMeasure).unwrap();
    let rect = result.root_rect().unwrap();

    // "hello world" = 11 chars * 8px = 88px
    assert_eq!(rect.width, 88.0);
    assert_eq!(rect.height, 16.0);
}

// --- Fragment/Component transparency ---

#[test]
fn fragment_children_flattened() {
    with_scope(|cx| {
        let frag = Node::Fragment(vec![
            el("A", cx)
                .style(Style {
                    width: Some(Length::Px(50.0)),
                    height: Some(Length::Px(50.0)),
                    ..Style::default()
                })
                .build_node(),
            el("B", cx)
                .style(Style {
                    width: Some(Length::Px(60.0)),
                    height: Some(Length::Px(50.0)),
                    ..Style::default()
                })
                .build_node(),
        ]);

        let node = el("Parent", cx)
            .style(Style {
                width: Some(Length::Px(300.0)),
                height: Some(Length::Px(100.0)),
                flex_direction: Some(FlexDirection::Row),
                ..Style::default()
            })
            .child_node(frag)
            .build_node();

        let result = compute_layout(&node, 300.0, 100.0, &MockTextMeasure).unwrap();
        // Parent(0) + A(1) + B(2) = 3 nodes (Fragment is transparent)
        assert_eq!(result.len(), 3);

        let a = result.get(LayoutNodeId(1)).unwrap();
        let b = result.get(LayoutNodeId(2)).unwrap();
        assert_eq!(a.x, 0.0);
        assert_eq!(a.width, 50.0);
        assert_eq!(b.x, 50.0);
        assert_eq!(b.width, 60.0);
    });
}

#[test]
fn component_node_transparent() {
    with_scope(|cx| {
        let inner = el("Inner", cx)
            .style(Style {
                width: Some(Length::Px(80.0)),
                height: Some(Length::Px(40.0)),
                ..Style::default()
            })
            .build_node();

        let comp = Node::Component(ComponentNode {
            name: "MyComponent",
            child: Box::new(inner),
        });

        let node = el("Parent", cx)
            .style(Style {
                width: Some(Length::Px(200.0)),
                height: Some(Length::Px(100.0)),
                flex_direction: Some(FlexDirection::Row),
                ..Style::default()
            })
            .child_node(comp)
            .build_node();

        let result = compute_layout(&node, 200.0, 100.0, &MockTextMeasure).unwrap();
        // Parent(0) + Inner(1) = 2 nodes (Component wrapper is transparent)
        assert_eq!(result.len(), 2);

        let inner = result.get(LayoutNodeId(1)).unwrap();
        assert_eq!(inner.width, 80.0);
        assert_eq!(inner.height, 40.0);
    });
}

// --- Border width ---

#[test]
fn border_width_affects_layout() {
    with_scope(|cx| {
        let node = el("Container", cx)
            .style(Style {
                width: Some(Length::Px(200.0)),
                height: Some(Length::Px(200.0)),
                border_width: Edges::all(5.0),
                flex_direction: Some(FlexDirection::Column),
                ..Style::default()
            })
            .child(
                el("Child", cx)
                    .style(Style {
                        width: Some(Length::Px(50.0)),
                        height: Some(Length::Px(50.0)),
                        ..Style::default()
                    })
                    .build_node(),
            )
            .build_node();

        let result = compute_layout(&node, 200.0, 200.0, &MockTextMeasure).unwrap();
        let child = result.get(LayoutNodeId(1)).unwrap();

        // Border acts like padding for layout purposes
        assert_eq!(child.x, 5.0);
        assert_eq!(child.y, 5.0);
    });
}

// --- Error cases ---

#[test]
fn empty_tree_returns_error() {
    let node = Node::Fragment(vec![]);
    let result = compute_layout(&node, 400.0, 300.0, &MockTextMeasure);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), LayoutError::EmptyTree);
}

// --- Deep nesting ---

#[test]
fn deeply_nested_absolute_positions() {
    with_scope(|cx| {
        // 4 levels deep, each with 10px padding
        let leaf = el("Leaf", cx)
            .style(Style {
                width: Some(Length::Px(20.0)),
                height: Some(Length::Px(20.0)),
                ..Style::default()
            })
            .build_node();

        let level3 = el("L3", cx)
            .style(Style {
                padding: Edges::all(LengthPercent::Px(10.0)),
                flex_direction: Some(FlexDirection::Column),
                ..Style::default()
            })
            .child_node(leaf)
            .build_node();

        let level2 = el("L2", cx)
            .style(Style {
                padding: Edges::all(LengthPercent::Px(10.0)),
                flex_direction: Some(FlexDirection::Column),
                ..Style::default()
            })
            .child_node(level3)
            .build_node();

        let level1 = el("L1", cx)
            .style(Style {
                padding: Edges::all(LengthPercent::Px(10.0)),
                flex_direction: Some(FlexDirection::Column),
                ..Style::default()
            })
            .child_node(level2)
            .build_node();

        let root = el("Root", cx)
            .style(Style {
                width: Some(Length::Px(400.0)),
                height: Some(Length::Px(400.0)),
                padding: Edges::all(LengthPercent::Px(10.0)),
                flex_direction: Some(FlexDirection::Column),
                ..Style::default()
            })
            .child_node(level1)
            .build_node();

        let result = compute_layout(&root, 400.0, 400.0, &MockTextMeasure).unwrap();
        assert_eq!(result.len(), 5);

        let leaf_rect = result.get(LayoutNodeId(4)).unwrap();
        // 4 levels * 10px padding each = 40px offset
        assert_eq!(leaf_rect.x, 40.0);
        assert_eq!(leaf_rect.y, 40.0);
        assert_eq!(leaf_rect.width, 20.0);
        assert_eq!(leaf_rect.height, 20.0);
    });
}

// --- Root Fragment ---

#[test]
fn root_fragment_with_children() {
    with_scope(|cx| {
        let node = Node::Fragment(vec![
            el("A", cx)
                .style(Style {
                    width: Some(Length::Px(100.0)),
                    height: Some(Length::Px(50.0)),
                    ..Style::default()
                })
                .build_node(),
            el("B", cx)
                .style(Style {
                    width: Some(Length::Px(100.0)),
                    height: Some(Length::Px(50.0)),
                    ..Style::default()
                })
                .build_node(),
        ]);

        let result = compute_layout(&node, 400.0, 300.0, &MockTextMeasure).unwrap();
        // A + B + synthetic container = 3
        assert_eq!(result.len(), 3);
    });
}

// --- Iterator ---

#[test]
fn layout_result_iter_covers_all_nodes() {
    with_scope(|cx| {
        let node = el("Parent", cx)
            .style(Style {
                width: Some(Length::Px(200.0)),
                height: Some(Length::Px(100.0)),
                ..Style::default()
            })
            .child(
                el("Child", cx)
                    .style(Style {
                        width: Some(Length::Px(50.0)),
                        height: Some(Length::Px(50.0)),
                        ..Style::default()
                    })
                    .build_node(),
            )
            .build_node();

        let result = compute_layout(&node, 200.0, 100.0, &MockTextMeasure).unwrap();
        assert_eq!(result.iter().count(), 2);
    });
}

// --- Text inside element ---

#[test]
fn text_inside_element_with_padding() {
    with_scope(|cx| {
        // align_items: FlexStart prevents cross-axis stretch so text keeps intrinsic width.
        let node = el("Container", cx)
            .style(Style {
                width: Some(Length::Px(200.0)),
                height: Some(Length::Px(100.0)),
                padding: Edges::all(LengthPercent::Px(10.0)),
                flex_direction: Some(FlexDirection::Column),
                align_items: Some(dusty_style::AlignItems::FlexStart),
                ..Style::default()
            })
            .child_text(text("hi"))
            .build_node();

        let result = compute_layout(&node, 200.0, 100.0, &MockTextMeasure).unwrap();
        assert_eq!(result.len(), 2);

        let text_rect = result.get(LayoutNodeId(1)).unwrap();
        assert_eq!(text_rect.x, 10.0); // padding
        assert_eq!(text_rect.y, 10.0);
        assert_eq!(text_rect.width, 16.0); // "hi" = 2*8
        assert_eq!(text_rect.height, 16.0);
    });
}

// --- Percent sizing ---

#[test]
fn percent_width_resolves_against_parent() {
    with_scope(|cx| {
        let node = el("Parent", cx)
            .style(Style {
                width: Some(Length::Px(400.0)),
                height: Some(Length::Px(200.0)),
                flex_direction: Some(FlexDirection::Row),
                ..Style::default()
            })
            .child(
                el("HalfChild", cx)
                    .style(Style {
                        width: Some(Length::Percent(0.5)),
                        height: Some(Length::Percent(1.0)),
                        ..Style::default()
                    })
                    .build_node(),
            )
            .build_node();

        let result = compute_layout(&node, 1024.0, 768.0, &MockTextMeasure).unwrap();
        let child = result.get(LayoutNodeId(1)).unwrap();
        assert_eq!(child.width, 200.0); // 50% of 400
        assert_eq!(child.height, 200.0); // 100% of 200
    });
}

#[test]
fn aspect_ratio_with_fixed_width_derives_height() {
    with_scope(|cx| {
        let node = el("Thumb", cx)
            .style(Style {
                width: Some(Length::Px(320.0)),
                aspect_ratio: Some(16.0 / 9.0),
                ..Style::default()
            })
            .build_node();

        let result = compute_layout(&node, 1024.0, 768.0, &MockTextMeasure).unwrap();
        let rect = result.root_rect().unwrap();
        assert_eq!(rect.width, 320.0);
        // 320 / (16/9) = 180
        assert!((rect.height - 180.0).abs() < 0.001, "got {}", rect.height);
    });
}
