//! Application builder for bootstrapping a Dusty application.

use dusty_core::Node;
use dusty_platform::{AppEvent, PlatformEvent, WindowConfig};
use dusty_reactive::Scope;
use dusty_style::theme::{provide_theme, Theme};

use crate::error::{DustyError, Result};

/// Creates a new [`App`] builder with the given window title.
///
/// # Example
///
/// ```no_run
/// use dusty::{app, prelude::*};
///
/// fn main() -> dusty::Result<()> {
///     app("My App")
///         .width(1024.0)
///         .height(768.0)
///         .root(|cx| Node::Text(text("Hello")))
///         .run()
/// }
/// ```
#[must_use]
pub fn app(title: impl Into<String>) -> App {
    App {
        config: WindowConfig::new(title),
        theme: None,
        root: None,
    }
}

/// Builder for configuring and running a Dusty application.
pub struct App {
    config: WindowConfig,
    theme: Option<Theme>,
    root: Option<Box<dyn FnOnce(Scope) -> Node>>,
}

impl App {
    /// Sets the window width in logical pixels.
    #[must_use]
    pub fn width(mut self, width: f64) -> Self {
        self.config = self.config.width(width);
        self
    }

    /// Sets the window height in logical pixels.
    #[must_use]
    pub fn height(mut self, height: f64) -> Self {
        self.config = self.config.height(height);
        self
    }

    /// Sets the minimum window size.
    #[must_use]
    pub fn min_size(mut self, width: f64, height: f64) -> Self {
        self.config = self.config.min_size(width, height);
        self
    }

    /// Sets the maximum window size.
    #[must_use]
    pub fn max_size(mut self, width: f64, height: f64) -> Self {
        self.config = self.config.max_size(width, height);
        self
    }

    /// Sets whether the window is resizable.
    #[must_use]
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.config = self.config.resizable(resizable);
        self
    }

    /// Sets whether the window has OS decorations.
    #[must_use]
    pub fn decorations(mut self, decorations: bool) -> Self {
        self.config = self.config.decorations(decorations);
        self
    }

    /// Sets whether the window background is transparent.
    #[must_use]
    pub fn transparent(mut self, transparent: bool) -> Self {
        self.config = self.config.transparent(transparent);
        self
    }

    /// Sets the theme for the application.
    #[must_use]
    pub const fn theme(mut self, theme: Theme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Sets the root component builder function.
    #[must_use]
    pub fn root(mut self, root_fn: impl FnOnce(Scope) -> Node + 'static) -> Self {
        self.root = Some(Box::new(root_fn));
        self
    }

    /// Runs the application, blocking until the window is closed.
    ///
    /// # Errors
    ///
    /// Returns [`DustyError::NoRoot`] if no root component was provided,
    /// [`DustyError::Reactive`] if the runtime fails to initialize,
    /// or [`DustyError::Platform`] if the event loop fails.
    pub fn run(self) -> Result<()> {
        let root_fn = self.root.ok_or(DustyError::NoRoot)?;
        let theme = self.theme.unwrap_or_else(Theme::light);

        dusty_reactive::initialize_runtime();

        let scope_result = dusty_reactive::create_scope(|cx| {
            // Provide theme — ignore error since scope is guaranteed active here
            let _ = provide_theme(theme);
            let _node = root_fn(cx);
            // Node is built; render integration happens in Phase 22
        });

        let root_scope = scope_result?;

        let platform_result = dusty_platform::run(self.config, |_window, event| {
            matches!(event, AppEvent::Platform(PlatformEvent::CloseRequested))
        });

        let _ = dusty_reactive::dispose_scope(root_scope);
        dusty_reactive::dispose_runtime();

        platform_result?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_constructor_sets_title() {
        let a = app("Test App");
        assert_eq!(a.config.title(), "Test App");
    }

    #[test]
    fn builder_chains_width_height() {
        let a = app("Test").width(1024.0).height(768.0);
        let size = a.config.size();
        assert!((size.width - 1024.0).abs() < f64::EPSILON);
        assert!((size.height - 768.0).abs() < f64::EPSILON);
    }

    #[test]
    fn builder_chains_min_max_size() {
        let a = app("Test").min_size(320.0, 240.0).max_size(1920.0, 1080.0);
        let min = a.config.min_size_value();
        let max = a.config.max_size_value();
        assert!(min.is_some());
        assert!(max.is_some());
    }

    #[test]
    fn builder_chains_window_flags() {
        let a = app("Test")
            .resizable(false)
            .decorations(false)
            .transparent(true);
        assert!(!a.config.is_resizable());
        assert!(!a.config.has_decorations());
        assert!(a.config.is_transparent());
    }

    #[test]
    fn builder_sets_theme() {
        let a = app("Test").theme(Theme::dark());
        assert_eq!(a.theme, Some(Theme::dark()));
    }

    #[test]
    fn builder_sets_root() {
        let a = app("Test").root(|_cx| dusty_core::Node::Text(dusty_core::text("hello")));
        assert!(a.root.is_some());
    }

    #[test]
    fn run_without_root_returns_no_root_error() {
        let result = app("Test").run();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, DustyError::NoRoot));
    }

    #[test]
    fn default_theme_is_none() {
        let a = app("Test");
        assert!(a.theme.is_none());
    }

    #[test]
    fn default_root_is_none() {
        let a = app("Test");
        assert!(a.root.is_none());
    }

    #[test]
    #[ignore] // Requires display server
    fn run_with_root_launches_event_loop() {
        let result = app("Integration Test")
            .width(400.0)
            .height(300.0)
            .root(|_cx| dusty_core::Node::Text(dusty_core::text("hello")))
            .run();
        // May fail on headless CI
        let _ = result;
    }
}
