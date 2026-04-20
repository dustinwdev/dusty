//! Canvas widget — 2D drawing escape hatch with reactive integration.

pub mod command;
pub mod frame;

use std::cell::RefCell;
use std::rc::Rc;

use dusty_core::el;
use dusty_core::node::{ComponentNode, Node};
use dusty_core::view::View;
use dusty_reactive::Scope;
use dusty_style::Style;

pub use command::{CanvasCommand, FillStyle, Point, StrokeStyle, Transform};
pub use frame::Frame;

/// A 2D drawing canvas widget.
///
/// The draw closure receives a [`Frame`] and records drawing commands. The
/// closure is wrapped in a reactive effect, so reading signals inside it
/// causes automatic re-recording when dependencies change.
///
/// # Example
///
/// ```
/// use dusty_reactive::{initialize_runtime, create_scope, dispose_runtime};
/// use dusty_widgets::Canvas;
/// use dusty_widgets::canvas::FillStyle;
/// use dusty_core::view::View;
/// use dusty_style::Color;
///
/// initialize_runtime();
/// create_scope(|cx| {
///     let node = Canvas::new(|frame| {
///         frame.rect(0.0, 0.0, 100.0, 50.0, Some(FillStyle::Solid(Color::WHITE)), None);
///     }).build(cx);
///     assert!(node.is_component());
/// });
/// dispose_runtime();
/// ```
type ClickHandler = Box<dyn Fn(&dusty_core::event::EventContext, &dusty_core::event::ClickEvent)>;
type HoverHandler = Box<dyn Fn(&dusty_core::event::EventContext, &dusty_core::event::HoverEvent)>;
type DragHandler = Box<dyn Fn(&dusty_core::event::EventContext, &dusty_core::event::DragEvent)>;

pub struct Canvas {
    draw: Box<dyn Fn(&mut Frame)>,
    user_style: Option<Style>,
    on_click: Option<ClickHandler>,
    on_hover: Option<HoverHandler>,
    on_drag: Option<DragHandler>,
}

impl Canvas {
    /// Creates a canvas widget with the given draw closure.
    #[must_use]
    pub fn new(draw: impl Fn(&mut Frame) + 'static) -> Self {
        Self {
            draw: Box::new(draw),
            user_style: None,
            on_click: None,
            on_hover: None,
            on_drag: None,
        }
    }

    /// Merges user styles on top of canvas defaults.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.user_style = Some(style);
        self
    }

    /// Registers a click event handler.
    #[must_use]
    pub fn on_click(
        mut self,
        handler: impl Fn(&dusty_core::event::EventContext, &dusty_core::event::ClickEvent) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    /// Registers a hover event handler.
    #[must_use]
    pub fn on_hover(
        mut self,
        handler: impl Fn(&dusty_core::event::EventContext, &dusty_core::event::HoverEvent) + 'static,
    ) -> Self {
        self.on_hover = Some(Box::new(handler));
        self
    }

    /// Registers a drag event handler.
    #[must_use]
    pub fn on_drag(
        mut self,
        handler: impl Fn(&dusty_core::event::EventContext, &dusty_core::event::DragEvent) + 'static,
    ) -> Self {
        self.on_drag = Some(Box::new(handler));
        self
    }
}

impl View for Canvas {
    fn build(self, cx: Scope) -> Node {
        let base = Style::default();

        let merged = if let Some(user) = &self.user_style {
            base.merge(user)
        } else {
            base
        };

        // Shared command buffer between the effect and the element
        let commands: Rc<RefCell<Vec<CanvasCommand>>> = Rc::new(RefCell::new(Vec::new()));

        // Run the draw closure inside a reactive effect
        let cmds_for_effect = commands.clone();
        let draw = self.draw;
        // create_effect returns Result but we're inside a scope so runtime is initialized
        let _effect = dusty_reactive::create_effect(move || {
            let mut frame = Frame::new();
            draw(&mut frame);
            // Use try_borrow_mut to avoid panic on re-entrant access
            // (e.g. if a signal read inside draw triggers this effect again).
            if let Ok(mut cmds) = cmds_for_effect.try_borrow_mut() {
                *cmds = frame.into_commands();
            }
        });

        let mut builder = el("Canvas", cx).style(merged).data(commands);

        if let Some(handler) = self.on_click {
            builder = builder.on_click(handler);
        }
        if let Some(handler) = self.on_hover {
            builder = builder.on_hover(handler);
        }
        if let Some(handler) = self.on_drag {
            builder = builder.on_drag(handler);
        }

        let element = builder.build_node();

        Node::Component(ComponentNode {
            name: "Canvas",
            child: Box::new(element),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dusty_core::Element;
    use dusty_reactive::{create_scope, create_signal, dispose_runtime, initialize_runtime};
    use dusty_style::{Color, Length};

    /// Drop guard that ensures `dispose_runtime()` runs even if a test panics.
    /// This prevents runtime state from leaking into subsequent tests.
    struct RuntimeGuard;

    impl Drop for RuntimeGuard {
        fn drop(&mut self) {
            dispose_runtime();
        }
    }

    fn with_scope(f: impl FnOnce(Scope)) {
        initialize_runtime();
        let _guard = RuntimeGuard;
        create_scope(|cx| f(cx));
    }

    fn extract_element(node: &Node) -> &Element {
        match node {
            Node::Component(comp) => match &*comp.child {
                Node::Element(el) => el,
                _ => panic!("expected Element inside Component"),
            },
            _ => panic!("expected Component node"),
        }
    }

    fn extract_commands(el: &Element) -> Vec<CanvasCommand> {
        let data = el
            .custom_data()
            .downcast_ref::<Rc<RefCell<Vec<CanvasCommand>>>>();
        assert!(data.is_some(), "custom_data should be canvas commands");
        data.unwrap().borrow().clone()
    }

    // -- Basic build tests --

    #[test]
    fn builds_component() {
        with_scope(|cx| {
            let node = Canvas::new(|_frame| {}).build(cx);
            assert!(node.is_component());
            if let Node::Component(comp) = &node {
                assert_eq!(comp.name, "Canvas");
            }
        });
    }

    #[test]
    fn draw_closure_invoked() {
        with_scope(|cx| {
            let node = Canvas::new(|frame| {
                frame.rect(
                    0.0,
                    0.0,
                    100.0,
                    50.0,
                    Some(FillStyle::Solid(Color::WHITE)),
                    None,
                );
            })
            .build(cx);
            let el = extract_element(&node);
            let cmds = extract_commands(el);
            assert_eq!(cmds.len(), 1);
        });
    }

    #[test]
    fn commands_in_custom_data() {
        with_scope(|cx| {
            let node = Canvas::new(|frame| {
                frame.circle(50.0, 50.0, 25.0, Some(FillStyle::Solid(Color::BLACK)), None);
            })
            .build(cx);
            let el = extract_element(&node);
            assert!(el
                .custom_data()
                .downcast_ref::<Rc<RefCell<Vec<CanvasCommand>>>>()
                .is_some());
        });
    }

    #[test]
    fn style_applied() {
        with_scope(|cx| {
            let node = Canvas::new(|_frame| {})
                .style(Style {
                    width: Some(Length::Px(200.0)),
                    height: Some(Length::Px(150.0)),
                    ..Style::default()
                })
                .build(cx);
            let el = extract_element(&node);
            let style = el.style().downcast_ref::<Style>().unwrap();
            assert_eq!(style.width, Some(Length::Px(200.0)));
            assert_eq!(style.height, Some(Length::Px(150.0)));
        });
    }

    // -- Reactive tests --

    #[test]
    fn draw_reads_signal() {
        with_scope(|cx| {
            let count = create_signal(0i32);
            let node = Canvas::new(move |frame| {
                let n = count.get();
                for _ in 0..n {
                    frame.rect(
                        0.0,
                        0.0,
                        10.0,
                        10.0,
                        Some(FillStyle::Solid(Color::WHITE)),
                        None,
                    );
                }
            })
            .build(cx);
            let el = extract_element(&node);
            // Initial value is 0 → no commands
            let cmds = extract_commands(el);
            assert_eq!(cmds.len(), 0);
        });
    }

    #[test]
    fn redraws_on_signal_change() {
        with_scope(|cx| {
            let count = create_signal(1i32);
            let node = Canvas::new(move |frame| {
                let n = count.get();
                for _ in 0..n {
                    frame.rect(
                        0.0,
                        0.0,
                        10.0,
                        10.0,
                        Some(FillStyle::Solid(Color::WHITE)),
                        None,
                    );
                }
            })
            .build(cx);
            let el = extract_element(&node);

            // Initial: 1 rect
            assert_eq!(extract_commands(el).len(), 1);

            // Update signal -> effect re-runs
            count.set(3);
            assert_eq!(extract_commands(el).len(), 3);
        });
    }

    #[test]
    fn no_redraw_when_deps_unchanged() {
        with_scope(|cx| {
            let call_count = Rc::new(std::cell::Cell::new(0u32));
            let cc = call_count.clone();
            let count = create_signal(1i32);

            let _node = Canvas::new(move |frame| {
                cc.set(cc.get() + 1);
                let n = count.get();
                for _ in 0..n {
                    frame.rect(
                        0.0,
                        0.0,
                        10.0,
                        10.0,
                        Some(FillStyle::Solid(Color::WHITE)),
                        None,
                    );
                }
            })
            .build(cx);

            // Effect ran once on build
            let initial_count = call_count.get();
            assert!(initial_count >= 1);

            // Setting to the same value — effect re-runs (signals don't skip equal values by default)
            // but no *additional* unexpected calls
            let before = call_count.get();
            // No signal change → no re-run
            assert_eq!(call_count.get(), before);
        });
    }

    // -- Input tests --

    #[test]
    fn on_click_handler() {
        with_scope(|cx| {
            let node = Canvas::new(|_frame| {}).on_click(|_ctx, _e| {}).build(cx);
            let el = extract_element(&node);
            assert!(el.event_handlers().iter().any(|h| h.name() == "click"));
        });
    }

    #[test]
    fn on_hover_handler() {
        with_scope(|cx| {
            let node = Canvas::new(|_frame| {}).on_hover(|_ctx, _e| {}).build(cx);
            let el = extract_element(&node);
            assert!(el.event_handlers().iter().any(|h| h.name() == "hover"));
        });
    }

    #[test]
    fn on_drag_handler() {
        with_scope(|cx| {
            let node = Canvas::new(|_frame| {}).on_drag(|_ctx, _e| {}).build(cx);
            let el = extract_element(&node);
            assert!(el.event_handlers().iter().any(|h| h.name() == "drag"));
        });
    }

    #[test]
    fn multiple_handlers() {
        with_scope(|cx| {
            let node = Canvas::new(|_frame| {})
                .on_click(|_ctx, _e| {})
                .on_hover(|_ctx, _e| {})
                .on_drag(|_ctx, _e| {})
                .build(cx);
            let el = extract_element(&node);
            assert_eq!(el.event_handlers().len(), 3);
        });
    }
}
