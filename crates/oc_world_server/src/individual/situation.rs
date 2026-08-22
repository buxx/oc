use derive_more::Constructor;
use oc_individual::IndividualIndex;
use oc_root::physics::Meters;
use oc_world::visibility::Visibility;

pub struct Situation<'a> {
    pub visible: Vec<Visible<'a>>,
}

impl<'a> Situation<'a> {
    pub fn imply_hide(&self) -> bool {
        !self.visible.is_empty()
    }
}

#[derive(Debug, Constructor)]
pub struct Visible<'a> {
    pub individual: IndividualIndex,
    pub visibility: &'a Visibility,
    pub distance: Meters,
}
