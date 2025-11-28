use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const FOLDER: &str = "data";
const CONFIG_FILE: &str = "data/config.toml";

#[derive(Deserialize, Serialize, Default)]
pub struct Config {
    pub entries: Vec<Entry>,
}

#[derive(Deserialize, Serialize)]
pub struct Entry {
    pub upstream_owner: String,
    pub upstream_repo: String,
    #[serde(default = "default_branch")]
    pub upstream_branch: String,
    pub fork_owner: String,
    pub fork_repo: String,
    #[serde(default = "default_branch")]
    pub fork_branch: String,
}

fn default_branch() -> String {
    "main".into()
}

impl Config {
    pub fn load() -> Self {
        match fs::read_to_string(CONFIG_FILE) {
            Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
            Err(_) => {
                let default = Config::default();
                default.save();
                default
            },
        }
    }

    pub fn save(&self) {
        fs::create_dir_all(FOLDER).unwrap();

        let contents = toml::to_string(&self).unwrap();
        fs::write(CONFIG_FILE, contents).unwrap();
    }
}
