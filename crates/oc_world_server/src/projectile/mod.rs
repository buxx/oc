use derive_more::Constructor;
use glam::Vec2;
use oc_mod::Mod;
use oc_physics::{Force, MetersSeconds};
use oc_projectile::{Projectile, bullet::Bullet, spawn::SpawnProjectile};

#[derive(Debug, Constructor)]
pub struct Builder<'a> {
    mod_: &'a Mod,
    spawn: SpawnProjectile,
}

impl<'a> Builder<'a> {
    pub fn build(&self) -> Projectile {
        let specs = &self.mod_.projectiles[*self.spawn.i as usize];
        let from = Vec2::from(self.spawn.from);
        let to = Vec2::from(self.spawn.to);
        let position = self.spawn.from;
        let direction = (to - from).normalize_or_zero();
        // TODO: speed according to weapon spec
        // TODO: Calculer angle (3D)
        let forces = vec![Force::Translation(direction.into(), MetersSeconds(100.0))];

        match specs.inner() {
            oc_mod::projectiles::Projectile::Bullet(_bullet) => {
                Projectile::Bullet(Bullet::new(position, forces))
            }
        }
    }
}
