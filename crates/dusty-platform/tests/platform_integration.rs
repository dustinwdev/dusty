//! Headless platform integration tests — no display server required.

use dusty_platform::{LogicalSize, PhysicalSize, ScaleFactor, WindowConfig};

// ---------------------------------------------------------------------------
// ScaleFactor arithmetic
// ---------------------------------------------------------------------------

#[test]
fn scale_factor_creation() {
    let scale = ScaleFactor::new(2.0).unwrap();
    assert!((scale.value() - 2.0).abs() < f64::EPSILON);
}

#[test]
fn scale_factor_rejects_zero() {
    assert!(ScaleFactor::new(0.0).is_none());
}

#[test]
fn scale_factor_rejects_negative() {
    assert!(ScaleFactor::new(-1.0).is_none());
}

#[test]
fn scale_factor_to_physical() {
    let scale = ScaleFactor::new(2.0).unwrap();
    let logical = LogicalSize {
        width: 100.0,
        height: 50.0,
    };
    let physical = scale.to_physical(logical);
    assert_eq!(physical.width, 200);
    assert_eq!(physical.height, 100);
}

#[test]
fn scale_factor_to_logical() {
    let scale = ScaleFactor::new(2.0).unwrap();
    let physical = PhysicalSize {
        width: 200,
        height: 100,
    };
    let logical = scale.to_logical(physical);
    assert!((logical.width - 100.0).abs() < f64::EPSILON);
    assert!((logical.height - 50.0).abs() < f64::EPSILON);
}

#[test]
fn scale_factor_position_round_trip() {
    use dusty_platform::PhysicalPosition;

    let scale = ScaleFactor::new(1.5).unwrap();
    let physical = PhysicalPosition { x: 300, y: 150 };
    let logical = scale.to_logical_position(physical);
    let back = scale.to_physical_position(logical);
    assert_eq!(back, physical);
}

// ---------------------------------------------------------------------------
// WindowConfig builder
// ---------------------------------------------------------------------------

#[test]
fn window_config_title() {
    let config = WindowConfig::new("My App");
    assert_eq!(config.title(), "My App");
}

#[test]
fn window_config_default_size() {
    let config = WindowConfig::new("Test");
    let size = config.size();
    assert!(size.width > 0.0);
    assert!(size.height > 0.0);
}

#[test]
fn window_config_builder_chain() {
    let config = WindowConfig::new("Test")
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .visible(false);
    assert!(!config.is_resizable());
    assert!(!config.has_decorations());
    assert!(config.is_transparent());
    assert!(!config.is_visible());
}

#[test]
fn window_config_min_max_size() {
    let config = WindowConfig::new("Test")
        .min_size(200.0, 100.0)
        .max_size(1920.0, 1080.0);
    assert!(config.min_size_value().is_some());
    assert!(config.max_size_value().is_some());
}

// ---------------------------------------------------------------------------
// Key translation
// ---------------------------------------------------------------------------

#[test]
fn translate_key_named_enter() {
    use dusty_core::event::Key;
    use winit::keyboard::{Key as WinitKey, NamedKey};

    let key = dusty_platform::translate_key(&WinitKey::Named(NamedKey::Enter));
    assert_eq!(key, Key("Enter".into()));
}

#[test]
fn translate_modifiers_shift_ctrl() {
    use winit::keyboard::ModifiersState;

    let mods = dusty_platform::translate_modifiers(ModifiersState::SHIFT | ModifiersState::CONTROL);
    assert!(mods.shift);
    assert!(mods.ctrl);
    assert!(!mods.alt);
    assert!(!mods.meta);
}

#[test]
fn translate_physical_key_letter() {
    use dusty_core::event::Key;
    use winit::keyboard::{KeyCode, PhysicalKey};

    let key = dusty_platform::translate_physical_key(PhysicalKey::Code(KeyCode::KeyZ));
    assert_eq!(key, Some(Key("z".into())));
}
