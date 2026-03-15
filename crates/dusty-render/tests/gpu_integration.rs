//! GPU integration tests.
//!
//! These tests require a GPU and are marked `#[ignore]` for CI environments
//! without GPU access. Run with: `cargo test -p dusty-render -- --ignored`

// GPU tests require a window handle, which isn't available in headless CI.
// These are placeholder tests that document the expected GPU behavior.

#[test]
#[ignore = "requires GPU"]
fn gpu_context_creation() {
    // In a real test with a window:
    // let ctx = pollster::block_on(GpuContext::new(window, 800, 600));
    // assert!(ctx.is_ok());
}

#[test]
#[ignore = "requires GPU"]
fn pipeline_creation() {
    // In a real test:
    // let ctx = pollster::block_on(GpuContext::new(window, 800, 600)).unwrap();
    // let pipeline = RenderPipeline::new(&ctx);
    // Pipeline creation succeeds (panics if shader compilation fails)
}

#[test]
#[ignore = "requires GPU"]
fn render_empty_commands() {
    // In a real test:
    // let mut renderer = pollster::block_on(Renderer::new(window, 800, 600)).unwrap();
    // let result = renderer.render(&[]);
    // assert!(result.is_ok());
}

#[test]
#[ignore = "requires GPU"]
fn render_single_rect() {
    // In a real test:
    // Render a white rect and verify the output texture contains white pixels
}

#[test]
#[ignore = "requires GPU"]
fn resize_updates_surface() {
    // In a real test:
    // renderer.resize(1024, 768);
    // assert_eq!(renderer.size(), (1024, 768));
}
