//! Text shaping, measurement, and rendering for Dusty.
//!
//! Wraps [`cosmic-text`] to provide text measurement for the layout engine
//! via [`TextMeasure`](dusty_layout::TextMeasure), plus richer APIs for
//! styled text, rich spans, and truncation.
//!
//! # Key Types
//!
//! - [`TextSystem`] — central text system, implements `TextMeasure`
//! - [`TextLayout`] — a shaped text buffer with size/line queries
//! - [`TextSpan`] — a styled range for rich text
//!
//! # Examples
//!
//! ```
//! use dusty_text::TextSystem;
//! use dusty_layout::TextMeasure;
//! use dusty_style::FontStyle;
//!
//! let system = TextSystem::new();
//! let (w, h) = system.measure("hello", None, &FontStyle::default());
//! assert!(w > 0.0);
//! ```

mod convert;
pub mod error;
pub mod layout;
pub mod rasterize;
pub mod rich;
pub mod system;
pub mod truncate;

pub use error::{Result, TextError};
pub use layout::TextLayout;
pub use rasterize::{GlyphRasterizer, RasterizedGlyph};
pub use rich::TextSpan;
pub use system::TextSystem;
pub use truncate::{TruncatedText, Truncation};

/// Re-export cosmic-text's `CacheKey` for glyph identification.
pub use cosmic_text::CacheKey;
