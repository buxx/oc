use oc_mod::{Mod, nature::Traversability};
use oc_root::{WorldConfig, material::MaterialKind};

use super::Force;
use crate::{Physic, collision::Material, volume::Volume};

#[derive(Debug)]
pub struct Corps<I: Clone + std::fmt::Debug> {
    pub i: I,
    pub position: [f32; 3],
    pub forces: Vec<Force>,
    pub material: Option<MaterialKind>,
    pub volumes: Vec<(Volume, Traversability)>,
}

impl<I: Clone + std::fmt::Debug> Corps<I> {
    pub fn new(
        i: I,
        position: [f32; 3],
        forces: Vec<Force>,
        material: Option<MaterialKind>,
        volumes: Vec<(Volume, Traversability)>,
    ) -> Self {
        Self {
            i,
            position,
            forces,
            material,
            volumes,
        }
    }
}

impl<I: Clone + std::fmt::Debug> Physic for Corps<I> {
    fn position(&self, _: &WorldConfig) -> [f32; 3] {
        self.position
    }

    fn forces(&self, _: &WorldConfig) -> &Vec<Force> {
        &self.forces
    }

    fn volumes(
        &self,
        ref_: [f32; 3],
        _: &WorldConfig,
        _mod_: &Mod,
    ) -> Vec<(Volume, Traversability)> {
        self.volumes
            .clone()
            .into_iter()
            .map(|(v, t)| (v.with_ref(ref_), t))
            .collect()
    }
}

impl<I: Clone + std::fmt::Debug> Material for Corps<I> {
    fn kind(&self) -> Option<MaterialKind> {
        self.material
    }
}
