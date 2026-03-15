use dusty_core::node::Node;
use dusty_macros::component;

#[component]
fn Bad(name: String) -> Node {
    dusty_core::node::Node::Fragment(vec![])
}

fn main() {}
