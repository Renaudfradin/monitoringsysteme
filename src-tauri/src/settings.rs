//! Persistent application settings (theme, alerts, launch-at-login).

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::error::MetricError;
use crate::models::AppSettings;

pub struct SettingsStore {
    path: Mutex<Option<PathBuf>>,
    settings: Mutex<AppSettings>,
}

impl SettingsStore {
    pub fn new() -> Self {
        Self {
            path: Mutex::new(None),
            settings: Mutex::new(AppSettings::default()),
        }
    }

    pub fn init(&self, path: PathBuf) {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(loaded) = serde_json::from_str::<AppSettings>(&data) {
                *self.settings.lock().unwrap_or_else(|e| e.into_inner()) = loaded;
            }
        }
        *self.path.lock().unwrap_or_else(|e| e.into_inner()) = Some(path);
    }

    pub fn get(&self) -> AppSettings {
        self.settings
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    pub fn set(&self, settings: AppSettings) -> Result<AppSettings, MetricError> {
        {
            *self.settings.lock().unwrap_or_else(|e| e.into_inner()) = settings.clone();
        }
        self.persist()?;
        Ok(settings)
    }

    fn persist(&self) -> Result<(), MetricError> {
        let path = self
            .path
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
            .ok_or_else(|| MetricError::Internal("settings path not initialized".into()))?;
        let settings = self.get();
        let json = serde_json::to_string_pretty(&settings)
            .map_err(|e| MetricError::Internal(e.to_string()))?;
        fs::write(path, json).map_err(|e| MetricError::Internal(e.to_string()))
    }
}

impl Default for SettingsStore {
    fn default() -> Self {
        Self::new()
    }
}
