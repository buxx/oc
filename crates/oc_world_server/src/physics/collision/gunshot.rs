use oc_individual::IndividualIndex;
use oc_projectile::ProjectileId;
use oc_root::Client;
#[cfg(feature = "debug")]
use oc_root::WorldConfig;

use crate::{physics::Processor, runner::update::Update};

impl<'x, E: Client> Processor<'x, E> {
    pub fn gunshot(&self, projectile: ProjectileId, i: IndividualIndex) -> Vec<Update> {
        let mut updates = vec![];
        let Some(_) = self.ctx.state.world().projectile(&projectile) else {
            return updates;
        };
        let world = self.ctx.state.world();
        let individual = world.individual(i);
        tracing::trace!(name="physics-collision-gunshot", projectile=?projectile, individual=?i);

        // TODO: probably dedicated code instead simple like this
        match individual.status {
            oc_individual::Status::Dead => {}
            // TODO: Compute something according to projectile nature and body status
            oc_individual::Status::Operational => {
                let dead = oc_individual::Status::Dead;
                let status = oc_individual::Update::SetStatus(dead);
                let forces = oc_individual::Update::SetForces(vec![]);
                let orders = oc_individual::Update::SetOrders(vec![]);
                let intent = oc_individual::Update::SetIntent(
                    oc_individual::behavior::Intent::Idle(individual.gesture.direction()),
                );
                #[cfg(feature = "tracker")]
                {
                    self.ctx
                        .tracker
                        .take()
                        .individuals
                        .push((i, status.clone()));
                }
                #[cfg(not(feature = "debug"))]
                {
                    updates.push(Update::UpdateIndividual(i, status));
                    updates.push(Update::UpdateIndividual(i, forces));
                    updates.push(Update::UpdateIndividual(i, orders));
                    updates.push(Update::UpdateIndividual(i, intent));
                }
                #[cfg(feature = "debug")]
                if !WorldConfig::immortality() {
                    updates.push(Update::UpdateIndividual(i, status));
                    updates.push(Update::UpdateIndividual(i, forces));
                    updates.push(Update::UpdateIndividual(i, orders));
                    updates.push(Update::UpdateIndividual(i, intent));
                }
                if let Some(sound) = self.ctx.state._mod().pick_die_sound() {
                    updates.push(Update::PlaySoundWithGeo(
                        sound,
                        individual.position,
                        individual.region(),
                    ));
                }
                updates.push(Update::RemoveProjectile(projectile));
            }
        }

        updates
    }
}
