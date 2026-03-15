//! Utility styling and theme engine for Dusty.
//!
//! Provides the core [`Style`] type with cascade/merge semantics,
//! design tokens for spacing/radius/shadow, a Tailwind-inspired
//! color palette, builder methods, and theme propagation.

mod builder;
mod color;
mod corners;
mod edges;
mod font;
mod gradient;
pub mod palette;
mod shadow;
mod style;
pub mod theme;
pub mod tokens;

pub use color::Color;
pub use corners::Corners;
pub use edges::Edges;
pub use font::{FontSlant, FontStyle, FontWeight};
pub use gradient::{ColorStop, GradientDirection, LinearGradient};
pub use palette::{ColorScale, Palette};
pub use shadow::BoxShadow;
pub use style::{
    AlignItems, AlignSelf, FlexDirection, FlexWrap, InteractionState, JustifyContent, Overflow,
    Style,
};
