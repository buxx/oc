use oc_projectile::Projectile;

pub trait ProjectilesGenerator {
    fn projectiles(&self) -> Vec<Projectile>;
}
