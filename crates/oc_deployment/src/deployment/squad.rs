use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct Squad {
    pub uuid: Uuid,
    pub label: String,
    pub individuals: Vec<super::individual::Individual>,
}

impl Squad {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            label: "".to_string(),
            individuals: vec![],
        }
    }
}
