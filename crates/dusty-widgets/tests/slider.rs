use dusty_core::event::{dispatch_event, ClickEvent, DragEvent, DragPhase};
use dusty_core::node::Node;
use dusty_core::view::View;
use dusty_reactive::{create_scope, create_signal, dispose_runtime, initialize_runtime};
use dusty_widgets::Slider;

fn inner_node(node: &Node) -> &Node {
    match node {
        Node::Component(comp) => &*comp.child,
        _ => panic!("expected Component node"),
    }
}

#[test]
fn drag_sequence_updates_value() {
    initialize_runtime();
    create_scope(|cx| {
        let sig = create_signal(50.0).unwrap();
        let node = Slider::new().controlled(sig).build(cx);
        let inner = inner_node(&node);

        // Drag right by 20px on a notional 200px track = 10% of 100 = +10
        let start = DragEvent {
            x: 100.0,
            y: 0.0,
            delta_x: 0.0,
            delta_y: 0.0,
            phase: DragPhase::Start,
        };
        dispatch_event(inner, &[], &start).unwrap();

        let drag_move = DragEvent {
            x: 120.0,
            y: 0.0,
            delta_x: 20.0,
            delta_y: 0.0,
            phase: DragPhase::Move,
        };
        dispatch_event(inner, &[], &drag_move).unwrap();

        let val = sig.get().unwrap();
        assert!((val - 60.0).abs() < 0.01, "expected ~60.0, got {val}");
    })
    .unwrap();
    dispose_runtime();
}

#[test]
fn click_jumps_to_position() {
    initialize_runtime();
    create_scope(|cx| {
        let sig = create_signal(0.0).unwrap();
        let node = Slider::new().controlled(sig).build(cx);
        let inner = inner_node(&node);

        // Click at x=100 on a 200px track = 50% of 100 = 50
        let event = ClickEvent { x: 100.0, y: 0.0 };
        dispatch_event(inner, &[], &event).unwrap();

        let val = sig.get().unwrap();
        assert!((val - 50.0).abs() < 0.01, "expected ~50.0, got {val}");
    })
    .unwrap();
    dispose_runtime();
}

#[test]
fn controlled_round_trip() {
    initialize_runtime();
    create_scope(|cx| {
        let sig = create_signal(25.0).unwrap();
        let node = Slider::new().controlled(sig).build(cx);
        let inner = inner_node(&node);

        // Click to change
        let event = ClickEvent { x: 150.0, y: 0.0 };
        dispatch_event(inner, &[], &event).unwrap();

        let val = sig.get().unwrap();
        assert!((val - 75.0).abs() < 0.01, "expected ~75.0, got {val}");
    })
    .unwrap();
    dispose_runtime();
}
