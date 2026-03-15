//! Platform and application event types.

use crate::config::LogicalSize;
use crate::scale::ScaleFactor;
use dusty_core::event::{
    ClickEvent, HoverEvent, KeyDownEvent, KeyUpEvent, ScrollEvent, TextInputEvent,
};

/// Window-level events with no dusty-core equivalent.
#[derive(Debug, Clone)]
pub enum PlatformEvent {
    /// The window was resized (new logical size).
    Resized(LogicalSize),
    /// The user requested the window to close.
    CloseRequested,
    /// The window needs to be redrawn.
    RedrawRequested,
    /// The display scale factor changed.
    ScaleFactorChanged(ScaleFactor),
    /// The window gained focus.
    WindowFocused,
    /// The window lost focus.
    WindowUnfocused,
}

/// Unified event type combining dusty-core input events and platform events.
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// A mouse click.
    Click(ClickEvent),
    /// Mouse cursor movement.
    Hover(HoverEvent),
    /// A key was pressed.
    KeyDown(KeyDownEvent),
    /// A key was released.
    KeyUp(KeyUpEvent),
    /// A scroll wheel or trackpad scroll.
    Scroll(ScrollEvent),
    /// Text input (from IME or direct character input).
    TextInput(TextInputEvent),
    /// A window-level platform event.
    Platform(PlatformEvent),
}

#[cfg(test)]
mod tests {
    use super::*;
    use dusty_core::event::{Key, Modifiers};

    #[test]
    fn app_event_click() {
        let event = AppEvent::Click(ClickEvent { x: 10.0, y: 20.0 });
        assert!(matches!(event, AppEvent::Click(ClickEvent { x, y }) if x == 10.0 && y == 20.0));
    }

    #[test]
    fn app_event_hover() {
        let event = AppEvent::Hover(HoverEvent { x: 5.0, y: 15.0 });
        assert!(matches!(event, AppEvent::Hover(_)));
    }

    #[test]
    fn app_event_key_down() {
        let event = AppEvent::KeyDown(KeyDownEvent {
            key: Key("Enter".into()),
            modifiers: Modifiers::default(),
        });
        assert!(matches!(event, AppEvent::KeyDown(_)));
    }

    #[test]
    fn app_event_key_up() {
        let event = AppEvent::KeyUp(KeyUpEvent {
            key: Key("a".into()),
            modifiers: Modifiers::default(),
        });
        assert!(matches!(event, AppEvent::KeyUp(_)));
    }

    #[test]
    fn app_event_scroll() {
        let event = AppEvent::Scroll(ScrollEvent {
            delta_x: 0.0,
            delta_y: -120.0,
        });
        assert!(matches!(event, AppEvent::Scroll(_)));
    }

    #[test]
    fn app_event_text_input() {
        let event = AppEvent::TextInput(TextInputEvent {
            text: "hello".into(),
        });
        assert!(matches!(event, AppEvent::TextInput(_)));
    }

    #[test]
    fn platform_event_close_requested() {
        let event = AppEvent::Platform(PlatformEvent::CloseRequested);
        assert!(matches!(
            event,
            AppEvent::Platform(PlatformEvent::CloseRequested)
        ));
    }

    #[test]
    fn platform_event_resized() {
        let event = AppEvent::Platform(PlatformEvent::Resized(LogicalSize {
            width: 1024.0,
            height: 768.0,
        }));
        assert!(matches!(
            event,
            AppEvent::Platform(PlatformEvent::Resized(_))
        ));
    }

    #[test]
    fn platform_event_redraw() {
        let event = AppEvent::Platform(PlatformEvent::RedrawRequested);
        assert!(matches!(
            event,
            AppEvent::Platform(PlatformEvent::RedrawRequested)
        ));
    }

    #[test]
    fn platform_event_focused() {
        let focused = AppEvent::Platform(PlatformEvent::WindowFocused);
        let unfocused = AppEvent::Platform(PlatformEvent::WindowUnfocused);
        assert!(matches!(
            focused,
            AppEvent::Platform(PlatformEvent::WindowFocused)
        ));
        assert!(matches!(
            unfocused,
            AppEvent::Platform(PlatformEvent::WindowUnfocused)
        ));
    }

    #[test]
    fn platform_event_scale_factor_changed() {
        let scale = ScaleFactor::new(2.0).unwrap();
        let event = AppEvent::Platform(PlatformEvent::ScaleFactorChanged(scale));
        assert!(matches!(
            event,
            AppEvent::Platform(PlatformEvent::ScaleFactorChanged(_))
        ));
    }
}
