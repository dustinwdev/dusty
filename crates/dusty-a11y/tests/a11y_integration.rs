//! Integration tests for the accessibility tree pipeline.
//!
//! Exercises the full path: build node → compute layout → build a11y tree.

#![allow(
    clippy::unwrap_used,
    clippy::float_cmp,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation
)]

use accesskit::{NodeId, Role};
use dusty_a11y::build_accessibility_tree;
use dusty_core::{el, text};
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
fn button_with_label_produces_correct_a11y_tree() {
    with_scope(|cx| {
        let node = el("Button", cx)
            .attr("label", "Submit")
            .style(Style {
                width: Some(120.0),
                height: Some(40.0),
                ..Style::default()
            })
            .build_node();

        let layout = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
        let update = build_accessibility_tree(&node, &layout, None).unwrap();

        // Should have root + button = 2 nodes
        assert_eq!(update.nodes.len(), 2);

        let button = update
            .nodes
            .iter()
            .find(|(id, _)| *id == NodeId(1))
            .map(|(_, n)| n)
            .unwrap();

        assert_eq!(button.role(), Role::Button);
        assert_eq!(button.label(), Some("Submit"));

        let bounds = button.bounds().unwrap();
        assert_eq!(bounds.x0, 0.0);
        assert_eq!(bounds.y0, 0.0);
        assert_eq!(bounds.x1, 120.0);
        assert_eq!(bounds.y1, 40.0);
    });
}

#[test]
fn form_with_multiple_widgets() {
    with_scope(|cx| {
        let node = el("Col", cx)
            .style(Style {
                width: Some(300.0),
                height: Some(200.0),
                flex_direction: Some(dusty_style::FlexDirection::Column),
                ..Style::default()
            })
            .child(
                el("Button", cx)
                    .attr("label", "OK")
                    .attr("disabled", true)
                    .style(Style {
                        width: Some(100.0),
                        height: Some(30.0),
                        ..Style::default()
                    })
                    .build_node(),
            )
            .child(
                el("TextInput", cx)
                    .attr("placeholder", "Type here")
                    .style(Style {
                        width: Some(200.0),
                        height: Some(30.0),
                        ..Style::default()
                    })
                    .build_node(),
            )
            .child(
                el("Checkbox", cx)
                    .attr("checked", true)
                    .attr("label", "Accept terms")
                    .style(Style {
                        width: Some(20.0),
                        height: Some(20.0),
                        ..Style::default()
                    })
                    .build_node(),
            )
            .build_node();

        let layout = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
        let update = build_accessibility_tree(&node, &layout, None).unwrap();

        // root(0) + Col(1) + Button(2) + TextInput(3) + Checkbox(4) = 5
        assert_eq!(update.nodes.len(), 5);

        // Button
        let btn = update
            .nodes
            .iter()
            .find(|(id, _)| *id == NodeId(2))
            .map(|(_, n)| n)
            .unwrap();
        assert_eq!(btn.role(), Role::Button);
        assert_eq!(btn.label(), Some("OK"));
        assert!(btn.is_disabled());

        // TextInput
        let input = update
            .nodes
            .iter()
            .find(|(id, _)| *id == NodeId(3))
            .map(|(_, n)| n)
            .unwrap();
        assert_eq!(input.role(), Role::TextInput);
        assert_eq!(input.label(), Some("Type here")); // placeholder fallback

        // Checkbox
        let cb = update
            .nodes
            .iter()
            .find(|(id, _)| *id == NodeId(4))
            .map(|(_, n)| n)
            .unwrap();
        assert_eq!(cb.role(), Role::CheckBox);
        assert_eq!(cb.label(), Some("Accept terms"));
        assert_eq!(cb.toggled(), Some(accesskit::Toggled::True));
    });
}

#[test]
fn nested_layout_bounds_match() {
    with_scope(|cx| {
        let node = el("Outer", cx)
            .style(Style {
                width: Some(400.0),
                height: Some(300.0),
                padding: dusty_style::Edges::all(20.0),
                flex_direction: Some(dusty_style::FlexDirection::Column),
                ..Style::default()
            })
            .child(
                el("Inner", cx)
                    .style(Style {
                        width: Some(200.0),
                        height: Some(100.0),
                        ..Style::default()
                    })
                    .build_node(),
            )
            .build_node();

        let layout = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
        let update = build_accessibility_tree(&node, &layout, None).unwrap();

        // Verify Inner bounds match layout exactly
        let layout_rect = layout.get(dusty_layout::LayoutNodeId(1)).unwrap();
        let inner = update
            .nodes
            .iter()
            .find(|(id, _)| *id == NodeId(2))
            .map(|(_, n)| n)
            .unwrap();

        let bounds = inner.bounds().unwrap();
        assert_eq!(bounds.x0, layout_rect.x as f64);
        assert_eq!(bounds.y0, layout_rect.y as f64);
        assert_eq!(bounds.x1, (layout_rect.x + layout_rect.width) as f64);
        assert_eq!(bounds.y1, (layout_rect.y + layout_rect.height) as f64);
    });
}

#[test]
fn disabled_button_propagated() {
    with_scope(|cx| {
        let node = el("Button", cx)
            .attr("disabled", true)
            .attr("label", "Can't click")
            .style(Style {
                width: Some(100.0),
                height: Some(40.0),
                ..Style::default()
            })
            .build_node();

        let layout = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
        let update = build_accessibility_tree(&node, &layout, None).unwrap();

        let btn = update
            .nodes
            .iter()
            .find(|(id, _)| *id == NodeId(1))
            .map(|(_, n)| n)
            .unwrap();

        assert!(btn.is_disabled());
        assert_eq!(btn.label(), Some("Can't click"));
    });
}

#[test]
fn live_region_on_container() {
    with_scope(|cx| {
        let node = el("Row", cx)
            .attr("aria-live", "polite")
            .style(Style {
                width: Some(200.0),
                height: Some(100.0),
                ..Style::default()
            })
            .child(text("Status: ready"))
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

        // Text child should be present
        let text_node = update
            .nodes
            .iter()
            .find(|(id, _)| *id == NodeId(2))
            .map(|(_, n)| n)
            .unwrap();
        assert_eq!(text_node.role(), Role::Label);
        assert_eq!(text_node.value(), Some("Status: ready"));
    });
}

#[test]
fn text_inside_element_correct_order() {
    with_scope(|cx| {
        let node = el("Button", cx)
            .attr("label", "Send")
            .style(Style {
                width: Some(100.0),
                height: Some(40.0),
                ..Style::default()
            })
            .child(text("Send"))
            .build_node();

        let layout = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
        let update = build_accessibility_tree(&node, &layout, None).unwrap();

        // root(0) + Button(1) + Text(2) = 3 nodes
        assert_eq!(update.nodes.len(), 3);

        // Button is parent of Text
        let btn = update
            .nodes
            .iter()
            .find(|(id, _)| *id == NodeId(1))
            .map(|(_, n)| n)
            .unwrap();
        assert_eq!(btn.children(), &[NodeId(2)]);
        assert_eq!(btn.role(), Role::Button);

        let text_ak = update
            .nodes
            .iter()
            .find(|(id, _)| *id == NodeId(2))
            .map(|(_, n)| n)
            .unwrap();
        assert_eq!(text_ak.role(), Role::Label);
        assert_eq!(text_ak.value(), Some("Send"));
    });
}
