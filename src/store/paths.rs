use std::{path::PathBuf, sync::OnceLock};

static CONFIG_DIR: OnceLock<PathBuf> = OnceLock::new();

const APP_CONFIG_NAME: &str = "com.shank03.somachron";

pub fn config_dir() -> &'static PathBuf {
    CONFIG_DIR.get_or_init(|| {
        dirs::config_dir()
            .expect("failed to determine config directory")
            .join(APP_CONFIG_NAME)
    })
}
