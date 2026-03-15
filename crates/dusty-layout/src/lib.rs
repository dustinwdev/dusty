//! Flexbox layout engine for Dusty.
//!
//! Integrates [taffy](https://docs.rs/taffy) to convert a [`Node`](dusty_core::Node) tree
//! with [`Style`](dusty_style::Style) data into computed layout rectangles.
//!
//! # Overview
//!
//! The single entry point is [`compute_layout`], which takes a node tree,
//! available space, and a [`TextMeasure`] callback, returning a
//! [`LayoutResult`] that maps each element/text node to an absolute [`Rect`].

mod convert;
pub mod error;
mod measure;
mod result;
mod tree;

pub use error::{LayoutError, Result};
pub use measure::TextMeasure;
pub use result::{LayoutNodeId, LayoutResult, Rect};
pub use tree::compute_layout;
