use oc_individual::{Individual, IndividualIndex};
use oc_world::World;

pub trait IntoSubject<T> {
    fn into_subject<'a>(&self, world: &'a World) -> &'a T;
}

impl IntoSubject<Individual> for IndividualIndex {
    fn into_subject<'a>(&self, world: &'a World) -> &'a Individual {
        world.individual(*self)
    }
}

pub trait IntoSubjectMut<T> {
    fn into_subject_mut<'a>(&self, world: &'a mut World) -> &'a mut T;
}

impl IntoSubjectMut<Individual> for IndividualIndex {
    fn into_subject_mut<'a>(&self, world: &'a mut World) -> &'a mut Individual {
        world.individual_mut(*self)
    }
}
