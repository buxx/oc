#[cfg(feature = "bevy")]
use bevy::prelude::*;
use oc_mod::nature::Prohibe;
use oc_root::material::MaterialKind;

pub trait Material {
    fn kind(&self) -> Option<MaterialKind> {
        None
    }

    fn prohibe(&self) -> &Prohibe {
        static PROHIBE_NONE: Prohibe = Prohibe::none();
        &PROHIBE_NONE
    }
}

#[cfg(feature = "bevy")]
#[derive(Debug, Deref, DerefMut, Component)]
pub struct Material_(pub Option<MaterialKind>);
