//! Frame — the drawing context passed to canvas draw closures.

use crate::canvas::command::{CanvasCommand, FillStyle, Point, StrokeStyle, Transform};

/// Drawing context for the [`Canvas`](super::Canvas) widget.
///
/// Records drawing commands that are later consumed by the renderer.
///
/// # Example
///
/// ```
/// use dusty_widgets::canvas::{Frame, FillStyle};
/// use dusty_style::Color;
///
/// let mut frame = Frame::new();
/// frame.rect(10.0, 10.0, 80.0, 40.0, Some(FillStyle::Solid(Color::WHITE)), None);
/// let cmds = frame.into_commands();
/// assert_eq!(cmds.len(), 1);
/// ```
pub struct Frame {
    commands: Vec<CanvasCommand>,
}

impl Frame {
    /// Creates a new empty frame.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    // -- Path commands --

    /// Moves the pen to the given point.
    pub fn move_to(&mut self, x: f32, y: f32) {
        self.commands.push(CanvasCommand::MoveTo(Point::new(x, y)));
    }

    /// Draws a line from the current pen position to the given point.
    pub fn line_to(&mut self, x: f32, y: f32) {
        self.commands.push(CanvasCommand::LineTo(Point::new(x, y)));
    }

    /// Draws a quadratic bezier curve.
    pub fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.commands.push(CanvasCommand::QuadTo {
            control: Point::new(cx, cy),
            to: Point::new(x, y),
        });
    }

    /// Draws a cubic bezier curve.
    pub fn cubic_to(&mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) {
        self.commands.push(CanvasCommand::CubicTo {
            c1: Point::new(c1x, c1y),
            c2: Point::new(c2x, c2y),
            to: Point::new(x, y),
        });
    }

    /// Closes the current path.
    pub fn close_path(&mut self) {
        self.commands.push(CanvasCommand::ClosePath);
    }

    // -- Paint commands --

    /// Fills the current path with the given style.
    pub fn fill(&mut self, style: FillStyle) {
        self.commands.push(CanvasCommand::Fill(style));
    }

    /// Strokes the current path with the given style.
    pub fn stroke(&mut self, style: StrokeStyle) {
        self.commands.push(CanvasCommand::Stroke(style));
    }

    // -- Shape shorthands --

    /// Draws a rectangle.
    pub fn rect(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        fill: Option<FillStyle>,
        stroke: Option<StrokeStyle>,
    ) {
        self.commands.push(CanvasCommand::Rect {
            x,
            y,
            width,
            height,
            fill,
            stroke,
        });
    }

    /// Draws a rounded rectangle.
    #[allow(clippy::too_many_arguments)]
    pub fn round_rect(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        radius: f32,
        fill: Option<FillStyle>,
        stroke: Option<StrokeStyle>,
    ) {
        self.commands.push(CanvasCommand::RoundRect {
            x,
            y,
            width,
            height,
            radius,
            fill,
            stroke,
        });
    }

    /// Draws a circle.
    pub fn circle(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        fill: Option<FillStyle>,
        stroke: Option<StrokeStyle>,
    ) {
        self.commands.push(CanvasCommand::Circle {
            cx,
            cy,
            radius,
            fill,
            stroke,
        });
    }

    // -- Content commands --

    /// Draws text at the given position.
    pub fn text(&mut self, content: impl Into<String>, x: f32, y: f32, fill: FillStyle) {
        self.commands.push(CanvasCommand::Text {
            content: content.into(),
            x,
            y,
            fill,
        });
    }

    /// Draws an image at the given position and size.
    pub fn image(&mut self, src: impl Into<String>, x: f32, y: f32, width: f32, height: f32) {
        self.commands.push(CanvasCommand::Image {
            src: src.into(),
            x,
            y,
            width,
            height,
        });
    }

    // -- Transform commands --

    /// Pushes a transform onto the transform stack.
    pub fn push_transform(&mut self, transform: Transform) {
        self.commands.push(CanvasCommand::PushTransform(transform));
    }

    /// Pops the most recent transform from the stack.
    pub fn pop_transform(&mut self) {
        self.commands.push(CanvasCommand::PopTransform);
    }

    // -- Clip commands --

    /// Pushes a clip rectangle.
    pub fn push_clip(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.commands.push(CanvasCommand::PushClip {
            x,
            y,
            width,
            height,
        });
    }

    /// Pops the most recent clip.
    pub fn pop_clip(&mut self) {
        self.commands.push(CanvasCommand::PopClip);
    }

    /// Consumes the frame and returns the recorded commands.
    #[must_use]
    pub fn into_commands(self) -> Vec<CanvasCommand> {
        self.commands
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dusty_style::Color;

    #[test]
    fn records_move_to() {
        let mut frame = Frame::new();
        frame.move_to(10.0, 20.0);
        let cmds = frame.into_commands();
        assert_eq!(cmds.len(), 1);
        assert!(matches!(
            cmds[0],
            CanvasCommand::MoveTo(Point { x: 10.0, y: 20.0 })
        ));
    }

    #[test]
    fn records_line_to() {
        let mut frame = Frame::new();
        frame.line_to(30.0, 40.0);
        let cmds = frame.into_commands();
        assert_eq!(cmds.len(), 1);
        assert!(matches!(
            cmds[0],
            CanvasCommand::LineTo(Point { x: 30.0, y: 40.0 })
        ));
    }

    #[test]
    fn records_fill() {
        let mut frame = Frame::new();
        frame.fill(FillStyle::Solid(Color::WHITE));
        let cmds = frame.into_commands();
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], CanvasCommand::Fill(FillStyle::Solid(_))));
    }

    #[test]
    fn rect_shorthand() {
        let mut frame = Frame::new();
        frame.rect(
            0.0,
            0.0,
            100.0,
            50.0,
            Some(FillStyle::Solid(Color::WHITE)),
            None,
        );
        let cmds = frame.into_commands();
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], CanvasCommand::Rect { .. }));
    }

    #[test]
    fn round_rect() {
        let mut frame = Frame::new();
        frame.round_rect(
            0.0,
            0.0,
            100.0,
            50.0,
            8.0,
            Some(FillStyle::Solid(Color::BLACK)),
            None,
        );
        let cmds = frame.into_commands();
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], CanvasCommand::RoundRect { .. }));
    }

    #[test]
    fn circle() {
        let mut frame = Frame::new();
        frame.circle(50.0, 50.0, 25.0, Some(FillStyle::Solid(Color::WHITE)), None);
        let cmds = frame.into_commands();
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], CanvasCommand::Circle { .. }));
    }

    #[test]
    fn transform_push_pop() {
        let mut frame = Frame::new();
        frame.push_transform(Transform::translate(10.0, 20.0));
        frame.rect(
            0.0,
            0.0,
            50.0,
            50.0,
            Some(FillStyle::Solid(Color::WHITE)),
            None,
        );
        frame.pop_transform();
        let cmds = frame.into_commands();
        assert_eq!(cmds.len(), 3);
        assert!(matches!(cmds[0], CanvasCommand::PushTransform(_)));
        assert!(matches!(cmds[2], CanvasCommand::PopTransform));
    }

    #[test]
    fn clip_push_pop() {
        let mut frame = Frame::new();
        frame.push_clip(0.0, 0.0, 100.0, 100.0);
        frame.pop_clip();
        let cmds = frame.into_commands();
        assert_eq!(cmds.len(), 2);
        assert!(matches!(cmds[0], CanvasCommand::PushClip { .. }));
        assert!(matches!(cmds[1], CanvasCommand::PopClip));
    }

    #[test]
    fn into_commands() {
        let mut frame = Frame::new();
        frame.move_to(0.0, 0.0);
        frame.line_to(10.0, 10.0);
        frame.close_path();
        frame.fill(FillStyle::Solid(Color::BLACK));
        let cmds = frame.into_commands();
        assert_eq!(cmds.len(), 4);
    }

    #[test]
    fn transform_constructors() {
        let t = Transform::translate(5.0, 10.0);
        assert_eq!(t.matrix[4], 5.0);
        assert_eq!(t.matrix[5], 10.0);

        let s = Transform::scale(2.0, 3.0);
        assert_eq!(s.matrix[0], 2.0);
        assert_eq!(s.matrix[3], 3.0);

        let r = Transform::rotate(0.0);
        assert_eq!(r, Transform::IDENTITY);
    }
}
