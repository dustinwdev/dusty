use dusty_core::event::{dispatch_event, ClickEvent};
use dusty_core::node::Node;
use dusty_core::view::View;
use dusty_core::Element;
use dusty_reactive::{create_scope, create_signal, dispose_runtime, initialize_runtime};
use dusty_widgets::Button;

fn extract_element(node: &Node) -> &Element {
    match node {
        Node::Component(comp) => match &*comp.child {
            Node::Element(el) => el,
            _ => panic!("expected Element inside Component"),
        },
        _ => panic!("expected Component node"),
    }
}

#[test]
fn button_with_signal_label() {
    initialize_runtime();
    create_scope(|cx| {
        let label = create_signal("Save".to_string()).unwrap();
        let node = Button::dynamic(move || label.get().unwrap()).build(cx);
        let el = extract_element(&node);

        if let Node::Text(text_node) = &el.children()[0] {
            assert_eq!(text_node.current_text(), "Save");
            label.set("Saving...".to_string()).unwrap();
            assert_eq!(text_node.current_text(), "Saving...");
        } else {
            panic!("expected Text child");
        }
    })
    .unwrap();
    dispose_runtime();
}

#[test]
fn click_updates_signal() {
    initialize_runtime();
    create_scope(|cx| {
        let count = create_signal(0i32).unwrap();
        let node = Button::new("Inc")
            .on_click(move |_ctx, _e| {
                let current = count.get().unwrap();
                count.set(current + 1).unwrap();
            })
            .build(cx);

        // Dispatch click event through the component → element
        let inner = match &node {
            Node::Component(comp) => &*comp.child,
            _ => panic!("expected Component node"),
        };
        let event = ClickEvent { x: 0.0, y: 0.0 };
        dispatch_event(inner, &[], &event).unwrap();
        assert_eq!(count.get().unwrap(), 1);

        dispatch_event(inner, &[], &event).unwrap();
        assert_eq!(count.get().unwrap(), 2);
    })
    .unwrap();
    dispose_runtime();
}
