use dusty_core::node::{Node, TextContent};
use dusty_core::view::View;
use dusty_core::Element;
use dusty_reactive::{create_scope, create_signal, dispose_runtime, initialize_runtime};
use dusty_widgets::Text;

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
fn text_with_signal_updates_dynamically() {
    initialize_runtime();
    create_scope(|cx| {
        let name = create_signal("Alice".to_string()).unwrap();
        let node = Text::dynamic(move || format!("Hello, {}!", name.get().unwrap())).build(cx);
        let el = extract_element(&node);

        if let Node::Text(text_node) = &el.children()[0] {
            assert_eq!(text_node.current_text(), "Hello, Alice!");
            assert!(matches!(text_node.content, TextContent::Dynamic(_)));

            // Update signal
            name.set("Bob".to_string()).unwrap();
            assert_eq!(text_node.current_text(), "Hello, Bob!");
        } else {
            panic!("expected Text child");
        }
    })
    .unwrap();
    dispose_runtime();
}

#[test]
fn text_static_content() {
    initialize_runtime();
    create_scope(|cx| {
        let node = Text::new("Static content").build(cx);
        let el = extract_element(&node);

        if let Node::Text(text_node) = &el.children()[0] {
            assert_eq!(text_node.current_text(), "Static content");
            assert!(matches!(text_node.content, TextContent::Static(_)));
        } else {
            panic!("expected Text child");
        }
    })
    .unwrap();
    dispose_runtime();
}
