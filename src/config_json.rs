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
    pub config_path: PathBuf,
    pub config: ConfigJson,
}

static DEFAULT_CONFIG_JSON: &str = r#"{
    "folders_to_prune": [".svn", ".git", ".hg", "target"],
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
            config_path: config_path.clone(),
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
            return Err(anyhow!(
                "config file already exists: {}",
                config_path.display()
            ));
        }

        println!("Saving default config in {}", config_path.display());
        let mut f = File::create(config_path)?;
        f.write_all(DEFAULT_CONFIG_JSON.as_bytes())?;
        Ok(())
    }

    pub fn show_config(&self) {
        println!(
            "smart find configuration from {}",
            self.config_file_path().unwrap().display()
        );
        println!("folders to prune:");
        for folder in &self.config.folders_to_prune {
            println!("    {}", folder);
        }
        println!("files to prune:");
        for filename in &self.config.files_to_prune {
            println!("    {}", filename);
        }
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
        use windows_sys::{
            Win32::UI::Shell::*,
            Win32::Foundation::*,
        };

        fn get_appdata_folder() -> Result<PathBuf> {
            let mut path_buf = [0u16; MAX_PATH as usize];
            let hresult = unsafe {
                SHGetFolderPathW(
                    0,
                    CSIDL_APPDATA as i32,
                    0,
                    SHGFP_TYPE_CURRENT as u32,
                    path_buf.as_mut_ptr())
            };
            if hresult != S_OK {
                Err(anyhow!("SHGetFolderPathW failed {}", hresult))
            } else {
                use std::ffi::OsString;
                use std::os::windows::ffi::OsStringExt as _;
                let Some(length) = path_buf.iter().position(|&ch| ch == 0)
                    else { return Err(anyhow!("missing 0 in SHGetFolderPathW buffer")); };
                Ok(OsString::from_wide(&path_buf[0..length]).into())
            }
        }

        fn config_file_path(app_name: &str) -> Result<PathBuf> {
            let mut path = PathBuf::from(get_appdata_folder()?);
            path.push(format!("{}.json", app_name));
            Ok(path)
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
