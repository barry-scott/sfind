use serde;
use serde_json;

#[derive(serde::Deserialize, Debug)]
pub struct ConfigJson {
    pub folders_to_prune:   Vec<String>,
    pub files_to_prune:     Vec<String>,
}

static DEFAULT_CONFIG_JSON: &str = r#"{
    "folders_to_prune": [".svn", ".git", ".hg"],
    "files_to_prune":   ["*~"]
}
"#;

impl ConfigJson {
    pub fn new() -> Result<ConfigJson, String> {
        let config: ConfigJson = serde_json::from_str(DEFAULT_CONFIG_JSON).unwrap();
        Ok(config)
    }
}

#[cfg(target_os = "macos")]
fn macos_only() {
  // ...
}
