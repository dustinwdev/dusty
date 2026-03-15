//! Render error types.

use core::fmt;

/// Errors that can occur during GPU rendering.
///
/// # Examples
///
/// ```
/// use dusty_render::RenderError;
///
/// let err = RenderError::NoAdapter;
/// assert_eq!(format!("{err}"), "no suitable GPU adapter found");
/// ```
#[derive(Debug)]
pub enum RenderError {
    /// No suitable GPU adapter was found.
    NoAdapter,
    /// Failed to create the GPU device.
    DeviceCreation(String),
    /// Failed to configure the rendering surface.
    SurfaceConfig(String),
    /// Shader compilation failed.
    ShaderCompilation(String),
    /// The rendering surface was lost and must be recreated.
    SurfaceLost,
    /// An error occurred during drawing.
    DrawError(String),
    /// The texture atlas is full and cannot allocate more space.
    AtlasFull,
    /// Failed to decode an image.
    ImageDecode(String),
    /// Failed to upload an image to the GPU.
    ImageUpload(String),
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoAdapter => write!(f, "no suitable GPU adapter found"),
            Self::DeviceCreation(msg) => write!(f, "GPU device creation failed: {msg}"),
            Self::SurfaceConfig(msg) => write!(f, "surface configuration failed: {msg}"),
            Self::ShaderCompilation(msg) => write!(f, "shader compilation failed: {msg}"),
            Self::SurfaceLost => write!(f, "rendering surface lost"),
            Self::DrawError(msg) => write!(f, "draw error: {msg}"),
            Self::AtlasFull => write!(f, "texture atlas is full"),
            Self::ImageDecode(msg) => write!(f, "image decode failed: {msg}"),
            Self::ImageUpload(msg) => write!(f, "image upload failed: {msg}"),
        }
    }
}

impl std::error::Error for RenderError {}

/// A specialized `Result` type for render operations.
pub type Result<T> = std::result::Result<T, RenderError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_no_adapter() {
        let err = RenderError::NoAdapter;
        assert_eq!(format!("{err}"), "no suitable GPU adapter found");
    }

    #[test]
    fn display_device_creation() {
        let err = RenderError::DeviceCreation("out of memory".into());
        assert_eq!(
            format!("{err}"),
            "GPU device creation failed: out of memory"
        );
    }

    #[test]
    fn display_surface_config() {
        let err = RenderError::SurfaceConfig("invalid format".into());
        assert_eq!(
            format!("{err}"),
            "surface configuration failed: invalid format"
        );
    }

    #[test]
    fn display_shader_compilation() {
        let err = RenderError::ShaderCompilation("syntax error".into());
        assert_eq!(format!("{err}"), "shader compilation failed: syntax error");
    }

    #[test]
    fn display_surface_lost() {
        let err = RenderError::SurfaceLost;
        assert_eq!(format!("{err}"), "rendering surface lost");
    }

    #[test]
    fn display_draw_error() {
        let err = RenderError::DrawError("buffer overflow".into());
        assert_eq!(format!("{err}"), "draw error: buffer overflow");
    }

    #[test]
    fn error_trait_is_implemented() {
        let err: &dyn std::error::Error = &RenderError::NoAdapter;
        // Verify the trait object is usable
        let _ = format!("{err}");
    }

    #[test]
    fn debug_output() {
        let err = RenderError::NoAdapter;
        let debug = format!("{err:?}");
        assert!(debug.contains("NoAdapter"));
    }

    #[test]
    fn display_atlas_full() {
        let err = RenderError::AtlasFull;
        assert_eq!(format!("{err}"), "texture atlas is full");
    }

    #[test]
    fn display_image_decode() {
        let err = RenderError::ImageDecode("invalid PNG".into());
        assert_eq!(format!("{err}"), "image decode failed: invalid PNG");
    }

    #[test]
    fn display_image_upload() {
        let err = RenderError::ImageUpload("out of VRAM".into());
        assert_eq!(format!("{err}"), "image upload failed: out of VRAM");
    }
}
