use oc_geo::tile::TileXy;
use oc_individual::network::Individual;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum ToClient {
    Hello,
    Individual(Individual),
}

impl From<Individual> for ToClient {
    fn from(value: Individual) -> Self {
        ToClient::Individual(value)
    }
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum ToServer {
    Listen(TileXy, TileXy),
    Refresh,
}
