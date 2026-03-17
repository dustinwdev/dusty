//! Event loop runner and window management.

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes};

use crate::config::{LogicalSize, PhysicalSize, WindowConfig};
use crate::convert::translate_window_event;
use crate::error::{PlatformError, Result};
use crate::event::AppEvent;
use crate::key::translate_modifiers;
use crate::scale::ScaleFactor;
use dusty_core::event::Modifiers;

/// A handle to the platform window.
///
/// Provides access to the underlying winit window for rendering integration
/// and window state queries.
pub struct PlatformWindow {
    window: Arc<Window>,
}

impl PlatformWindow {
    /// Returns a reference to the underlying winit window.
    ///
    /// Used by the render layer (Phase 13) to create a wgpu surface.
    #[must_use]
    pub fn raw_window(&self) -> &Window {
        &self.window
    }

    /// Returns the current scale factor.
    #[must_use]
    pub fn scale_factor(&self) -> ScaleFactor {
        // winit guarantees scale_factor > 0 and finite
        ScaleFactor::new(self.window.scale_factor()).unwrap_or(ScaleFactor::default_scale())
    }

    /// Returns the inner size in logical pixels.
    #[must_use]
    pub fn inner_size(&self) -> LogicalSize {
        let physical = self.window.inner_size();
        let scale = self.scale_factor();
        scale.to_logical(PhysicalSize {
            width: physical.width,
            height: physical.height,
        })
    }

    /// Requests that the window be redrawn.
    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }
}

/// Internal state for the winit `ApplicationHandler`.
struct AppHandler<F> {
    config: WindowConfig,
    handler: F,
    window: Option<PlatformWindow>,
    cursor_x: f64,
    cursor_y: f64,
    modifiers: Modifiers,
    error: Option<PlatformError>,
}

impl<F> AppHandler<F> {
    fn new(config: WindowConfig, handler: F) -> Self {
        Self {
            config,
            handler,
            window: None,
            cursor_x: 0.0,
            cursor_y: 0.0,
            modifiers: Modifiers::default(),
            error: None,
        }
    }

    fn scale_factor(&self) -> ScaleFactor {
        self.window
            .as_ref()
            .map_or_else(ScaleFactor::default_scale, PlatformWindow::scale_factor)
    }
}

impl<F> ApplicationHandler for AppHandler<F>
where
    F: FnMut(&PlatformWindow, &AppEvent) -> bool + 'static,
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let size = self.config.size();
        let mut attrs = WindowAttributes::default()
            .with_title(self.config.title())
            .with_inner_size(winit::dpi::LogicalSize::new(size.width, size.height))
            .with_resizable(self.config.is_resizable())
            .with_decorations(self.config.has_decorations())
            .with_transparent(self.config.is_transparent())
            .with_visible(self.config.is_visible());

        if let Some(min) = self.config.min_size_value() {
            attrs = attrs.with_min_inner_size(winit::dpi::LogicalSize::new(min.width, min.height));
        }
        if let Some(max) = self.config.max_size_value() {
            attrs = attrs.with_max_inner_size(winit::dpi::LogicalSize::new(max.width, max.height));
        }

        match event_loop.create_window(attrs) {
            Ok(window) => {
                self.window = Some(PlatformWindow {
                    window: Arc::new(window),
                });
            }
            Err(err) => {
                self.error = Some(PlatformError::WindowCreation(err.to_string()));
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        // Update stored modifier state
        if let WindowEvent::ModifiersChanged(mods) = &event {
            self.modifiers = translate_modifiers(mods.state());
            return;
        }

        // Track cursor position
        if let WindowEvent::CursorMoved { position, .. } = &event {
            let scale = self.scale_factor();
            self.cursor_x = position.x / scale.value();
            self.cursor_y = position.y / scale.value();
        }

        let scale = self.scale_factor();

        if let Some(app_event) =
            translate_window_event(&event, self.cursor_x, self.cursor_y, self.modifiers, scale)
        {
            if let Some(window) = &self.window {
                let should_exit = (self.handler)(window, &app_event);
                if should_exit {
                    event_loop.exit();
                }
            }
        }
    }
}

/// Runs the platform event loop with the given window configuration and event handler.
///
/// The handler receives each [`AppEvent`] and returns `true` to exit the event loop.
///
/// This function blocks until the event loop exits.
///
/// # Errors
///
/// Returns [`PlatformError::EventLoopCreation`] if the event loop cannot be created,
/// or [`PlatformError::EventLoopExit`] if it exits with an error.
///
/// # Example
///
/// ```no_run
/// use dusty_platform::{run, WindowConfig, AppEvent, PlatformEvent};
///
/// run(WindowConfig::new("My App"), |_window, event| {
///     matches!(event, AppEvent::Platform(PlatformEvent::CloseRequested))
/// }).unwrap();
/// ```
pub fn run(
    config: WindowConfig,
    handler: impl FnMut(&PlatformWindow, &AppEvent) -> bool + 'static,
) -> Result<()> {
    let event_loop =
        EventLoop::new().map_err(|e| PlatformError::EventLoopCreation(format!("{e}")))?;
    let mut app = AppHandler::new(config, handler);
    event_loop
        .run_app(&mut app)
        .map_err(|e| PlatformError::EventLoopExit(format!("{e}")))?;
    if let Some(err) = app.error {
        return Err(err);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_handler_initial_state() {
        let config = WindowConfig::new("Test");
        let handler = AppHandler::new(config, |_window: &PlatformWindow, _event: &AppEvent| false);
        assert!(handler.window.is_none());
        assert!((handler.cursor_x).abs() < f64::EPSILON);
        assert!((handler.cursor_y).abs() < f64::EPSILON);
        assert_eq!(handler.modifiers, Modifiers::default());
    }

    #[test]
    fn app_handler_initial_error_is_none() {
        let config = WindowConfig::new("Test");
        let handler = AppHandler::new(config, |_window: &PlatformWindow, _event: &AppEvent| false);
        assert!(handler.error.is_none());
    }

    #[test]
    fn app_handler_default_scale() {
        let config = WindowConfig::new("Test");
        let handler = AppHandler::new(config, |_window: &PlatformWindow, _event: &AppEvent| false);
        let scale = handler.scale_factor();
        assert!((scale.value() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    #[ignore] // Requires display server
    fn run_with_immediate_close() {
        let result = run(
            WindowConfig::new("Test").visible(false),
            |_window, event| {
                matches!(
                    event,
                    AppEvent::Platform(crate::event::PlatformEvent::CloseRequested)
                )
            },
        );
        // May fail on headless CI, but tests the API compiles
        let _ = result;
    }
}
