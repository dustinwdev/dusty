//! Built-in widget library for Dusty.
//!
//! Provides display widgets ([`Text`], [`Image`], [`Divider`], [`Spacer`],
//! [`Canvas`]), interactive widgets ([`Button`], [`Checkbox`], [`Toggle`],
//! [`Radio`], [`Slider`], [`TextInput`]), and container widgets
//! ([`ScrollView`], [`Show`], [`MatchView`], [`For`], [`ErrorBoundary`],
//! [`Suspense`]).

mod button;
pub mod canvas;
mod checkbox;
mod divider;
mod error;
mod error_boundary;
mod for_each;
mod image;
mod macros;
mod match_view;
mod radio;
mod scroll_view;
mod show;
mod slider;
mod spacer;
mod suspense;
mod text;
pub mod text_input;
mod toggle;

/// Re-exports for macro hygiene. Not part of the public API.
#[doc(hidden)]
pub mod __macro_internals {
    pub use dusty_core::{el, ElementBuilder};
    pub use dusty_style::{FlexDirection, Style};
}

pub use button::{Button, ButtonVariant};
pub use canvas::Canvas;
pub use checkbox::{Checkbox, CheckedSource};
pub use divider::{Divider, Orientation};
pub use error::{Result, WidgetError};
pub use error_boundary::ErrorBoundary;
pub use for_each::For;
pub use image::{Image, SizingMode};
pub use match_view::MatchView;
pub use radio::Radio;
pub use scroll_view::{ScrollAxis, ScrollView};
pub use show::Show;
pub use slider::{Slider, SliderSource};
pub use spacer::Spacer;
pub use suspense::Suspense;
pub use text::Text;
pub use text_input::{InputSource, TextInput, TextInputState};
pub use toggle::{Toggle, ToggleSource};
