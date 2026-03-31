use std::fmt::Display;

use oc_geo::{
    Geo, UpdateGeo,
    region::{Region, RegionXy},
    tile::TileXy,
};
use oc_physics::{
    Force, Physic, UpdatePhysic,
    collision::{Material, Materials},
    volume::Volume,
};
use oc_root::ids::Ids;
use oc_utils::collections::WithIds;
use rkyv::{Archive, Deserialize, Serialize};

use crate::bullet::Bullet;

pub mod bullet;
pub mod network;
pub mod spawn;

#[derive(Archive, Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct ProjectileId(pub u64);

impl Display for ProjectileId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}", self.0))
    }
}

impl From<&ProjectileId> for ProjectileId {
    fn from(value: &ProjectileId) -> Self {
        *value
    }
}

#[derive(Archive, Deserialize, Serialize, Clone, Debug, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Projectile {
    Bullet(Bullet),
}

impl Projectile {
    pub fn position(&self) -> &[f32; 3] {
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
    fn position(&self) -> [f32; 3] {
        match self {
            Projectile::Bullet(bullet) => bullet.position,
        }
    }

    fn forces(&self) -> &Vec<Force> {
        match self {
            Projectile::Bullet(bullet) => &bullet.forces,
        }
    }

    fn volume(&self, ref_: [f32; 3]) -> Volume {
        Volume::Point {
            x: ref_[0],
            y: ref_[1],
            z: ref_[2],
        }
    }
}

impl UpdatePhysic for Projectile {
    fn set_position(&mut self, value: [f32; 3]) {
        match self {
            Projectile::Bullet(bullet) => bullet.position = value,
        }
    }

    fn push_force(&mut self, value: Force) {
        match self {
            Projectile::Bullet(bullet) => bullet.forces.push(value),
        }
    }

    fn remove_force(&mut self, value: &Force) {
        match self {
            Projectile::Bullet(bullet) => bullet.forces.retain(|f| f != value),
        }
    }

    fn set_volume(&self, _value: Volume) {
        // Never need to update projectile volume
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

impl<'a> WithIds<ProjectileId, &'a Projectile> for Vec<(&'a ProjectileId, &'a Projectile)> {
    fn with_ids(&self) -> Vec<(ProjectileId, &'a Projectile)> {
        self.into_iter()
            .map(|(i, projectile)| (**i, *projectile))
            .collect()
    }
}

pub trait NextProjectileId {
    fn next_projectile_id(&self) -> ProjectileId;
}

impl NextProjectileId for Ids {
    fn next_projectile_id(&self) -> ProjectileId {
        ProjectileId(
            self.projectiles
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        )
    }
}
