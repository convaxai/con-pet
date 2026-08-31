use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct AppConfig {
    pub locale: AppLocale,
    pub enabled: bool,
    pub keyword: String,
    pub cooldown_ms: u64,
    pub sequence_timeout_ms: u64,
    pub loops: u8,
    pub scale: f64,
    pub position: PositionConfig,
    pub pet_manifest_path: Option<String>,
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

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            locale: AppLocale::ZhCn,
            enabled: true,
            keyword: "codex".into(),
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
        self.keyword = self.keyword.trim().to_lowercase();
        let keyword_len = self.keyword.chars().count();
        if keyword_len == 0 || keyword_len > 64 {
            return Err(if english {
                "Keyword must contain 1–64 characters"
            } else {
                "关键词长度必须为 1–64 个字符"
            }
            .into());
        }
        if self.keyword.chars().any(char::is_control) {
            return Err(if english {
                "Keyword cannot contain control characters"
            } else {
                "关键词不能包含控制字符"
            }
            .into());
        }
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

    #[test]
    fn defaults_are_valid() {
        assert!(AppConfig::default().normalize_and_validate().is_ok());
    }

    #[test]
    fn unicode_keyword_is_preserved_and_normalized() {
        let config = AppConfig {
            keyword: " 你好Codex ".into(),
            ..Default::default()
        }
        .normalize_and_validate()
        .unwrap();
        assert_eq!(config.keyword, "你好codex");
    }

    #[test]
    fn legacy_config_defaults_to_chinese_locale() {
        let config: AppConfig =
            serde_json::from_str(r#"{"keyword":"mj","animation":"failed"}"#).unwrap();
        assert_eq!(config.locale, AppLocale::ZhCn);
        assert_eq!(config.keyword, "mj");
        assert!(!serde_json::to_string(&config)
            .unwrap()
            .contains("animation"));
    }
}
