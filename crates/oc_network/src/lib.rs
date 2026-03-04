use oc_geo::region::WorldRegionIndex;
use oc_individual::network::Individual;
use oc_projectile::network::Projectile;
#[cfg(feature = "debug")]
use oc_projectile::network::SpawnProjectile;
use oc_root::config::Config;
use oc_world::meta::Meta;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum ToClient {
    Config(Config),
    Meta(Meta),
    Individual(Individual),
    Projectile(Projectile),
}

impl From<Individual> for ToClient {
    fn from(value: Individual) -> Self {
        ToClient::Individual(value)
    }
}

impl From<Projectile> for ToClient {
    fn from(value: Projectile) -> Self {
        ToClient::Projectile(value)
    }
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum ToServer {
    ListenRegion(WorldRegionIndex),
    ForgotRegion(WorldRegionIndex),
    Refresh,
    #[cfg(feature = "debug")]
    SpawnProjectile(SpawnProjectile),
}
