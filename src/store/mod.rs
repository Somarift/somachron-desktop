use std::{collections::HashMap, fs::File, io::Read};

use gpui::{App, Global};
use serde::{Deserialize, Serialize};

mod paths;

const APP_SETTINGS: &str = "settings.json";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cookie {
    pub value: String,
    pub expires: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Store {
    #[serde(default)]
    pub cookies: HashMap<String, Cookie>,
    pub client_id: Option<String>,
    pub session_id: Option<String>,
}

impl Global for Store {}

impl Store {
    pub fn global_get(cx: &mut App) -> &Self {
        cx.global()
    }

    pub fn global_mut(cx: &mut App) -> &mut Self {
        cx.global_mut()
    }

    pub fn load() -> Self {
        let config_path = paths::config_dir().join(APP_SETTINGS);
        let data = if !config_path.exists() {
            File::create(&config_path).expect("Failed to create app settings file");
            serde_json::json!({})
        } else {
            let mut file = File::open(&config_path).expect("Failed to open app settings file");
            let mut buf = Vec::with_capacity(4096);
            file.read_to_end(&mut buf).expect("Failed to read file");

            if buf.is_empty() {
                serde_json::json!({})
            } else {
                serde_json::from_slice(&buf).expect("Failed to parse settings file")
            }
        };

        serde_json::from_value(data).expect("Failed to parse settings")
    }

    pub fn save(&self) {
        let config_path = paths::config_dir().join(APP_SETTINGS);
        let file = File::create(&config_path).expect("Failed to open app settings file");
        serde_json::to_writer(file, &self).expect("Failed to save settings to file")
    }
}
