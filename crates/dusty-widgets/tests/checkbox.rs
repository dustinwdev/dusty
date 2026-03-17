use dusty_core::event::{dispatch_event, ClickEvent};
use dusty_core::node::Node;
use dusty_core::view::View;
use dusty_reactive::{create_scope, create_signal, dispose_runtime, initialize_runtime};
use dusty_widgets::Checkbox;

fn inner_node(node: &Node) -> &Node {
    match node {
        Node::Component(comp) => &*comp.child,
        _ => panic!("expected Component node"),
    }
}

#[test]
fn controlled_round_trip() {
    initialize_runtime();
    create_scope(|cx| {
        let sig = create_signal(false);
        let node = Checkbox::new().controlled(sig).build(cx);

        assert_eq!(sig.get(), false);

        // Click toggles via dispatch
        let inner = inner_node(&node);
        let event = ClickEvent { x: 0.0, y: 0.0 };
        dispatch_event(inner, &[], &event).unwrap();

        assert_eq!(sig.get(), true);

        // Click again toggles back
        dispatch_event(inner, &[], &event).unwrap();
        assert_eq!(sig.get(), false);
    });
    dispose_runtime();
}

#[test]
fn on_change_receives_new_value() {
    initialize_runtime();
    create_scope(|cx| {
        let received = std::rc::Rc::new(std::cell::Cell::new(None));
        let received_clone = received.clone();
        let node = Checkbox::new()
            .on_change(move |val| {
                received_clone.set(Some(val));
            })
            .build(cx);

        let inner = inner_node(&node);
        let event = ClickEvent { x: 0.0, y: 0.0 };
        dispatch_event(inner, &[], &event).unwrap();

        assert_eq!(received.get(), Some(true));
    });
    dispose_runtime();
}
