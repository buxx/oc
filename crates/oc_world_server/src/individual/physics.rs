use oc_geo::tile::TileXy;
use oc_individual::{Individual, Update};
use oc_physics::{Force, Physic};

pub fn changes(individual: &Individual, position: &[f32; 2], forces: &Vec<Force>) -> Vec<Update> {
    let mut updates = vec![];

    if individual.position() != position {
        updates.push(Update::UpdatePosition(*position));

        let xy: TileXy = position.clone().into();
        if individual.tile() != &xy {
            updates.push(Update::UpdateXy(xy));
        }
    }

    for force in individual.forces() {
        if !forces.contains(force) {
            updates.push(Update::RemoveForce(force.clone()));
        }
    }

    for force in forces {
        if !individual.forces().contains(force) {
            updates.push(Update::PushForce(force.clone()));
        }
    }

    updates
}
