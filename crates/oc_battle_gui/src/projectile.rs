use oc_projectile::spawn::SpawnProjectile;

use crate::ingame::debug::projectile::SpawnProjectileProfile;

pub trait IntoSpawnProjectile {
    fn spawn(&self, start: [f32; 3], end: [f32; 3]) -> SpawnProjectile;
}

impl IntoSpawnProjectile for SpawnProjectileProfile {
    fn spawn(&self, start: [f32; 3], end: [f32; 3]) -> SpawnProjectile {
        let weapon = self.weapon;
        let ammo = self.ammunition;
        let shot = self.shot;
        let repeat = self.repeat;
        SpawnProjectile::new(weapon, ammo, shot, repeat, start, end)
    }
}
