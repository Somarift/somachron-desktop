use std::{fs, path::PathBuf, sync::OnceLock};

static CONFIG_DIR: OnceLock<PathBuf> = OnceLock::new();

const APP_CONFIG_NAME: &str = "com.shank03.somachron";

pub(super) fn config_dir() -> &'static PathBuf {
    CONFIG_DIR.get_or_init(|| {
        dirs::config_dir()
            .expect("failed to determine config directory")
            .join(APP_CONFIG_NAME)
    })
}

pub fn cache_dir() -> Result<PathBuf, std::io::Error> {
    let config_dir = config_dir();
    let cache_dir = config_dir.join("cache");

    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir)?;
    }
    Ok(cache_dir)
}
