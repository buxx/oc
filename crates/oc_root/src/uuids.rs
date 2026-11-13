use derive_more::Constructor;
use rkyv::{Archive, Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Default, PartialEq, Archive, Deserialize, Serialize, Constructor)]
pub struct Uuids {
    pub individuals: Vec<Uuid>,
}

impl Uuids {
    pub fn individual(&self, i: usize) -> &Uuid {
        &self.individuals[i]
    }
}
