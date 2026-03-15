//! Winit key and modifier translation.

use dusty_core::event::{Key, Modifiers};
use winit::keyboard::{KeyCode, ModifiersState, NamedKey, PhysicalKey};

/// Translates a winit `winit::keyboard::Key` into a dusty [`Key`].
///
/// Named keys follow the W3C UI Events key values (`"Enter"`, `"Escape"`, etc.).
/// Character keys pass through directly.
///
/// # Example
///
/// ```
/// use dusty_platform::translate_key;
/// use dusty_core::event::Key;
/// use winit::keyboard::{Key as WinitKey, NamedKey};
///
/// let key = translate_key(&WinitKey::Named(NamedKey::Enter));
/// assert_eq!(key, Key("Enter".into()));
/// ```
#[must_use]
pub fn translate_key(key: &winit::keyboard::Key) -> Key {
    match key {
        winit::keyboard::Key::Named(named) => Key(translate_named_key(*named).into()),
        winit::keyboard::Key::Character(c) => Key(c.to_string()),
        winit::keyboard::Key::Unidentified(_) | winit::keyboard::Key::Dead(_) => {
            Key("Unidentified".into())
        }
    }
}

/// Translates a winit `PhysicalKey` to an optional key name string.
///
/// Used as a fallback when the logical key is not informative.
#[must_use]
pub fn translate_physical_key(physical: PhysicalKey) -> Option<Key> {
    match physical {
        PhysicalKey::Code(code) => translate_key_code(code).map(|s| Key(s.into())),
        PhysicalKey::Unidentified(_) => None,
    }
}

/// Translates winit `ModifiersState` into dusty [`Modifiers`].
///
/// # Example
///
/// ```
/// use dusty_platform::translate_modifiers;
/// use winit::keyboard::ModifiersState;
///
/// let mods = translate_modifiers(ModifiersState::SHIFT | ModifiersState::CONTROL);
/// assert!(mods.shift);
/// assert!(mods.ctrl);
/// assert!(!mods.alt);
/// assert!(!mods.meta);
/// ```
#[must_use]
pub fn translate_modifiers(state: ModifiersState) -> Modifiers {
    Modifiers {
        shift: state.shift_key(),
        ctrl: state.control_key(),
        alt: state.alt_key(),
        meta: state.super_key(),
    }
}

const fn translate_named_key(named: NamedKey) -> &'static str {
    match named {
        NamedKey::Enter => "Enter",
        NamedKey::Tab => "Tab",
        NamedKey::Space => " ",
        NamedKey::Backspace => "Backspace",
        NamedKey::Delete => "Delete",
        NamedKey::Escape => "Escape",
        NamedKey::ArrowUp => "ArrowUp",
        NamedKey::ArrowDown => "ArrowDown",
        NamedKey::ArrowLeft => "ArrowLeft",
        NamedKey::ArrowRight => "ArrowRight",
        NamedKey::Home => "Home",
        NamedKey::End => "End",
        NamedKey::PageUp => "PageUp",
        NamedKey::PageDown => "PageDown",
        NamedKey::Insert => "Insert",
        NamedKey::F1 => "F1",
        NamedKey::F2 => "F2",
        NamedKey::F3 => "F3",
        NamedKey::F4 => "F4",
        NamedKey::F5 => "F5",
        NamedKey::F6 => "F6",
        NamedKey::F7 => "F7",
        NamedKey::F8 => "F8",
        NamedKey::F9 => "F9",
        NamedKey::F10 => "F10",
        NamedKey::F11 => "F11",
        NamedKey::F12 => "F12",
        NamedKey::CapsLock => "CapsLock",
        NamedKey::NumLock => "NumLock",
        NamedKey::ScrollLock => "ScrollLock",
        NamedKey::PrintScreen => "PrintScreen",
        NamedKey::Pause => "Pause",
        NamedKey::ContextMenu => "ContextMenu",
        NamedKey::Shift => "Shift",
        NamedKey::Control => "Control",
        NamedKey::Alt => "Alt",
        NamedKey::Meta | NamedKey::Super => "Meta",
        _ => "Unidentified",
    }
}

const fn translate_key_code(code: KeyCode) -> Option<&'static str> {
    match code {
        KeyCode::KeyA => Some("a"),
        KeyCode::KeyB => Some("b"),
        KeyCode::KeyC => Some("c"),
        KeyCode::KeyD => Some("d"),
        KeyCode::KeyE => Some("e"),
        KeyCode::KeyF => Some("f"),
        KeyCode::KeyG => Some("g"),
        KeyCode::KeyH => Some("h"),
        KeyCode::KeyI => Some("i"),
        KeyCode::KeyJ => Some("j"),
        KeyCode::KeyK => Some("k"),
        KeyCode::KeyL => Some("l"),
        KeyCode::KeyM => Some("m"),
        KeyCode::KeyN => Some("n"),
        KeyCode::KeyO => Some("o"),
        KeyCode::KeyP => Some("p"),
        KeyCode::KeyQ => Some("q"),
        KeyCode::KeyR => Some("r"),
        KeyCode::KeyS => Some("s"),
        KeyCode::KeyT => Some("t"),
        KeyCode::KeyU => Some("u"),
        KeyCode::KeyV => Some("v"),
        KeyCode::KeyW => Some("w"),
        KeyCode::KeyX => Some("x"),
        KeyCode::KeyY => Some("y"),
        KeyCode::KeyZ => Some("z"),
        KeyCode::Digit0 => Some("0"),
        KeyCode::Digit1 => Some("1"),
        KeyCode::Digit2 => Some("2"),
        KeyCode::Digit3 => Some("3"),
        KeyCode::Digit4 => Some("4"),
        KeyCode::Digit5 => Some("5"),
        KeyCode::Digit6 => Some("6"),
        KeyCode::Digit7 => Some("7"),
        KeyCode::Digit8 => Some("8"),
        KeyCode::Digit9 => Some("9"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_key_enter() {
        let key = translate_key(&winit::keyboard::Key::Named(NamedKey::Enter));
        assert_eq!(key, Key("Enter".into()));
    }

    #[test]
    fn named_key_escape() {
        let key = translate_key(&winit::keyboard::Key::Named(NamedKey::Escape));
        assert_eq!(key, Key("Escape".into()));
    }

    #[test]
    fn named_key_space() {
        let key = translate_key(&winit::keyboard::Key::Named(NamedKey::Space));
        assert_eq!(key, Key(" ".into()));
    }

    #[test]
    fn named_key_arrows() {
        assert_eq!(
            translate_key(&winit::keyboard::Key::Named(NamedKey::ArrowUp)),
            Key("ArrowUp".into())
        );
        assert_eq!(
            translate_key(&winit::keyboard::Key::Named(NamedKey::ArrowDown)),
            Key("ArrowDown".into())
        );
        assert_eq!(
            translate_key(&winit::keyboard::Key::Named(NamedKey::ArrowLeft)),
            Key("ArrowLeft".into())
        );
        assert_eq!(
            translate_key(&winit::keyboard::Key::Named(NamedKey::ArrowRight)),
            Key("ArrowRight".into())
        );
    }

    #[test]
    fn named_key_function_keys() {
        assert_eq!(
            translate_key(&winit::keyboard::Key::Named(NamedKey::F1)),
            Key("F1".into())
        );
        assert_eq!(
            translate_key(&winit::keyboard::Key::Named(NamedKey::F12)),
            Key("F12".into())
        );
    }

    #[test]
    fn named_key_modifiers() {
        assert_eq!(
            translate_key(&winit::keyboard::Key::Named(NamedKey::Shift)),
            Key("Shift".into())
        );
        assert_eq!(
            translate_key(&winit::keyboard::Key::Named(NamedKey::Control)),
            Key("Control".into())
        );
        assert_eq!(
            translate_key(&winit::keyboard::Key::Named(NamedKey::Alt)),
            Key("Alt".into())
        );
        assert_eq!(
            translate_key(&winit::keyboard::Key::Named(NamedKey::Meta)),
            Key("Meta".into())
        );
    }

    #[test]
    fn character_key() {
        let key = translate_key(&winit::keyboard::Key::Character(
            winit::keyboard::SmolStr::new("a"),
        ));
        assert_eq!(key, Key("a".into()));
    }

    #[test]
    fn character_key_multi() {
        let key = translate_key(&winit::keyboard::Key::Character(
            winit::keyboard::SmolStr::new("abc"),
        ));
        assert_eq!(key, Key("abc".into()));
    }

    #[test]
    fn dead_key() {
        let key = translate_key(&winit::keyboard::Key::Dead(None));
        assert_eq!(key, Key("Unidentified".into()));
    }

    #[test]
    fn modifiers_none() {
        let mods = translate_modifiers(ModifiersState::empty());
        assert_eq!(mods, Modifiers::default());
    }

    #[test]
    fn modifiers_shift() {
        let mods = translate_modifiers(ModifiersState::SHIFT);
        assert!(mods.shift);
        assert!(!mods.ctrl);
        assert!(!mods.alt);
        assert!(!mods.meta);
    }

    #[test]
    fn modifiers_all() {
        let mods = translate_modifiers(
            ModifiersState::SHIFT
                | ModifiersState::CONTROL
                | ModifiersState::ALT
                | ModifiersState::SUPER,
        );
        assert!(mods.shift);
        assert!(mods.ctrl);
        assert!(mods.alt);
        assert!(mods.meta);
    }

    #[test]
    fn physical_key_a() {
        let key = translate_physical_key(PhysicalKey::Code(KeyCode::KeyA));
        assert_eq!(key, Some(Key("a".into())));
    }

    #[test]
    fn physical_key_digit() {
        let key = translate_physical_key(PhysicalKey::Code(KeyCode::Digit5));
        assert_eq!(key, Some(Key("5".into())));
    }

    #[test]
    fn physical_key_unknown() {
        let key = translate_physical_key(PhysicalKey::Unidentified(
            winit::keyboard::NativeKeyCode::Unidentified,
        ));
        assert!(key.is_none());
    }
}
