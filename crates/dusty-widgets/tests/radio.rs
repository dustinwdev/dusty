use dusty_core::event::{dispatch_event, ClickEvent};
use dusty_core::node::Node;
use dusty_core::view::View;
use dusty_core::{AttributeValue, Element};
use dusty_reactive::{create_scope, create_signal, dispose_runtime, initialize_runtime};
use dusty_widgets::Radio;

fn extract_element(node: &Node) -> &Element {
    match node {
        Node::Component(comp) => match &*comp.child {
            Node::Element(el) => el,
            _ => panic!("expected Element inside Component"),
        },
        _ => panic!("expected Component node"),
    }
}

fn inner_node(node: &Node) -> &Node {
    match node {
        Node::Component(comp) => &*comp.child,
        _ => panic!("expected Component node"),
    }
}

#[test]
fn selecting_one_deselects_other() {
    initialize_runtime();
    create_scope(|cx| {
        let group = create_signal("a".to_string()).unwrap();

        // Build two radios sharing the same group signal.
        // Radio A starts selected, Radio B starts unselected.
        let node_a = Radio::new("a".to_string(), group).build(cx);
        let node_b = Radio::new("b".to_string(), group).build(cx);

        let el_a = extract_element(&node_a);
        let el_b = extract_element(&node_b);

        assert_eq!(el_a.attr("checked"), Some(&AttributeValue::Bool(true)));
        assert_eq!(el_b.attr("checked"), Some(&AttributeValue::Bool(false)));

        // Click radio B
        let inner_b = inner_node(&node_b);
        let event = ClickEvent { x: 0.0, y: 0.0 };
        dispatch_event(inner_b, &[], &event).unwrap();

        // Group signal should now be "b"
        assert_eq!(group.get().unwrap(), "b");
    })
    .unwrap();
    dispose_runtime();
}

#[test]
fn on_select_fires() {
    initialize_runtime();
    create_scope(|cx| {
        let group = create_signal(0i32).unwrap();
        let selected = std::rc::Rc::new(std::cell::Cell::new(0i32));
        let selected_clone = selected.clone();

        let node = Radio::new(42, group)
            .on_select(move |v| {
                selected_clone.set(*v);
            })
            .build(cx);

        let inner = inner_node(&node);
        let event = ClickEvent { x: 0.0, y: 0.0 };
        dispatch_event(inner, &[], &event).unwrap();

        assert_eq!(selected.get(), 42);
        assert_eq!(group.get().unwrap(), 42);
    })
    .unwrap();
    dispose_runtime();
}
