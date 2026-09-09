use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::deployment::{magazine::Magazine, weapon::Weapons};

#[derive(Debug, Deserialize, Serialize)]
pub struct Individual {
    pub uuid: Uuid,
    pub weapons: Weapons,
    pub magazines: Vec<Magazine>,
}

impl Individual {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            weapons: Weapons::empty(),
            magazines: vec![],
        }
    }
}
