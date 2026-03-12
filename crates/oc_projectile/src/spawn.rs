use derive_more::Constructor;
use oc_mod::projectiles::ProjectileIndex;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Deserialize, Serialize, Clone, Debug, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct SpawnProfile {
    pub count: u32,
    pub interval_ms: u32,
}

impl Default for SpawnProfile {
    fn default() -> Self {
        Self {
            count: 1,
            interval_ms: 1000,
        }
    }
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq, Constructor)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct SpawnProjectile {
    pub projectile: ProjectileIndex,
    pub profile: SpawnProfile,
    pub from: [f32; 2],
    pub to: [f32; 2],
}
