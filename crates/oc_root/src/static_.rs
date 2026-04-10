use std::path::PathBuf;

use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum StaticSource {
    Remote(u16), // Http port of static server
    Local { mod_: String, world: String },
}
impl StaticSource {
    pub fn cache(&self) -> PathBuf {
        match self {
            StaticSource::Remote(_) => PathBuf::from("cache"),
            StaticSource::Local { mod_: _, world: _ } => PathBuf::from("cache_"),
        }
    }
}
