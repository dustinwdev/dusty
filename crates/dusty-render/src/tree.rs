//! Render tree walker — converts a `Node` tree + layout into draw commands.
//!
//! Traverses the node tree in pre-order depth-first order (mirroring
//! `TreeBuilder::build_node` in `dusty-layout/src/tree.rs`) and emits
//! [`DrawCommand`]s for each visible node.
//!
//! # Examples
//!
//! ```no_run
//! use dusty_render::tree::walk_tree;
//! # // walk_tree requires a full node tree, layout, text system, etc.
//! ```

use std::cell::RefCell;
use std::rc::Rc;

use dusty_core::Node;
use dusty_layout::{LayoutNodeId, LayoutResult};
use dusty_reactive::Signal;
use dusty_style::{Color, FontStyle, InteractionState, Overflow, Style};
use dusty_text::{GlyphRasterizer, TextLayout, TextSystem};

use crate::command::CommandEncoder;
use crate::glyph_cache::GlyphCache;
use crate::image_cache::ImageCache;
use crate::primitive::{
    DrawCommand, DrawPrimitive, ImageId, ImagePrimitive, PrimitiveFlags, Rect, TextGlyph,
    TextPrimitive,
};

/// Default foreground (text) color when none is specified.
const DEFAULT_FOREGROUND: Color = Color::BLACK;

/// Identifies which elements are in hover, focus, or active states
/// so the render tree can resolve style overrides.
///
/// Pass this to [`walk_tree_interactive`] to enable visual feedback
/// for hover, focus, and active states.
#[derive(Debug, Clone, Default)]
pub struct InteractionContext {
    /// The layout node ID of the element currently under the cursor.
    pub hovered_id: Option<LayoutNodeId>,
    /// The layout node ID of the element that currently has focus.
    pub focused_id: Option<LayoutNodeId>,
    /// Whether the primary mouse button is currently pressed.
    pub active: bool,
}

/// Walks the node tree and emits draw commands.
///
/// The traversal mirrors `TreeBuilder::build_node` in `dusty-layout`:
/// - `Element` and `Text` nodes consume a `LayoutNodeId` (incrementing).
/// - `Fragment` and `Component` nodes are transparent (no ID consumed).
///
/// # Parameters
///
/// - `root` — the root of the node tree.
/// - `layout` — the computed layout result.
/// - `text_system` — text system for creating `TextLayout`s.
/// - `glyph_cache` — cache for rasterized glyph bitmaps.
/// - `rasterizer` — glyph rasterizer.
/// - `scale_factor` — display scale factor (1.0 for 1x, 2.0 for Retina).
pub fn walk_tree(
    root: &Node,
    layout: &LayoutResult,
    text_system: &TextSystem,
    glyph_cache: &mut GlyphCache,
    rasterizer: &mut GlyphRasterizer,
    scale_factor: f32,
) -> Vec<DrawCommand> {
    walk_tree_with_images(
        root,
        layout,
        text_system,
        glyph_cache,
        rasterizer,
        scale_factor,
        None,
    )
}

/// Walks the node tree with an optional image cache for Image element support.
pub fn walk_tree_with_images(
    root: &Node,
    layout: &LayoutResult,
    text_system: &TextSystem,
    glyph_cache: &mut GlyphCache,
    rasterizer: &mut GlyphRasterizer,
    scale_factor: f32,
    image_cache: Option<&ImageCache>,
) -> Vec<DrawCommand> {
    walk_tree_interactive(
        root,
        layout,
        text_system,
        glyph_cache,
        rasterizer,
        scale_factor,
        image_cache,
        InteractionContext::default(),
    )
}

/// Walks the node tree with interaction state for hover/focus/active style resolution.
#[allow(clippy::too_many_arguments)]
pub fn walk_tree_interactive(
    root: &Node,
    layout: &LayoutResult,
    text_system: &TextSystem,
    glyph_cache: &mut GlyphCache,
    rasterizer: &mut GlyphRasterizer,
    scale_factor: f32,
    image_cache: Option<&ImageCache>,
    interaction: InteractionContext,
) -> Vec<DrawCommand> {
    let mut ctx = WalkContext {
        commands: Vec::new(),
        encoder: CommandEncoder::with_scale_factor(scale_factor),
        next_id: 0,
        text_system,
        glyph_cache,
        rasterizer,
        scale_factor,
        image_cache,
        interaction,
        available_width: None,
        scroll_offset: (0.0, 0.0),
    };

    ctx.walk_node(root, layout, &FontStyle::default(), DEFAULT_FOREGROUND, 1.0);
    ctx.commands
}

struct WalkContext<'a> {
    commands: Vec<DrawCommand>,
    encoder: CommandEncoder,
    next_id: usize,
    text_system: &'a TextSystem,
    glyph_cache: &'a mut GlyphCache,
    rasterizer: &'a mut GlyphRasterizer,
    scale_factor: f32,
    image_cache: Option<&'a ImageCache>,
    interaction: InteractionContext,
    /// Available width for text wrapping, inherited from the nearest
    /// ancestor element. Text nodes use this instead of their own
    /// computed width to avoid rounding-induced re-wrapping.
    available_width: Option<f32>,
    /// Accumulated scroll offset from ancestor scroll containers.
    /// Applied to child positions during rendering.
    scroll_offset: (f32, f32),
}

impl WalkContext<'_> {
    fn alloc_id(&mut self) -> LayoutNodeId {
        let id = LayoutNodeId(self.next_id);
        self.next_id += 1;
        id
    }

    fn walk_node(
        &mut self,
        node: &Node,
        layout: &LayoutResult,
        inherited_font: &FontStyle,
        inherited_fg: Color,
        inherited_opacity: f32,
    ) {
        match node {
            Node::Element(el) => {
                self.walk_element(el, layout, inherited_font, inherited_fg, inherited_opacity);
            }
            Node::Text(text_node) => {
                self.walk_text(
                    text_node,
                    layout,
                    inherited_font,
                    inherited_fg,
                    inherited_opacity,
                );
            }
            Node::Fragment(children) => {
                for child in children {
                    self.walk_node(
                        child,
                        layout,
                        inherited_font,
                        inherited_fg,
                        inherited_opacity,
                    );
                }
            }
            Node::Component(comp) => {
                self.walk_node(
                    &comp.child,
                    layout,
                    inherited_font,
                    inherited_fg,
                    inherited_opacity,
                );
            }
            Node::Dynamic(dn) => {
                let resolved = dn.current_node();
                self.walk_node(
                    &resolved,
                    layout,
                    inherited_font,
                    inherited_fg,
                    inherited_opacity,
                );
            }
        }
    }

    fn walk_element(
        &mut self,
        el: &dusty_core::Element,
        layout: &LayoutResult,
        inherited_font: &FontStyle,
        inherited_fg: Color,
        inherited_opacity: f32,
    ) {
        let layout_id = self.alloc_id();
        let Some(layout_rect) = layout.get(layout_id) else {
            return;
        };

        // Apply accumulated scroll offset from ancestor scroll containers
        let adjusted = dusty_layout::Rect {
            x: layout_rect.x - self.scroll_offset.0,
            y: layout_rect.y - self.scroll_offset.1,
            width: layout_rect.width,
            height: layout_rect.height,
        };
        let render_rect = scale_rect(to_render_rect(&adjusted), self.scale_factor);

        // Downcast style
        let raw_style = if el.style().is::<()>() {
            Style::default()
        } else {
            el.style()
                .downcast_ref::<Style>()
                .cloned()
                .unwrap_or_default()
        };

        // Resolve interaction state for rendering.
        let is_disabled = el
            .attr("disabled")
            .and_then(|v| {
                if let dusty_core::AttributeValue::Bool(b) = v {
                    Some(*b)
                } else {
                    None
                }
            })
            .unwrap_or(false);
        let interaction_state = InteractionState {
            hovered: self.interaction.hovered_id == Some(layout_id),
            focused: self.interaction.focused_id == Some(layout_id),
            active: self.interaction.active && self.interaction.focused_id == Some(layout_id),
            disabled: is_disabled,
        };
        let dusty_style = raw_style.resolve(&interaction_state);

        // Merge font for children
        let child_font = inherited_font.merge(&dusty_style.font);
        let child_fg = dusty_style.foreground.unwrap_or(inherited_fg);

        // Maybe push clip
        let has_clip = self.encoder.maybe_push_clip(&dusty_style, &render_rect);

        // Encode element visuals (shadows, rect)
        let element_cmds = self.encoder.encode_element(&dusty_style, &render_rect);
        self.commands.extend(element_cmds);

        // Image element — emit DrawCommand::Image
        if el.name() == "Image" {
            self.walk_image_element(el, &render_rect, &dusty_style);
        }

        // Canvas element — emit draw commands from custom_data
        if el.name() == "Canvas" {
            self.walk_canvas_element(el, &render_rect, &dusty_style);
        }

        // Detect scroll container and read scroll offset
        #[allow(clippy::cast_possible_truncation)]
        let scroll_offset_delta = if matches!(
            dusty_style.overflow,
            Some(Overflow::Scroll | Overflow::Auto)
        ) {
            el.custom_data()
                .downcast_ref::<Signal<(f64, f64)>>()
                .map_or((0.0, 0.0), |sig| {
                    let (sx, sy) = sig.get();
                    (sx as f32, sy as f32)
                })
        } else {
            (0.0, 0.0)
        };

        // Recurse children — pass this element's logical width so text
        // nodes can use it as their wrapping constraint instead of their
        // own intrinsic width (which causes rounding-induced re-wrapping).
        let parent_width = self.available_width;
        let parent_scroll = self.scroll_offset;
        let child_opacity = dusty_style.opacity.unwrap_or(1.0) * inherited_opacity;
        self.available_width = Some(layout_rect.width);
        self.scroll_offset = (
            self.scroll_offset.0 + scroll_offset_delta.0,
            self.scroll_offset.1 + scroll_offset_delta.1,
        );
        for child in el.children() {
            self.walk_node(child, layout, &child_font, child_fg, child_opacity);
        }
        self.available_width = parent_width;
        self.scroll_offset = parent_scroll;

        // Pop clip if we pushed one
        if has_clip {
            self.encoder.pop_clip();
        }
    }

    #[allow(clippy::cast_precision_loss)]
    fn walk_image_element(&mut self, el: &dusty_core::Element, render_rect: &Rect, style: &Style) {
        // Read attributes
        let src = el.attr("src").and_then(|v| match v {
            dusty_core::AttributeValue::String(s) => Some(s.as_str()),
            _ => None,
        });
        let sizing_mode = el
            .attr("sizing_mode")
            .and_then(|v| match v {
                dusty_core::AttributeValue::String(s) => Some(s.as_str()),
                _ => None,
            })
            .unwrap_or("cover");

        let Some(src) = src else {
            return;
        };

        // Look up image in cache to get intrinsic dimensions
        let image_source = crate::image_cache::ImageSource::Path(std::path::PathBuf::from(src));
        let (intrinsic_w, intrinsic_h) = if let Some(cache) = self.image_cache {
            if let Some(&id) = cache.source_to_id().get(&image_source) {
                if let Some(decoded) = cache.get(id) {
                    (decoded.width as f32, decoded.height as f32)
                } else {
                    return;
                }
            } else {
                // Image not loaded — use fill UV as fallback
                (render_rect.width, render_rect.height)
            }
        } else {
            // No image cache — use fill UV
            (render_rect.width, render_rect.height)
        };

        let uv = compute_uv(sizing_mode, render_rect, intrinsic_w, intrinsic_h);
        let opacity = style.opacity.unwrap_or(1.0);
        let clip_rect = self.encoder.clip_stack().current().map(|c| c.rect);

        // Use a deterministic texture ID from the source path hash
        let texture_id = ImageId(hash_string(src));

        self.commands.push(DrawCommand::Image(ImagePrimitive {
            rect: *render_rect,
            texture_id,
            uv,
            opacity,
            clip_rect,
        }));
    }

    fn walk_canvas_element(&mut self, el: &dusty_core::Element, render_rect: &Rect, style: &Style) {
        type CanvasCommands = Rc<RefCell<Vec<dusty_widgets_canvas::CanvasCommand>>>;

        let Some(cmds_rc) = el.custom_data().downcast_ref::<CanvasCommands>() else {
            return;
        };

        let cmds = cmds_rc.borrow();
        if cmds.is_empty() {
            return;
        }

        let opacity = style.opacity.unwrap_or(1.0);
        let clip_rect = self.encoder.clip_stack().current().map(|c| c.rect);
        let ox = render_rect.x;
        let oy = render_rect.y;

        for cmd in cmds.iter() {
            self.emit_canvas_command(cmd, ox, oy, opacity, clip_rect);
        }
    }

    fn emit_canvas_command(
        &mut self,
        cmd: &dusty_widgets_canvas::CanvasCommand,
        ox: f32,
        oy: f32,
        opacity: f32,
        clip_rect: Option<Rect>,
    ) {
        use dusty_widgets_canvas::CanvasCommand as CC;

        // Canvas commands are in logical pixels; scale to physical
        let s = self.scale_factor;

        match cmd {
            CC::Rect {
                x,
                y,
                width,
                height,
                fill: Some(dusty_widgets_canvas::FillStyle::Solid(color)),
                ..
            } => {
                self.commands.push(DrawCommand::Rect(DrawPrimitive {
                    rect: Rect {
                        x: ox + x * s,
                        y: oy + y * s,
                        width: *width * s,
                        height: *height * s,
                    },
                    radii: [0.0; 4],
                    fill_color: *color,
                    border_color: Color::TRANSPARENT,
                    border_widths: [0.0; 4],
                    opacity,
                    clip_rect,
                    clip_radii: [0.0; 4],
                    flags: PrimitiveFlags::empty(),
                    gradient: None,
                }));
            }
            CC::RoundRect {
                x,
                y,
                width,
                height,
                radius,
                fill: Some(dusty_widgets_canvas::FillStyle::Solid(color)),
                ..
            } => {
                let r = *radius * s;
                self.commands.push(DrawCommand::Rect(DrawPrimitive {
                    rect: Rect {
                        x: ox + x * s,
                        y: oy + y * s,
                        width: *width * s,
                        height: *height * s,
                    },
                    radii: [r; 4],
                    fill_color: *color,
                    border_color: Color::TRANSPARENT,
                    border_widths: [0.0; 4],
                    opacity,
                    clip_rect,
                    clip_radii: [0.0; 4],
                    flags: PrimitiveFlags::ROUNDED,
                    gradient: None,
                }));
            }
            CC::Circle {
                cx,
                cy,
                radius,
                fill: Some(dusty_widgets_canvas::FillStyle::Solid(color)),
                ..
            } => {
                let r = *radius * s;
                self.commands.push(DrawCommand::Rect(DrawPrimitive {
                    rect: Rect {
                        x: (*cx - *radius).mul_add(s, ox),
                        y: (*cy - *radius).mul_add(s, oy),
                        width: r * 2.0,
                        height: r * 2.0,
                    },
                    radii: [r; 4],
                    fill_color: *color,
                    border_color: Color::TRANSPARENT,
                    border_widths: [0.0; 4],
                    opacity,
                    clip_rect,
                    clip_radii: [0.0; 4],
                    flags: PrimitiveFlags::ROUNDED,
                    gradient: None,
                }));
            }
            // Path commands, transforms, clips, text, image — deferred to later phases
            _ => {}
        }
    }

    #[allow(clippy::cast_precision_loss)]
    fn walk_text(
        &mut self,
        text_node: &dusty_core::TextNode,
        layout: &LayoutResult,
        inherited_font: &FontStyle,
        inherited_fg: Color,
        inherited_opacity: f32,
    ) {
        let layout_id = self.alloc_id();
        let Some(layout_rect) = layout.get(layout_id) else {
            return;
        };

        // Apply accumulated scroll offset from ancestor scroll containers
        let adjusted = dusty_layout::Rect {
            x: layout_rect.x - self.scroll_offset.0,
            y: layout_rect.y - self.scroll_offset.1,
            width: layout_rect.width,
            height: layout_rect.height,
        };
        let logical_rect = to_render_rect(&adjusted);
        let text = text_node.current_text();

        if text.is_empty() {
            return;
        }

        // Use the parent element's width for text wrapping, not the text
        // node's own computed width. The layout system measures text with
        // the full available width (e.g., 500px) and then assigns the text
        // node its intrinsic width (e.g., 121px). Re-constraining to 121px
        // during rendering causes wrapping due to FP precision differences.
        let max_width = self.available_width.filter(|&w| w > 0.0).or_else(|| {
            let w = logical_rect.width;
            if w > 0.0 {
                Some(w)
            } else {
                None
            }
        });
        let Ok(text_layout) = TextLayout::new(self.text_system, &text, inherited_font, max_width)
        else {
            eprintln!("dusty: FontSystem borrow conflict during text layout");
            return;
        };

        let (tw, th) = text_layout.size();
        if tw == 0.0 && th == 0.0 {
            eprintln!("dusty: text measurement returned (0, 0) for non-empty text, font system may be unavailable");
        }

        let clip_rect = self.encoder.clip_stack().current().map(|c| c.rect);
        let fg_color = [
            inherited_fg.r,
            inherited_fg.g,
            inherited_fg.b,
            inherited_fg.a,
        ];

        // Glyph positioning in physical pixels
        let scale = self.scale_factor;
        let phys_x = logical_rect.x * scale;
        let phys_y = logical_rect.y * scale;

        let mut glyphs = Vec::new();
        let Ok(mut font_system) = self.text_system.font_system_mut() else {
            eprintln!("dusty: FontSystem borrow conflict during glyph rasterization");
            return;
        };

        for run in text_layout.buffer().layout_runs() {
            for layout_glyph in run.glyphs {
                let physical = layout_glyph.physical((0., run.line_y), scale);

                let cached = self.glyph_cache.get_or_rasterize(
                    physical.cache_key,
                    self.rasterizer,
                    &mut font_system,
                );

                if let Some(cached) = cached {
                    let glyph_x = phys_x + physical.x as f32 + cached.offset[0] as f32;
                    let glyph_y = phys_y + physical.y as f32 - cached.offset[1] as f32;

                    glyphs.push(TextGlyph {
                        x: glyph_x,
                        y: glyph_y,
                        width: cached.size[0] as f32,
                        height: cached.size[1] as f32,
                        uv: cached.uv,
                        color: fg_color,
                        opacity: inherited_opacity,
                        clip_rect,
                    });
                }
            }
        }

        drop(font_system);

        if !glyphs.is_empty() {
            self.commands
                .push(DrawCommand::Text(TextPrimitive { glyphs }));
        }
    }
}

/// Converts a layout `Rect` to a render `Rect`.
const fn to_render_rect(layout_rect: &dusty_layout::Rect) -> Rect {
    Rect {
        x: layout_rect.x,
        y: layout_rect.y,
        width: layout_rect.width,
        height: layout_rect.height,
    }
}

/// Scales a rect from logical to physical pixels.
fn scale_rect(rect: Rect, scale: f32) -> Rect {
    Rect {
        x: rect.x * scale,
        y: rect.y * scale,
        width: rect.width * scale,
        height: rect.height * scale,
    }
}

/// Compute UV coordinates based on sizing mode.
///
/// - `"fill"` — stretch to fill: `[0, 0, 1, 1]`
/// - `"cover"` — crop to fill, center-aligned
/// - `"contain"` — letterbox, center-aligned
fn compute_uv(
    sizing_mode: &str,
    render_rect: &Rect,
    intrinsic_w: f32,
    intrinsic_h: f32,
) -> [f32; 4] {
    if intrinsic_w <= 0.0 || intrinsic_h <= 0.0 {
        return [0.0, 0.0, 1.0, 1.0];
    }

    match sizing_mode {
        "cover" => {
            let aspect_src = intrinsic_w / intrinsic_h;
            let aspect_dst = render_rect.width / render_rect.height;
            if aspect_dst > aspect_src {
                // Wider than image — crop vertically
                let visible_h = intrinsic_w / aspect_dst;
                let offset_v = (intrinsic_h - visible_h) / (2.0 * intrinsic_h);
                let end_v = 1.0 - offset_v;
                [0.0, offset_v, 1.0, end_v]
            } else {
                // Taller than image — crop horizontally
                let visible_w = intrinsic_h * aspect_dst;
                let offset_u = (intrinsic_w - visible_w) / (2.0 * intrinsic_w);
                let end_u = 1.0 - offset_u;
                [offset_u, 0.0, end_u, 1.0]
            }
        }
        // "fill", "contain", and unknown modes all use full UV range.
        // Contain adjusts the dest rect at a higher level, not UVs.
        _ => [0.0, 0.0, 1.0, 1.0],
    }
}

/// Hashes a string to produce a deterministic image ID using `SipHash`.
fn hash_string(s: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

/// Module alias for canvas command types used in the renderer.
mod dusty_widgets_canvas {
    pub use dusty_widgets::canvas::{CanvasCommand, FillStyle};
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::float_cmp,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation
)]
mod tests {
    use super::*;
    use dusty_core::{el, text, ComponentNode};
    use dusty_layout::compute_layout;
    use dusty_reactive::{create_scope, dispose_runtime, initialize_runtime};
    use dusty_style::{Color, Edges, Length, LengthPercent, Overflow};

    struct MockMeasure;
    impl dusty_layout::TextMeasure for MockMeasure {
        fn measure(&self, text: &str, max_width: Option<f32>, _font: &FontStyle) -> (f32, f32) {
            let char_width = 8.0;
            let line_height = 16.0;
            let text_width = text.len() as f32 * char_width;
            if let Some(max) = max_width {
                if text_width > max {
                    let chars_per_line = (max / char_width).floor() as usize;
                    if chars_per_line == 0 {
                        return (char_width, line_height);
                    }
                    let lines = text.len().div_ceil(chars_per_line);
                    return (max, lines as f32 * line_height);
                }
            }
            (text_width, line_height)
        }
    }

    fn with_scope(f: impl FnOnce(dusty_reactive::Scope)) {
        initialize_runtime();
        create_scope(f);
        dispose_runtime();
    }

    fn walk_with_mock(root: &Node, width: f32, height: f32) -> Vec<DrawCommand> {
        walk_with_mock_scale(root, width, height, 1.0)
    }

    fn walk_with_mock_scale(root: &Node, width: f32, height: f32, scale: f32) -> Vec<DrawCommand> {
        let layout = compute_layout(root, width, height, &MockMeasure).unwrap();
        let text_system = TextSystem::new();
        let mut glyph_cache = GlyphCache::new(256, 256);
        let mut rasterizer = GlyphRasterizer::new();

        walk_tree(
            root,
            &layout,
            &text_system,
            &mut glyph_cache,
            &mut rasterizer,
            scale,
        )
    }

    #[test]
    fn single_element_emits_rect_commands() {
        with_scope(|cx| {
            let node = el("Box", cx)
                .style(Style {
                    width: Some(Length::Px(100.0)),
                    height: Some(Length::Px(50.0)),
                    background: Some(Color::WHITE),
                    ..Style::default()
                })
                .build_node();

            let cmds = walk_with_mock(&node, 400.0, 300.0);
            assert!(!cmds.is_empty());
            assert!(
                cmds.iter().any(|c| matches!(c, DrawCommand::Rect(_))),
                "should contain Rect command"
            );
        });
    }

    #[test]
    fn text_node_emits_text_command() {
        // Text nodes produce DrawCommand::Text with glyphs
        let node = Node::Text(text("hello"));
        let layout = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
        let text_system = TextSystem::new();
        let mut glyph_cache = GlyphCache::new(512, 512);
        let mut rasterizer = GlyphRasterizer::new();

        let cmds = walk_tree(
            &node,
            &layout,
            &text_system,
            &mut glyph_cache,
            &mut rasterizer,
            1.0,
        );

        let has_text = cmds.iter().any(|c| matches!(c, DrawCommand::Text(_)));
        assert!(has_text, "should contain Text command, got: {cmds:?}");
    }

    #[test]
    fn parent_background_before_child_text() {
        with_scope(|cx| {
            let node = el("Parent", cx)
                .style(Style {
                    width: Some(Length::Px(200.0)),
                    height: Some(Length::Px(100.0)),
                    background: Some(Color::WHITE),
                    ..Style::default()
                })
                .child(text("hello"))
                .build_node();

            let layout = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
            let text_system = TextSystem::new();
            let mut glyph_cache = GlyphCache::new(512, 512);
            let mut rasterizer = GlyphRasterizer::new();

            let cmds = walk_tree(
                &node,
                &layout,
                &text_system,
                &mut glyph_cache,
                &mut rasterizer,
                1.0,
            );

            // Find first Rect and first Text
            let rect_idx = cmds.iter().position(|c| matches!(c, DrawCommand::Rect(_)));
            let text_idx = cmds.iter().position(|c| matches!(c, DrawCommand::Text(_)));

            if let (Some(ri), Some(ti)) = (rect_idx, text_idx) {
                assert!(ri < ti, "Rect should come before Text for z-ordering");
            }
        });
    }

    #[test]
    fn fragment_children_flattened_in_order() {
        with_scope(|cx| {
            let frag = Node::Fragment(vec![
                el("A", cx)
                    .style(Style {
                        width: Some(Length::Px(50.0)),
                        height: Some(Length::Px(50.0)),
                        background: Some(Color::WHITE),
                        ..Style::default()
                    })
                    .build_node(),
                el("B", cx)
                    .style(Style {
                        width: Some(Length::Px(50.0)),
                        height: Some(Length::Px(50.0)),
                        background: Some(Color::BLACK),
                        ..Style::default()
                    })
                    .build_node(),
            ]);

            let parent = el("Parent", cx)
                .style(Style {
                    width: Some(Length::Px(200.0)),
                    height: Some(Length::Px(100.0)),
                    ..Style::default()
                })
                .child_node(frag)
                .build_node();

            let cmds = walk_with_mock(&parent, 400.0, 300.0);
            // Should have rects for both A and B
            let rect_count = cmds
                .iter()
                .filter(|c| matches!(c, DrawCommand::Rect(_)))
                .count();
            assert!(
                rect_count >= 2,
                "should have at least 2 rects, got {rect_count}"
            );
        });
    }

    #[test]
    fn component_node_transparent() {
        with_scope(|cx| {
            let inner = el("Inner", cx)
                .style(Style {
                    width: Some(Length::Px(80.0)),
                    height: Some(Length::Px(40.0)),
                    background: Some(Color::WHITE),
                    ..Style::default()
                })
                .build_node();

            let comp = Node::Component(ComponentNode {
                name: "MyComponent",
                child: Box::new(inner),
            });

            let parent = el("Parent", cx)
                .style(Style {
                    width: Some(Length::Px(200.0)),
                    height: Some(Length::Px(100.0)),
                    ..Style::default()
                })
                .child_node(comp)
                .build_node();

            let cmds = walk_with_mock(&parent, 400.0, 300.0);
            // Should have rect for Inner (Component is transparent)
            let rect_count = cmds
                .iter()
                .filter(|c| matches!(c, DrawCommand::Rect(_)))
                .count();
            assert!(rect_count >= 1, "should have at least 1 rect for Inner");
        });
    }

    #[test]
    fn nested_overflow_hidden_bakes_clip_rects() {
        with_scope(|cx| {
            let node = el("Outer", cx)
                .style(Style {
                    width: Some(Length::Px(200.0)),
                    height: Some(Length::Px(200.0)),
                    overflow: Some(Overflow::Hidden),
                    background: Some(Color::WHITE),
                    ..Style::default()
                })
                .child(
                    el("Inner", cx)
                        .style(Style {
                            width: Some(Length::Px(100.0)),
                            height: Some(Length::Px(100.0)),
                            overflow: Some(Overflow::Hidden),
                            background: Some(Color::BLACK),
                            ..Style::default()
                        })
                        .build_node(),
                )
                .build_node();

            let cmds = walk_with_mock(&node, 400.0, 300.0);

            // Inner element should have a clip_rect baked from the outer clip
            let inner_rect = cmds.iter().find_map(|c| {
                if let DrawCommand::Rect(prim) = c {
                    if prim.fill_color == Color::BLACK {
                        return Some(prim);
                    }
                }
                None
            });
            assert!(
                inner_rect.is_some(),
                "inner element should produce a Rect command"
            );
            assert!(
                inner_rect.unwrap().clip_rect.is_some(),
                "inner element should have a clip_rect from the outer overflow:hidden"
            );
        });
    }

    #[test]
    fn empty_tree_produces_no_commands() {
        let node = Node::Fragment(vec![]);
        // Empty fragment returns error from compute_layout, so we skip.
        // Instead test with a styled but invisible element.
        with_scope(|cx| {
            let node = el("Empty", cx)
                .style(Style {
                    width: Some(Length::Px(0.0)),
                    height: Some(Length::Px(0.0)),
                    ..Style::default()
                })
                .build_node();

            let cmds = walk_with_mock(&node, 400.0, 300.0);
            // No background = no commands
            assert!(
                cmds.is_empty(),
                "empty/unstyled element should produce no commands"
            );
        });
    }

    #[test]
    fn deeply_nested_tree_correct_order() {
        with_scope(|cx| {
            let node = el("L1", cx)
                .style(Style {
                    width: Some(Length::Px(400.0)),
                    height: Some(Length::Px(400.0)),
                    background: Some(Color::WHITE),
                    padding: Edges::all(LengthPercent::Px(10.0)),
                    ..Style::default()
                })
                .child(
                    el("L2", cx)
                        .style(Style {
                            width: Some(Length::Px(300.0)),
                            height: Some(Length::Px(300.0)),
                            background: Some(Color::hex(0xAAAAAA)),
                            padding: Edges::all(LengthPercent::Px(10.0)),
                            ..Style::default()
                        })
                        .child(
                            el("L3", cx)
                                .style(Style {
                                    width: Some(Length::Px(200.0)),
                                    height: Some(Length::Px(200.0)),
                                    background: Some(Color::BLACK),
                                    ..Style::default()
                                })
                                .build_node(),
                        )
                        .build_node(),
                )
                .build_node();

            let cmds = walk_with_mock(&node, 400.0, 400.0);

            // Pre-order: L1 rect, L2 rect, L3 rect
            let rects: Vec<&DrawCommand> = cmds
                .iter()
                .filter(|c| matches!(c, DrawCommand::Rect(_)))
                .collect();
            assert_eq!(rects.len(), 3, "should have 3 rect commands");
        });
    }

    #[test]
    fn node_id_assignment_matches_layout() {
        with_scope(|cx| {
            // Build the exact same tree used for layout and verify
            // the walker consumes IDs in the same order.
            let node = el("Root", cx)
                .style(Style {
                    width: Some(Length::Px(200.0)),
                    height: Some(Length::Px(100.0)),
                    ..Style::default()
                })
                .child(text("hello"))
                .build_node();

            let layout = compute_layout(&node, 200.0, 100.0, &MockMeasure).unwrap();

            // Layout should have 2 nodes: Root(0) + Text(1)
            assert_eq!(layout.len(), 2);

            let text_system = TextSystem::new();
            let mut glyph_cache = GlyphCache::new(256, 256);
            let mut rasterizer = GlyphRasterizer::new();

            // Should not panic — means IDs are assigned correctly
            let _cmds = walk_tree(
                &node,
                &layout,
                &text_system,
                &mut glyph_cache,
                &mut rasterizer,
                1.0,
            );
        });
    }

    // -- Image element tests --

    #[test]
    fn image_element_emits_image_command() {
        with_scope(|cx| {
            let node = el("Image", cx)
                .attr("src", "photo.png")
                .attr("sizing_mode", "fill")
                .style(Style {
                    width: Some(Length::Px(200.0)),
                    height: Some(Length::Px(100.0)),
                    ..Style::default()
                })
                .build_node();

            let cmds = walk_with_mock(&node, 400.0, 300.0);
            let has_image = cmds.iter().any(|c| matches!(c, DrawCommand::Image(_)));
            assert!(has_image, "should contain Image command, got: {cmds:?}");
        });
    }

    #[test]
    fn sizing_fill_uv() {
        let uv = super::compute_uv(
            "fill",
            &Rect {
                x: 0.0,
                y: 0.0,
                width: 200.0,
                height: 100.0,
            },
            400.0,
            200.0,
        );
        assert_eq!(uv, [0.0, 0.0, 1.0, 1.0]);
    }

    #[test]
    fn sizing_cover_uv() {
        // Container is 200x100 (aspect 2.0), image is 100x100 (aspect 1.0)
        // Container is wider — crop vertically
        let uv = super::compute_uv(
            "cover",
            &Rect {
                x: 0.0,
                y: 0.0,
                width: 200.0,
                height: 100.0,
            },
            100.0,
            100.0,
        );
        // Visible height = 100/2 = 50, offset_v = (100-50)/(200) = 0.25
        assert!(
            (uv[1] - 0.25).abs() < 0.01,
            "v_start should be ~0.25, got {}",
            uv[1]
        );
        assert!(
            (uv[3] - 0.75).abs() < 0.01,
            "v_end should be ~0.75, got {}",
            uv[3]
        );
    }

    #[test]
    fn sizing_contain_uv() {
        let uv = super::compute_uv(
            "contain",
            &Rect {
                x: 0.0,
                y: 0.0,
                width: 200.0,
                height: 100.0,
            },
            400.0,
            200.0,
        );
        assert_eq!(uv, [0.0, 0.0, 1.0, 1.0]);
    }

    // -- Canvas element tests --

    #[test]
    fn canvas_emits_rect_commands() {
        with_scope(|cx| {
            use dusty_widgets::canvas::{CanvasCommand, FillStyle};

            let commands: Rc<RefCell<Vec<CanvasCommand>>> =
                Rc::new(RefCell::new(vec![CanvasCommand::Rect {
                    x: 10.0,
                    y: 10.0,
                    width: 80.0,
                    height: 40.0,
                    fill: Some(FillStyle::Solid(Color::WHITE)),
                    stroke: None,
                }]));

            let node = el("Canvas", cx)
                .style(Style {
                    width: Some(Length::Px(200.0)),
                    height: Some(Length::Px(100.0)),
                    ..Style::default()
                })
                .data(commands)
                .build_node();

            let cmds = walk_with_mock(&node, 400.0, 300.0);
            let rect_count = cmds
                .iter()
                .filter(|c| matches!(c, DrawCommand::Rect(_)))
                .count();
            assert!(
                rect_count >= 1,
                "should contain Rect from canvas, got {rect_count}"
            );
        });
    }

    #[test]
    fn commands_offset_by_position() {
        with_scope(|cx| {
            use dusty_widgets::canvas::{CanvasCommand, FillStyle};

            let commands: Rc<RefCell<Vec<CanvasCommand>>> =
                Rc::new(RefCell::new(vec![CanvasCommand::Rect {
                    x: 5.0,
                    y: 5.0,
                    width: 20.0,
                    height: 20.0,
                    fill: Some(FillStyle::Solid(Color::BLACK)),
                    stroke: None,
                }]));

            // Wrap in a parent so the canvas gets a non-zero position
            let canvas_node = el("Canvas", cx)
                .style(Style {
                    width: Some(Length::Px(100.0)),
                    height: Some(Length::Px(100.0)),
                    ..Style::default()
                })
                .data(commands)
                .build_node();

            let parent = el("Root", cx)
                .style(Style {
                    width: Some(Length::Px(400.0)),
                    height: Some(Length::Px(300.0)),
                    padding: Edges::all(LengthPercent::Px(20.0)),
                    ..Style::default()
                })
                .child_node(canvas_node)
                .build_node();

            let cmds = walk_with_mock(&parent, 400.0, 300.0);

            // Find the canvas rect (drawn from the canvas element)
            let canvas_rects: Vec<_> = cmds
                .iter()
                .filter_map(|c| {
                    if let DrawCommand::Rect(p) = c {
                        if p.rect.width == 20.0 && p.rect.height == 20.0 {
                            Some(p)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();
            assert!(!canvas_rects.is_empty(), "should find the canvas rect");
            // Canvas is at (20, 20) due to parent padding, + (5, 5) from command offset
            let r = &canvas_rects[0].rect;
            assert_eq!(r.x, 25.0, "x should be parent_padding + command_x");
            assert_eq!(r.y, 25.0, "y should be parent_padding + command_y");
        });
    }

    #[test]
    fn empty_commands_no_output() {
        with_scope(|cx| {
            use dusty_widgets::canvas::CanvasCommand;

            let commands: Rc<RefCell<Vec<CanvasCommand>>> = Rc::new(RefCell::new(vec![]));

            let node = el("Canvas", cx)
                .style(Style {
                    width: Some(Length::Px(100.0)),
                    height: Some(Length::Px(100.0)),
                    ..Style::default()
                })
                .data(commands)
                .build_node();

            let cmds = walk_with_mock(&node, 400.0, 300.0);
            // No background and no canvas commands → empty
            assert!(cmds.is_empty(), "empty canvas should produce no commands");
        });
    }

    #[test]
    fn hash_string_known_djb2_collision_produces_distinct_ids() {
        // "Az" and "BY" collide under DJB2 but not under SipHash
        use super::hash_string;
        let h1 = hash_string("Az");
        let h2 = hash_string("BY");
        assert_ne!(
            h1, h2,
            "DJB2 collision pair must produce distinct SipHash values"
        );
    }

    // -- Scroll offset tests --

    fn extract_rect_positions(cmds: &[DrawCommand]) -> Vec<(f32, f32)> {
        cmds.iter()
            .filter_map(|c| {
                if let DrawCommand::Rect(p) = c {
                    Some((p.rect.x, p.rect.y))
                } else {
                    None
                }
            })
            .collect()
    }

    #[test]
    fn scroll_view_translates_child_positions() {
        with_scope(|cx| {
            use dusty_reactive::create_signal;

            let scroll_signal: Signal<(f64, f64)> = create_signal((0.0, 50.0));

            let node = el("ScrollContainer", cx)
                .style(Style {
                    width: Some(Length::Px(200.0)),
                    height: Some(Length::Px(200.0)),
                    overflow: Some(Overflow::Scroll),
                    background: Some(Color::WHITE),
                    ..Style::default()
                })
                .data(scroll_signal)
                .child(
                    el("Child", cx)
                        .style(Style {
                            width: Some(Length::Px(100.0)),
                            height: Some(Length::Px(100.0)),
                            background: Some(Color::BLACK),
                            ..Style::default()
                        })
                        .build_node(),
                )
                .build_node();

            let cmds = walk_with_mock(&node, 400.0, 300.0);
            let positions = extract_rect_positions(&cmds);

            // Container at (0,0), child should be shifted up by scroll offset (50)
            assert!(positions.len() >= 2);
            // Container rect is not shifted
            assert_eq!(positions[0].1, 0.0, "container should be at y=0");
            // Child rect should be shifted by -50
            assert_eq!(
                positions[1].1, -50.0,
                "child should be at y=-50 due to scroll"
            );
        });
    }

    #[test]
    fn scroll_view_clip_region_not_shifted() {
        with_scope(|cx| {
            use dusty_reactive::create_signal;

            let scroll_signal: Signal<(f64, f64)> = create_signal((0.0, 30.0));

            let node = el("ScrollContainer", cx)
                .style(Style {
                    width: Some(Length::Px(200.0)),
                    height: Some(Length::Px(200.0)),
                    overflow: Some(Overflow::Scroll),
                    background: Some(Color::WHITE),
                    ..Style::default()
                })
                .data(scroll_signal)
                .child(
                    el("Child", cx)
                        .style(Style {
                            width: Some(Length::Px(100.0)),
                            height: Some(Length::Px(100.0)),
                            background: Some(Color::BLACK),
                            ..Style::default()
                        })
                        .build_node(),
                )
                .build_node();

            let cmds = walk_with_mock(&node, 400.0, 300.0);

            // The child element should have a clip_rect baked from the scroll
            // container. That clip rect should use the container's unshifted position.
            let child_prim = cmds.iter().find_map(|c| {
                if let DrawCommand::Rect(prim) = c {
                    if prim.fill_color == Color::BLACK {
                        return Some(prim);
                    }
                }
                None
            });
            assert!(
                child_prim.is_some(),
                "child element should produce a Rect command"
            );
            let clip = child_prim.unwrap().clip_rect.as_ref();
            assert!(
                clip.is_some(),
                "child should have clip_rect from scroll container"
            );
            assert_eq!(clip.unwrap().y, 0.0, "clip rect should be at container y=0");
        });
    }

    #[test]
    fn scroll_view_translates_child_at_hidpi_2x() {
        // P0-#4 verification: with scale_factor=2.0 (Retina), the scroll
        // offset must be subtracted in logical space *before* scaling, so a
        // child at logical (10, 100) inside a container scrolled by (0, 50)
        // appears at physical (20, 100) — i.e. (10 * 2, (100 - 50) * 2).
        with_scope(|cx| {
            use dusty_reactive::create_signal;

            let scroll_signal: Signal<(f64, f64)> = create_signal((0.0, 50.0));

            let node = el("ScrollContainer", cx)
                .style(Style {
                    width: Some(Length::Px(200.0)),
                    height: Some(Length::Px(200.0)),
                    overflow: Some(Overflow::Scroll),
                    background: Some(Color::WHITE),
                    ..Style::default()
                })
                .data(scroll_signal)
                .child(
                    el("Child", cx)
                        .style(Style {
                            width: Some(Length::Px(100.0)),
                            height: Some(Length::Px(100.0)),
                            background: Some(Color::BLACK),
                            ..Style::default()
                        })
                        .build_node(),
                )
                .build_node();

            let cmds = walk_with_mock_scale(&node, 400.0, 300.0, 2.0);
            let positions = extract_rect_positions(&cmds);

            assert!(positions.len() >= 2);
            // Container is at logical (0, 0), no scroll affects it: physical (0, 0).
            assert_eq!(
                positions[0],
                (0.0, 0.0),
                "container at scale 2x: physical (0, 0)"
            );
            // Child is at logical (0, 0) within container; with scroll y=50
            // the adjusted logical y is -50; scale 2x gives physical y = -100.
            assert_eq!(
                positions[1],
                (0.0, -100.0),
                "child at scale 2x with y=50 scroll: physical (0, -100)"
            );
        });
    }

    #[test]
    fn scroll_view_zero_offset_unchanged() {
        with_scope(|cx| {
            use dusty_reactive::create_signal;

            let scroll_signal: Signal<(f64, f64)> = create_signal((0.0, 0.0));

            let node = el("ScrollContainer", cx)
                .style(Style {
                    width: Some(Length::Px(200.0)),
                    height: Some(Length::Px(200.0)),
                    overflow: Some(Overflow::Scroll),
                    background: Some(Color::WHITE),
                    ..Style::default()
                })
                .data(scroll_signal)
                .child(
                    el("Child", cx)
                        .style(Style {
                            width: Some(Length::Px(100.0)),
                            height: Some(Length::Px(100.0)),
                            background: Some(Color::BLACK),
                            ..Style::default()
                        })
                        .build_node(),
                )
                .build_node();

            let cmds = walk_with_mock(&node, 400.0, 300.0);
            let positions = extract_rect_positions(&cmds);

            assert!(positions.len() >= 2);
            // With zero scroll offset, child should be at normal position
            assert_eq!(
                positions[1].1, 0.0,
                "child should be at y=0 with zero offset"
            );
        });
    }

    // -- Text Y-position tests --

    fn extract_glyph_ys(cmds: &[DrawCommand]) -> Vec<f32> {
        cmds.iter()
            .filter_map(|c| {
                if let DrawCommand::Text(tp) = c {
                    Some(tp)
                } else {
                    None
                }
            })
            .flat_map(|tp| tp.glyphs.iter().map(|g| g.y))
            .collect()
    }

    #[test]
    fn text_multiline_glyphs_dont_overlap() {
        // Two-line text: verify second line glyphs have Y > first line glyph Y values
        with_scope(|cx| {
            // Narrow container to force wrapping
            let node = el("Container", cx)
                .style(Style {
                    width: Some(Length::Px(60.0)),
                    height: Some(Length::Px(200.0)),
                    ..Style::default()
                })
                .child(text("Hello World this is a long text that wraps"))
                .build_node();

            let layout = compute_layout(&node, 60.0, 200.0, &MockMeasure).unwrap();
            let text_system = TextSystem::new();
            let mut glyph_cache = GlyphCache::new(512, 512);
            let mut rasterizer = GlyphRasterizer::new();

            let cmds = walk_tree(
                &node,
                &layout,
                &text_system,
                &mut glyph_cache,
                &mut rasterizer,
                1.0,
            );

            let ys = extract_glyph_ys(&cmds);
            if ys.len() < 2 {
                // Font system may be unavailable in CI — skip gracefully
                return;
            }

            // Group glyphs by approximate Y (within 2px tolerance)
            let mut y_levels: Vec<f32> = Vec::new();
            for &y in &ys {
                if !y_levels.iter().any(|&ly| (ly - y).abs() < 2.0) {
                    y_levels.push(y);
                }
            }
            y_levels.sort_by(|a, b| a.partial_cmp(b).unwrap());

            assert!(
                y_levels.len() >= 2,
                "expected at least 2 Y levels (multiple lines), got {}: {y_levels:?}",
                y_levels.len()
            );

            // Each successive line should be strictly below the previous
            for pair in y_levels.windows(2) {
                assert!(
                    pair[1] > pair[0],
                    "line Y values should increase: {} should be > {}",
                    pair[1],
                    pair[0]
                );
            }
        });
    }

    #[test]
    fn text_glyph_y_within_layout_rect() {
        // Single-line text: glyph Y values should be within the text node's layout rect
        with_scope(|cx| {
            let node = el("Container", cx)
                .style(Style {
                    width: Some(Length::Px(400.0)),
                    height: Some(Length::Px(100.0)),
                    ..Style::default()
                })
                .child(text("Hello"))
                .build_node();

            let layout = compute_layout(&node, 400.0, 100.0, &MockMeasure).unwrap();

            // Text node is ID 1 (Element=0, Text=1)
            let text_rect = layout.get(LayoutNodeId(1)).unwrap();

            let text_system = TextSystem::new();
            let mut glyph_cache = GlyphCache::new(512, 512);
            let mut rasterizer = GlyphRasterizer::new();

            let cmds = walk_tree(
                &node,
                &layout,
                &text_system,
                &mut glyph_cache,
                &mut rasterizer,
                1.0,
            );

            let ys = extract_glyph_ys(&cmds);
            if ys.is_empty() {
                return; // Font system unavailable
            }

            for &y in &ys {
                assert!(
                    y >= text_rect.y - 1.0,
                    "glyph y ({y}) should be >= layout rect y ({})",
                    text_rect.y
                );
                assert!(
                    y < text_rect.y + text_rect.height + 16.0, // generous margin for descenders
                    "glyph y ({y}) should be within layout rect (y={}, h={})",
                    text_rect.y,
                    text_rect.height
                );
            }
        });
    }

    #[test]
    fn text_glyph_positions_scale_proportionally() {
        // Multi-line text at 2x scale should still produce multiple Y levels
        // with correct ascending order (same invariant as 1x)
        with_scope(|cx| {
            let node = el("Container", cx)
                .style(Style {
                    width: Some(Length::Px(60.0)),
                    height: Some(Length::Px(200.0)),
                    ..Style::default()
                })
                .child(text("Hello World this is a long text that wraps"))
                .build_node();

            let layout = compute_layout(&node, 60.0, 200.0, &MockMeasure).unwrap();
            let text_system = TextSystem::new();

            let mut gc = GlyphCache::new(512, 512);
            let mut rast = GlyphRasterizer::new();
            let cmds = walk_tree(&node, &layout, &text_system, &mut gc, &mut rast, 2.0);
            let ys = extract_glyph_ys(&cmds);

            if ys.len() < 2 {
                return; // Font system unavailable
            }

            // Group into Y levels
            let mut y_levels: Vec<f32> = Vec::new();
            for &y in &ys {
                if !y_levels.iter().any(|&ly| (ly - y).abs() < 4.0) {
                    y_levels.push(y);
                }
            }
            y_levels.sort_by(|a, b| a.partial_cmp(b).unwrap());

            assert!(
                y_levels.len() >= 2,
                "at 2x scale, should still have multiple Y levels: {y_levels:?}"
            );
            for pair in y_levels.windows(2) {
                assert!(
                    pair[1] > pair[0],
                    "at 2x scale, Y levels should increase: {} > {}",
                    pair[1],
                    pair[0]
                );
            }
        });
    }

    #[test]
    fn inherited_foreground_color_used() {
        with_scope(|cx| {
            let node = el("Parent", cx)
                .style(Style {
                    width: Some(Length::Px(200.0)),
                    height: Some(Length::Px(100.0)),
                    foreground: Some(Color::rgb(1.0, 0.0, 0.0)), // red
                    ..Style::default()
                })
                .child(text("red text"))
                .build_node();

            let layout = compute_layout(&node, 200.0, 100.0, &MockMeasure).unwrap();
            let text_system = TextSystem::new();
            let mut glyph_cache = GlyphCache::new(512, 512);
            let mut rasterizer = GlyphRasterizer::new();

            let cmds = walk_tree(
                &node,
                &layout,
                &text_system,
                &mut glyph_cache,
                &mut rasterizer,
                1.0,
            );

            // Check that text glyphs have red color
            for cmd in &cmds {
                if let DrawCommand::Text(tp) = cmd {
                    for glyph in &tp.glyphs {
                        assert_eq!(glyph.color[0], 1.0, "text glyph red channel should be 1.0");
                        assert_eq!(
                            glyph.color[1], 0.0,
                            "text glyph green channel should be 0.0"
                        );
                    }
                }
            }
        });
    }
}
