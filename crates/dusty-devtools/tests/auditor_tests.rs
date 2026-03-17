//! Integration tests for the accessibility auditor.

#![allow(clippy::unwrap_used)]

use dusty_core::event::ClickEvent;
use dusty_core::{el, text, Node};
use dusty_devtools::auditor::{audit, AuditRule, Severity};
use dusty_reactive::{create_scope, dispose_runtime, initialize_runtime};

fn with_scope(f: impl FnOnce(dusty_reactive::Scope)) {
    initialize_runtime();
    create_scope(|cx| f(cx));
    dispose_runtime();
}

#[test]
fn button_missing_label_is_error() {
    with_scope(|cx| {
        let node = el("Button", cx).build_node();
        let result = audit(&node);

        assert_eq!(result.issues.len(), 1);
        assert_eq!(result.issues[0].rule, AuditRule::MissingLabel);
        assert_eq!(result.issues[0].severity, Severity::Error);
        assert_eq!(result.issues[0].element_name, Some("Button".to_string()));
    });
}

#[test]
fn button_with_label_no_label_issue() {
    with_scope(|cx| {
        let node = el("Button", cx).attr("label", "Submit").build_node();
        let result = audit(&node);
        assert!(!result
            .issues
            .iter()
            .any(|i| i.rule == AuditRule::MissingLabel));
    });
}

#[test]
fn button_with_aria_label_no_label_issue() {
    with_scope(|cx| {
        let node = el("Button", cx).attr("aria-label", "Close").build_node();
        let result = audit(&node);
        assert!(!result
            .issues
            .iter()
            .any(|i| i.rule == AuditRule::MissingLabel));
    });
}

#[test]
fn checkbox_missing_label_is_error() {
    with_scope(|cx| {
        let node = el("Checkbox", cx).build_node();
        let result = audit(&node);
        let issue = result
            .issues
            .iter()
            .find(|i| i.rule == AuditRule::MissingLabel)
            .unwrap();
        assert_eq!(issue.severity, Severity::Error);
    });
}

#[test]
fn radio_missing_label_is_error() {
    with_scope(|cx| {
        let node = el("Radio", cx).build_node();
        let result = audit(&node);
        assert!(result
            .issues
            .iter()
            .any(|i| i.rule == AuditRule::MissingLabel));
    });
}

#[test]
fn toggle_missing_label_is_error() {
    with_scope(|cx| {
        let node = el("Toggle", cx).build_node();
        let result = audit(&node);
        assert!(result
            .issues
            .iter()
            .any(|i| i.rule == AuditRule::MissingLabel));
    });
}

#[test]
fn slider_missing_label_is_error() {
    with_scope(|cx| {
        let node = el("Slider", cx).build_node();
        let result = audit(&node);
        assert!(result
            .issues
            .iter()
            .any(|i| i.rule == AuditRule::MissingLabel));
    });
}

#[test]
fn image_missing_alt_is_warning() {
    with_scope(|cx| {
        let node = el("Image", cx).build_node();
        let result = audit(&node);
        let issue = result
            .issues
            .iter()
            .find(|i| i.rule == AuditRule::MissingImageAlt)
            .unwrap();
        assert_eq!(issue.severity, Severity::Warning);
    });
}

#[test]
fn image_with_label_no_alt_issue() {
    with_scope(|cx| {
        let node = el("Image", cx)
            .attr("aria-label", "Profile photo")
            .build_node();
        let result = audit(&node);
        assert!(!result
            .issues
            .iter()
            .any(|i| i.rule == AuditRule::MissingImageAlt));
    });
}

#[test]
fn text_input_missing_label_is_error() {
    with_scope(|cx| {
        let node = el("TextInput", cx).build_node();
        let result = audit(&node);
        let issue = result
            .issues
            .iter()
            .find(|i| i.rule == AuditRule::MissingInputLabel)
            .unwrap();
        assert_eq!(issue.severity, Severity::Error);
    });
}

#[test]
fn input_missing_label_is_error() {
    with_scope(|cx| {
        let node = el("Input", cx).build_node();
        let result = audit(&node);
        assert!(result
            .issues
            .iter()
            .any(|i| i.rule == AuditRule::MissingInputLabel));
    });
}

#[test]
fn input_with_placeholder_no_label_issue() {
    with_scope(|cx| {
        let node = el("TextInput", cx)
            .attr("placeholder", "Type here")
            .build_node();
        let result = audit(&node);
        assert!(!result
            .issues
            .iter()
            .any(|i| i.rule == AuditRule::MissingInputLabel));
    });
}

#[test]
fn slider_missing_value_is_warning() {
    with_scope(|cx| {
        let node = el("Slider", cx).attr("label", "Volume").build_node();
        let result = audit(&node);
        let issue = result
            .issues
            .iter()
            .find(|i| i.rule == AuditRule::MissingSliderValue)
            .unwrap();
        assert_eq!(issue.severity, Severity::Warning);
    });
}

#[test]
fn slider_with_value_no_value_issue() {
    with_scope(|cx| {
        let node = el("Slider", cx)
            .attr("label", "Volume")
            .attr("value", "50")
            .build_node();
        let result = audit(&node);
        assert!(!result
            .issues
            .iter()
            .any(|i| i.rule == AuditRule::MissingSliderValue));
    });
}

#[test]
fn interactive_generic_container_is_warning() {
    with_scope(|cx| {
        let node = el("CustomWidget", cx)
            .on_click(|_e: &ClickEvent| {})
            .build_node();
        let result = audit(&node);
        let issue = result
            .issues
            .iter()
            .find(|i| i.rule == AuditRule::MissingRole)
            .unwrap();
        assert_eq!(issue.severity, Severity::Warning);
    });
}

#[test]
fn non_interactive_element_no_role_issue() {
    with_scope(|cx| {
        let node = el("CustomWidget", cx).build_node();
        let result = audit(&node);
        assert!(!result
            .issues
            .iter()
            .any(|i| i.rule == AuditRule::MissingRole));
    });
}

#[test]
fn known_interactive_element_no_role_issue() {
    with_scope(|cx| {
        let node = el("Button", cx)
            .attr("label", "OK")
            .on_click(|_e: &ClickEvent| {})
            .build_node();
        let result = audit(&node);
        // Button maps to Role::Button, not GenericContainer
        assert!(!result
            .issues
            .iter()
            .any(|i| i.rule == AuditRule::MissingRole));
    });
}

#[test]
fn empty_fragment_zero_issues() {
    let node = Node::Fragment(vec![]);
    let result = audit(&node);
    assert!(result.issues.is_empty());
    assert_eq!(result.total_nodes_audited, 0);
    assert_eq!(result.nodes_with_issues, 0);
}

#[test]
fn text_node_no_issues() {
    let node = Node::Text(text("hello"));
    let result = audit(&node);
    assert!(result.issues.is_empty());
    assert_eq!(result.total_nodes_audited, 1);
}

#[test]
fn mixed_tree_correct_counts() {
    with_scope(|cx| {
        let node = el("Row", cx)
            .child(el("Button", cx).build_node()) // MissingLabel
            .child(el("Button", cx).attr("label", "OK").build_node()) // passes
            .child(el("Image", cx).build_node()) // MissingImageAlt
            .child(text("hello"))
            .build_node();
        let result = audit(&node);

        // 5 nodes total: Row + 2 Buttons + Image + Text
        assert_eq!(result.total_nodes_audited, 5);
        // 2 issues: MissingLabel on first Button, MissingImageAlt on Image
        assert_eq!(result.issues.len(), 2);
        assert_eq!(result.nodes_with_issues, 2);
    });
}

#[test]
fn nested_issues_found() {
    with_scope(|cx| {
        let node = el("Row", cx)
            .child(
                el("Col", cx)
                    .child(el("Button", cx).build_node()) // nested MissingLabel
                    .build_node(),
            )
            .build_node();
        let result = audit(&node);

        assert_eq!(result.issues.len(), 1);
        assert_eq!(result.issues[0].rule, AuditRule::MissingLabel);
    });
}
