use derive_more::Constructor;
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
        let position = self.spawn.from;
        // TODO: direction according to from/to
        // TODO: speed according to weapon spec
        let forces = vec![Force::Translation([0.5, 0.5], MetersSeconds(100.0))];

        match specs.inner() {
            oc_mod::projectiles::Projectile::Bullet(_bullet) => {
                Projectile::Bullet(Bullet::new(position, forces))
            }
        }
    }
}
