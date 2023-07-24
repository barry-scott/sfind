use anyhow::{anyhow, Result};
use cfg_if;
use serde;
use serde_json;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

#[derive(serde::Deserialize, Debug)]
pub struct ConfigJson {
    pub folders_to_prune: Vec<String>,
    pub files_to_prune: Vec<String>,
}

#[derive(Debug)]
pub struct AppConfig {
    app_name: String,
    pub config: ConfigJson,
}

static DEFAULT_CONFIG_JSON: &str = r#"{
    "folders_to_prune": [".svn", ".git", ".hg"],
    "files_to_prune":   ["*~"]
}
"#;

impl AppConfig {
    pub fn new(app_name: &str) -> Result<AppConfig> {
        let config_path = config_file_path(app_name)?;

        let config_data = if config_path.exists() {
            fs::read_to_string(&config_path).map_err(|e| {
                anyhow!(
                    "Error reading {} - {}",
                    &config_path.display(),
                    e.to_string()
                )
            })?
        } else {
            DEFAULT_CONFIG_JSON.to_string()
        };

        let app_config = AppConfig {
            app_name: app_name.to_string(),
            config: serde_json::from_str(&config_data).map_err(|e| {
                anyhow!(
                    "Error parsing config {} - {}",
                    &config_path.display(),
                    e.to_string()
                )
            })?,
        };
        Ok(app_config)
    }

    pub fn config_file_path(&self) -> Result<PathBuf> {
        config_file_path(&self.app_name)
    }

    pub fn save_default_config(&self) -> Result<()> {
        let config_path = self.config_file_path()?;

        if config_path.exists() {
            return Err(anyhow!("config file already exists"));
        }

        let mut f = File::create(config_path)?;
        f.write_all(DEFAULT_CONFIG_JSON.as_bytes())?;
        Ok(())
    }
}

cfg_if::cfg_if! {
    if #[cfg(target_os = "macos")] {
        use std::env;

        fn config_file_path(app_name: &str) -> Result<PathBuf> {
            let home_dir = env::var("HOME")?;
            Ok(PathBuf::from(
                format!("{}/Library/Preferences/{}.json", home_dir, &app_name)))
        }

    } else if #[cfg(target_os = "windows")] {
        fn config_file_path(app_name: &str) -> Result<PathBuf> {
            Err(anyhow!("No windows support yet!"))
        }

    } else { // assume not mac and not windows can use xdg standard
        use xdg;

        fn config_file_path(app_name: &str) -> Result<PathBuf> {
            let xdg_dirs = xdg::BaseDirectories::new()?;

            Ok(xdg_dirs.place_config_file(
                    format!("{}.json", &app_name))?)
        }
    }
}
