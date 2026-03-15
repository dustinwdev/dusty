//! Window configuration and size types.

/// A size in logical pixels (DPI-independent).
///
/// # Example
///
/// ```
/// use dusty_platform::LogicalSize;
///
/// let size = LogicalSize { width: 800.0, height: 600.0 };
/// assert_eq!(size.width, 800.0);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct LogicalSize {
    /// Width in logical pixels.
    pub width: f64,
    /// Height in logical pixels.
    pub height: f64,
}

impl PartialEq for LogicalSize {
    fn eq(&self, other: &Self) -> bool {
        self.width.total_cmp(&other.width).is_eq() && self.height.total_cmp(&other.height).is_eq()
    }
}

/// A size in physical pixels.
///
/// # Example
///
/// ```
/// use dusty_platform::PhysicalSize;
///
/// let size = PhysicalSize { width: 1600, height: 1200 };
/// assert_eq!(size.width, 1600);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalSize {
    /// Width in physical pixels.
    pub width: u32,
    /// Height in physical pixels.
    pub height: u32,
}

/// Configuration for creating a window.
///
/// Uses a builder pattern with consuming `self` methods.
///
/// # Example
///
/// ```
/// use dusty_platform::WindowConfig;
///
/// let config = WindowConfig::new("My App")
///     .width(1024.0)
///     .height(768.0)
///     .resizable(false);
/// assert_eq!(config.title(), "My App");
/// ```
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct WindowConfig {
    title: String,
    width: f64,
    height: f64,
    min_size: Option<LogicalSize>,
    max_size: Option<LogicalSize>,
    resizable: bool,
    decorations: bool,
    transparent: bool,
    visible: bool,
}

impl WindowConfig {
    /// Creates a new window configuration with the given title and default settings.
    #[must_use]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            width: 800.0,
            height: 600.0,
            min_size: None,
            max_size: None,
            resizable: true,
            decorations: true,
            transparent: false,
            visible: true,
        }
    }

    /// Sets the window width in logical pixels.
    #[must_use]
    pub const fn width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }

    /// Sets the window height in logical pixels.
    #[must_use]
    pub const fn height(mut self, height: f64) -> Self {
        self.height = height;
        self
    }

    /// Sets the minimum window size.
    #[must_use]
    pub const fn min_size(mut self, width: f64, height: f64) -> Self {
        self.min_size = Some(LogicalSize { width, height });
        self
    }

    /// Sets the maximum window size.
    #[must_use]
    pub const fn max_size(mut self, width: f64, height: f64) -> Self {
        self.max_size = Some(LogicalSize { width, height });
        self
    }

    /// Sets whether the window is resizable.
    #[must_use]
    pub const fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Sets whether the window has OS decorations (title bar, borders).
    #[must_use]
    pub const fn decorations(mut self, decorations: bool) -> Self {
        self.decorations = decorations;
        self
    }

    /// Sets whether the window background is transparent.
    #[must_use]
    pub const fn transparent(mut self, transparent: bool) -> Self {
        self.transparent = transparent;
        self
    }

    /// Sets whether the window is initially visible.
    #[must_use]
    pub const fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Returns the window title.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the logical size of the window.
    #[must_use]
    pub const fn size(&self) -> LogicalSize {
        LogicalSize {
            width: self.width,
            height: self.height,
        }
    }

    /// Returns the minimum size, if set.
    #[must_use]
    pub const fn min_size_value(&self) -> Option<LogicalSize> {
        self.min_size
    }

    /// Returns the maximum size, if set.
    #[must_use]
    pub const fn max_size_value(&self) -> Option<LogicalSize> {
        self.max_size
    }

    /// Returns whether the window is resizable.
    #[must_use]
    pub const fn is_resizable(&self) -> bool {
        self.resizable
    }

    /// Returns whether the window has decorations.
    #[must_use]
    pub const fn has_decorations(&self) -> bool {
        self.decorations
    }

    /// Returns whether the window is transparent.
    #[must_use]
    pub const fn is_transparent(&self) -> bool {
        self.transparent
    }

    /// Returns whether the window is initially visible.
    #[must_use]
    pub const fn is_visible(&self) -> bool {
        self.visible
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let config = WindowConfig::new("Test");
        assert_eq!(config.title(), "Test");
        assert_eq!(
            config.size(),
            LogicalSize {
                width: 800.0,
                height: 600.0
            }
        );
        assert!(config.is_resizable());
        assert!(config.has_decorations());
        assert!(!config.is_transparent());
        assert!(config.is_visible());
        assert!(config.min_size_value().is_none());
        assert!(config.max_size_value().is_none());
    }

    #[test]
    fn builder_chain() {
        let config = WindowConfig::new("App")
            .width(1024.0)
            .height(768.0)
            .min_size(320.0, 240.0)
            .max_size(1920.0, 1080.0)
            .resizable(false)
            .decorations(false)
            .transparent(true)
            .visible(false);

        assert_eq!(config.title(), "App");
        assert_eq!(
            config.size(),
            LogicalSize {
                width: 1024.0,
                height: 768.0
            }
        );
        assert!(!config.is_resizable());
        assert!(!config.has_decorations());
        assert!(config.is_transparent());
        assert!(!config.is_visible());
        assert_eq!(
            config.min_size_value(),
            Some(LogicalSize {
                width: 320.0,
                height: 240.0
            })
        );
        assert_eq!(
            config.max_size_value(),
            Some(LogicalSize {
                width: 1920.0,
                height: 1080.0
            })
        );
    }

    #[test]
    fn title_from_string() {
        let title = String::from("Dynamic Title");
        let config = WindowConfig::new(title);
        assert_eq!(config.title(), "Dynamic Title");
    }

    #[test]
    fn logical_size_equality() {
        let a = LogicalSize {
            width: 800.0,
            height: 600.0,
        };
        let b = LogicalSize {
            width: 800.0,
            height: 600.0,
        };
        assert_eq!(a, b);
    }

    #[test]
    fn logical_size_inequality() {
        let a = LogicalSize {
            width: 800.0,
            height: 600.0,
        };
        let b = LogicalSize {
            width: 1024.0,
            height: 600.0,
        };
        assert_ne!(a, b);
    }

    #[test]
    fn physical_size_equality() {
        let a = PhysicalSize {
            width: 1600,
            height: 1200,
        };
        let b = PhysicalSize {
            width: 1600,
            height: 1200,
        };
        assert_eq!(a, b);
    }
}
