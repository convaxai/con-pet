use crate::{matcher::SequenceMatcher, refresh_tray, AppState};
use keytap::{EventKind, Key, Tap};
use serde::Serialize;
use std::{
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

#[derive(Default)]
struct InputRuntime {
    matcher: SequenceMatcher,
    control: bool,
    alt: bool,
    meta: bool,
    last_trigger: Option<Duration>,
}

impl InputRuntime {
    fn handle(&mut self, event: EventKind, app: &AppHandle, state: &AppState, elapsed: Duration) {
        match event {
            EventKind::KeyDown(key) => {
                self.update_modifier(key, true);
                if is_modifier(key) {
                    return;
                }

                let Ok(config) = state.config.read().map(|value| value.clone()) else {
                    return;
                };
                if !config.enabled || state.paused.load(Ordering::Relaxed) {
                    self.matcher.clear();
                    return;
                }
                if self.control || self.alt || self.meta {
                    self.matcher.clear();
                    return;
                }

                if key == Key::Backspace {
                    self.matcher.backspace();
                    return;
                }
                if resets_sequence(key) {
                    self.matcher.clear();
                    return;
                }

                let Some(text) = key_to_ascii(key) else {
                    self.matcher.clear();
                    return;
                };
                let matched = self.matcher.feed_text(
                    text,
                    &config.keyword,
                    elapsed,
                    Duration::from_millis(config.sequence_timeout_ms),
                );
                let cooled_down = self.last_trigger.is_none_or(|last| {
                    elapsed.saturating_sub(last) >= Duration::from_millis(config.cooldown_ms)
                });
                if matched && cooled_down {
                    self.last_trigger = Some(elapsed);
                    state.trigger_count.fetch_add(1, Ordering::Relaxed);
                    let _ = emit_trigger(app, "keyboard");
                }
            }
            EventKind::KeyUp(key) => {
                self.update_modifier(key, false);
            }
            EventKind::KeyRepeat(_) => {}
        }
    }

    fn update_modifier(&mut self, key: Key, pressed: bool) {
        match key {
            Key::ControlLeft | Key::ControlRight => self.control = pressed,
            Key::AltLeft | Key::AltRight => self.alt = pressed,
            Key::MetaLeft | Key::MetaRight => self.meta = pressed,
            _ => {}
        }
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

fn resets_sequence(key: Key) -> bool {
    matches!(
        key,
        Key::Enter | Key::Tab | Key::Space | Key::Escape | Key::Delete
    )
}

fn key_to_ascii(key: Key) -> Option<&'static str> {
    Some(match key {
        Key::A => "a",
        Key::B => "b",
        Key::C => "c",
        Key::D => "d",
        Key::E => "e",
        Key::F => "f",
        Key::G => "g",
        Key::H => "h",
        Key::I => "i",
        Key::J => "j",
        Key::K => "k",
        Key::L => "l",
        Key::M => "m",
        Key::N => "n",
        Key::O => "o",
        Key::P => "p",
        Key::Q => "q",
        Key::R => "r",
        Key::S => "s",
        Key::T => "t",
        Key::U => "u",
        Key::V => "v",
        Key::W => "w",
        Key::X => "x",
        Key::Y => "y",
        Key::Z => "z",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_physical_letter_keys_without_interpreting_text() {
        assert_eq!(key_to_ascii(Key::A), Some("a"));
        assert_eq!(key_to_ascii(Key::Z), Some("z"));
        assert_eq!(key_to_ascii(Key::Space), None);
        assert!(is_modifier(Key::MetaRight));
    }
}
