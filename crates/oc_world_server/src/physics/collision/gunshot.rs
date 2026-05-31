use oc_individual::IndividualIndex;
use oc_projectile::ProjectileId;
use oc_root::Client;

use crate::physics::Processor;

impl<'x, E: Client> Processor<'x, E> {
    pub fn gunshot(
        &self,
        projectile: ProjectileId,
        individual: IndividualIndex,
    ) -> Vec<crate::runner::update::Update> {
        let projectile = self.ctx.state.world().projectile(&projectile);
        let individual = self.ctx.state.world().individual(individual);

        vec![]
    }
}
