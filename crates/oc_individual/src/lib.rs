use std::fmt::Display;
use std::ops::Deref;

use derive_more::Constructor;
use oc_geo::Geo;
use oc_geo::UpdateGeo;
use oc_geo::region::Region;
use oc_geo::region::WorldRegionIndex;
use oc_geo::tile::WorldTileIndex;
use oc_physics::Force;
use oc_physics::Physic;
use oc_physics::UpdatePhysic;
use oc_physics::collision::Material;
use oc_physics::collision::Materials;
use oc_physics::volume::Volume;
use oc_root::WorldConfig;
use oc_root::physics::Meters;
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
    pub position: [f32; 3],
    pub tile: WorldTileIndex,
    pub region: WorldRegionIndex,
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
    fn region(&self) -> WorldRegionIndex {
        self.region
    }

    fn set_region(&mut self, value: WorldRegionIndex) {
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
    pub fn tile(&self) -> WorldTileIndex {
        self.tile
    }

    pub fn region(&self) -> WorldRegionIndex {
        self.region
    }
}

impl Physic for Individual {
    fn position(&self, _: &WorldConfig) -> [f32; 3] {
        self.position
    }

    fn forces(&self, _: &WorldConfig) -> &Vec<Force> {
        &self.forces
    }

    fn volume(&self, ref_: [f32; 3], w: &WorldConfig) -> Volume {
        Volume::Cube {
            x: ref_[0],
            y: ref_[1],
            z: ref_[2],
            width: Meters(0.5).pixels(w),
            height: Meters(0.5).pixels(w),
            depth: Meters(1.8).pixels(w),
        }
    }
}

impl UpdatePhysic for Individual {
    fn set_position(&mut self, value: [f32; 3]) {
        self.position = value;
    }

    fn push_force(&mut self, value: Force) {
        self.forces.push(value)
    }

    fn remove_force(&mut self, value: &Force) {
        self.forces.retain(|f| f != value)
    }

    fn set_volume(&self, _value: Volume) {
        // No update volume of an individual (for now ...)
    }
}

impl Geo for Individual {
    fn tile(&self) -> WorldTileIndex {
        self.tile
    }
}

impl UpdateGeo for Individual {
    fn set_tile(&mut self, value: WorldTileIndex) {
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
        self.iter()
            .enumerate()
            .map(|(i, individual)| (i.into(), individual))
            .collect()
    }
}
