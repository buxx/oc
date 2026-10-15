use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::deployment::{magazine::Magazine, weapon::Weapons};

#[derive(Debug, Deserialize, Serialize)]
pub struct Individual {
    pub uuid: Uuid,
    pub illustration: String,
    #[serde(default = "default_xp")]
    pub xp: u8,
    pub weapons: Weapons,
    pub magazines: Vec<Magazine>,
}

impl Default for Individual {
    fn default() -> Self {
        Self::new()
    }
}

impl Individual {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            illustration: "default".to_string(),
            xp: default_xp(),
            weapons: Weapons::empty(),
            magazines: vec![],
        }
    }
}

fn default_xp() -> u8 {
    128
}
