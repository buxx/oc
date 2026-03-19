#[cfg(feature = "bevy")]
use bevy::prelude::*;
use oc_root::tile::Tile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Materials {
    Solid,
    Traversable,
}

impl Materials {
    pub fn is_solid(&self) -> bool {
        match self {
            Materials::Solid => true,
            Materials::Traversable => false,
        }
    }
}

pub trait Material {
    fn material(&self) -> Materials;
}

#[cfg(feature = "bevy")]
#[derive(Debug, Deref, DerefMut, Component)]
pub struct Material_(pub Materials);

#[derive(Debug, Clone)]
pub struct Collision<L, R>(pub L, pub R);

impl Material for Tile {
    fn material(&self) -> Materials {
        // TODO: depending on tile
        Materials::Traversable
    }
}
