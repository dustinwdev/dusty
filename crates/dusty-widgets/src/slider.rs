use dusty_core::el;
use dusty_core::event::{ClickEvent, DragEvent, EventContext};
use dusty_core::node::{ComponentNode, Node};
use dusty_core::view::View;
use dusty_reactive::{create_signal, Scope, Signal};
use dusty_style::theme::use_theme;
use dusty_style::{Corners, Length, Style};

type SliderChangeCallback = std::rc::Rc<Option<Box<dyn Fn(f64)>>>;

/// Source of truth for the slider value.
pub enum SliderSource {
    /// Widget manages its own signal internally.
    Uncontrolled(f64),
    /// Caller provides the signal.
    Controlled(Signal<f64>),
}

/// A range slider widget.
///
/// # Example
///
/// ```
/// use dusty_reactive::{initialize_runtime, create_scope, dispose_runtime};
/// use dusty_widgets::Slider;
/// use dusty_core::view::View;
///
/// initialize_runtime();
/// create_scope(|cx| {
///     let node = Slider::new().build(cx);
///     assert!(node.is_component());
/// });
/// dispose_runtime();
/// ```
pub struct Slider {
    source: SliderSource,
    min: f64,
    max: f64,
    step: Option<f64>,
    track_width: f32,
    disabled: bool,
    user_style: Option<Style>,
    on_change: Option<Box<dyn Fn(f64)>>,
}

impl Slider {
    /// Creates a slider with default range 0.0–100.0.
    #[must_use]
    pub fn new() -> Self {
        Self {
            source: SliderSource::Uncontrolled(0.0),
            min: 0.0,
            max: 100.0,
            step: None,
            track_width: 200.0,
            disabled: false,
            user_style: None,
            on_change: None,
        }
    }

    /// Sets the initial value (uncontrolled mode).
    #[must_use]
    pub const fn value(mut self, value: f64) -> Self {
        self.source = SliderSource::Uncontrolled(value);
        self
    }

    /// Uses an external signal as the source of truth (controlled mode).
    #[must_use]
    pub const fn controlled(mut self, signal: Signal<f64>) -> Self {
        self.source = SliderSource::Controlled(signal);
        self
    }

    /// Sets the minimum value.
    #[must_use]
    pub const fn min(mut self, min: f64) -> Self {
        self.min = min;
        self
    }

    /// Sets the maximum value.
    #[must_use]
    pub const fn max(mut self, max: f64) -> Self {
        self.max = max;
        self
    }

    /// Sets the step increment.
    #[must_use]
    pub const fn step(mut self, step: f64) -> Self {
        self.step = Some(step);
        self
    }

    /// Sets the track width used for drag/click fraction calculations.
    #[must_use]
    pub const fn track_width(mut self, width: f32) -> Self {
        self.track_width = width;
        self
    }

    /// Disables the slider, suppressing drag and click events.
    #[must_use]
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Merges user styles on top of slider defaults.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.user_style = Some(style);
        self
    }

    /// Registers a change handler called with the new value.
    #[must_use]
    pub fn on_change(mut self, handler: impl Fn(f64) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }
}

impl Default for Slider {
    fn default() -> Self {
        Self::new()
    }
}

/// Snap a value to the nearest step.
fn snap(val: f64, min: f64, step: f64) -> f64 {
    ((val - min) / step).round().mul_add(step, min)
}

/// Clamp value to [min, max].
fn clamp(val: f64, min: f64, max: f64) -> f64 {
    if val < min {
        min
    } else if val > max {
        max
    } else {
        val
    }
}

impl View for Slider {
    #[allow(clippy::unreadable_literal)]
    fn build(self, cx: Scope) -> Node {
        let theme = use_theme();
        let base = Style {
            height: Some(Length::Px(24.0)),
            flex_grow: Some(1.0),
            border_radius: Corners::all(4.0),
            background: Some(
                theme
                    .secondary
                    .get(200)
                    .unwrap_or_else(|| dusty_style::Color::hex(0xe2e8f0)),
            ),
            ..Style::default()
        };

        let merged = if let Some(user) = &self.user_style {
            base.merge(user)
        } else {
            base
        };

        let merged = if self.disabled {
            merged.merge(&Style {
                opacity: Some(0.5),
                ..Style::default()
            })
        } else {
            merged
        };

        let signal = match self.source {
            SliderSource::Uncontrolled(initial) => create_signal(initial),
            SliderSource::Controlled(sig) => sig,
        };

        let mut builder = el("Slider", cx)
            .attr("value", signal.get())
            .attr("min", self.min)
            .attr("max", self.max)
            .attr("disabled", self.disabled)
            .style(merged)
            .data(signal);

        if let Some(step_val) = self.step {
            builder = builder.attr("step", step_val);
        }

        if !self.disabled {
            let min = self.min;
            let max = self.max;
            let step_opt = self.step;
            let on_change = self.on_change;
            let track_w = f64::from(self.track_width);

            let sig_drag = signal;
            let on_change_drag: SliderChangeCallback = std::rc::Rc::new(on_change);
            let on_change_click = on_change_drag.clone();

            builder = builder.on_drag(move |_ctx: &EventContext, e: &DragEvent| {
                let range = max - min;
                if range <= 0.0 {
                    return;
                }
                let current = sig_drag.get();
                let delta_frac = e.delta_x / track_w;
                let mut new_val = delta_frac.mul_add(range, current);
                if let Some(s) = step_opt {
                    new_val = snap(new_val, min, s);
                }
                new_val = clamp(new_val, min, max);
                sig_drag.set_if_changed(new_val);
                if let Some(ref cb) = *on_change_drag {
                    cb(new_val);
                }
            });

            let sig_click = signal;
            builder = builder.on_click(move |_ctx: &EventContext, e: &ClickEvent| {
                let range = max - min;
                if range <= 0.0 {
                    return;
                }
                let frac = e.x / track_w;
                let mut new_val = frac.mul_add(range, min);
                if let Some(s) = step_opt {
                    new_val = snap(new_val, min, s);
                }
                new_val = clamp(new_val, min, max);
                sig_click.set_if_changed(new_val);
                if let Some(ref cb) = *on_change_click {
                    cb(new_val);
                }
            });
        }

        let element = builder.build_node();

        Node::Component(ComponentNode {
            name: "Slider",
            child: Box::new(element),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dusty_core::{AttributeValue, Element};
    use dusty_reactive::{create_scope, create_signal, dispose_runtime, initialize_runtime};

    fn with_scope(f: impl FnOnce(Scope)) {
        initialize_runtime();
        create_scope(|cx| f(cx));
        dispose_runtime();
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

    #[test]
    fn builds_component() {
        with_scope(|cx| {
            let node = Slider::new().build(cx);
            assert!(node.is_component());
            if let Node::Component(comp) = &node {
                assert_eq!(comp.name, "Slider");
            }
        });
    }

    #[test]
    fn default_value_and_range() {
        with_scope(|cx| {
            let node = Slider::new().build(cx);
            let el = extract_element(&node);
            assert_eq!(el.attr("value"), Some(&AttributeValue::Float(0.0)));
            assert_eq!(el.attr("min"), Some(&AttributeValue::Float(0.0)));
            assert_eq!(el.attr("max"), Some(&AttributeValue::Float(100.0)));
        });
    }

    #[test]
    fn custom_range_attrs() {
        with_scope(|cx| {
            let node = Slider::new().min(10.0).max(50.0).value(25.0).build(cx);
            let el = extract_element(&node);
            assert_eq!(el.attr("min"), Some(&AttributeValue::Float(10.0)));
            assert_eq!(el.attr("max"), Some(&AttributeValue::Float(50.0)));
            assert_eq!(el.attr("value"), Some(&AttributeValue::Float(25.0)));
        });
    }

    #[test]
    fn step_attr() {
        with_scope(|cx| {
            let node = Slider::new().step(5.0).build(cx);
            let el = extract_element(&node);
            assert_eq!(el.attr("step"), Some(&AttributeValue::Float(5.0)));
        });
    }

    #[test]
    fn step_snapping_logic() {
        assert_eq!(snap(12.0, 0.0, 5.0), 10.0);
        assert_eq!(snap(13.0, 0.0, 5.0), 15.0);
        assert_eq!(snap(2.5, 0.0, 5.0), 5.0);
        assert_eq!(snap(7.0, 0.0, 5.0), 5.0);
        assert_eq!(snap(8.0, 0.0, 5.0), 10.0);
        // With non-zero min
        assert_eq!(snap(14.0, 10.0, 5.0), 15.0);
        assert_eq!(snap(11.0, 10.0, 5.0), 10.0);
    }

    #[test]
    fn clamp_to_range() {
        assert_eq!(clamp(-5.0, 0.0, 100.0), 0.0);
        assert_eq!(clamp(150.0, 0.0, 100.0), 100.0);
        assert_eq!(clamp(50.0, 0.0, 100.0), 50.0);
    }

    #[test]
    fn disabled_suppresses_handlers() {
        with_scope(|cx| {
            let node = Slider::new().disabled(true).build(cx);
            let el = extract_element(&node);
            assert!(el.event_handlers().is_empty());
        });
    }

    #[test]
    fn drag_and_click_handlers_registered() {
        with_scope(|cx| {
            let node = Slider::new().build(cx);
            let el = extract_element(&node);
            assert!(el.event_handlers().iter().any(|h| h.name() == "drag"));
            assert!(el.event_handlers().iter().any(|h| h.name() == "click"));
        });
    }

    #[test]
    fn controlled_reads_signal() {
        with_scope(|cx| {
            let sig = create_signal(42.0);
            let node = Slider::new().controlled(sig).build(cx);
            let el = extract_element(&node);
            assert_eq!(el.attr("value"), Some(&AttributeValue::Float(42.0)));
        });
    }

    #[test]
    fn style_merges() {
        with_scope(|cx| {
            let node = Slider::new()
                .style(Style {
                    width: Some(Length::Px(300.0)),
                    ..Style::default()
                })
                .build(cx);
            let el = extract_element(&node);
            let style = el.style().downcast_ref::<Style>().unwrap();
            assert_eq!(style.width, Some(Length::Px(300.0)));
            // Base height still present
            assert_eq!(style.height, Some(Length::Px(24.0)));
        });
    }

    #[test]
    fn stores_signal_in_custom_data() {
        with_scope(|cx| {
            let node = Slider::new().build(cx);
            let el = extract_element(&node);
            assert!(el.custom_data().downcast_ref::<Signal<f64>>().is_some());
        });
    }
}
