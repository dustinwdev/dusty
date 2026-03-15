//! Accessibility tree integration for Dusty.
//!
//! Generates an accesskit [`TreeUpdate`](accesskit::TreeUpdate) from a Dusty
//! [`Node`](dusty_core::Node) tree and its computed
//! [`LayoutResult`](dusty_layout::LayoutResult), so assistive technologies
//! (screen readers, switch access, voice control) can interact with the UI.

mod error;
mod role;
mod tree;

pub use error::{A11yError, Result};
pub use role::element_role;
pub use tree::build_accessibility_tree;
