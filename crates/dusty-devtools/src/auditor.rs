//! Accessibility auditor — walks a node tree and reports common a11y issues.

use dusty_a11y::element_role;
use dusty_core::{Element, Node};

/// Issue severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Must be fixed for accessibility compliance.
    Error,
    /// Should be reviewed and likely fixed.
    Warning,
}

/// The specific audit rule that was violated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditRule {
    /// Interactive element (Button, Checkbox, Radio, Toggle, Slider) has no
    /// `label` or `aria-label` attribute.
    MissingLabel,
    /// Element has event handlers but maps to `GenericContainer` — likely
    /// needs an explicit semantic role.
    MissingRole,
    /// Image element has no `label` or `aria-label` for alt text.
    MissingImageAlt,
    /// TextInput/Input has no `label`, `aria-label`, or `placeholder`.
    MissingInputLabel,
    /// Slider has no `value` attribute.
    MissingSliderValue,
}

/// A single audit issue found in the node tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditIssue {
    /// Severity of this issue.
    pub severity: Severity,
    /// Which audit rule was violated.
    pub rule: AuditRule,
    /// Human-readable description of the issue.
    pub message: String,
    /// Index of the node in the walk order.
    pub node_index: usize,
    /// Element name, if applicable.
    pub element_name: Option<String>,
}

/// Result of auditing a node tree.
#[derive(Debug, Clone)]
pub struct AuditResult {
    /// All issues found.
    pub issues: Vec<AuditIssue>,
    /// Total number of nodes examined.
    pub total_nodes_audited: usize,
    /// Number of nodes that have at least one issue.
    pub nodes_with_issues: usize,
}

/// Audits a node tree for common accessibility issues.
///
/// Walks the node tree (no layout required) and checks each element against
/// a set of rules. Returns an [`AuditResult`] with all issues found.
///
/// # Rules
///
/// | Rule | Severity | Trigger |
/// |------|----------|---------|
/// | `MissingLabel` | Error | Button, Checkbox, Radio, Toggle, Slider without `label`/`aria-label` |
/// | `MissingRole` | Warning | Element with event handlers mapping to `GenericContainer` |
/// | `MissingImageAlt` | Warning | Image without `label`/`aria-label` |
/// | `MissingInputLabel` | Error | TextInput/Input without `label`/`aria-label`/`placeholder` |
/// | `MissingSliderValue` | Warning | Slider without `value` attribute |
///
/// # Examples
///
/// ```
/// use dusty_reactive::{initialize_runtime, create_scope, dispose_runtime};
/// use dusty_core::el;
/// use dusty_devtools::auditor::{audit, AuditRule};
///
/// initialize_runtime();
/// create_scope(|cx| {
///     let node = el("Button", cx).build_node();
///     let result = audit(&node);
///     assert!(result.issues.iter().any(|i| i.rule == AuditRule::MissingLabel));
/// }).unwrap();
/// dispose_runtime();
/// ```
#[must_use]
pub fn audit(root: &Node) -> AuditResult {
    let mut walker = AuditWalker {
        issues: Vec::new(),
        node_count: 0,
        nodes_with_issues: std::collections::HashSet::new(),
    };

    walker.walk_node(root);

    AuditResult {
        issues: walker.issues,
        total_nodes_audited: walker.node_count,
        nodes_with_issues: walker.nodes_with_issues.len(),
    }
}

struct AuditWalker {
    issues: Vec<AuditIssue>,
    node_count: usize,
    nodes_with_issues: std::collections::HashSet<usize>,
}

impl AuditWalker {
    fn walk_node(&mut self, node: &Node) {
        match node {
            Node::Element(el) => self.audit_element(el),
            Node::Text(_) => {
                self.node_count += 1;
            }
            Node::Fragment(children) => {
                for child in children {
                    self.walk_node(child);
                }
            }
            Node::Component(comp) => {
                self.walk_node(&comp.child);
            }
            Node::Dynamic(dn) => {
                let resolved = dusty_reactive::untrack(|| dn.current_node());
                self.walk_node(&resolved);
            }
        }
    }

    fn audit_element(&mut self, el: &Element) {
        let node_index = self.node_count;
        self.node_count += 1;
        let name = el.name();

        self.check_missing_label(el, name, node_index);
        self.check_missing_role(el, name, node_index);
        self.check_missing_image_alt(el, name, node_index);
        self.check_missing_input_label(el, name, node_index);
        self.check_missing_slider_value(el, name, node_index);

        // Recurse into children
        for child in el.children() {
            self.walk_node(child);
        }
    }

    fn check_missing_label(&mut self, el: &Element, name: &str, node_index: usize) {
        let needs_label = matches!(name, "Button" | "Checkbox" | "Radio" | "Toggle" | "Slider");
        if !needs_label {
            return;
        }

        let has_label = el.attr("label").is_some() || el.attr("aria-label").is_some();
        let has_child_text = el
            .children()
            .iter()
            .any(|child| matches!(child, Node::Text(t) if !t.current_text().trim().is_empty()));
        if !has_label && !has_child_text {
            self.add_issue(AuditIssue {
                severity: Severity::Error,
                rule: AuditRule::MissingLabel,
                message: format!("{name} element is missing a label or aria-label attribute"),
                node_index,
                element_name: Some(name.to_string()),
            });
        }
    }

    fn check_missing_role(&mut self, el: &Element, name: &str, node_index: usize) {
        if el.event_handlers().is_empty() {
            return;
        }

        let role = element_role(name);
        if role == accesskit::Role::GenericContainer {
            self.add_issue(AuditIssue {
                severity: Severity::Warning,
                rule: AuditRule::MissingRole,
                message: format!(
                    "{name} element has event handlers but maps to GenericContainer — \
                     consider using a semantic element name or adding a role"
                ),
                node_index,
                element_name: Some(name.to_string()),
            });
        }
    }

    fn check_missing_image_alt(&mut self, el: &Element, name: &str, node_index: usize) {
        if name != "Image" {
            return;
        }

        let has_alt = el.attr("label").is_some() || el.attr("aria-label").is_some();
        if !has_alt {
            self.add_issue(AuditIssue {
                severity: Severity::Warning,
                rule: AuditRule::MissingImageAlt,
                message: "Image element is missing alt text (label or aria-label)".to_string(),
                node_index,
                element_name: Some(name.to_string()),
            });
        }
    }

    fn check_missing_input_label(&mut self, el: &Element, name: &str, node_index: usize) {
        if !matches!(name, "TextInput" | "Input") {
            return;
        }

        let has_label = el.attr("label").is_some()
            || el.attr("aria-label").is_some()
            || el.attr("placeholder").is_some();
        if !has_label {
            self.add_issue(AuditIssue {
                severity: Severity::Error,
                rule: AuditRule::MissingInputLabel,
                message: format!(
                    "{name} element is missing a label, aria-label, or placeholder attribute"
                ),
                node_index,
                element_name: Some(name.to_string()),
            });
        }
    }

    fn check_missing_slider_value(&mut self, el: &Element, name: &str, node_index: usize) {
        if name != "Slider" {
            return;
        }

        if el.attr("value").is_none() {
            self.add_issue(AuditIssue {
                severity: Severity::Warning,
                rule: AuditRule::MissingSliderValue,
                message: "Slider element is missing a value attribute".to_string(),
                node_index,
                element_name: Some(name.to_string()),
            });
        }
    }

    fn add_issue(&mut self, issue: AuditIssue) {
        self.nodes_with_issues.insert(issue.node_index);
        self.issues.push(issue);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use dusty_core::{el, text};
    use dusty_reactive::{create_scope, dispose_runtime, initialize_runtime};

    fn with_scope(f: impl FnOnce(dusty_reactive::Scope)) {
        initialize_runtime();
        create_scope(|cx| f(cx)).unwrap();
        dispose_runtime();
    }

    #[test]
    fn button_missing_label() {
        with_scope(|cx| {
            let node = el("Button", cx).build_node();
            let result = audit(&node);
            assert_eq!(result.issues.len(), 1);
            assert_eq!(result.issues[0].rule, AuditRule::MissingLabel);
            assert_eq!(result.issues[0].severity, Severity::Error);
        });
    }

    #[test]
    fn button_with_label_passes() {
        with_scope(|cx| {
            let node = el("Button", cx).attr("label", "Submit").build_node();
            let result = audit(&node);
            let label_issues: Vec<_> = result
                .issues
                .iter()
                .filter(|i| i.rule == AuditRule::MissingLabel)
                .collect();
            assert!(label_issues.is_empty());
        });
    }

    #[test]
    fn button_with_aria_label_passes() {
        with_scope(|cx| {
            let node = el("Button", cx).attr("aria-label", "Close").build_node();
            let result = audit(&node);
            let label_issues: Vec<_> = result
                .issues
                .iter()
                .filter(|i| i.rule == AuditRule::MissingLabel)
                .collect();
            assert!(label_issues.is_empty());
        });
    }

    #[test]
    fn checkbox_missing_label() {
        with_scope(|cx| {
            let node = el("Checkbox", cx).build_node();
            let result = audit(&node);
            assert!(result
                .issues
                .iter()
                .any(|i| i.rule == AuditRule::MissingLabel));
        });
    }

    #[test]
    fn image_missing_alt() {
        with_scope(|cx| {
            let node = el("Image", cx).build_node();
            let result = audit(&node);
            assert!(result
                .issues
                .iter()
                .any(|i| i.rule == AuditRule::MissingImageAlt));
            assert_eq!(
                result
                    .issues
                    .iter()
                    .find(|i| i.rule == AuditRule::MissingImageAlt)
                    .map(|i| i.severity),
                Some(Severity::Warning)
            );
        });
    }

    #[test]
    fn image_with_label_passes() {
        with_scope(|cx| {
            let node = el("Image", cx).attr("label", "Photo").build_node();
            let result = audit(&node);
            assert!(!result
                .issues
                .iter()
                .any(|i| i.rule == AuditRule::MissingImageAlt));
        });
    }

    #[test]
    fn input_missing_label() {
        with_scope(|cx| {
            let node = el("TextInput", cx).build_node();
            let result = audit(&node);
            assert!(result
                .issues
                .iter()
                .any(|i| i.rule == AuditRule::MissingInputLabel));
            assert_eq!(
                result
                    .issues
                    .iter()
                    .find(|i| i.rule == AuditRule::MissingInputLabel)
                    .map(|i| i.severity),
                Some(Severity::Error)
            );
        });
    }

    #[test]
    fn input_with_placeholder_passes() {
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
    fn slider_missing_value() {
        with_scope(|cx| {
            let node = el("Slider", cx).attr("label", "Volume").build_node();
            let result = audit(&node);
            assert!(result
                .issues
                .iter()
                .any(|i| i.rule == AuditRule::MissingSliderValue));
        });
    }

    #[test]
    fn slider_with_value_passes() {
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
    fn interactive_generic_container() {
        with_scope(|cx| {
            let node = el("CustomWidget", cx).on_click(|_ctx, _e| {}).build_node();
            let result = audit(&node);
            assert!(result
                .issues
                .iter()
                .any(|i| i.rule == AuditRule::MissingRole));
            assert_eq!(
                result
                    .issues
                    .iter()
                    .find(|i| i.rule == AuditRule::MissingRole)
                    .map(|i| i.severity),
                Some(Severity::Warning)
            );
        });
    }

    #[test]
    fn non_interactive_generic_container_passes() {
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
    fn known_element_with_handlers_passes_role_check() {
        with_scope(|cx| {
            let node = el("Button", cx)
                .attr("label", "OK")
                .on_click(|_ctx, _e| {})
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
    fn auditor_passes_when_child_text_present() {
        with_scope(|cx| {
            let node = el("Button", cx).child(text("Submit")).build_node();
            let result = audit(&node);
            let label_issues: Vec<_> = result
                .issues
                .iter()
                .filter(|i| i.rule == AuditRule::MissingLabel)
                .collect();
            assert!(
                label_issues.is_empty(),
                "Button with child text should not trigger MissingLabel"
            );
        });
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

            // Row has no issues, Button#1 has MissingLabel, Button#2 passes,
            // Image has MissingImageAlt, text has no issues
            assert_eq!(result.total_nodes_audited, 5); // Row + 2 Buttons + Image + Text
            assert_eq!(result.issues.len(), 2); // MissingLabel + MissingImageAlt
            assert_eq!(result.nodes_with_issues, 2);
        });
    }
}
