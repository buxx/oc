use oc_individual::{IndividualIndex, Update, network};
use oc_network::ToClient;
use oc_world::World;

use crate::routing::Listening;

pub fn write(
    world: &mut World,
    update: Update,
    i: IndividualIndex,
) -> Vec<(Listening, Vec<ToClient>)> {
    let individual = world.individual_mut(i);

    match &update {
        Update::SetBehavior(behavior) => {
            individual.behavior = behavior.clone();
        }
        Update::SetOrders(orders) => {
            individual.orders = orders.clone();
        }
        Update::SetForces(forces) => {
            individual.forces = forces.clone();
        }
        Update::SetStatus(status) => {
            individual.status = *status;
        }
        Update::SetGesture(gesture) => {
            individual.gesture = gesture.clone();
        }
        Update::SetIntent(intent) => individual.intent = intent.clone(),
    }

    let region = individual.region;
    let update = network::Individual::Update(i, update);
    let update = ToClient::Individual(update);
    vec![(Listening::Regions(vec![region]), vec![update])]
}
