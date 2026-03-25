use derive_more::Constructor;
use glam::Vec3;
use oc_mod::Mod;
use oc_physics::Force;
use oc_projectile::{Projectile, bullet::Bullet, spawn::SpawnProjectile};

#[derive(Debug, Constructor)]
pub struct Builder<'a> {
    mod_: &'a Mod,
    spawn: SpawnProjectile,
}

impl<'a> Builder<'a> {
    pub fn build(&self) -> Projectile {
        let weapon = self.mod_.weapon(self.spawn.weapon);
        let ammunition = self.mod_.ammunition(self.spawn.ammunition);
        let from = Vec3::from(self.spawn.from);
        let to = Vec3::from(self.spawn.to);
        let position = self.spawn.from;
        let direction = (to - from).normalize_or_zero();
        let speed = weapon.velocity();
        let forces = vec![Force::Translation(direction.into(), speed)];

        // FIXME BS NOW: shot mode burst (to introduce dalayed spawn !)
        match ammunition {
            oc_mod::ammunition::Ammunition::Cartridge(_) => {
                Projectile::Bullet(Bullet::new(position, forces))
            }
        }
    }
}
