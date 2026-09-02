use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

const TRIGGER_COMMAND_VERSION: u8 = 1;
const MAX_TRIGGER_STEPS: usize = 64;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct AppConfig {
    pub locale: AppLocale,
    pub enabled: bool,
    /// Read-only compatibility bridge for configs written before triggerCommand.
    #[serde(default = "default_legacy_keyword", skip_serializing)]
    pub keyword: String,
    /// Read-only compatibility bridge for the single Space/Enter trigger format.
    #[serde(default, skip_serializing)]
    pub trigger_key: Option<TriggerKey>,
    /// Field-level `default` distinguishes a missing legacy field from the
    /// command installed by `AppConfig::default()`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_command: Option<TriggerCommand>,
    pub cooldown_ms: u64,
    pub sequence_timeout_ms: u64,
    pub loops: u8,
    pub scale: f64,
    pub position: PositionConfig,
    pub pet_manifest_path: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TriggerCommand {
    #[serde(default = "trigger_command_version")]
    pub version: u8,
    #[serde(default)]
    pub steps: Vec<TriggerChord>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TriggerChord {
    #[serde(default)]
    pub modifiers: Vec<TriggerModifier>,
    pub key: CommandKey,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum TriggerModifier {
    Primary,
    Control,
    Alt,
    Shift,
    Meta,
}

/// Stable physical key identifiers shared with `KeyboardEvent.code` in the UI.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[allow(clippy::enum_variant_names)]
pub enum CommandKey {
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    KeyG,
    KeyH,
    KeyI,
    KeyJ,
    KeyK,
    KeyL,
    KeyM,
    KeyN,
    KeyO,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyT,
    KeyU,
    KeyV,
    KeyW,
    KeyX,
    KeyY,
    KeyZ,
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    Delete,
    Escape,
    Tab,
    Space,
    Enter,
    Backspace,
    Backquote,
    Minus,
    Equal,
    BracketLeft,
    BracketRight,
    Backslash,
    Semicolon,
    Quote,
    Comma,
    Period,
    Slash,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadSubtract,
    NumpadMultiply,
    NumpadDivide,
    NumpadEnter,
    NumpadDecimal,
    NumLock,
    PrintScreen,
    ScrollLock,
    Pause,
    ContextMenu,
    IntlBackslash,
}

impl CommandKey {
    pub fn display_label(self) -> &'static str {
        match self {
            Self::KeyA => "A",
            Self::KeyB => "B",
            Self::KeyC => "C",
            Self::KeyD => "D",
            Self::KeyE => "E",
            Self::KeyF => "F",
            Self::KeyG => "G",
            Self::KeyH => "H",
            Self::KeyI => "I",
            Self::KeyJ => "J",
            Self::KeyK => "K",
            Self::KeyL => "L",
            Self::KeyM => "M",
            Self::KeyN => "N",
            Self::KeyO => "O",
            Self::KeyP => "P",
            Self::KeyQ => "Q",
            Self::KeyR => "R",
            Self::KeyS => "S",
            Self::KeyT => "T",
            Self::KeyU => "U",
            Self::KeyV => "V",
            Self::KeyW => "W",
            Self::KeyX => "X",
            Self::KeyY => "Y",
            Self::KeyZ => "Z",
            Self::Digit0 => "0",
            Self::Digit1 => "1",
            Self::Digit2 => "2",
            Self::Digit3 => "3",
            Self::Digit4 => "4",
            Self::Digit5 => "5",
            Self::Digit6 => "6",
            Self::Digit7 => "7",
            Self::Digit8 => "8",
            Self::Digit9 => "9",
            Self::F1 => "F1",
            Self::F2 => "F2",
            Self::F3 => "F3",
            Self::F4 => "F4",
            Self::F5 => "F5",
            Self::F6 => "F6",
            Self::F7 => "F7",
            Self::F8 => "F8",
            Self::F9 => "F9",
            Self::F10 => "F10",
            Self::F11 => "F11",
            Self::F12 => "F12",
            Self::F13 => "F13",
            Self::F14 => "F14",
            Self::F15 => "F15",
            Self::F16 => "F16",
            Self::F17 => "F17",
            Self::F18 => "F18",
            Self::F19 => "F19",
            Self::F20 => "F20",
            Self::F21 => "F21",
            Self::F22 => "F22",
            Self::F23 => "F23",
            Self::F24 => "F24",
            Self::ArrowUp => "↑",
            Self::ArrowDown => "↓",
            Self::ArrowLeft => "←",
            Self::ArrowRight => "→",
            Self::Home => "Home",
            Self::End => "End",
            Self::PageUp => "PgUp",
            Self::PageDown => "PgDn",
            Self::Insert => "Insert",
            Self::Delete => "Delete",
            Self::Escape => "Esc",
            Self::Tab => "Tab",
            Self::Space => "Space",
            Self::Enter => "Enter",
            Self::Backspace => "Backspace",
            Self::Backquote => "`",
            Self::Minus => "-",
            Self::Equal => "=",
            Self::BracketLeft => "[",
            Self::BracketRight => "]",
            Self::Backslash => "\\",
            Self::Semicolon => ";",
            Self::Quote => "'",
            Self::Comma => ",",
            Self::Period => ".",
            Self::Slash => "/",
            Self::Numpad0 => "Num 0",
            Self::Numpad1 => "Num 1",
            Self::Numpad2 => "Num 2",
            Self::Numpad3 => "Num 3",
            Self::Numpad4 => "Num 4",
            Self::Numpad5 => "Num 5",
            Self::Numpad6 => "Num 6",
            Self::Numpad7 => "Num 7",
            Self::Numpad8 => "Num 8",
            Self::Numpad9 => "Num 9",
            Self::NumpadAdd => "Num +",
            Self::NumpadSubtract => "Num -",
            Self::NumpadMultiply => "Num ×",
            Self::NumpadDivide => "Num ÷",
            Self::NumpadEnter => "Num Enter",
            Self::NumpadDecimal => "Num .",
            Self::NumLock => "Num Lock",
            Self::PrintScreen => "Print Screen",
            Self::ScrollLock => "Scroll Lock",
            Self::Pause => "Pause",
            Self::ContextMenu => "Menu",
            Self::IntlBackslash => "Intl Backslash",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct PositionConfig {
    pub mode: PositionMode,
    pub monitor: MonitorMode,
    pub margin: f64,
    pub fixed_x: f64,
    pub fixed_y: f64,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub enum AppLocale {
    #[serde(rename = "zh-CN")]
    ZhCn,
    #[serde(rename = "en")]
    En,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TriggerKey {
    Space,
    Enter,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum PositionMode {
    Random,
    TopLeft,
    TopCenter,
    TopRight,
    BottomLeft,
    BottomRight,
    Fixed,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum MonitorMode {
    Cursor,
    Primary,
    Random,
}

impl TriggerChord {
    pub fn new(key: CommandKey, mut modifiers: Vec<TriggerModifier>) -> Self {
        modifiers.sort_unstable();
        modifiers.dedup();
        Self { modifiers, key }
    }

    pub fn plain(key: CommandKey) -> Self {
        Self::new(key, Vec::new())
    }

    fn normalize(&mut self) {
        self.modifiers.sort_unstable();
        self.modifiers.dedup();
        // keytap's Windows hook reports both Return keys as Enter. Keep saved
        // commands portable instead of allowing a Windows-unmatchable step.
        if self.key == CommandKey::NumpadEnter {
            self.key = CommandKey::Enter;
        }
    }
}

impl Default for TriggerCommand {
    fn default() -> Self {
        Self {
            version: TRIGGER_COMMAND_VERSION,
            steps: vec![
                TriggerChord::plain(CommandKey::KeyM),
                TriggerChord::plain(CommandKey::KeyJ),
            ],
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            locale: AppLocale::ZhCn,
            enabled: true,
            keyword: String::new(),
            trigger_key: None,
            trigger_command: Some(TriggerCommand::default()),
            cooldown_ms: 1_200,
            sequence_timeout_ms: 3_000,
            loops: 2,
            scale: 1.0,
            position: PositionConfig::default(),
            pet_manifest_path: None,
        }
    }
}

impl Default for PositionConfig {
    fn default() -> Self {
        Self {
            mode: PositionMode::Random,
            monitor: MonitorMode::Cursor,
            margin: 24.0,
            fixed_x: 40.0,
            fixed_y: 40.0,
        }
    }
}

impl AppConfig {
    pub fn normalize_and_validate(mut self) -> Result<Self, String> {
        let english = self.locale == AppLocale::En;
        let mut command = self
            .trigger_command
            .take()
            .unwrap_or_else(|| migrate_legacy_command(self.trigger_key, &self.keyword));

        // Once normalized, legacy fields cannot take part in future saves or matching.
        self.keyword.clear();
        self.trigger_key = None;

        if command.version != TRIGGER_COMMAND_VERSION {
            return Err(if english {
                "Unsupported trigger command version"
            } else {
                "不支持的触发指令版本"
            }
            .into());
        }
        if command.steps.is_empty() || command.steps.len() > MAX_TRIGGER_STEPS {
            return Err(if english {
                "Trigger command must contain 1–64 steps"
            } else {
                "触发指令必须包含 1–64 个步骤"
            }
            .into());
        }
        for step in &mut command.steps {
            step.normalize();
        }
        self.trigger_command = Some(command);

        if !(250..=60_000).contains(&self.cooldown_ms) {
            return Err(if english {
                "Cooldown must be between 250 and 60000 ms"
            } else {
                "冷却时间必须在 250–60000ms 之间"
            }
            .into());
        }
        if !(500..=30_000).contains(&self.sequence_timeout_ms) {
            return Err(if english {
                "Typing timeout must be between 500 and 30000 ms"
            } else {
                "连续输入超时必须在 500–30000ms 之间"
            }
            .into());
        }
        if !(1..=10).contains(&self.loops) {
            return Err(if english {
                "Animation loops must be between 1 and 10"
            } else {
                "动画循环次数必须在 1–10 之间"
            }
            .into());
        }
        if !(0.4..=2.5).contains(&self.scale) {
            return Err(if english {
                "Animation size must be between 0.4 and 2.5"
            } else {
                "动画缩放必须在 0.4–2.5 之间"
            }
            .into());
        }
        if !(0.0..=400.0).contains(&self.position.margin) {
            return Err(if english {
                "Screen margin must be between 0 and 400"
            } else {
                "屏幕边距必须在 0–400 之间"
            }
            .into());
        }
        Ok(self)
    }
}

fn trigger_command_version() -> u8 {
    TRIGGER_COMMAND_VERSION
}

fn default_legacy_keyword() -> String {
    "mj".into()
}

fn migrate_legacy_command(trigger_key: Option<TriggerKey>, keyword: &str) -> TriggerCommand {
    if let Some(key) = trigger_key {
        return TriggerCommand {
            version: TRIGGER_COMMAND_VERSION,
            steps: vec![TriggerChord::plain(match key {
                TriggerKey::Space => CommandKey::Space,
                TriggerKey::Enter => CommandKey::Enter,
            })],
        };
    }

    let trimmed = keyword.trim();
    let steps = (1..=MAX_TRIGGER_STEPS)
        .contains(&trimmed.chars().count())
        .then(|| {
            trimmed
                .chars()
                .map(legacy_character_to_key)
                .collect::<Option<Vec<_>>>()
        })
        .flatten();

    steps
        .map(|steps| TriggerCommand {
            version: TRIGGER_COMMAND_VERSION,
            steps: steps.into_iter().map(TriggerChord::plain).collect(),
        })
        // Legacy text could contain Unicode, while global keytap exposes physical
        // keys. Falling back is deterministic and avoids persisting an unmatchable
        // command.
        .unwrap_or_default()
}

fn legacy_character_to_key(character: char) -> Option<CommandKey> {
    Some(match character.to_ascii_lowercase() {
        'a' => CommandKey::KeyA,
        'b' => CommandKey::KeyB,
        'c' => CommandKey::KeyC,
        'd' => CommandKey::KeyD,
        'e' => CommandKey::KeyE,
        'f' => CommandKey::KeyF,
        'g' => CommandKey::KeyG,
        'h' => CommandKey::KeyH,
        'i' => CommandKey::KeyI,
        'j' => CommandKey::KeyJ,
        'k' => CommandKey::KeyK,
        'l' => CommandKey::KeyL,
        'm' => CommandKey::KeyM,
        'n' => CommandKey::KeyN,
        'o' => CommandKey::KeyO,
        'p' => CommandKey::KeyP,
        'q' => CommandKey::KeyQ,
        'r' => CommandKey::KeyR,
        's' => CommandKey::KeyS,
        't' => CommandKey::KeyT,
        'u' => CommandKey::KeyU,
        'v' => CommandKey::KeyV,
        'w' => CommandKey::KeyW,
        'x' => CommandKey::KeyX,
        'y' => CommandKey::KeyY,
        'z' => CommandKey::KeyZ,
        '0' => CommandKey::Digit0,
        '1' => CommandKey::Digit1,
        '2' => CommandKey::Digit2,
        '3' => CommandKey::Digit3,
        '4' => CommandKey::Digit4,
        '5' => CommandKey::Digit5,
        '6' => CommandKey::Digit6,
        '7' => CommandKey::Digit7,
        '8' => CommandKey::Digit8,
        '9' => CommandKey::Digit9,
        ' ' => CommandKey::Space,
        '\t' => CommandKey::Tab,
        '\n' | '\r' => CommandKey::Enter,
        '-' => CommandKey::Minus,
        '=' => CommandKey::Equal,
        '[' => CommandKey::BracketLeft,
        ']' => CommandKey::BracketRight,
        '\\' => CommandKey::Backslash,
        ';' => CommandKey::Semicolon,
        '\'' => CommandKey::Quote,
        ',' => CommandKey::Comma,
        '.' => CommandKey::Period,
        '/' => CommandKey::Slash,
        '`' => CommandKey::Backquote,
        _ => return None,
    })
}

pub fn load(path: &Path) -> AppConfig {
    let Ok(raw) = fs::read_to_string(path) else {
        return AppConfig::default();
    };
    serde_json::from_str::<AppConfig>(&raw)
        .ok()
        .and_then(|config| config.normalize_and_validate().ok())
        .unwrap_or_default()
}

pub fn save(path: &Path, config: &AppConfig) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let temporary = temporary_path(path);
    let raw = serde_json::to_vec_pretty(config).map_err(|error| error.to_string())?;
    fs::write(&temporary, raw).map_err(|error| error.to_string())?;
    fs::rename(&temporary, path).map_err(|error| error.to_string())
}

fn temporary_path(path: &Path) -> PathBuf {
    let mut value = path.as_os_str().to_owned();
    value.push(".tmp");
    PathBuf::from(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn command(config: &AppConfig) -> &TriggerCommand {
        config.trigger_command.as_ref().unwrap()
    }

    #[test]
    fn defaults_to_an_m_then_j_command() {
        let config = AppConfig::default().normalize_and_validate().unwrap();
        assert_eq!(
            command(&config).steps,
            vec![
                TriggerChord::plain(CommandKey::KeyM),
                TriggerChord::plain(CommandKey::KeyJ)
            ]
        );
        assert_eq!(config.keyword, "");
        assert_eq!(config.trigger_key, None);
        assert_eq!(config.pet_manifest_path, None);
    }

    #[test]
    fn serializes_only_the_versioned_command_shape() {
        let config = AppConfig::default().normalize_and_validate().unwrap();
        let raw = serde_json::to_value(config).unwrap();

        assert!(raw.get("keyword").is_none());
        assert!(raw.get("triggerKey").is_none());
        assert_eq!(raw["triggerCommand"]["version"], 1);
        assert_eq!(raw["triggerCommand"]["steps"][0]["key"], "KeyM");
        assert_eq!(
            raw["triggerCommand"]["steps"][0]["modifiers"],
            serde_json::json!([])
        );
    }

    #[test]
    fn a_missing_command_field_remains_distinguishable_until_migration() {
        let config: AppConfig = serde_json::from_str(r#"{"keyword":"mj"}"#).unwrap();
        assert_eq!(config.trigger_command, None);
    }

    #[test]
    fn migrates_the_legacy_mj_keyword() {
        let config = serde_json::from_str::<AppConfig>(r#"{"keyword":"MJ"}"#)
            .unwrap()
            .normalize_and_validate()
            .unwrap();

        assert_eq!(
            command(&config).steps,
            vec![
                TriggerChord::plain(CommandKey::KeyM),
                TriggerChord::plain(CommandKey::KeyJ)
            ]
        );
        assert_eq!(config.keyword, "");
    }

    #[test]
    fn migrates_legacy_space_and_enter_triggers() {
        for (legacy, expected) in [("space", CommandKey::Space), ("enter", CommandKey::Enter)] {
            let config = serde_json::from_str::<AppConfig>(&format!(
                r#"{{"keyword":"stale","triggerKey":"{legacy}"}}"#
            ))
            .unwrap()
            .normalize_and_validate()
            .unwrap();
            assert_eq!(command(&config).steps, vec![TriggerChord::plain(expected)]);
            assert_eq!(config.trigger_key, None);
        }
    }

    #[test]
    fn explicit_new_command_takes_priority_over_legacy_fields() {
        let config = serde_json::from_str::<AppConfig>(
            r#"{
                "keyword":"mj",
                "triggerKey":"space",
                "triggerCommand":{
                    "version":1,
                    "steps":[{"modifiers":["primary"],"key":"KeyC"}]
                }
            }"#,
        )
        .unwrap()
        .normalize_and_validate()
        .unwrap();

        assert_eq!(
            command(&config).steps,
            vec![TriggerChord::new(
                CommandKey::KeyC,
                vec![TriggerModifier::Primary]
            )]
        );
    }

    #[test]
    fn unrepresentable_legacy_unicode_falls_back_to_default_command() {
        let config = serde_json::from_str::<AppConfig>(r#"{"keyword":"你好"}"#)
            .unwrap()
            .normalize_and_validate()
            .unwrap();
        assert_eq!(command(&config), &TriggerCommand::default());
    }

    #[test]
    fn normalizes_modifier_order_and_duplicates() {
        let config = AppConfig {
            trigger_command: Some(TriggerCommand {
                version: 1,
                steps: vec![TriggerChord {
                    modifiers: vec![
                        TriggerModifier::Shift,
                        TriggerModifier::Primary,
                        TriggerModifier::Shift,
                        TriggerModifier::Alt,
                    ],
                    key: CommandKey::KeyC,
                }],
            }),
            ..Default::default()
        }
        .normalize_and_validate()
        .unwrap();

        assert_eq!(
            command(&config).steps[0].modifiers,
            vec![
                TriggerModifier::Primary,
                TriggerModifier::Alt,
                TriggerModifier::Shift
            ]
        );
    }

    #[test]
    fn normalizes_numpad_enter_to_portable_enter() {
        let config = AppConfig {
            trigger_command: Some(TriggerCommand {
                version: 1,
                steps: vec![TriggerChord::plain(CommandKey::NumpadEnter)],
            }),
            ..Default::default()
        }
        .normalize_and_validate()
        .unwrap();

        assert_eq!(
            command(&config).steps,
            vec![TriggerChord::plain(CommandKey::Enter)]
        );
    }

    #[test]
    fn rejects_unknown_command_version_and_empty_or_long_commands() {
        let invalid_version = AppConfig {
            trigger_command: Some(TriggerCommand {
                version: 2,
                steps: vec![TriggerChord::plain(CommandKey::KeyM)],
            }),
            ..Default::default()
        };
        assert!(invalid_version.normalize_and_validate().is_err());

        for steps in [Vec::new(), vec![TriggerChord::plain(CommandKey::KeyM); 65]] {
            let config = AppConfig {
                trigger_command: Some(TriggerCommand { version: 1, steps }),
                ..Default::default()
            };
            assert!(config.normalize_and_validate().is_err());
        }
    }
}
