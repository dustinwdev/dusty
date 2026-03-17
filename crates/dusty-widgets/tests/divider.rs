use dusty_core::node::Node;
use dusty_core::view::View;
use dusty_core::{AttributeValue, Element};
use dusty_reactive::{create_scope, dispose_runtime, initialize_runtime};
use dusty_style::Style;
use dusty_widgets::Divider;

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
fn horizontal_divider_dimensions() {
    initialize_runtime();
    create_scope(|cx| {
        let node = Divider::horizontal().build(cx);
        let el = extract_element(&node);
        let style = el.style().downcast_ref::<Style>().unwrap();
        assert_eq!(style.height, Some(1.0));
        assert_eq!(style.width, None); // stretches via flex_grow
        assert_eq!(style.flex_grow, Some(1.0));
        assert_eq!(
            el.attr("orientation"),
            Some(&AttributeValue::String("horizontal".into()))
        );
    });
    dispose_runtime();
}

#[test]
fn vertical_divider_dimensions() {
    initialize_runtime();
    create_scope(|cx| {
        let node = Divider::vertical().build(cx);
        let el = extract_element(&node);
        let style = el.style().downcast_ref::<Style>().unwrap();
        assert_eq!(style.width, Some(1.0));
        assert_eq!(style.height, None); // stretches via flex_grow
        assert_eq!(style.flex_grow, Some(1.0));
        assert_eq!(
            el.attr("orientation"),
            Some(&AttributeValue::String("vertical".into()))
        );
    });
    dispose_runtime();
}
