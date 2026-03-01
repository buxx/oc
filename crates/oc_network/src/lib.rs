use oc_geo::region::WorldRegionIndex;
use oc_individual::network::Individual;
use oc_world::meta::Meta;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum ToClient {
    Meta(Meta),
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
    ListenRegion(WorldRegionIndex),
    ForgotRegion(WorldRegionIndex),
    Refresh,
}
