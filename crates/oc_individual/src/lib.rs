use std::fmt::Display;
use std::ops::Deref;

use derive_more::Constructor;
use oc_geo::Geo;
use oc_geo::UpdateGeo;
use oc_geo::region::Region;
use oc_geo::region::RegionXy;
use oc_geo::tile::TileXy;
use oc_physics::Force;
use oc_physics::Physic;
use oc_physics::UpdatePhysic;
use oc_physics::collision::Material;
use oc_physics::collision::Materials;
use oc_utils::collections::WithIds;
use rkyv::{Archive, Deserialize, Serialize};

use crate::behavior::Behavior;

pub mod behavior;
pub mod network;

#[derive(Archive, Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq, Constructor, Hash)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct IndividualIndex(pub u64);

impl Display for IndividualIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}", self.0))
    }
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Constructor, Clone)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Individual {
    pub position: [f32; 2],
    pub tile: TileXy,
    pub region: RegionXy,
    pub behavior: Behavior,
    pub forces: Vec<Force>,
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Update {
    SetForces(Vec<Force>),
    SetBehavior(Behavior),
}

impl Region for Individual {
    fn region(&self) -> &RegionXy {
        &self.region
    }

    fn set_region(&mut self, value: RegionXy) {
        self.region = value;
    }
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
        &self.tile
    }

    pub fn region(&self) -> &RegionXy {
        &self.region
    }
}

impl Physic for Individual {
    fn position(&self) -> &[f32; 2] {
        &self.position
    }

    fn forces(&self) -> &Vec<Force> {
        &self.forces
    }
}

impl UpdatePhysic for Individual {
    fn set_position(&mut self, value: [f32; 2]) {
        self.position = value;
    }

    fn push_force(&mut self, value: Force) {
        self.forces.push(value)
    }

    fn remove_force(&mut self, value: &Force) {
        self.forces.retain(|f| f != value)
    }
}

impl Geo for Individual {
    fn tile(&self) -> &TileXy {
        &self.tile
    }
}

impl UpdateGeo for Individual {
    fn set_tile(&mut self, value: TileXy) {
        self.tile = value;
    }
}

impl Material for Individual {
    fn material(&self) -> Materials {
        Materials::Traversable
    }
}

impl<'a> WithIds<IndividualIndex, &'a Individual> for &'a [Individual] {
    fn with_ids(&self) -> Vec<(IndividualIndex, &'a Individual)> {
        self.into_iter()
            .enumerate()
            .map(|(i, individual)| (i.into(), individual))
            .collect()
    }
}
