use oc_individual::{Individual, IndividualIndex};
use oc_world::World;

pub trait IntoSubject<'a, T> {
    fn into_subject(&self, world: &'a World) -> &'a T;
}

impl<'a> IntoSubject<'a, Individual> for IndividualIndex {
    fn into_subject(&self, world: &'a World) -> &'a Individual {
        world.individual(*self)
    }
}
