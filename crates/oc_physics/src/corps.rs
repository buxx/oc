use oc_mod::{Mod, nature::Traversability};
use oc_root::{WorldConfig, geo::WorldVec3, material::MaterialKind, side::Side};
use oc_utils::d2::Direction;

use super::Force;
use crate::{IgnoreSide, Physic, collision::Material, volume::Volume};

#[derive(Debug)]
pub struct Corps<I: Clone + std::fmt::Debug> {
    pub i: I,
    pub position: WorldVec3,
    pub direction: Direction,
    pub forces: Vec<Force>,
    pub material: Option<MaterialKind>,
    pub volumes: Vec<(Volume, Traversability)>,
    pub side: Option<Side>,
    pub ignore_side: IgnoreSide,
}

impl<I: Clone + std::fmt::Debug> Corps<I> {
    pub fn new(
        i: I,
        position: WorldVec3,
        direction: Direction,
        forces: Vec<Force>,
        material: Option<MaterialKind>,
        volumes: Vec<(Volume, Traversability)>,
        side: Option<Side>,
        ignore_side: IgnoreSide,
    ) -> Self {
        Self {
            i,
            position,
            direction,
            forces,
            material,
            volumes,
            side,
            ignore_side,
        }
    }
}

impl<I: Clone + std::fmt::Debug> Physic for Corps<I> {
    fn position(&self, _: &WorldConfig) -> WorldVec3 {
        self.position
    }

    fn forces(&self, _: &WorldConfig) -> &Vec<Force> {
        &self.forces
    }

    fn volumes(
        &self,
        ref_: WorldVec3,
        _: &WorldConfig,
        _mod_: &Mod,
    ) -> Vec<(Volume, Traversability, Direction)> {
        self.volumes
            .clone()
            .into_iter()
            .map(|(v, t)| (v.with_ref(ref_), t, self.direction))
            .collect()
    }

    fn ignore_side(&self) -> IgnoreSide {
        self.ignore_side
    }

    fn side(&self) -> Option<Side> {
        self.side
    }
}

impl<I: Clone + std::fmt::Debug> Material for Corps<I> {
    fn kind(&self) -> Option<MaterialKind> {
        self.material
    }
}
