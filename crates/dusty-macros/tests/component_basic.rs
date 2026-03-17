use dusty_core::node::Node;
use dusty_core::view::View;
use dusty_macros::component;
use dusty_reactive::{create_scope, dispose_runtime, initialize_runtime, Scope};

fn with_scope(f: impl FnOnce(Scope)) {
    initialize_runtime();
    create_scope(|cx| f(cx));
    dispose_runtime();
}

#[component]
fn Empty(cx: Scope) -> Node {
    dusty_core::el("Empty", cx).build_node()
}

#[component]
fn Greeting(cx: Scope, name: String) -> Node {
    dusty_core::el("Greeting", cx)
        .child(dusty_core::text(format!("Hello, {}!", name)))
        .build_node()
}

#[component]
fn Counter(cx: Scope, initial: i32, max: i32) -> Node {
    dusty_core::el("Counter", cx)
        .child(dusty_core::text(format!("{}/{}", initial, max)))
        .build_node()
}

#[test]
fn empty_component_builds() {
    with_scope(|cx| {
        let node = Empty::new().build(cx);
        assert!(node.is_component());
    });
}

#[test]
fn empty_component_name() {
    with_scope(|cx| {
        let node = Empty::new().build(cx);
        if let Node::Component(comp) = &node {
            assert_eq!(comp.name, "Empty");
        } else {
            panic!("expected Component node");
        }
    });
}

#[test]
fn single_required_prop_flows_through() {
    with_scope(|cx| {
        let node = Greeting::new("World".to_string()).build(cx);
        assert!(node.is_component());
        if let Node::Component(comp) = &node {
            assert_eq!(comp.name, "Greeting");
            // The child should be an element containing the text
            if let Node::Element(el) = &*comp.child {
                assert_eq!(el.children().len(), 1);
                if let Node::Text(t) = &el.children()[0] {
                    assert_eq!(t.current_text(), "Hello, World!");
                } else {
                    panic!("expected Text child");
                }
            } else {
                panic!("expected Element child");
            }
        }
    });
}

#[test]
fn multiple_required_props() {
    with_scope(|cx| {
        let node = Counter::new(5, 10).build(cx);
        assert!(node.is_component());
        if let Node::Component(comp) = &node {
            if let Node::Element(el) = &*comp.child {
                if let Node::Text(t) = &el.children()[0] {
                    assert_eq!(t.current_text(), "5/10");
                } else {
                    panic!("expected Text child");
                }
            } else {
                panic!("expected Element child");
            }
        }
    });
}

#[test]
fn component_wraps_in_component_node() {
    with_scope(|cx| {
        let node = Empty::new().build(cx);
        // Must be Component, not Element
        assert!(node.is_component());
        assert!(!node.is_element());
    });
}
