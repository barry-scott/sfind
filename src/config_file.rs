use serde::Deserialize;
use anyhow::Result;

#[derive(Deserialize, Debug)]
pub struct ConfigJson {
    pub folders_to_prune:   Vec<String>,
    pub files_to_prune:     Vec<String>,
}

static default_config_json = String::from(r#"{
    "folders_to_prune": [".svn", ".git", ".hg"],
    "files_to_prune":   ["*~"]
}
"#);

impl ConfigJson{
    pub fn new() -> Result<ConfigJson> {
        Ok(serde_json::from_str(default_config_json)?)
    }
