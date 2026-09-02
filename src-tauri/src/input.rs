use crate::{
    config::{CommandKey, TriggerChord, TriggerModifier},
    matcher::SequenceMatcher,
    refresh_tray, AppState,
};
use keytap::{EventKind, Key, Tap};
use serde::Serialize;
use std::{
    collections::HashSet,
    sync::atomic::Ordering,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerPayload {
    pub source: &'static str,
    pub timestamp_ms: u64,
}

pub fn emit_trigger(app: &AppHandle, source: &'static str) -> Result<(), String> {
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    app.emit(
        "keyword-triggered",
        TriggerPayload {
            source,
            timestamp_ms,
        },
    )
    .map_err(|error| error.to_string())
}

pub fn start(app: AppHandle, state: AppState) {
    thread::Builder::new()
        .name("pc-pet-global-input".into())
        .spawn(move || {
            loop {
                let tap = match Tap::new() {
                    Ok(tap) => tap,
                    Err(error) => {
                        update_listener_state(
                            &app,
                            &state,
                            false,
                            Some(format!("全局键盘监听启动失败：{error}")),
                        );
                        // Input Monitoring can be granted while the app is open. Keep the
                        // background process alive and recover without requiring a restart.
                        thread::sleep(Duration::from_secs(2));
                        continue;
                    }
                };

                update_listener_state(&app, &state, true, None);
                let started = Instant::now();
                let mut runtime = InputRuntime::default();
                for event in tap.iter() {
                    runtime.handle(event.kind, &app, &state, started.elapsed());
                }
                update_listener_state(
                    &app,
                    &state,
                    false,
                    Some("全局键盘监听已停止，正在重新连接".into()),
                );
                thread::sleep(Duration::from_millis(500));
            }
        })
        .expect("failed to spawn global input listener");
}

fn update_listener_state(app: &AppHandle, state: &AppState, running: bool, error: Option<String>) {
    let mut changed = state.listener_running.swap(running, Ordering::Relaxed) != running;
    if let Ok(mut slot) = state.listener_error.write() {
        changed |= *slot != error;
        *slot = error.clone();
    }
    if changed {
        if let Some(message) = error {
            let _ = app.emit("listener-error", message);
        }
        refresh_tray(app);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ModifierPlatform {
    MacOs,
    WindowsOrLinux,
}

const CURRENT_MODIFIER_PLATFORM: ModifierPlatform = if cfg!(target_os = "macos") {
    ModifierPlatform::MacOs
} else {
    ModifierPlatform::WindowsOrLinux
};

#[derive(Default)]
struct InputRuntime {
    matcher: SequenceMatcher,
    held_modifiers: HashSet<Key>,
    last_trigger: Option<Duration>,
}

impl InputRuntime {
    fn handle(&mut self, event: EventKind, app: &AppHandle, state: &AppState, elapsed: Duration) {
        let key_down = self.observe_event(event);
        let Ok(config) = state.config.read().map(|value| value.clone()) else {
            self.matcher.clear();
            return;
        };
        if !config.enabled
            || state.paused.load(Ordering::Relaxed)
            || state.recording_command.load(Ordering::Relaxed)
        {
            self.matcher.clear();
            return;
        }

        // Modifier-only, key-up and auto-repeat events update state at most. In
        // particular, holding Space cannot synthesize a double-Space command.
        let Some(key) = key_down else {
            return;
        };
        let Some(command) = config.trigger_command.as_ref() else {
            self.matcher.clear();
            return;
        };
        let Some(key) = key_to_command_key(key) else {
            self.matcher.clear();
            return;
        };
        let chord = TriggerChord::new(
            key,
            modifiers_for_platform(&self.held_modifiers, CURRENT_MODIFIER_PLATFORM),
        );
        let matched = self.matcher.feed(
            chord,
            command,
            elapsed,
            Duration::from_millis(config.sequence_timeout_ms),
        );
        if matched && self.mark_trigger_if_ready(elapsed, Duration::from_millis(config.cooldown_ms))
        {
            state.trigger_count.fetch_add(1, Ordering::Relaxed);
            let _ = emit_trigger(app, "keyboard");
        }
    }

    fn observe_event(&mut self, event: EventKind) -> Option<Key> {
        match event {
            EventKind::KeyDown(key) if is_modifier(key) => {
                self.held_modifiers.insert(key);
                None
            }
            EventKind::KeyDown(key) => Some(key),
            EventKind::KeyUp(key) => {
                if is_modifier(key) {
                    self.held_modifiers.remove(&key);
                }
                None
            }
            EventKind::KeyRepeat(_) => None,
        }
    }

    fn mark_trigger_if_ready(&mut self, elapsed: Duration, cooldown: Duration) -> bool {
        let cooled_down = self
            .last_trigger
            .is_none_or(|last| elapsed.saturating_sub(last) >= cooldown);
        if cooled_down {
            self.last_trigger = Some(elapsed);
        }
        cooled_down
    }
}

fn is_modifier(key: Key) -> bool {
    matches!(
        key,
        Key::ShiftLeft
            | Key::ShiftRight
            | Key::ControlLeft
            | Key::ControlRight
            | Key::AltLeft
            | Key::AltRight
            | Key::MetaLeft
            | Key::MetaRight
    )
}

fn modifiers_for_platform(held: &HashSet<Key>, platform: ModifierPlatform) -> Vec<TriggerModifier> {
    let has_shift = held.contains(&Key::ShiftLeft) || held.contains(&Key::ShiftRight);
    let has_control = held.contains(&Key::ControlLeft) || held.contains(&Key::ControlRight);
    let has_alt = held.contains(&Key::AltLeft) || held.contains(&Key::AltRight);
    let has_meta = held.contains(&Key::MetaLeft) || held.contains(&Key::MetaRight);
    let mut modifiers = Vec::with_capacity(4);

    match platform {
        ModifierPlatform::MacOs => {
            if has_meta {
                modifiers.push(TriggerModifier::Primary);
            }
            if has_control {
                modifiers.push(TriggerModifier::Control);
            }
        }
        ModifierPlatform::WindowsOrLinux => {
            if has_control {
                modifiers.push(TriggerModifier::Primary);
            }
        }
    }
    if has_alt {
        modifiers.push(TriggerModifier::Alt);
    }
    if has_shift {
        modifiers.push(TriggerModifier::Shift);
    }
    if platform == ModifierPlatform::WindowsOrLinux && has_meta {
        modifiers.push(TriggerModifier::Meta);
    }
    modifiers
}

fn key_to_command_key(key: Key) -> Option<CommandKey> {
    Some(match key {
        Key::A => CommandKey::KeyA,
        Key::B => CommandKey::KeyB,
        Key::C => CommandKey::KeyC,
        Key::D => CommandKey::KeyD,
        Key::E => CommandKey::KeyE,
        Key::F => CommandKey::KeyF,
        Key::G => CommandKey::KeyG,
        Key::H => CommandKey::KeyH,
        Key::I => CommandKey::KeyI,
        Key::J => CommandKey::KeyJ,
        Key::K => CommandKey::KeyK,
        Key::L => CommandKey::KeyL,
        Key::M => CommandKey::KeyM,
        Key::N => CommandKey::KeyN,
        Key::O => CommandKey::KeyO,
        Key::P => CommandKey::KeyP,
        Key::Q => CommandKey::KeyQ,
        Key::R => CommandKey::KeyR,
        Key::S => CommandKey::KeyS,
        Key::T => CommandKey::KeyT,
        Key::U => CommandKey::KeyU,
        Key::V => CommandKey::KeyV,
        Key::W => CommandKey::KeyW,
        Key::X => CommandKey::KeyX,
        Key::Y => CommandKey::KeyY,
        Key::Z => CommandKey::KeyZ,
        Key::Digit0 => CommandKey::Digit0,
        Key::Digit1 => CommandKey::Digit1,
        Key::Digit2 => CommandKey::Digit2,
        Key::Digit3 => CommandKey::Digit3,
        Key::Digit4 => CommandKey::Digit4,
        Key::Digit5 => CommandKey::Digit5,
        Key::Digit6 => CommandKey::Digit6,
        Key::Digit7 => CommandKey::Digit7,
        Key::Digit8 => CommandKey::Digit8,
        Key::Digit9 => CommandKey::Digit9,
        Key::F1 => CommandKey::F1,
        Key::F2 => CommandKey::F2,
        Key::F3 => CommandKey::F3,
        Key::F4 => CommandKey::F4,
        Key::F5 => CommandKey::F5,
        Key::F6 => CommandKey::F6,
        Key::F7 => CommandKey::F7,
        Key::F8 => CommandKey::F8,
        Key::F9 => CommandKey::F9,
        Key::F10 => CommandKey::F10,
        Key::F11 => CommandKey::F11,
        Key::F12 => CommandKey::F12,
        Key::F13 => CommandKey::F13,
        Key::F14 => CommandKey::F14,
        Key::F15 => CommandKey::F15,
        Key::F16 => CommandKey::F16,
        Key::F17 => CommandKey::F17,
        Key::F18 => CommandKey::F18,
        Key::F19 => CommandKey::F19,
        Key::F20 => CommandKey::F20,
        Key::F21 => CommandKey::F21,
        Key::F22 => CommandKey::F22,
        Key::F23 => CommandKey::F23,
        Key::F24 => CommandKey::F24,
        Key::ArrowUp => CommandKey::ArrowUp,
        Key::ArrowDown => CommandKey::ArrowDown,
        Key::ArrowLeft => CommandKey::ArrowLeft,
        Key::ArrowRight => CommandKey::ArrowRight,
        Key::Home => CommandKey::Home,
        Key::End => CommandKey::End,
        Key::PageUp => CommandKey::PageUp,
        Key::PageDown => CommandKey::PageDown,
        Key::Insert => CommandKey::Insert,
        Key::Delete => CommandKey::Delete,
        Key::Escape => CommandKey::Escape,
        Key::Tab => CommandKey::Tab,
        Key::Space => CommandKey::Space,
        Key::Enter => CommandKey::Enter,
        Key::Backspace => CommandKey::Backspace,
        Key::Backtick => CommandKey::Backquote,
        Key::Minus => CommandKey::Minus,
        Key::Equal => CommandKey::Equal,
        Key::BracketLeft => CommandKey::BracketLeft,
        Key::BracketRight => CommandKey::BracketRight,
        Key::Backslash => CommandKey::Backslash,
        Key::Semicolon => CommandKey::Semicolon,
        Key::Quote => CommandKey::Quote,
        Key::Comma => CommandKey::Comma,
        Key::Period => CommandKey::Period,
        Key::Slash => CommandKey::Slash,
        Key::Numpad0 => CommandKey::Numpad0,
        Key::Numpad1 => CommandKey::Numpad1,
        Key::Numpad2 => CommandKey::Numpad2,
        Key::Numpad3 => CommandKey::Numpad3,
        Key::Numpad4 => CommandKey::Numpad4,
        Key::Numpad5 => CommandKey::Numpad5,
        Key::Numpad6 => CommandKey::Numpad6,
        Key::Numpad7 => CommandKey::Numpad7,
        Key::Numpad8 => CommandKey::Numpad8,
        Key::Numpad9 => CommandKey::Numpad9,
        Key::NumpadAdd => CommandKey::NumpadAdd,
        Key::NumpadSubtract => CommandKey::NumpadSubtract,
        Key::NumpadMultiply => CommandKey::NumpadMultiply,
        Key::NumpadDivide => CommandKey::NumpadDivide,
        Key::NumpadEnter => CommandKey::Enter,
        Key::NumpadDecimal => CommandKey::NumpadDecimal,
        Key::NumLock => CommandKey::NumLock,
        Key::PrintScreen => CommandKey::PrintScreen,
        Key::ScrollLock => CommandKey::ScrollLock,
        Key::Pause => CommandKey::Pause,
        Key::Menu => CommandKey::ContextMenu,
        Key::IntlBackslash => CommandKey::IntlBackslash,
        Key::ShiftLeft
        | Key::ShiftRight
        | Key::ControlLeft
        | Key::ControlRight
        | Key::AltLeft
        | Key::AltRight
        | Key::MetaLeft
        | Key::MetaRight
        | Key::CapsLock
        | Key::Function
        | Key::Unknown(_) => return None,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TriggerCommand;

    #[test]
    fn maps_physical_keys_to_dom_codes() {
        assert_eq!(key_to_command_key(Key::A), Some(CommandKey::KeyA));
        assert_eq!(key_to_command_key(Key::Digit7), Some(CommandKey::Digit7));
        assert_eq!(key_to_command_key(Key::Space), Some(CommandKey::Space));
        assert_eq!(
            key_to_command_key(Key::NumpadEnter),
            Some(CommandKey::Enter)
        );
        assert_eq!(key_to_command_key(Key::Function), None);
        assert!(is_modifier(Key::MetaRight));
    }

    #[test]
    fn macos_maps_command_to_primary_and_keeps_control_explicit() {
        let held = HashSet::from([Key::MetaLeft, Key::ControlRight, Key::ShiftLeft]);
        assert_eq!(
            modifiers_for_platform(&held, ModifierPlatform::MacOs),
            vec![
                TriggerModifier::Primary,
                TriggerModifier::Control,
                TriggerModifier::Shift
            ]
        );
    }

    #[test]
    fn windows_maps_control_to_primary_and_keeps_meta_explicit() {
        let held = HashSet::from([Key::ControlLeft, Key::MetaRight, Key::AltLeft]);
        assert_eq!(
            modifiers_for_platform(&held, ModifierPlatform::WindowsOrLinux),
            vec![
                TriggerModifier::Primary,
                TriggerModifier::Alt,
                TriggerModifier::Meta
            ]
        );
    }

    #[test]
    fn tracks_left_and_right_modifiers_independently() {
        let mut runtime = InputRuntime::default();
        assert_eq!(
            runtime.observe_event(EventKind::KeyDown(Key::ControlLeft)),
            None
        );
        assert_eq!(
            runtime.observe_event(EventKind::KeyDown(Key::ControlRight)),
            None
        );
        runtime.observe_event(EventKind::KeyUp(Key::ControlLeft));
        assert_eq!(
            modifiers_for_platform(&runtime.held_modifiers, ModifierPlatform::WindowsOrLinux),
            vec![TriggerModifier::Primary]
        );
        runtime.observe_event(EventKind::KeyUp(Key::ControlRight));
        assert!(runtime.held_modifiers.is_empty());
    }

    #[test]
    fn key_repeat_is_not_fed_as_a_second_space() {
        let mut runtime = InputRuntime::default();
        let command = TriggerCommand {
            version: 1,
            steps: vec![
                TriggerChord::plain(CommandKey::Space),
                TriggerChord::plain(CommandKey::Space),
            ],
        };
        let timeout = Duration::from_secs(3);

        let first = runtime
            .observe_event(EventKind::KeyDown(Key::Space))
            .and_then(key_to_command_key)
            .unwrap();
        assert!(!runtime.matcher.feed(
            TriggerChord::plain(first),
            &command,
            Duration::ZERO,
            timeout
        ));
        assert_eq!(
            runtime.observe_event(EventKind::KeyRepeat(Key::Space)),
            None
        );
        runtime.observe_event(EventKind::KeyUp(Key::Space));
        let second = runtime
            .observe_event(EventKind::KeyDown(Key::Space))
            .and_then(key_to_command_key)
            .unwrap();
        assert!(runtime.matcher.feed(
            TriggerChord::plain(second),
            &command,
            Duration::from_millis(100),
            timeout
        ));
    }

    #[test]
    fn all_command_sources_share_cooldown_bookkeeping() {
        let mut runtime = InputRuntime::default();
        let cooldown = Duration::from_millis(1_200);

        assert!(runtime.mark_trigger_if_ready(Duration::ZERO, cooldown));
        assert!(!runtime.mark_trigger_if_ready(Duration::from_millis(1_199), cooldown));
        assert!(runtime.mark_trigger_if_ready(Duration::from_millis(1_200), cooldown));
    }
}
