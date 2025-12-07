use octocrab::models::repos::Tag;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

const FOLDER: &str = "data";
const STATE_FILE: &str = "data/state.toml";

#[derive(Deserialize, Serialize, Default)]
pub struct State {
    // owner -> repo -> TagState
    pub repos: HashMap<String, HashMap<String, TagState>>,
}

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct TagState {
    pub latest_tag: Option<TagInfo>,
    pub previous_tag: Option<TagInfo>,
}

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct TagInfo {
    pub name: String,
    pub sha: String,
}

impl State {
    pub fn load() -> Self {
        match fs::read_to_string(STATE_FILE) {
            Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
            Err(_) => State::default(),
        }
    }

    pub fn save(&self) {
        fs::create_dir_all(FOLDER).unwrap();

        let contents = toml::to_string(&self).unwrap();
        fs::write(STATE_FILE, contents).unwrap();
    }

    pub fn repo_mut(&mut self, owner: impl Into<String>, repo: impl Into<String>) -> &mut TagState {
        self.repos
            .entry(owner.into())
            .or_default()
            .entry(repo.into())
            .or_default()
    }
}

impl TagState {
    pub fn swap_with_new(&mut self, new_tag: &Tag) {
        self.previous_tag = self.latest_tag.clone();
        self.latest_tag = Some(TagInfo {
            name: new_tag.name.clone(),
            sha: new_tag.commit.sha.clone(),
        });
    }
}
