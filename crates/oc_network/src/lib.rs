use derive_more::Constructor;
use oc_geo::{region::WorldRegionIndex, tile::WorldTileIndex};
use oc_individual::{
    IndividualIndex,
    network::{Individual, Squad},
    order::{Order, OrderIndex},
    squad::SquadIndex,
};
use oc_mod::Mod;
use oc_physics::fx::Fx;
use oc_projectile::network::Projectile;
use oc_projectile::spawn::SpawnProjectiles;
#[cfg(feature = "debug")]
use oc_root::geo::WorldVec3;
use oc_root::{WorldConfig, geo::WorldVec2, identity::Identity, static_::StaticSource};
use oc_world::{meta::Meta, resume::WorldResume, tile::Tile, visibility::Visibility};
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
    UpdateVisibilities(Vec<(IndividualIndex, IndividualIndex, Visibility)>),
    #[cfg(feature = "debug")]
    Debug(Debug),
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
    ExplodeProjectile(SpawnProjectiles),
    Squad(SquadIndex, SquadMessage),
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq, Constructor)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct GameConfig {
    pub w: WorldConfig,
    pub mod_: Mod,
    pub meta: Meta,
    pub static_: StaticSource,
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum SquadMessage {
    SetOrders(Vec<Order>),
    SetPositionOrderPosition(OrderIndex, WorldVec2),
}

#[cfg(feature = "debug")]
#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Debug {
    Collision(WorldVec3),
}
