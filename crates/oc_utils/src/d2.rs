use oc_root::WORLD_WIDTH;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Xy(pub u64, pub u64);

#[derive(Debug, Clone, Copy)]
pub struct XyIndex(pub usize);

impl From<XyIndex> for Xy {
    fn from(XyIndex(i): XyIndex) -> Self {
        let x = i % WORLD_WIDTH;
        let y = i / WORLD_WIDTH;
        Self(x as u64, y as u64)
    }
}

impl From<Xy> for XyIndex {
    fn from(Xy(x, y): Xy) -> Self {
        Self(y as usize * WORLD_WIDTH + x as usize)
    }
}

impl From<Xy> for (u64, u64) {
    fn from(value: Xy) -> Self {
        (value.0, value.1)
    }
}
