//! Global PTT hotkey registration via rdev.
//! This is the ONLY module that registers global event listeners.
//! See architecture.md § System API Boundary.

use crate::error::AppError;
use rdev::{EventType, Key};
use std::collections::HashSet;

/// Internal representation of a parsed PTT hotkey binding.
#[derive(Debug, Clone)]
pub struct HotkeyBinding {
    /// The primary key that triggers PTT (e.g., Key::AltGr for "AltRight", Key::Space for combos).
    pub key: Key,
    /// Modifier keys that must be held for combo bindings. Empty for bare key/modifier bindings.
    pub modifiers: Vec<Key>,
}

/// Signals sent from the hotkey listener to the audio capture pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PttEvent {
    Pressed,
    Released,
}

pub type PttSender = std::sync::mpsc::Sender<PttEvent>;
pub type PttReceiver = std::sync::mpsc::Receiver<PttEvent>;

/// Maps a single key name (from `KeyboardEvent.code` or combo segment) to an `rdev::Key`.
fn map_key(name: &str) -> Result<Key, AppError> {
    match name {
        // Sided modifier keys (from KeyboardEvent.code in onboarding)
        "AltRight" => Ok(Key::AltGr),
        "AltLeft" => Ok(Key::Alt),
        "MetaLeft" => Ok(Key::MetaLeft),
        "MetaRight" => Ok(Key::MetaRight),
        "ControlLeft" => Ok(Key::ControlLeft),
        "ControlRight" => Ok(Key::ControlRight),
        "ShiftLeft" => Ok(Key::ShiftLeft),
        "ShiftRight" => Ok(Key::ShiftRight),

        // Logical modifier names (used in combo prefixes like "Alt+Space")
        "Alt" => Ok(Key::Alt),
        "Meta" => Ok(Key::MetaLeft),
        "Control" => Ok(Key::ControlLeft),
        "Shift" => Ok(Key::ShiftLeft),

        // Common keys
        "Space" => Ok(Key::Space),
        "Enter" => Ok(Key::Return),
        "Tab" => Ok(Key::Tab),
        "Backspace" => Ok(Key::Backspace),
        "Escape" => Ok(Key::Escape),
        "CapsLock" => Ok(Key::CapsLock),
        "Delete" => Ok(Key::Delete),

        // Function keys
        "F1" => Ok(Key::F1),
        "F2" => Ok(Key::F2),
        "F3" => Ok(Key::F3),
        "F4" => Ok(Key::F4),
        "F5" => Ok(Key::F5),
        "F6" => Ok(Key::F6),
        "F7" => Ok(Key::F7),
        "F8" => Ok(Key::F8),
        "F9" => Ok(Key::F9),
        "F10" => Ok(Key::F10),
        "F11" => Ok(Key::F11),
        "F12" => Ok(Key::F12),

        // Letter keys (KeyA..KeyZ from KeyboardEvent.code)
        "KeyA" => Ok(Key::KeyA),
        "KeyB" => Ok(Key::KeyB),
        "KeyC" => Ok(Key::KeyC),
        "KeyD" => Ok(Key::KeyD),
        "KeyE" => Ok(Key::KeyE),
        "KeyF" => Ok(Key::KeyF),
        "KeyG" => Ok(Key::KeyG),
        "KeyH" => Ok(Key::KeyH),
        "KeyI" => Ok(Key::KeyI),
        "KeyJ" => Ok(Key::KeyJ),
        "KeyK" => Ok(Key::KeyK),
        "KeyL" => Ok(Key::KeyL),
        "KeyM" => Ok(Key::KeyM),
        "KeyN" => Ok(Key::KeyN),
        "KeyO" => Ok(Key::KeyO),
        "KeyP" => Ok(Key::KeyP),
        "KeyQ" => Ok(Key::KeyQ),
        "KeyR" => Ok(Key::KeyR),
        "KeyS" => Ok(Key::KeyS),
        "KeyT" => Ok(Key::KeyT),
        "KeyU" => Ok(Key::KeyU),
        "KeyV" => Ok(Key::KeyV),
        "KeyW" => Ok(Key::KeyW),
        "KeyX" => Ok(Key::KeyX),
        "KeyY" => Ok(Key::KeyY),
        "KeyZ" => Ok(Key::KeyZ),

        _ => Err(AppError::Hotkey(format!("Unknown hotkey: {}", name))),
    }
}

/// Parses a hotkey string from config into a `HotkeyBinding`.
///
/// Supported formats (from onboarding.js):
/// - Bare modifier: `"AltRight"`, `"MetaLeft"` — key is the modifier, no modifiers vec
/// - Bare key: `"F5"`, `"Space"` — key is the key, no modifiers
/// - Combo: `"Alt+Space"`, `"Control+Shift+F5"` — last segment is key, preceding are modifiers
pub fn parse_hotkey(hotkey_str: &str) -> Result<HotkeyBinding, AppError> {
    if hotkey_str.is_empty() {
        return Err(AppError::Hotkey("Empty hotkey string".to_string()));
    }

    let parts: Vec<&str> = hotkey_str.split('+').map(str::trim).collect();

    if parts.len() == 1 {
        // Bare key or bare modifier
        let key = map_key(parts[0])?;
        Ok(HotkeyBinding {
            key,
            modifiers: Vec::new(),
        })
    } else {
        // Combo: all but last are modifiers, last is the key
        let key = map_key(parts[parts.len() - 1])?;
        let modifiers = parts[..parts.len() - 1]
            .iter()
            .map(|&m| map_key(m))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(HotkeyBinding { key, modifiers })
    }
}

/// Starts the global PTT hotkey listener on a dedicated thread.
///
/// The listener uses `rdev::listen()` which installs a CGEventTap on macOS.
/// It blocks the thread for the lifetime of the app. On key events matching
/// the configured binding, it sends `PttEvent::Pressed` or `PttEvent::Released`
/// through the provided channel.
pub fn start_hotkey_listener(hotkey_str: &str, tx: PttSender) -> Result<(), AppError> {
    let binding = parse_hotkey(hotkey_str)?;
    let has_modifiers = !binding.modifiers.is_empty();

    tracing::info!(
        hotkey = hotkey_str,
        "Registering global PTT hotkey listener"
    );

    std::thread::Builder::new()
        .name("ptt-hotkey".to_string())
        .spawn(move || {
            let mut is_pressed = false;
            let mut held_modifiers: HashSet<Key> = HashSet::new();

            let callback = move |event: rdev::Event| {
                match event.event_type {
                    EventType::KeyPress(key) => {
                        // Track modifier state for combo bindings
                        if has_modifiers && is_modifier(&key) {
                            held_modifiers.insert(key);
                        }

                        if key == binding.key && !is_pressed {
                            // For combos, check all required modifiers are held
                            if has_modifiers {
                                let all_held = binding.modifiers.iter().all(|m| {
                                    held_modifiers.iter().any(|h| modifier_matches(m, h))
                                });
                                if !all_held {
                                    return;
                                }
                            }
                            is_pressed = true;
                            let _ = tx.send(PttEvent::Pressed);
                            tracing::debug!("PTT pressed");
                        }
                    }
                    EventType::KeyRelease(key) => {
                        if has_modifiers && is_modifier(&key) {
                            held_modifiers.remove(&key);
                        }

                        if key == binding.key && is_pressed {
                            is_pressed = false;
                            let _ = tx.send(PttEvent::Released);
                            tracing::debug!("PTT released");
                        }
                    }
                    _ => {}
                }
            };

            if let Err(e) = rdev::listen(callback) {
                tracing::error!("rdev listener exited with error: {:?}", e);
            }
        })
        .map_err(|e| AppError::Hotkey(format!("Failed to spawn hotkey thread: {}", e)))?;

    Ok(())
}

/// Returns true if the key is a modifier key (used for combo tracking).
fn is_modifier(key: &Key) -> bool {
    matches!(
        key,
        Key::Alt
            | Key::AltGr
            | Key::MetaLeft
            | Key::MetaRight
            | Key::ControlLeft
            | Key::ControlRight
            | Key::ShiftLeft
            | Key::ShiftRight
    )
}

/// Returns true if `held` satisfies the requirement for modifier `required`.
/// Accepts either left or right variant for generic modifier mappings.
/// E.g., if the combo specifies `Key::Alt` (left), holding `Key::AltGr` (right) also matches.
fn modifier_matches(required: &Key, held: &Key) -> bool {
    if required == held {
        return true;
    }
    matches!(
        (required, held),
        (Key::Alt, Key::AltGr)
            | (Key::AltGr, Key::Alt)
            | (Key::MetaLeft, Key::MetaRight)
            | (Key::MetaRight, Key::MetaLeft)
            | (Key::ControlLeft, Key::ControlRight)
            | (Key::ControlRight, Key::ControlLeft)
            | (Key::ShiftLeft, Key::ShiftRight)
            | (Key::ShiftRight, Key::ShiftLeft)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bare_modifier_alt_right() {
        let binding = parse_hotkey("AltRight").expect("should parse");
        assert!(matches!(binding.key, Key::AltGr));
        assert!(binding.modifiers.is_empty());
    }

    #[test]
    fn test_parse_bare_modifier_meta_left() {
        let binding = parse_hotkey("MetaLeft").expect("should parse");
        assert!(matches!(binding.key, Key::MetaLeft));
        assert!(binding.modifiers.is_empty());
    }

    #[test]
    fn test_parse_combo_alt_space() {
        let binding = parse_hotkey("Alt+Space").expect("should parse");
        assert!(matches!(binding.key, Key::Space));
        assert_eq!(binding.modifiers.len(), 1);
        assert!(matches!(binding.modifiers[0], Key::Alt));
    }

    #[test]
    fn test_parse_combo_control_shift_f5() {
        let binding = parse_hotkey("Control+Shift+F5").expect("should parse");
        assert!(matches!(binding.key, Key::F5));
        assert_eq!(binding.modifiers.len(), 2);
        assert!(matches!(binding.modifiers[0], Key::ControlLeft));
        assert!(matches!(binding.modifiers[1], Key::ShiftLeft));
    }

    #[test]
    fn test_parse_unknown_key_returns_error() {
        let result = parse_hotkey("FooBar");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Unknown hotkey"), "got: {}", err);
    }

    #[test]
    fn test_parse_empty_string_returns_error() {
        let result = parse_hotkey("");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Empty hotkey"), "got: {}", err);
    }
}
