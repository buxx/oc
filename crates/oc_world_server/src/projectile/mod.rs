use derive_more::Constructor;
use glam::Vec3;
use oc_mod::Mod;
use oc_physics::Force;
use oc_projectile::{Projectile, bullet::Bullet, spawn::SpawnProjectile};
use oc_root::physics::{Meters, MetersSeconds};

#[derive(Debug, Constructor)]
pub struct Builder<'a> {
    mod_: &'a Mod,
    spawn: SpawnProjectile,
}

impl<'a> Builder<'a> {
    pub fn build(&self) -> Projectile {
        let specs = &self.mod_.projectiles[*self.spawn.i as usize];
        let from = Vec3::from(self.spawn.from);
        let to = Vec3::from(self.spawn.to);
        let position = self.spawn.from;
        let direction = (to - from).normalize_or_zero();
        // TODO: speed according to weapon spec
        // TODO: Calculer angle (3D)
        let speed = MetersSeconds(Meters(100.0));
        let forces = vec![Force::Translation(direction.into(), speed)];

        match specs.inner() {
            oc_mod::projectiles::Projectile::Bullet(_bullet) => {
                Projectile::Bullet(Bullet::new(position, forces))
            }
        }
    }
}
