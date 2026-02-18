use std::ops::Deref;

use derive_more::Constructor;
use oc_geo::tile::TileXy;
use oc_physics::Force;
use oc_physics::Physic;
use oc_physics::collision::Material;
use oc_physics::collision::Materials;
use oc_utils::d2::Xy;
use rkyv::{Archive, Deserialize, Serialize};

use crate::behavior::Behavior;

pub mod behavior;
pub mod network;

#[derive(Archive, Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Constructor)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct IndividualIndex(pub u64);

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Constructor, Clone)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Individual {
    pub position: [f32; 2],
    pub xy: TileXy,
    pub behavior: Behavior,
    pub forces: Vec<Force>,
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Constructor, Clone)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct IndividualPublic {
    pub position: [f32; 2],
    pub xy: TileXy,
    pub behavior: Behavior,
    pub forces: Vec<Force>,
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Update {
    UpdatePosition([f32; 2]),
    UpdateXy(TileXy),
    UpdateBehavior(Behavior),
    PushForce(Force),
    RemoveForce(Force),
}

impl Deref for IndividualIndex {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<usize> for IndividualIndex {
    fn from(value: usize) -> Self {
        Self(value as u64)
    }
}

impl From<u64> for IndividualIndex {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl Individual {
    pub fn tile(&self) -> &TileXy {
        &self.xy
    }
}

impl Physic for &Individual {
    fn position(&self) -> &[f32; 2] {
        &self.position
    }

    fn xy(&self) -> &Xy {
        &self.xy.0
    }

    fn forces(&self) -> &Vec<Force> {
        &self.forces
    }
}

impl Material for &Individual {
    fn material(&self) -> Materials {
        Materials::Traversable
    }
}

impl From<Individual> for IndividualPublic {
    fn from(value: Individual) -> Self {
        Self {
            position: value.position,
            xy: value.xy,
            behavior: value.behavior,
            forces: value.forces,
        }
    }
}
