use octocrab::models::repos::Tag;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Deserialize, Serialize, Default)]
pub struct State {
    // owner -> repo -> TagState
    pub repos: HashMap<String, HashMap<String, TagState>>,
}

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct TagState {
    pub latest_tag: Option<String>,
    pub previous_tag: Option<String>,
}

impl State {
    pub fn load(location: &String) -> Self {
        match fs::read_to_string(format!("{location}/state.toml")) {
            Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
            Err(_) => State::default(),
        }
    }

    pub fn save(&self, location: &String) {
        let contents = toml::to_string(&self).unwrap();
        fs::write(format!("{location}/state.toml"), contents).unwrap();
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
        self.latest_tag = Some(new_tag.name.clone());
    }
}
