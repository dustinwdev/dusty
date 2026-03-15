use dusty_core::el;
use dusty_core::node::{ComponentNode, Node};
use dusty_core::view::View;
use dusty_reactive::Scope;
use dusty_style::Style;

/// How the image should be sized within its container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SizingMode {
    /// Scale to cover the container, cropping if needed (default).
    #[default]
    Cover,
    /// Scale to fit entirely within the container, letterboxing if needed.
    Contain,
    /// Stretch to fill the container exactly.
    Fill,
}

/// An image display widget.
///
/// # Example
///
/// ```
/// use dusty_reactive::{initialize_runtime, create_scope, dispose_runtime};
/// use dusty_widgets::Image;
/// use dusty_core::view::View;
///
/// initialize_runtime();
/// create_scope(|cx| {
///     let node = Image::new("photo.png").build(cx);
///     assert!(node.is_component());
/// }).unwrap();
/// dispose_runtime();
/// ```
pub struct Image {
    src: String,
    sizing: SizingMode,
    alt: Option<String>,
    user_style: Option<Style>,
}

impl Image {
    /// Creates a new image widget with the given source path.
    #[must_use]
    pub fn new(src: impl Into<String>) -> Self {
        Self {
            src: src.into(),
            sizing: SizingMode::default(),
            alt: None,
            user_style: None,
        }
    }

    /// Sets the sizing mode for the image.
    #[must_use]
    pub const fn sizing(mut self, mode: SizingMode) -> Self {
        self.sizing = mode;
        self
    }

    /// Sets the alt text for accessibility.
    #[must_use]
    pub fn alt(mut self, text: impl Into<String>) -> Self {
        self.alt = Some(text.into());
        self
    }

    /// Merges user styles on top of image defaults.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.user_style = Some(style);
        self
    }
}

impl View for Image {
    fn build(self, cx: Scope) -> Node {
        let base = Style::default();

        let merged = if let Some(user) = &self.user_style {
            base.merge(user)
        } else {
            base
        };

        let sizing_str = match self.sizing {
            SizingMode::Cover => "cover",
            SizingMode::Contain => "contain",
            SizingMode::Fill => "fill",
        };

        let mut builder = el("Image", cx)
            .attr("src", self.src)
            .attr("sizing_mode", sizing_str)
            .style(merged);

        if let Some(alt_text) = self.alt {
            builder = builder.attr("alt", alt_text);
        }

        let element = builder.build_node();

        Node::Component(ComponentNode {
            name: "Image",
            child: Box::new(element),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dusty_core::{AttributeValue, Element};
    use dusty_reactive::{create_scope, dispose_runtime, initialize_runtime};

    fn with_scope(f: impl FnOnce(Scope)) {
        initialize_runtime();
        create_scope(|cx| f(cx)).unwrap();
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
    fn src_attribute() {
        with_scope(|cx| {
            let node = Image::new("photo.png").build(cx);
            let el = extract_element(&node);
            assert_eq!(
                el.attr("src"),
                Some(&AttributeValue::String("photo.png".into()))
            );
        });
    }

    #[test]
    fn default_cover() {
        with_scope(|cx| {
            let node = Image::new("test.png").build(cx);
            let el = extract_element(&node);
            assert_eq!(
                el.attr("sizing_mode"),
                Some(&AttributeValue::String("cover".into()))
            );
        });
    }

    #[test]
    fn sizing_modes() {
        with_scope(|cx| {
            let contain = Image::new("a.png").sizing(SizingMode::Contain).build(cx);
            let el = extract_element(&contain);
            assert_eq!(
                el.attr("sizing_mode"),
                Some(&AttributeValue::String("contain".into()))
            );

            let fill = Image::new("b.png").sizing(SizingMode::Fill).build(cx);
            let el = extract_element(&fill);
            assert_eq!(
                el.attr("sizing_mode"),
                Some(&AttributeValue::String("fill".into()))
            );
        });
    }

    #[test]
    fn alt_text() {
        with_scope(|cx| {
            let node = Image::new("photo.png").alt("A sunset").build(cx);
            let el = extract_element(&node);
            assert_eq!(
                el.attr("alt"),
                Some(&AttributeValue::String("A sunset".into()))
            );
        });
    }

    #[test]
    fn style_merges() {
        with_scope(|cx| {
            let node = Image::new("photo.png")
                .style(Style {
                    width: Some(200.0),
                    height: Some(150.0),
                    ..Style::default()
                })
                .build(cx);
            let el = extract_element(&node);
            let style = el.style().downcast_ref::<Style>().unwrap();
            assert_eq!(style.width, Some(200.0));
            assert_eq!(style.height, Some(150.0));
        });
    }
}
