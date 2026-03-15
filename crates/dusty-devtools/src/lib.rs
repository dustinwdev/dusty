//! Inspector, profiler, and accessibility auditor for Dusty.
//!
//! Provides three analysis modules that operate on existing Dusty data
//! structures (node trees, layout results, styles) to produce testable
//! snapshots and reports. No rendering integration — this is a pure
//! analysis layer.
//!
//! Feature-gated behind `devtools` on the `dusty` facade crate so there
//! is zero cost when disabled.

pub mod auditor;
pub mod error;
pub mod inspector;
pub mod profiler;

pub use error::{DevtoolsError, Result};
