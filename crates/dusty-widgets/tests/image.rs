use dusty_core::node::Node;
use dusty_core::view::View;
use dusty_core::{AttributeValue, Element};
use dusty_reactive::{create_scope, dispose_runtime, initialize_runtime};
use dusty_widgets::{Image, SizingMode};

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
fn image_attribute_propagation() {
    initialize_runtime();
    create_scope(|cx| {
        let node = Image::new("landscape.jpg")
            .sizing(SizingMode::Contain)
            .alt("Beautiful landscape")
            .build(cx);
        let el = extract_element(&node);

        assert_eq!(
            el.attr("src"),
            Some(&AttributeValue::String("landscape.jpg".into()))
        );
        assert_eq!(
            el.attr("sizing_mode"),
            Some(&AttributeValue::String("contain".into()))
        );
        assert_eq!(
            el.attr("alt"),
            Some(&AttributeValue::String("Beautiful landscape".into()))
        );
    })
    .unwrap();
    dispose_runtime();
}

#[test]
fn image_element_name() {
    initialize_runtime();
    create_scope(|cx| {
        let node = Image::new("test.png").build(cx);
        let el = extract_element(&node);
        assert_eq!(el.name(), "Image");
    })
    .unwrap();
    dispose_runtime();
}
