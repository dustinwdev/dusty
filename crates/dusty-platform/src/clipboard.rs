//! Clipboard access wrapper.

use crate::error::{PlatformError, Result};

/// Clipboard accessor wrapping the system clipboard.
///
/// # Example
///
/// ```no_run
/// use dusty_platform::Clipboard;
///
/// fn example() -> dusty_platform::Result<()> {
///     let mut clipboard = Clipboard::new()?;
///     clipboard.write("hello")?;
///     assert_eq!(clipboard.read()?, "hello");
///     Ok(())
/// }
/// ```
pub struct Clipboard {
    inner: arboard::Clipboard,
}

impl Clipboard {
    /// Creates a new clipboard instance.
    ///
    /// # Errors
    ///
    /// Returns [`PlatformError::ClipboardError`] if the system clipboard
    /// cannot be accessed.
    pub fn new() -> Result<Self> {
        let inner = arboard::Clipboard::new().map_err(PlatformError::from)?;
        Ok(Self { inner })
    }

    /// Reads text from the clipboard.
    ///
    /// # Errors
    ///
    /// Returns [`PlatformError::ClipboardError`] if reading fails.
    pub fn read(&mut self) -> Result<String> {
        self.inner.get_text().map_err(PlatformError::from)
    }

    /// Writes text to the clipboard.
    ///
    /// # Errors
    ///
    /// Returns [`PlatformError::ClipboardError`] if writing fails.
    pub fn write(&mut self, text: impl AsRef<str>) -> Result<()> {
        self.inner
            .set_text(text.as_ref())
            .map_err(PlatformError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires a display server / system clipboard
    fn clipboard_round_trip() {
        let mut clipboard = Clipboard::new().unwrap();
        clipboard.write("dusty-platform test").unwrap();
        let text = clipboard.read().unwrap();
        assert_eq!(text, "dusty-platform test");
    }

    #[test]
    #[ignore]
    fn clipboard_overwrite() {
        let mut clipboard = Clipboard::new().unwrap();
        clipboard.write("first").unwrap();
        clipboard.write("second").unwrap();
        let text = clipboard.read().unwrap();
        assert_eq!(text, "second");
    }

    #[test]
    #[ignore]
    fn clipboard_empty_string() {
        let mut clipboard = Clipboard::new().unwrap();
        clipboard.write("").unwrap();
        // Reading empty may return empty or error depending on platform
        let _ = clipboard.read();
    }
}
