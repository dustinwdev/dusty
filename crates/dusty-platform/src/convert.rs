//! Winit event translation to [`AppEvent`].

use crate::event::{AppEvent, PlatformEvent};
use crate::key::translate_key;
use crate::scale::ScaleFactor;
use dusty_core::event::{
    ClickEvent, HoverEvent, KeyDownEvent, KeyUpEvent, Modifiers, ScrollEvent, TextInputEvent,
};
use winit::event::{ElementState, Ime, MouseButton, MouseScrollDelta, WindowEvent};

/// Translates a winit `WindowEvent` into an optional [`AppEvent`].
///
/// Requires current state: cursor position, modifiers, and scale factor.
/// Returns `None` for events we don't translate (e.g., `AxisMotion`).
#[must_use]
pub fn translate_window_event(
    event: &WindowEvent,
    cursor_x: f64,
    cursor_y: f64,
    modifiers: Modifiers,
    scale: ScaleFactor,
) -> Option<AppEvent> {
    match event {
        WindowEvent::CursorMoved { position, .. } => {
            Some(translate_cursor_moved(position.x, position.y, scale))
        }
        WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: MouseButton::Left,
            ..
        } => Some(translate_mouse_click(cursor_x, cursor_y)),
        WindowEvent::MouseWheel { delta, .. } => Some(translate_mouse_wheel(*delta, scale)),
        WindowEvent::KeyboardInput { event, .. } => {
            Some(translate_keyboard_input(event, modifiers))
        }
        WindowEvent::Ime(Ime::Commit(text)) => Some(translate_ime_commit(text)),
        WindowEvent::Resized(size) => Some(translate_resized(size.width, size.height, scale)),
        WindowEvent::CloseRequested => Some(AppEvent::Platform(PlatformEvent::CloseRequested)),
        WindowEvent::RedrawRequested => Some(AppEvent::Platform(PlatformEvent::RedrawRequested)),
        WindowEvent::Focused(focused) => Some(if *focused {
            AppEvent::Platform(PlatformEvent::WindowFocused)
        } else {
            AppEvent::Platform(PlatformEvent::WindowUnfocused)
        }),
        WindowEvent::ScaleFactorChanged { scale_factor, .. } => ScaleFactor::new(*scale_factor)
            .map(|sf| AppEvent::Platform(PlatformEvent::ScaleFactorChanged(sf))),
        _ => None,
    }
}

/// Translates cursor movement to a hover event in logical coordinates.
#[must_use]
pub(crate) fn translate_cursor_moved(
    physical_x: f64,
    physical_y: f64,
    scale: ScaleFactor,
) -> AppEvent {
    let factor = scale.value();
    AppEvent::Hover(HoverEvent {
        x: physical_x / factor,
        y: physical_y / factor,
    })
}

/// Translates a left mouse button press to a click event.
#[must_use]
pub(crate) const fn translate_mouse_click(x: f64, y: f64) -> AppEvent {
    AppEvent::Click(ClickEvent { x, y })
}

/// Translates mouse wheel delta to a scroll event.
#[must_use]
pub(crate) fn translate_mouse_wheel(delta: MouseScrollDelta, scale: ScaleFactor) -> AppEvent {
    let (dx, dy) = match delta {
        MouseScrollDelta::LineDelta(x, y) => (f64::from(x) * 40.0, f64::from(y) * 40.0),
        MouseScrollDelta::PixelDelta(pos) => {
            let factor = scale.value();
            (pos.x / factor, pos.y / factor)
        }
    };
    AppEvent::Scroll(ScrollEvent {
        delta_x: dx,
        delta_y: dy,
    })
}

/// Translates a keyboard input event.
#[must_use]
pub(crate) fn translate_keyboard_input(
    event: &winit::event::KeyEvent,
    modifiers: Modifiers,
) -> AppEvent {
    let key = translate_key(&event.logical_key);
    match event.state {
        ElementState::Pressed => AppEvent::KeyDown(KeyDownEvent { key, modifiers }),
        ElementState::Released => AppEvent::KeyUp(KeyUpEvent { key, modifiers }),
    }
}

/// Translates an IME commit to a text input event.
#[must_use]
pub(crate) fn translate_ime_commit(text: &str) -> AppEvent {
    AppEvent::TextInput(TextInputEvent {
        text: text.to_owned(),
    })
}

/// Translates a resize event from physical to logical pixels.
#[must_use]
pub(crate) fn translate_resized(
    physical_width: u32,
    physical_height: u32,
    scale: ScaleFactor,
) -> AppEvent {
    use crate::config::PhysicalSize;
    let logical = scale.to_logical(PhysicalSize {
        width: physical_width,
        height: physical_height,
    });
    AppEvent::Platform(PlatformEvent::Resized(logical))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::LogicalSize;

    fn scale_1x() -> ScaleFactor {
        ScaleFactor::new(1.0).unwrap()
    }

    fn scale_2x() -> ScaleFactor {
        ScaleFactor::new(2.0).unwrap()
    }

    // -- cursor moved --

    #[test]
    fn cursor_moved_1x() {
        let event = translate_cursor_moved(100.0, 200.0, scale_1x());
        match event {
            AppEvent::Hover(h) => {
                assert!((h.x - 100.0).abs() < f64::EPSILON);
                assert!((h.y - 200.0).abs() < f64::EPSILON);
            }
            _ => panic!("expected Hover"),
        }
    }

    #[test]
    fn cursor_moved_2x() {
        let event = translate_cursor_moved(200.0, 400.0, scale_2x());
        match event {
            AppEvent::Hover(h) => {
                assert!((h.x - 100.0).abs() < f64::EPSILON);
                assert!((h.y - 200.0).abs() < f64::EPSILON);
            }
            _ => panic!("expected Hover"),
        }
    }

    // -- mouse click --

    #[test]
    fn mouse_click() {
        let event = translate_mouse_click(50.0, 75.0);
        match event {
            AppEvent::Click(c) => {
                assert!((c.x - 50.0).abs() < f64::EPSILON);
                assert!((c.y - 75.0).abs() < f64::EPSILON);
            }
            _ => panic!("expected Click"),
        }
    }

    // -- mouse wheel --

    #[test]
    fn mouse_wheel_line_delta() {
        let event = translate_mouse_wheel(MouseScrollDelta::LineDelta(1.0, -2.0), scale_1x());
        match event {
            AppEvent::Scroll(s) => {
                assert!((s.delta_x - 40.0).abs() < f64::EPSILON);
                assert!((s.delta_y - -80.0).abs() < f64::EPSILON);
            }
            _ => panic!("expected Scroll"),
        }
    }

    #[test]
    fn mouse_wheel_pixel_delta_1x() {
        let delta = MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition::new(10.0, 20.0));
        let event = translate_mouse_wheel(delta, scale_1x());
        match event {
            AppEvent::Scroll(s) => {
                assert!((s.delta_x - 10.0).abs() < f64::EPSILON);
                assert!((s.delta_y - 20.0).abs() < f64::EPSILON);
            }
            _ => panic!("expected Scroll"),
        }
    }

    #[test]
    fn mouse_wheel_pixel_delta_2x() {
        let delta = MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition::new(20.0, 40.0));
        let event = translate_mouse_wheel(delta, scale_2x());
        match event {
            AppEvent::Scroll(s) => {
                assert!((s.delta_x - 10.0).abs() < f64::EPSILON);
                assert!((s.delta_y - 20.0).abs() < f64::EPSILON);
            }
            _ => panic!("expected Scroll"),
        }
    }

    // -- IME commit --

    #[test]
    fn ime_commit() {
        let event = translate_ime_commit("hello");
        match event {
            AppEvent::TextInput(t) => assert_eq!(t.text, "hello"),
            _ => panic!("expected TextInput"),
        }
    }

    #[test]
    fn ime_commit_empty() {
        let event = translate_ime_commit("");
        match event {
            AppEvent::TextInput(t) => assert_eq!(t.text, ""),
            _ => panic!("expected TextInput"),
        }
    }

    // -- resized --

    #[test]
    fn resized_1x() {
        let event = translate_resized(800, 600, scale_1x());
        match event {
            AppEvent::Platform(PlatformEvent::Resized(size)) => {
                assert_eq!(
                    size,
                    LogicalSize {
                        width: 800.0,
                        height: 600.0
                    }
                );
            }
            _ => panic!("expected Resized"),
        }
    }

    #[test]
    fn resized_2x() {
        let event = translate_resized(1600, 1200, scale_2x());
        match event {
            AppEvent::Platform(PlatformEvent::Resized(size)) => {
                assert_eq!(
                    size,
                    LogicalSize {
                        width: 800.0,
                        height: 600.0
                    }
                );
            }
            _ => panic!("expected Resized"),
        }
    }
}
