use oc_individual::{Individual, IndividualIndex};
use oc_projectile::{Projectile, ProjectileId};
use oc_world::World;

pub trait IntoSubject<T> {
    fn into_subject<'a>(&self, world: &'a World) -> Option<&'a T>;
}

impl IntoSubject<Individual> for IndividualIndex {
    fn into_subject<'a>(&self, world: &'a World) -> Option<&'a Individual> {
        Some(world.individual(*self))
    }
}

impl IntoSubject<Projectile> for ProjectileId {
    fn into_subject<'a>(&self, world: &'a World) -> Option<&'a Projectile> {
        world.projectile(self)
    }
}

pub trait IntoSubjectMut<T> {
    fn into_subject_mut<'a>(&self, world: &'a mut World) -> Option<&'a mut T>;
}

impl IntoSubjectMut<Individual> for IndividualIndex {
    fn into_subject_mut<'a>(&self, world: &'a mut World) -> Option<&'a mut Individual> {
        Some(world.individual_mut(*self))
    }
}

impl IntoSubjectMut<Projectile> for ProjectileId {
    fn into_subject_mut<'a>(&self, world: &'a mut World) -> Option<&'a mut Projectile> {
        world.projectile_mut(self)
    }
}
