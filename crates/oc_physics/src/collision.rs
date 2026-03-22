#[cfg(feature = "bevy")]
use bevy::prelude::*;

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
