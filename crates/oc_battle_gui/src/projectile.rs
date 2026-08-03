use oc_projectile::spawn::SpawnProjectile;
use oc_root::geo::WorldVec3;

use crate::ingame::debug::projectile::SpawnProjectileProfile;

pub trait IntoSpawnProjectile {
    fn spawn(&self, start: WorldVec3, end: WorldVec3) -> SpawnProjectile;
}

impl IntoSpawnProjectile for SpawnProjectileProfile {
    fn spawn(&self, start: WorldVec3, end: WorldVec3) -> SpawnProjectile {
        let weapon = self.weapon;
        let ammo = self.ammunition;
        let shot = self.shot;
        let repeat = self.repeat;
        SpawnProjectile::new(weapon, ammo, shot, repeat, start, end)
    }
}
