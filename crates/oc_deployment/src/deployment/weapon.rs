use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Weapons {
    pub main: Option<String>,
}

impl Weapons {
    pub fn empty() -> Weapons {
        Weapons { main: None }
    }
}
