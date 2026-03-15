use dusty_macros::component;
use dusty_reactive::Scope;

#[component]
fn Bad(cx: Scope) -> String {
    "hello".to_string()
}

fn main() {}
