use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Xy(pub u64, pub u64);

impl From<Xy> for (u64, u64) {
    fn from(value: Xy) -> Self {
        (value.0, value.1)
    }
}
