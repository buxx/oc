use std::ops::Deref;

use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Height(pub u8);

impl Deref for Height {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
