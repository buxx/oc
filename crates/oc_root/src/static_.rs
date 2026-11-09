use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum StaticSource {
    /// Files must be downloaded from the http static server of this port
    Remote(u16),
    /// Server and client share files
    Local,
}
