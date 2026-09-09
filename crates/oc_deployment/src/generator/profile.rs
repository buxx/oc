use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Profile {
    pub squads: Vec<Squad>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Squad {
    #[serde(default = "default_count")]
    pub count: usize,
    pub label: String,
    pub individuals: Vec<Individual>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Individual {
    #[serde(default = "default_count")]
    pub count: usize,
    pub weapons: Weapons,
    pub magazines: Vec<Magazine>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Weapons {
    pub main: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Magazine {
    #[serde(default = "default_count")]
    pub count: usize,
    pub name: String,
}

fn default_count() -> usize {
    1
}
