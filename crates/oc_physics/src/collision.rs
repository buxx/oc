// TODO ... réfléchir au concept
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
