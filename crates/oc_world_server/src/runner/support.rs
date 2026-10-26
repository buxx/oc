use std::time::{Duration, Instant};

use crate::routing::Listening;
use crate::runner::update::Update;
use crate::utils::d2::random_position_around;
use oc_network::ToClient;
use oc_root::Client;
use oc_root::geo::WorldVec2;
use oc_root::side::Side;
use oc_utils::let_some;
use oc_world::support::SupportType;

impl<E: Client> super::State<E> {
    pub fn realize_support(
        &self,
        side: Side,
        type_: SupportType,
    ) -> Vec<(Listening, Vec<ToClient>)> {
        let world = self.world_mut();
        let mod_ = world.mod_();
        let pending = world
            .pending_supports
            .iter()
            .find(|p| p.side == side && p.type_ == type_);
        let_some!(pending = pending, return vec![]);

        tracing::debug!(name="runner-update-realize-support", side=?side, type_=?type_);

        match type_ {
            SupportType::NavalBombard => {
                let support = &world.mod_.support;
                let shell = mod_.shell(mod_.support.naval_bombard_shell);
                let series: Vec<Vec<WorldVec2>> = (0..support.naval_bombard_series)
                    .map(|_| {
                        (0..support.naval_bombard_shells_by_serie)
                            .map(|_| {
                                random_position_around(
                                    &world.w,
                                    pending.position,
                                    support.naval_bombard_radius,
                                )
                            })
                            .collect()
                    })
                    .collect();

                let mut decal = Duration::ZERO;
                for serie in series {
                    for position in serie {
                        let instant = Instant::now() + decal;
                        let explosion = shell.inner().clone().into();
                        decal = decal + support.naval_bombard_shell_interval();
                        tracing::debug!(name="runner-update-realize-support-nava-bombard-spawn-explosion", explosion=?explosion, position=?position, decal=?decal);
                        self.schedule(instant, Update::SpawnExplosion(explosion, position));
                    }
                    decal = decal + support.naval_bombard_serie_interval();
                }
            }
        }

        vec![]
    }
}
