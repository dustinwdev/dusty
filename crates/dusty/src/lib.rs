//! Dusty — Reactive, declarative GUI framework for Rust.
//!
//! This is the facade crate that re-exports all Dusty subcrates under
//! a single dependency. For most applications, add `dusty` to your
//! `Cargo.toml` and use the [`prelude`] for common imports.
//!
//! # Quick start
//!
//! ```no_run
//! use dusty::prelude::*;
//!
//! fn main() -> dusty::Result<()> {
//!     dusty::app("Hello Dusty")
//!         .root(|cx| Node::Text(text("Hello, world!")))
//!         .run()
//! }
//! ```

mod app;
mod error;
pub mod prelude;

// Subcrate module aliases
pub use dusty_a11y as a11y;
pub use dusty_layout as layout;
pub use dusty_platform as platform;
pub use dusty_reactive as reactive;
pub use dusty_render as render;
pub use dusty_style as style;
pub use dusty_widgets as widgets;

#[cfg(feature = "devtools")]
pub use dusty_devtools as devtools;

// Root-level exports
pub use app::{app, App};
pub use dusty_macros::component;
pub use dusty_widgets::{col, row};
pub use error::{DustyError, Result};
