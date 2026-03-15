use std::cell::RefCell;
use std::rc::Rc;

use dusty_core::node::Node;
use dusty_core::view::View;
use dusty_core::Element;
use dusty_reactive::{create_scope, create_signal, dispose_runtime, initialize_runtime};
use dusty_style::Color;
use dusty_widgets::canvas::{CanvasCommand, FillStyle};
use dusty_widgets::Canvas;

fn extract_element(node: &Node) -> &Element {
    match node {
        Node::Component(comp) => match &*comp.child {
            Node::Element(el) => el,
            _ => panic!("expected Element inside Component"),
        },
        _ => panic!("expected Component node"),
    }
}

fn extract_commands(el: &Element) -> Vec<CanvasCommand> {
    let data = el
        .custom_data()
        .downcast_ref::<Rc<RefCell<Vec<CanvasCommand>>>>();
    assert!(data.is_some(), "custom_data should be canvas commands");
    data.unwrap().borrow().clone()
}

#[test]
fn canvas_with_signal_updates_commands() {
    initialize_runtime();
    create_scope(|cx| {
        let radius = create_signal(10.0f32).unwrap();
        let node = Canvas::new(move |frame| {
            let r = radius.get().unwrap_or(0.0);
            frame.circle(50.0, 50.0, r, Some(FillStyle::Solid(Color::WHITE)), None);
        })
        .build(cx);
        let el = extract_element(&node);

        // Initial render
        let cmds = extract_commands(el);
        assert_eq!(cmds.len(), 1);
        if let CanvasCommand::Circle { radius: r, .. } = &cmds[0] {
            assert_eq!(*r, 10.0);
        } else {
            panic!("expected Circle command");
        }

        // Update signal
        radius.set(25.0).unwrap();
        let cmds = extract_commands(el);
        assert_eq!(cmds.len(), 1);
        if let CanvasCommand::Circle { radius: r, .. } = &cmds[0] {
            assert_eq!(*r, 25.0);
        } else {
            panic!("expected Circle command after update");
        }
    })
    .unwrap();
    dispose_runtime();
}

#[test]
fn canvas_empty_draw() {
    initialize_runtime();
    create_scope(|cx| {
        let node = Canvas::new(|_frame| {}).build(cx);
        let el = extract_element(&node);
        let cmds = extract_commands(el);
        assert!(cmds.is_empty());
    })
    .unwrap();
    dispose_runtime();
}
