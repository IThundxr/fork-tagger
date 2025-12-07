use config::ConfigError;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
pub struct Config {
    pub github_token: String,
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
    pub fn load() -> Result<Self, ConfigError> {
        let config = config::Config::builder()
            .add_source(config::File::with_name("data/config"))
            .add_source(config::Environment::with_prefix("APP"))
            .build()?
            .try_deserialize()?;

        Ok(config)
    }
}
