use crate::error::{AppError, AppResult};
use dirs_next::config_dir;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

const APP_DIR: &str = "Voxta";
const LEGACY_APP_DIR: &str = "LocalType";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InsertionMode {
    Auto,
    Direct,
    Clipboard,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub enabled: bool,
    pub shortcut: String,
    pub microphone_id: String,
    pub model: String,
    pub language: String,
    pub launch_at_login: bool,
    pub play_sounds: bool,
    pub spoken_commands: bool,
    pub trailing_space: bool,
    pub insertion_mode: InsertionMode,
    pub onboarding_complete: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            shortcut: default_shortcut(),
            microphone_id: String::new(),
            model: "base".to_string(),
            language: "auto".to_string(),
            launch_at_login: false,
            play_sounds: true,
            spoken_commands: true,
            trailing_space: true,
            insertion_mode: InsertionMode::Auto,
            onboarding_complete: false,
        }
    }
}

pub fn default_shortcut() -> String {
    "Ctrl+Alt+Space".to_string()
}

impl AppConfig {
    pub fn validate(mut self) -> Self {
        if self.shortcut.trim().is_empty() {
            self.shortcut = default_shortcut();
        }
        if self.model.trim().is_empty() {
            self.model = "base".to_string();
        }
        if self.language.trim().is_empty() {
            self.language = "auto".to_string();
        }
        self
    }
}

#[derive(Debug, Clone)]
pub struct ConfigStore {
    path: PathBuf,
}

impl ConfigStore {
    pub fn new() -> AppResult<Self> {
        let base = config_dir().ok_or_else(|| {
            AppError::Config("could not resolve user configuration directory".to_string())
        })?;
        let path = migrate_config_if_needed(base)?;
        Ok(Self { path })
    }

    #[cfg(test)]
    pub fn from_path(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn load(&self) -> AppResult<AppConfig> {
        if !self.path.exists() {
            let config = AppConfig::default();
            self.save(&config)?;
            return Ok(config);
        }

        let text = fs::read_to_string(&self.path)?;
        match serde_json::from_str::<AppConfig>(&text) {
            Ok(config) => Ok(config.validate()),
            Err(err) => {
                let backup = self.path.with_extension("json.corrupt");
                let _ = fs::copy(&self.path, backup);
                let config = AppConfig::default();
                self.save(&config)?;
                Err(AppError::Config(format!(
                    "configuration was corrupt and has been reset: {err}"
                )))
            }
        }
    }

    pub fn save(&self, config: &AppConfig) -> AppResult<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = serde_json::to_string_pretty(&config.clone().validate())?;
        fs::write(&self.path, text)?;
        Ok(())
    }
}

fn migrate_config_if_needed(base: PathBuf) -> AppResult<PathBuf> {
    let new_dir = base.join(APP_DIR);
    let legacy_path = base.join(LEGACY_APP_DIR).join("config.json");
    let new_path = new_dir.join("config.json");

    fs::create_dir_all(&new_dir)?;
    if !new_path.exists() && legacy_path.exists() {
        fs::copy(&legacy_path, &new_path)?;
    }

    Ok(new_path)
}

#[cfg(test)]
mod tests {
    use super::{AppConfig, ConfigStore};
    use std::fs;

    #[test]
    fn saves_and_loads_config() {
        let dir = tempfile::tempdir().unwrap();
        let store = ConfigStore::from_path(dir.path().join("config.json"));
        let config = AppConfig {
            shortcut: "Ctrl+Alt+D".to_string(),
            ..AppConfig::default()
        };
        store.save(&config).unwrap();
        assert_eq!(store.load().unwrap().shortcut, "Ctrl+Alt+D");
    }

    #[test]
    fn corrupted_config_is_reset_and_reported() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        fs::write(&path, "{not valid json").unwrap();
        let store = ConfigStore::from_path(path.clone());
        assert!(store.load().is_err());
        let recovered =
            serde_json::from_str::<AppConfig>(&fs::read_to_string(path).unwrap()).unwrap();
        assert_eq!(recovered.model, "base");
    }

    #[test]
    fn migrates_legacy_config_when_new_config_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let legacy_dir = dir.path().join(super::LEGACY_APP_DIR);
        fs::create_dir_all(&legacy_dir).unwrap();
        fs::write(
            legacy_dir.join("config.json"),
            serde_json::to_string(&AppConfig {
                shortcut: "Ctrl+Alt+V".to_string(),
                ..AppConfig::default()
            })
            .unwrap(),
        )
        .unwrap();

        let migrated = super::migrate_config_if_needed(dir.path().to_path_buf()).unwrap();
        let config =
            serde_json::from_str::<AppConfig>(&fs::read_to_string(migrated).unwrap()).unwrap();
        assert_eq!(config.shortcut, "Ctrl+Alt+V");
    }
}
