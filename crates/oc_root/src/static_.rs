use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum StaticSource {
    Remote(u16), // Http port of static server
    Local { mod_: String, map: String },
}
