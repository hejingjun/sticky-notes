use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsConfig {
    pub penetrate: String,
    pub auto_purge: bool,
    pub opacity: f64,
    pub theme: String,
    pub webdav_url: String,
    pub webdav_user: String,
    pub webdav_password: String,
}

impl Default for SettingsConfig {
    fn default() -> Self {
        Self {
            penetrate: "Ctrl+Alt+Shift+P".into(),
            auto_purge: true,
            opacity: 0.6,
            theme: "green".into(),
            webdav_url: String::new(),
            webdav_user: String::new(),
            webdav_password: String::new(),
        }
    }
}

pub struct SettingsManager {
    config: SettingsConfig,
    path: PathBuf,
}

impl SettingsManager {
    pub fn new(app_dir: &PathBuf) -> Self {
        let path = app_dir.join("shortcuts.json");
        let config = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        if !path.exists() {
            let _ = std::fs::write(&path, serde_json::to_string_pretty(&config).unwrap());
        }
        Self { config, path }
    }

    fn save(&self) {
        if let Err(e) = std::fs::write(&self.path, serde_json::to_string_pretty(&self.config).unwrap()) {
            eprintln!("[sticky-notes] 设置写入失败: {e}");
        }
    }

    pub fn get_config(&self) -> &SettingsConfig { &self.config }

    pub fn update_penetrate(&mut self, accelerator: &str) {
        self.config.penetrate = accelerator.to_string();
        self.save();
    }

    pub fn update_auto_purge(&mut self, v: bool) {
        self.config.auto_purge = v;
        self.save();
    }

    pub fn update_opacity(&mut self, v: f64) {
        self.config.opacity = v;
        self.save();
    }

    pub fn update_webdav(&mut self, url: &str, user: &str, password: &str) {
        self.config.webdav_url = url.to_string();
        self.config.webdav_user = user.to_string();
        self.config.webdav_password = password.to_string();
        self.save();
    }

    pub fn update_theme(&mut self, theme: &str) {
        self.config.theme = theme.to_string();
        self.save();
    }
}
