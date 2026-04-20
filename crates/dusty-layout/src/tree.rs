//! Node tree → `TaffyTree` builder, layout computation, and result extraction.

use std::collections::HashMap;

use dusty_core::Node;
use dusty_style::{FontStyle, InteractionState};
use smallvec::SmallVec;
use taffy::{AvailableSpace, NodeId, TaffyTree};

use crate::convert::to_taffy_style;
use crate::error::{LayoutError, Result};
use crate::measure::TextMeasure;
use crate::result::{LayoutNodeId, LayoutResult, Rect};

/// Context stored on taffy leaf nodes for text measurement.
struct TextNodeContext {
    content: String,
    font: FontStyle,
}

/// Computes absolute-position layout for a node tree.
///
/// Walks the `Node` tree, converts styles to taffy, runs flexbox layout,
/// and returns a [`LayoutResult`] mapping each element/text node to its
/// screen-space [`Rect`].
///
/// # Errors
///
/// Returns [`LayoutError::EmptyTree`] if the tree contains no layout nodes.
///
/// # Examples
///
/// ```
/// use dusty_core::{Node, Element, el, text};
/// use dusty_style::{Style, FontStyle, Length};
/// use dusty_layout::{compute_layout, TextMeasure};
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
///     let node = el("Root", cx)
///         .style(Style { width: Some(Length::Px(400.0)), height: Some(Length::Px(300.0)), ..Style::default() })
///         .build_node();
///     let result = compute_layout(&node, 400.0, 300.0, &Mock).unwrap();
///     assert_eq!(result.len(), 1);
/// });
/// dispose_runtime();
/// ```
pub fn compute_layout(
    root: &Node,
    available_width: f32,
    available_height: f32,
    text_measure: &dyn TextMeasure,
) -> Result<LayoutResult> {
    let mut engine = LayoutEngine::new();
    engine.compute(root, available_width, available_height, text_measure)
}

/// A reusable layout engine that caches the internal `TaffyTree` across calls.
///
/// Each [`compute`](Self::compute) call clears and reuses the underlying allocations,
/// avoiding repeated heap allocation for the tree and map.
///
/// # Examples
///
/// ```
/// use dusty_core::{Node, Element, el, text};
/// use dusty_style::{Style, FontStyle, Length};
/// use dusty_layout::{LayoutEngine, TextMeasure};
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
///     let mut engine = LayoutEngine::new();
///     let node = el("Root", cx)
///         .style(Style { width: Some(Length::Px(400.0)), height: Some(Length::Px(300.0)), ..Style::default() })
///         .build_node();
///     let result = engine.compute(&node, 400.0, 300.0, &Mock).unwrap();
///     assert_eq!(result.len(), 1);
/// });
/// dispose_runtime();
/// ```
pub struct LayoutEngine {
    taffy: TaffyTree<TextNodeContext>,
    taffy_to_layout: HashMap<NodeId, LayoutNodeId>,
}

impl LayoutEngine {
    /// Creates a new layout engine with empty internal state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            taffy: TaffyTree::new(),
            taffy_to_layout: HashMap::new(),
        }
    }

    /// Computes layout for a node tree, reusing internal allocations.
    ///
    /// # Errors
    ///
    /// Returns [`LayoutError::EmptyTree`] if the tree contains no layout nodes.
    pub fn compute(
        &mut self,
        root: &Node,
        available_width: f32,
        available_height: f32,
        text_measure: &dyn TextMeasure,
    ) -> Result<LayoutResult> {
        self.taffy.clear();
        self.taffy_to_layout.clear();

        let mut builder = TreeBuilder {
            taffy: &mut self.taffy,
            taffy_to_layout: &mut self.taffy_to_layout,
            next_id: 0,
        };

        let root_taffy_ids = builder.build_node(root, &FontStyle::default())?;

        if root_taffy_ids.is_empty() {
            return Err(LayoutError::EmptyTree);
        }

        let taffy_root = if root_taffy_ids.len() == 1 {
            root_taffy_ids[0]
        } else {
            let layout_id = builder.alloc_id();
            let synthetic = builder
                .taffy
                .new_with_children(taffy::Style::DEFAULT, &root_taffy_ids)?;
            builder.taffy_to_layout.insert(synthetic, layout_id);
            synthetic
        };

        let total_nodes = builder.next_id;

        self.taffy.compute_layout_with_measure(
            taffy_root,
            taffy::Size {
                width: AvailableSpace::Definite(available_width),
                height: AvailableSpace::Definite(available_height),
            },
            |known_dimensions, available_space, _node_id, context, _style| {
                if let Some(ctx) = context {
                    let max_width = match available_space.width {
                        AvailableSpace::Definite(w) => Some(w),
                        _ => None,
                    };
                    let (w, h) = text_measure.measure(&ctx.content, max_width, &ctx.font);
                    taffy::Size {
                        width: known_dimensions.width.unwrap_or(w),
                        height: known_dimensions.height.unwrap_or(h),
                    }
                } else {
                    taffy::Size::ZERO
                }
            },
        )?;

        let mut rects = vec![
            Rect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            };
            total_nodes
        ];

        extract_absolute(
            &self.taffy,
            taffy_root,
            0.0,
            0.0,
            &self.taffy_to_layout,
            &mut rects,
        )?;

        let root_layout_id = self
            .taffy_to_layout
            .get(&taffy_root)
            .copied()
            .ok_or(LayoutError::EmptyTree)?;

        Ok(LayoutResult::new(rects, root_layout_id))
    }
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

struct TreeBuilder<'a> {
    taffy: &'a mut TaffyTree<TextNodeContext>,
    taffy_to_layout: &'a mut HashMap<NodeId, LayoutNodeId>,
    next_id: usize,
}

impl TreeBuilder<'_> {
    fn alloc_id(&mut self) -> LayoutNodeId {
        let id = LayoutNodeId(self.next_id);
        self.next_id += 1;
        id
    }

    fn build_node(
        &mut self,
        node: &Node,
        inherited_font: &FontStyle,
    ) -> Result<SmallVec<[NodeId; 8]>> {
        match node {
            Node::Element(el) => {
                let layout_id = self.alloc_id();

                // Downcast style; `()` means no style was set, so use default.
                // Any other non-Style type is a programmer error.
                let raw_style = if el.style().is::<()>() {
                    dusty_style::Style::default()
                } else {
                    el.style()
                        .downcast_ref::<dusty_style::Style>()
                        .cloned()
                        .ok_or(LayoutError::StyleDowncastFailed)?
                };

                // Resolve interaction state (disabled) for layout purposes.
                // Hover/focus/active resolution deferred until state tracking
                // is implemented in the app runner.
                let is_disabled = el
                    .attr("disabled")
                    .and_then(|v| {
                        if let dusty_core::AttributeValue::Bool(b) = v {
                            Some(*b)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(false);
                let dusty_style = if is_disabled {
                    raw_style.resolve(&InteractionState {
                        disabled: true,
                        ..InteractionState::default()
                    })
                } else {
                    raw_style
                };

                let taffy_style = to_taffy_style(&dusty_style);

                // Merge element's font onto the inherited font for children.
                let child_font = inherited_font.merge(&dusty_style.font);

                // Build children (Fragment/Component flatten here).
                let child_taffy_ids = self.build_children(el.children(), &child_font)?;

                let taffy_id = self
                    .taffy
                    .new_with_children(taffy_style, &child_taffy_ids)?;

                self.taffy_to_layout.insert(taffy_id, layout_id);
                Ok(smallvec::smallvec![taffy_id])
            }
            Node::Text(text_node) => {
                let layout_id = self.alloc_id();
                let content = text_node.current_text().into_owned();
                let ctx = TextNodeContext {
                    content,
                    font: inherited_font.clone(),
                };
                let taffy_id = self
                    .taffy
                    .new_leaf_with_context(taffy::Style::DEFAULT, ctx)?;

                self.taffy_to_layout.insert(taffy_id, layout_id);
                Ok(smallvec::smallvec![taffy_id])
            }
            Node::Fragment(children) => self.build_children(children, inherited_font),
            Node::Component(comp) => self.build_node(&comp.child, inherited_font),
            Node::Dynamic(dn) => {
                let resolved = dn.current_node();
                self.build_node(&resolved, inherited_font)
            }
        }
    }

    fn build_children(
        &mut self,
        children: &[Node],
        inherited_font: &FontStyle,
    ) -> Result<SmallVec<[NodeId; 8]>> {
        let mut ids = SmallVec::new();
        for child in children {
            ids.extend(self.build_node(child, inherited_font)?);
        }
        Ok(ids)
    }
}

fn extract_absolute(
    taffy: &TaffyTree<TextNodeContext>,
    node_id: NodeId,
    parent_x: f32,
    parent_y: f32,
    taffy_to_layout: &HashMap<NodeId, LayoutNodeId>,
    rects: &mut [Rect],
) -> Result<()> {
    let layout = taffy.layout(node_id)?;
    let abs_x = parent_x + layout.location.x;
    let abs_y = parent_y + layout.location.y;

    if let Some(&layout_id) = taffy_to_layout.get(&node_id) {
        debug_assert!(
            layout_id.0 < rects.len(),
            "layout_id {} out of bounds (rects len {})",
            layout_id.0,
            rects.len()
        );
        if let Some(rect) = rects.get_mut(layout_id.0) {
            *rect = Rect {
                x: abs_x,
                y: abs_y,
                width: layout.size.width,
                height: layout.size.height,
            };
        }
    }

    let children = taffy.children(node_id)?;
    for child_id in children {
        extract_absolute(taffy, child_id, abs_x, abs_y, taffy_to_layout, rects)?;
    }

    Ok(())
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
    use dusty_core::{el, text, ComponentNode};
    use dusty_reactive::{create_scope, dispose_runtime, initialize_runtime};
    use dusty_style::{Length, LengthPercent, Style};

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
        create_scope(f);
        dispose_runtime();
    }

    #[test]
    fn single_element_fixed_size() {
        with_scope(|cx| {
            let node = el("Box", cx)
                .style(Style {
                    width: Some(Length::Px(200.0)),
                    height: Some(Length::Px(100.0)),
                    ..Style::default()
                })
                .build_node();

            let result = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
            assert_eq!(result.len(), 1);
            let rect = result.root_rect().unwrap();
            assert_eq!(rect.width, 200.0);
            assert_eq!(rect.height, 100.0);
            assert_eq!(rect.x, 0.0);
            assert_eq!(rect.y, 0.0);
        });
    }

    #[test]
    fn element_no_style_uses_default() {
        with_scope(|cx| {
            // No .style() call — style is `()`, downcast fails, falls back to default
            let node = el("Box", cx).build_node();
            let result = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
            assert_eq!(result.len(), 1);
        });
    }

    #[test]
    fn empty_fragment_returns_error() {
        let node = Node::Fragment(vec![]);
        let result = compute_layout(&node, 400.0, 300.0, &MockMeasure);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), LayoutError::EmptyTree);
    }

    #[test]
    fn text_node_uses_measure() {
        let node = Node::Text(text("hello"));
        let result = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
        assert_eq!(result.len(), 1);
        let rect = result.root_rect().unwrap();
        // "hello" = 5 chars * 8px = 40px wide, 16px tall
        assert_eq!(rect.width, 40.0);
        assert_eq!(rect.height, 16.0);
    }

    #[test]
    fn row_layout_distributes_children() {
        with_scope(|cx| {
            let node = el("Row", cx)
                .style(Style {
                    width: Some(Length::Px(300.0)),
                    height: Some(Length::Px(100.0)),
                    flex_direction: Some(dusty_style::FlexDirection::Row),
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
                .child(
                    el("C", cx)
                        .style(Style {
                            flex_grow: Some(1.0),
                            ..Style::default()
                        })
                        .build_node(),
                )
                .build_node();

            let result = compute_layout(&node, 300.0, 100.0, &MockMeasure).unwrap();
            assert_eq!(result.len(), 4); // parent + 3 children

            let a = result.get(LayoutNodeId(1)).unwrap();
            let b = result.get(LayoutNodeId(2)).unwrap();
            let c = result.get(LayoutNodeId(3)).unwrap();

            assert_eq!(a.width, 100.0);
            assert_eq!(b.width, 100.0);
            assert_eq!(c.width, 100.0);

            assert_eq!(a.x, 0.0);
            assert_eq!(b.x, 100.0);
            assert_eq!(c.x, 200.0);
        });
    }

    #[test]
    fn column_layout_stacks_vertically() {
        with_scope(|cx| {
            let node = el("Col", cx)
                .style(Style {
                    width: Some(Length::Px(200.0)),
                    height: Some(Length::Px(200.0)),
                    flex_direction: Some(dusty_style::FlexDirection::Column),
                    ..Style::default()
                })
                .child(
                    el("A", cx)
                        .style(Style {
                            height: Some(Length::Px(50.0)),
                            ..Style::default()
                        })
                        .build_node(),
                )
                .child(
                    el("B", cx)
                        .style(Style {
                            height: Some(Length::Px(50.0)),
                            ..Style::default()
                        })
                        .build_node(),
                )
                .build_node();

            let result = compute_layout(&node, 200.0, 200.0, &MockMeasure).unwrap();
            let a = result.get(LayoutNodeId(1)).unwrap();
            let b = result.get(LayoutNodeId(2)).unwrap();

            assert_eq!(a.y, 0.0);
            assert_eq!(a.height, 50.0);
            assert_eq!(b.y, 50.0);
            assert_eq!(b.height, 50.0);
        });
    }

    #[test]
    fn nested_containers_absolute_positions() {
        with_scope(|cx| {
            let node = el("Outer", cx)
                .style(Style {
                    width: Some(Length::Px(400.0)),
                    height: Some(Length::Px(400.0)),
                    padding: dusty_style::Edges::all(dusty_style::LengthPercent::Px(20.0)),
                    flex_direction: Some(dusty_style::FlexDirection::Column),
                    ..Style::default()
                })
                .child(
                    el("Inner", cx)
                        .style(Style {
                            width: Some(Length::Px(200.0)),
                            height: Some(Length::Px(100.0)),
                            padding: dusty_style::Edges::all(dusty_style::LengthPercent::Px(10.0)),
                            ..Style::default()
                        })
                        .child(
                            el("Leaf", cx)
                                .style(Style {
                                    width: Some(Length::Px(50.0)),
                                    height: Some(Length::Px(50.0)),
                                    ..Style::default()
                                })
                                .build_node(),
                        )
                        .build_node(),
                )
                .build_node();

            let result = compute_layout(&node, 400.0, 400.0, &MockMeasure).unwrap();

            let outer = result.get(LayoutNodeId(0)).unwrap();
            assert_eq!(outer.x, 0.0);
            assert_eq!(outer.y, 0.0);

            let inner = result.get(LayoutNodeId(1)).unwrap();
            assert_eq!(inner.x, 20.0); // outer padding
            assert_eq!(inner.y, 20.0);

            let leaf = result.get(LayoutNodeId(2)).unwrap();
            assert_eq!(leaf.x, 30.0); // outer padding + inner padding
            assert_eq!(leaf.y, 30.0);
        });
    }

    #[test]
    fn gap_between_children() {
        with_scope(|cx| {
            let node = el("Row", cx)
                .style(Style {
                    width: Some(Length::Px(300.0)),
                    height: Some(Length::Px(100.0)),
                    flex_direction: Some(dusty_style::FlexDirection::Row),
                    gap: Some(LengthPercent::Px(10.0)),
                    ..Style::default()
                })
                .child(
                    el("A", cx)
                        .style(Style {
                            width: Some(Length::Px(100.0)),
                            height: Some(Length::Px(100.0)),
                            ..Style::default()
                        })
                        .build_node(),
                )
                .child(
                    el("B", cx)
                        .style(Style {
                            width: Some(Length::Px(100.0)),
                            height: Some(Length::Px(100.0)),
                            ..Style::default()
                        })
                        .build_node(),
                )
                .build_node();

            let result = compute_layout(&node, 300.0, 100.0, &MockMeasure).unwrap();
            let a = result.get(LayoutNodeId(1)).unwrap();
            let b = result.get(LayoutNodeId(2)).unwrap();

            assert_eq!(a.x, 0.0);
            assert_eq!(b.x, 110.0); // 100 + 10 gap
        });
    }

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
                        width: Some(Length::Px(50.0)),
                        height: Some(Length::Px(50.0)),
                        ..Style::default()
                    })
                    .build_node(),
            ]);

            let node = el("Parent", cx)
                .style(Style {
                    width: Some(Length::Px(200.0)),
                    height: Some(Length::Px(100.0)),
                    flex_direction: Some(dusty_style::FlexDirection::Row),
                    ..Style::default()
                })
                .child_node(frag)
                .build_node();

            let result = compute_layout(&node, 200.0, 100.0, &MockMeasure).unwrap();
            // Parent + A + B = 3 (Fragment is transparent)
            assert_eq!(result.len(), 3);

            let a = result.get(LayoutNodeId(1)).unwrap();
            let b = result.get(LayoutNodeId(2)).unwrap();
            assert_eq!(a.x, 0.0);
            assert_eq!(b.x, 50.0);
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
                    ..Style::default()
                })
                .child_node(comp)
                .build_node();

            let result = compute_layout(&node, 200.0, 100.0, &MockMeasure).unwrap();
            // Parent + Inner = 2 (Component is transparent)
            assert_eq!(result.len(), 2);
        });
    }

    #[test]
    fn flex_grow_proportional() {
        with_scope(|cx| {
            let node = el("Row", cx)
                .style(Style {
                    width: Some(Length::Px(300.0)),
                    height: Some(Length::Px(100.0)),
                    flex_direction: Some(dusty_style::FlexDirection::Row),
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

            let result = compute_layout(&node, 300.0, 100.0, &MockMeasure).unwrap();
            let a = result.get(LayoutNodeId(1)).unwrap();
            let b = result.get(LayoutNodeId(2)).unwrap();

            assert_eq!(a.width, 100.0);
            assert_eq!(b.width, 200.0);
        });
    }

    #[test]
    fn wrong_style_type_returns_error() {
        with_scope(|cx| {
            let node = el("Box", cx).style("wrong type".to_string()).build_node();
            let result = compute_layout(&node, 400.0, 300.0, &MockMeasure);
            assert_eq!(result.unwrap_err(), LayoutError::StyleDowncastFailed);
        });
    }

    #[test]
    fn bare_fragment_root_rect_covers_children() {
        with_scope(|cx| {
            let node = Node::Fragment(vec![
                el("A", cx)
                    .style(Style {
                        width: Some(Length::Px(50.0)),
                        height: Some(Length::Px(50.0)),
                        ..Style::default()
                    })
                    .build_node(),
                el("B", cx)
                    .style(Style {
                        width: Some(Length::Px(50.0)),
                        height: Some(Length::Px(50.0)),
                        ..Style::default()
                    })
                    .build_node(),
            ]);
            let result = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
            let root = result.root_rect().unwrap();
            // Default flex direction is row, so width >= 100
            assert!(
                root.width >= 100.0,
                "root width should be >= 100, got {}",
                root.width
            );
        });
    }

    #[test]
    fn margin_offsets_position() {
        with_scope(|cx| {
            let node = el("Container", cx)
                .style(Style {
                    width: Some(Length::Px(200.0)),
                    height: Some(Length::Px(200.0)),
                    flex_direction: Some(dusty_style::FlexDirection::Column),
                    ..Style::default()
                })
                .child(
                    el("Child", cx)
                        .style(Style {
                            width: Some(Length::Px(100.0)),
                            height: Some(Length::Px(50.0)),
                            margin: dusty_style::Edges::new(
                                dusty_style::Length::Px(10.0),
                                dusty_style::Length::Px(0.0),
                                dusty_style::Length::Px(0.0),
                                dusty_style::Length::Px(20.0),
                            ),
                            ..Style::default()
                        })
                        .build_node(),
                )
                .build_node();

            let result = compute_layout(&node, 200.0, 200.0, &MockMeasure).unwrap();
            let child = result.get(LayoutNodeId(1)).unwrap();
            assert_eq!(child.x, 20.0); // left margin
            assert_eq!(child.y, 10.0); // top margin
        });
    }

    #[test]
    fn layout_engine_reuse_produces_correct_results() {
        with_scope(|cx| {
            let mut engine = LayoutEngine::new();

            // First compute
            let node1 = el("Box", cx)
                .style(Style {
                    width: Some(Length::Px(200.0)),
                    height: Some(Length::Px(100.0)),
                    ..Style::default()
                })
                .build_node();
            let result1 = engine.compute(&node1, 400.0, 300.0, &MockMeasure).unwrap();
            assert_eq!(result1.root_rect().unwrap().width, 200.0);

            // Second compute reuses engine
            let node2 = el("Box2", cx)
                .style(Style {
                    width: Some(Length::Px(300.0)),
                    height: Some(Length::Px(150.0)),
                    ..Style::default()
                })
                .build_node();
            let result2 = engine.compute(&node2, 400.0, 300.0, &MockMeasure).unwrap();
            assert_eq!(result2.root_rect().unwrap().width, 300.0);
            assert_eq!(result2.root_rect().unwrap().height, 150.0);
        });
    }

    #[test]
    fn layout_engine_clear_resets_ids() {
        with_scope(|cx| {
            let mut engine = LayoutEngine::new();

            let node = el("A", cx)
                .style(Style {
                    width: Some(Length::Px(100.0)),
                    height: Some(Length::Px(100.0)),
                    ..Style::default()
                })
                .build_node();

            let result1 = engine.compute(&node, 400.0, 300.0, &MockMeasure).unwrap();
            assert_eq!(result1.len(), 1);

            // IDs should start fresh
            let result2 = engine.compute(&node, 400.0, 300.0, &MockMeasure).unwrap();
            assert_eq!(result2.len(), 1);
            // Root should be at LayoutNodeId(0) both times
            assert!(result2.get(LayoutNodeId(0)).is_some());
        });
    }
}
