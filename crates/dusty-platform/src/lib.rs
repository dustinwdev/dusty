//! Windowing, input, and event loop for Dusty.
//!
//! This crate bridges the OS platform (windowing, input, clipboard) and
//! Dusty's event/rendering pipeline. It translates winit events into
//! dusty-core event types and provides the main application entry point.
//!
//! # Example
//!
//! ```no_run
//! use dusty_platform::{run, WindowConfig, AppEvent, PlatformEvent};
//!
//! run(WindowConfig::new("My App"), |_window, event| {
//!     matches!(event, AppEvent::Platform(PlatformEvent::CloseRequested))
//! }).unwrap();
//! ```

pub mod clipboard;
pub mod config;
pub mod convert;
pub mod error;
pub mod event;
pub mod key;
pub mod runner;
pub mod scale;

pub use clipboard::Clipboard;
pub use config::{LogicalSize, PhysicalSize, WindowConfig};
pub use error::{PlatformError, Result};
pub use event::{AppEvent, PlatformEvent};
pub use key::{translate_key, translate_modifiers, translate_physical_key};
pub use runner::{run, PlatformWindow};
pub use scale::{LogicalPosition, PhysicalPosition, ScaleFactor};
