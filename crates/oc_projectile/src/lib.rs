use oc_geo::{
    Geo, UpdateGeo,
    region::{Region, RegionXy},
    tile::TileXy,
};
use oc_physics::{
    Force, Physic, UpdatePhysic,
    collision::{Material, Materials},
};
use rkyv::{Archive, Deserialize, Serialize};

use crate::bullet::Bullet;

pub mod bullet;
pub mod network;

#[derive(Archive, Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct ProjectileId(pub u64);

#[derive(Archive, Deserialize, Serialize, Clone, Debug, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Projectile {
    Bullet(Bullet),
}

// TODO: seems lot of common code with individual. Think to refactor that
#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Update {
    UpdatePosition([f32; 2]),
    UpdateTile(TileXy),
    UpdateRegion(RegionXy),
    PushForce(Force),
    RemoveForce(Force),
}

impl Projectile {
    pub fn position(&self) -> &[f32; 2] {
        match self {
            Projectile::Bullet(bullet) => &bullet.position,
        }
    }

    pub fn tile(&self) -> &TileXy {
        match self {
            Projectile::Bullet(bullet) => &bullet.tile,
        }
    }
}

impl Region for Projectile {
    fn region(&self) -> &RegionXy {
        match self {
            Projectile::Bullet(bullet) => &bullet.region,
        }
    }

    fn set_region(&mut self, value: RegionXy) {
        match self {
            Projectile::Bullet(bullet) => bullet.region = value,
        }
    }
}

impl Physic for Projectile {
    fn position(&self) -> &[f32; 2] {
        match self {
            Projectile::Bullet(bullet) => &bullet.position,
        }
    }

    fn forces(&self) -> &Vec<Force> {
        match self {
            Projectile::Bullet(bullet) => &bullet.forces,
        }
    }
}

impl UpdatePhysic for Projectile {
    fn set_position(&mut self, value: [f32; 2]) {
        match self {
            Projectile::Bullet(bullet) => bullet.position = value,
        }
    }

    fn push_force(&mut self, value: Force) {
        match self {
            Projectile::Bullet(bullet) => bullet.forces.push(value),
        }
    }

    fn remove_force(&mut self, value: Force) {
        match self {
            Projectile::Bullet(bullet) => bullet.forces.retain(|f| f != &value),
        }
    }
}

impl Geo for Projectile {
    fn tile(&self) -> &TileXy {
        match self {
            Projectile::Bullet(bullet) => &bullet.tile,
        }
    }
}

impl UpdateGeo for Projectile {
    fn set_tile(&mut self, value: TileXy) {
        match self {
            Projectile::Bullet(bullet) => bullet.tile = value,
        }
    }
}

impl Material for Projectile {
    fn material(&self) -> Materials {
        Materials::Traversable
    }
}
