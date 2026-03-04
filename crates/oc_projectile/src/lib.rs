use rkyv::{Archive, Deserialize, Serialize};

use crate::bullet::Bullet;

pub mod bullet;

#[derive(Archive, Deserialize, Serialize, Clone, Debug)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Projectile {
    Bullet(Bullet),
}
