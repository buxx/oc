use glam::Vec2;
use oc_individual::{BodyGesture, IndividualIndex, Update, order::Order};
use oc_root::WcfgFrom;
use oc_root::{
    geo::{WorldVec2, WorldVec3},
    physics::Meters,
    y::V,
};
use oc_utils::{
    d2::{AlmostEqual, Direction},
    let_some,
};

use super::{POSITION_TOLERANCE, Processor};
use crate::{
    runner,
    visibility::{path_objects_at, visibility},
};

// When defending/hiding/engaging/suppress, formation positions must permit to fire at least this distance (meters)
// in the order direction.
const FORMATION_MIN_FIRE: Meters = Meters(10.0);

impl Processor<'_> {
    /// Build updates to ensure each member of squad receive order according to situation.
    pub fn distribute(&self) -> Vec<runner::update::Update> {
        let squad_i = self.index.individual_squad(self.i);
        let squad = self.world.squad(squad_i);
        let is_squad_leader = self.i == squad.leader();
        let leader = self.world.individual(squad.leader());

        // Skip function because executed on non leader
        if !is_squad_leader {
            tracing::trace!(name="individual-step-distribute-not-leader", i=?self.i);
            return vec![];
        }

        // Skip function because leader can't leadership (eeg. dead and news leader has not been set)
        if !leader.can_leadership() {
            tracing::trace!(name="individual-step-distribute-leader-cant-leadership", i=?self.i);
            return vec![];
        }

        let order = squad.orders.first();
        let mut updates = vec![];
        let mut distribution: Vec<(IndividualIndex, Vec<Order>)> =
            Vec::with_capacity(squad.members.len());

        let idle = Order::Idle;
        let order = order.unwrap_or(&idle);

        // Squad leader own the squad order
        let orders = Update::SetOrders(vec![order.clone()]);
        let update = runner::update::Update::UpdateIndividual(squad.leader(), orders);
        if squad.orders.first().is_some() {
            updates.push(update);
        }

        let gesture = &leader.gesture;
        let count = squad.actives as usize;
        let direction = gesture.direction();
        let angle = direction.angle(V::Server);

        let individual_position = |i| self.world.individual(i).position.into();

        // Members receive orders according to this order when leader accept it.
        match order {
            Order::Idle => {
                let reference = leader.position.into();
                let places = self.places(squad_i, squad, reference, count, direction, angle, None);

                for (member, position) in places {
                    let individual = self.world.individual(member);

                    if !position.almost_equal(individual.position, POSITION_TOLERANCE) {
                        distribution.push((member, vec![Order::MoveTo(position)]));
                    }
                }
            }
            Order::MoveTo(target) | Order::MoveFastTo(target) | Order::SneakTo(target) => {
                let leader_position: Vec2 = leader.position.into();
                let direction = order.direction(leader_position, |_| leader_position);
                let_some!(direction = direction, return updates);
                let angle = direction.angle(V::Server);
                let places = self.places(
                    squad_i,
                    squad,
                    (*target).into(),
                    count,
                    direction,
                    angle,
                    None,
                );

                for (member, position) in places {
                    let individual = self.world.individual(member);

                    if position.almost_equal(individual.position, POSITION_TOLERANCE) {
                        tracing::trace!(name="individual-step-distribute-already-on-position", i=?self.i, squad_i=?squad_i, member=?member, position=?position);
                        continue;
                    }

                    let order = order.clone().with_position(position);
                    distribution.push((member, vec![order]));
                }
            }
            // Members hold the same order as leader, after moving to their place if needed
            Order::Defend(_) | Order::Hide(_) | Order::Engage(_) | Order::Suppress(_) => {
                let reference: Vec2 = leader.position.into();
                let direction = order.direction(reference, individual_position);
                let_some!(direction = direction, return updates);
                let places = self.places(
                    squad_i,
                    squad,
                    reference,
                    count,
                    direction,
                    angle,
                    Some(direction),
                );

                for (member, position) in places {
                    let member_ = self.world.individual(member);
                    let orders = match position.almost_equal(member_.position, POSITION_TOLERANCE) {
                        true => vec![order.clone()],
                        // FIXME BS NOW NOW: how to choose move/moveFast/sneak ?
                        false => vec![Order::MoveFastTo(position), order.clone()],
                    };

                    distribution.push((member, orders));
                }
            }
        }

        for (member_i, orders) in distribution {
            let member = self.world.individual(member_i);

            // Dead individuals must keep no order
            if !member.status.is_target() {
                continue;
            }

            if member.orders != orders {
                let orders = Update::SetOrders(orders);
                let update = runner::update::Update::UpdateIndividual(member_i, orders);
                updates.push(update);
            }
        }

        updates
    }

    /// Find places according to formation and position. When `fire_direction` is given, places
    /// individual which can't fire in this direction to the nearest tile which can.
    #[allow(clippy::too_many_arguments)]
    fn places(
        &self,
        squad_i: oc_individual::squad::SquadIndex,
        squad: &oc_individual::squad::Squad,
        reference: Vec2,
        count: usize,
        direction: Direction,
        angle: oc_utils::d2::Angle,
        fire: Option<Direction>,
    ) -> Vec<(IndividualIndex, WorldVec2)> {
        let mut available: Vec<WorldVec2> = squad
            .formation
            // List one more position to prevent position in in opposite place
            .positions(&self.world.w, V::Server, reference, angle, count + 1)
            .into_iter()
            .skip(1) // drop leader's own slot
            .map(|p| WorldVec2::new(p.x, p.y))
            .collect();
        if let Some(fire) = fire {
            let leader = self.world.individual(squad.leader());
            available = self.fireable_positions(leader.position.z, available, fire);
        }
        tracing::trace!(name="individual-step-distribute-formation", i=?self.i, squad_i=?squad_i, reference=?reference, direction=?direction, angle=?angle, available_positions=?available);

        let mut remaining_members: Vec<IndividualIndex> = squad
            .members
            .iter()
            .skip(1)
            .take(count.saturating_sub(1))
            .copied()
            .collect();

        let mut assignments: Vec<(IndividualIndex, WorldVec2)> =
            Vec::with_capacity(remaining_members.len());

        while !remaining_members.is_empty() && !available.is_empty() {
            let mut best: Option<(usize, usize, f32)> = None;

            for (mi, member_i) in remaining_members.iter().enumerate() {
                let member_position = self.world.individual(*member_i).position;
                let member_xy = Vec2::new(member_position.x, member_position.y);
                for (pi, position) in available.iter().enumerate() {
                    let distance = member_xy.distance(Vec2::from(*position));
                    if best.is_none_or(|(_, _, best_distance)| distance < best_distance) {
                        best = Some((mi, pi, distance));
                    }
                }
            }

            let (mi, pi, _) = best.expect("remaining_members and available_positions non-empty");
            let member = remaining_members.remove(mi);
            let position = available.remove(pi);

            assignments.push((member, position));
        }
        assignments
    }

    // Warning: AI generated
    /// Positions which don't permit to fire in direction to the nearest ones which do.
    fn fireable_positions(
        &self,
        start_z: f32,
        positions: Vec<WorldVec2>,
        direction: Direction,
    ) -> Vec<WorldVec2> {
        let w = &self.world.w;
        let mod_ = &self.world.mod_;
        let distance = FORMATION_MIN_FIRE;
        let at = |xy, z| path_objects_at(w, mod_, self.world, xy, z);
        let tile_px = w.geo_pixels_per_tile() as f32;
        let radius = w.formation_tiles_radius_placement() as i64;
        let fire = Vec2::from(direction).normalize_or_zero() * distance.pixels(w);
        let weapon_z = BodyGesture::Crawling(Default::default())
            .weapon_z()
            .pixels(w);

        let height = start_z + weapon_z;
        let can_fire = |position: Vec2| -> bool {
            let end = position + fire;
            let from = WorldVec3::new(position.x, position.y, height);
            let to = WorldVec3::new(end.x, end.y, height);
            let modifier = oc_lov::Modifier::None(true);
            visibility(w, at, from, to, modifier).visible
        };

        // Opacity of the tile under a position
        let opacity_at = |position: Vec2| -> f32 {
            let xy = oc_geo::tile::TileXy::from_([position.x, position.y], w);
            let index = oc_geo::tile::WorldTileIndex::from_(xy, w);
            self.world
                .tile(index)
                .map(|t| mod_.nature(t.nature).opacity)
                .unwrap_or(0.0)
        };

        // offsets to list tiles where test fire
        let mut offsets: Vec<(i64, i64)> = (-radius..=radius)
            .flat_map(|dx| (-radius..=radius).map(move |dy| (dx, dy)))
            .collect();
        offsets.sort_by_key(|(dx, dy)| dx * dx + dy * dy);

        // Starts with original positions; each one is replaced by its chosen position in turn.
        let mut occupied: Vec<Vec2> = positions.iter().map(|p| Vec2::from(*p)).collect();

        for index in 0..occupied.len() {
            let original = occupied[index];
            // Best opacity first (cover), then nearest (offsets are sorted by distance)
            let mut candidates: Vec<(f32, Vec2)> = offsets
                .iter()
                .map(|(dx, dy)| original + Vec2::new(*dx as f32, *dy as f32) * tile_px)
                .filter(|c| {
                    *c == original
                        || occupied
                            .iter()
                            .enumerate()
                            .all(|(i, o)| i == index || o.distance(*c) >= tile_px)
                })
                .map(|c| (opacity_at(c), c))
                .collect();
            candidates.sort_by(|a, b| b.0.total_cmp(&a.0));
            occupied[index] = candidates
                .into_iter()
                .find(|(_, c)| can_fire(*c))
                .map(|(_, c)| c)
                .unwrap_or(original);
        }

        occupied
            .into_iter()
            .map(|p| WorldVec2::new(p.x, p.y))
            .collect()
    }
}
