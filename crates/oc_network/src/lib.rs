use derive_more::Constructor;
use oc_geo::{region::WorldRegionIndex, tile::WorldTileIndex};
use oc_individual::network::{Individual, Squad};
use oc_mod::Mod;
use oc_physics::fx::Fx;
use oc_projectile::network::Projectile;
use oc_projectile::spawn::SpawnProjectile;
use oc_root::{WorldConfig, identity::Identity, static_::StaticSource};
use oc_world::{meta::Meta, resume::WorldResume, tile::Tile};
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum ToClient {
    GameConfig(GameConfig),
    WorldResume(WorldResume),
    Individual(Individual),
    Squad(Squad),
    Projectile(Projectile),
    Tiles(WorldRegionIndex, Vec<(WorldTileIndex, Tile)>),
    Fx(Fx),
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
    RequestInit(Identity),
    ListenRegion(WorldRegionIndex),
    ForgotRegion(WorldRegionIndex),
    Refresh,
    SpawnProjectile(SpawnProjectile),
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq, Constructor)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct GameConfig {
    pub w: WorldConfig,
    pub mod_: Mod,
    pub meta: Meta,
    pub static_: StaticSource,
}
