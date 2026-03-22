use derive_more::Constructor;
use oc_mod::projectiles::IndexedProjectile;
use oc_projectile::spawn::SpawnProfile;
use oc_root::physics::Meters;

#[derive(Debug, Clone, Constructor)]
pub struct SpawnProjectileProfile {
    pub projectile: IndexedProjectile,
    pub profile: SpawnProfile,
    pub plus_z: Meters,
}
