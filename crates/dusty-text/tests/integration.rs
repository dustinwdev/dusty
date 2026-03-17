//! Integration tests: TextSystem as TextMeasure with the layout engine.

use dusty_core::{el, text};
use dusty_layout::{compute_layout, TextMeasure};
use dusty_reactive::{create_scope, dispose_runtime, initialize_runtime};
use dusty_style::{FontStyle, FontWeight, Style};
use dusty_text::{TextLayout, TextSpan, TextSystem, Truncation};

fn with_scope(f: impl FnOnce(dusty_reactive::Scope)) {
    initialize_runtime();
    create_scope(f);
    dispose_runtime();
}

#[test]
fn text_system_implements_text_measure() {
    let system = TextSystem::new();
    let measure: &dyn TextMeasure = &system;
    let (w, h) = measure.measure("hello", None, &FontStyle::default());
    assert!(w > 0.0);
    assert!(h > 0.0);
}

#[test]
fn text_node_gets_real_metrics_in_layout() {
    let system = TextSystem::new();

    let node = dusty_core::Node::Text(text("hello world"));

    #[allow(clippy::unwrap_used)]
    let result = compute_layout(&node, 800.0, 600.0, &system).unwrap();
    assert_eq!(result.len(), 1);

    #[allow(clippy::unwrap_used)]
    let rect = result.root_rect().unwrap();
    assert!(rect.width > 0.0, "text should have width");
    assert!(rect.height > 0.0, "text should have height");
}

#[test]
fn text_node_wraps_in_constrained_container() {
    let system = TextSystem::new();
    let long_text = "This is a fairly long piece of text that should definitely wrap when placed inside a narrow container";

    with_scope(|cx| {
        let node = el("Container", cx)
            .style(Style {
                width: Some(100.0),
                height: Some(400.0),
                ..Style::default()
            })
            .child(dusty_core::Node::Text(text(long_text)))
            .build_node();

        #[allow(clippy::unwrap_used)]
        let result = compute_layout(&node, 100.0, 400.0, &system).unwrap();

        // Text node
        #[allow(clippy::unwrap_used)]
        let text_rect = result.get(dusty_layout::LayoutNodeId(1)).unwrap();

        // Text should be taller than a single line since it wraps
        assert!(
            text_rect.height > 20.0,
            "wrapped text should be multi-line tall: {}",
            text_rect.height
        );
    });
}

#[test]
fn font_size_affects_layout() {
    let system = TextSystem::new();
    let font_small = FontStyle {
        size: Some(12.0),
        ..FontStyle::default()
    };
    let font_large = FontStyle {
        size: Some(48.0),
        ..FontStyle::default()
    };

    let (_, small_h) = system.measure("hello", None, &font_small);
    let (_, large_h) = system.measure("hello", None, &font_large);

    assert!(
        large_h > small_h,
        "larger font should produce taller text: {large_h} > {small_h}"
    );
}

#[test]
fn rich_text_measurement() {
    let system = TextSystem::new();
    let spans = [
        TextSpan::new("normal "),
        TextSpan::new("bold ").weight(FontWeight::BOLD),
        TextSpan::new("text"),
    ];
    let font = FontStyle::default();

    let (w, h) = system.measure_rich(&spans, None, &font).unwrap();
    assert!(w > 0.0, "rich text width should be positive: {w}");
    assert!(h > 0.0, "rich text height should be positive: {h}");
}

#[test]
fn text_layout_line_count() {
    let system = TextSystem::new();
    let font = FontStyle::default();

    let single = TextLayout::new(&system, "hello", &font, None).unwrap();
    assert_eq!(single.line_count(), 1);

    let multi = TextLayout::new(
        &system,
        "this is a long text that should wrap across several lines",
        &font,
        Some(50.0),
    )
    .unwrap();
    assert!(
        multi.line_count() > 1,
        "should wrap into multiple lines: {}",
        multi.line_count()
    );
}

#[test]
fn truncation_ellipsis_in_layout_context() {
    let system = TextSystem::new();
    let font = FontStyle::default();
    let text = "The quick brown fox jumps over the lazy dog";
    let max_width = 120.0;

    let result = system.truncate(text, max_width, &font, Truncation::Ellipsis);
    assert!(result.was_truncated);
    assert!(result.text.ends_with('…'));
    assert!(result.size.0 <= max_width + 1.0);

    // Verify we can lay out the truncated text
    let layout = TextLayout::new(&system, &result.text, &font, None).unwrap();
    let (w, h) = layout.size();
    assert!(w > 0.0);
    assert!(h > 0.0);
}

#[test]
fn rich_text_layout_line_count() {
    let system = TextSystem::new();
    let font = FontStyle::default();
    let spans = [
        TextSpan::new("hello "),
        TextSpan::new("world "),
        TextSpan::new("this is a long text"),
    ];

    let layout = TextLayout::new_rich(&system, &spans, &font, Some(50.0)).unwrap();
    assert!(
        layout.line_count() > 1,
        "rich text should wrap: {}",
        layout.line_count()
    );
}

// --- Truncation edge-case tests ---

#[test]
fn truncate_single_char_tiny_max_width() {
    let system = TextSystem::new();
    let font = FontStyle::default();

    // A single character with a max_width too small to even fit the ellipsis.
    let result = system.truncate("X", 0.5, &font, Truncation::Ellipsis);

    // The text overflows but the binary search may only produce "…" or empty.
    // Key invariant: must not panic, and was_truncated should be true.
    assert!(
        result.was_truncated,
        "single char should be truncated at tiny width"
    );
}

#[test]
fn truncate_unicode_multibyte_at_boundaries() {
    let system = TextSystem::new();
    let font = FontStyle::default();

    // Mix of multi-byte code points: CJK, emoji, accented Latin
    let text = "\u{4e16}\u{754c}\u{00e9}\u{00fc}\u{1f600}hello";
    let (full_w, _) = system.measure(text, None, &font);

    // Use a width that is roughly half the full width so truncation
    // must split somewhere among the multi-byte characters.
    let max_width = full_w * 0.5;
    let result = system.truncate(text, max_width, &font, Truncation::Ellipsis);

    // Must not panic from slicing mid-character.
    assert!(result.was_truncated, "multi-byte text should be truncated");
    assert!(
        result.text.ends_with('\u{2026}'),
        "should end with ellipsis: {:?}",
        result.text
    );
    // The truncated text must be valid UTF-8 (Rust enforces this, but the
    // assertion documents intent).
    assert!(
        result.text.is_char_boundary(result.text.len()),
        "truncated text must end on a char boundary"
    );
}

#[test]
fn truncate_max_width_exactly_matches_text() {
    let system = TextSystem::new();
    let font = FontStyle::default();
    let text = "exact fit";

    let (full_w, _) = system.measure(text, None, &font);

    // When max_width == measured width, text should not be truncated.
    let result = system.truncate(text, full_w, &font, Truncation::Ellipsis);
    assert!(
        !result.was_truncated,
        "text that fits exactly should not be truncated"
    );
    assert_eq!(result.text, text);
}

#[test]
fn truncate_ellipsis_one_char_too_wide() {
    let system = TextSystem::new();
    let font = FontStyle::default();
    let text = "abcdefgh";

    let (full_w, _) = system.measure(text, None, &font);

    // Subtract a tiny amount so the full text just barely overflows.
    let max_width = full_w - 0.5;
    let result = system.truncate(text, max_width, &font, Truncation::Ellipsis);

    assert!(
        result.was_truncated,
        "text slightly wider than budget should be truncated"
    );
    assert!(
        result.text.ends_with('\u{2026}'),
        "should end with ellipsis: {:?}",
        result.text
    );
    // The truncated width should fit within the budget (with small tolerance).
    assert!(
        result.size.0 <= max_width + 1.0,
        "truncated width {} should be <= max_width {} (with tolerance)",
        result.size.0,
        max_width
    );
    // Should have removed at least one character from the original text
    // (the ellipsis replaces it).
    let prefix_len = result.text.trim_end_matches('\u{2026}').len();
    assert!(
        prefix_len < text.len(),
        "prefix should be shorter than original: {} < {}",
        prefix_len,
        text.len()
    );
}
