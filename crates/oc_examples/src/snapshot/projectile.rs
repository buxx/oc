use oc_projectile::Projectile;
use oc_root::WorldConfig;

pub trait ProjectilesGenerator {
    fn projectiles(&self, w: &WorldConfig) -> Vec<Projectile>;
}
