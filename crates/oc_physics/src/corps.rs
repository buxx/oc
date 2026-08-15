use oc_mod::{Mod, nature::Traversability};
use oc_root::{WorldConfig, geo::WorldVec3, material::MaterialKind};
use oc_utils::d2::Direction;

use super::Force;
use crate::{Physic, collision::Material, volume::Volume};

#[derive(Debug)]
pub struct Corps<I: Clone + std::fmt::Debug> {
    pub i: I,
    pub position: WorldVec3,
    pub direction: Direction,
    pub forces: Vec<Force>,
    pub material: Option<MaterialKind>,
    pub volumes: Vec<(Volume, Traversability)>,
}

impl<I: Clone + std::fmt::Debug> Corps<I> {
    pub fn new(
        i: I,
        position: WorldVec3,
        direction: Direction,
        forces: Vec<Force>,
        material: Option<MaterialKind>,
        volumes: Vec<(Volume, Traversability)>,
    ) -> Self {
        Self {
            i,
            position,
            direction,
            forces,
            material,
            volumes,
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
}

impl<I: Clone + std::fmt::Debug> Material for Corps<I> {
    fn kind(&self) -> Option<MaterialKind> {
        self.material
    }
}
