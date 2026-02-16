use oc_physics::collision::{Material, Materials};

#[derive(Debug, Clone, Copy)]
pub enum Tile {
    ShortGrass,
}

impl Material for Tile {
    fn material(&self) -> Materials {
        Materials::Solid
    }
}
