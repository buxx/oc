use derive_more::Constructor;
use glam::{Vec2, Vec3};
use oc_geo::tile::{TileXy, WorldTileIndex};
use oc_individual::{
    BodyGesture, Gesture, Individual, IndividualIndex, Update, WeaponKind,
    behavior::{Behavior, Intent, MovePath},
    order::Order,
};
use oc_physics::Force;
use oc_projectile::spawn::SpawnProjectiles;
use oc_root::WorldConfig;
use oc_root::{
    U8Progress, WcfgFrom,
    geo::{WorldVec2, WorldVec3},
    opacity::CumulatedOpacity,
    physics::Meters,
    y::V,
};
use oc_utils::{
    d2::{AlmostEqual, Direction},
    let_some,
    random::direction_with_inaccuracy,
};
use oc_world::World;

use crate::{
    index::Indexes,
    individual::situation::{Situation, Visible},
    runner,
    visibility::{path_objects_at, visibility},
};

pub mod situation;
pub mod update;

const POSITION_TOLERANCE: f32 = 3.0;

// Below this distance to the current waypoint, the walking force is scaled
// down proportionally instead of applied at full strength. This prevents the
// individual from overshooting POSITION_TOLERANCE every tick and bouncing
// back and forth ("circling") around its target.
const ARRIVAL_RADIUS: f32 = 8.0;

// Force is never scaled below this factor, so approach still makes progress
// instead of asymptotically crawling forever.
const MIN_FORCE_FACTOR: f32 = 0.05;

// TODO: When click hide order, permit use key (like Shift) to specify distance
const HIDE_ENGAGE_DISTANCE: Meters = Meters(30.0);

#[derive(Constructor)]
pub struct Processor<'a> {
    world: &'a World,
    index: &'a Indexes,
    i: IndividualIndex,
}

impl<'a> Processor<'a> {
    /// Compute individual changes.
    ///
    /// Individual behavior is splitted in several steps:
    ///   - resolve order into intent (`decide` function): Order can be follow or not. For example,
    ///     if order is to engage an enemy but individual is suppressed, intent will be hide.
    ///   - resolve an intent into behavior (`act` function): Here the intent can be specialized in
    ///     behavior to reach the intent goal. For example, a move intent will be splitted in move
    ///     behavior which target the next move step instead move target.
    ///   - resolve behavior in gesture: body and hands gesture are computed from the behavior and
    ///     individual state. For example, engage behavior can result a reloading hands, or aiming
    ///     hands.
    ///   - resolve forces from behavior: Here, behavior forces are computed. For example, a move
    ///     behavior result a translation force.
    ///
    pub fn step(self) -> Vec<runner::update::Update> {
        tracing::trace!(name="individual-step", i=?self.i);
        let mut updates = vec![];
        let individual = self.world.individual(self.i);

        self.suppress(&mut updates, individual);

        if !individual.status.can_step() {
            tracing::trace!(name="individual-step-cant-step", i=?self.i);
            return vec![];
        }
        let situation = &self.situation(individual);

        if let Some(updates) = self.accomplished(situation) {
            tracing::trace!(name = "individual-step-accomplished-updates", i=?self.i, updates=?updates);
            return updates;
        }

        let distribute = self.distribute();
        let intent = self.decide(situation);
        let behavior = self.act(situation, &intent);
        let (gesture, updates_) = self.gesture(individual, situation, &behavior);
        let forces = self.forces(individual, &behavior, &intent);

        tracing::trace!(
            name = "individual-step-with",
            i = ?self.i,
            individual = ?individual,
            distribute = ?distribute,
            intent = ?intent,
            behavior = ?behavior,
            gesture = ?gesture,
            forces = ?forces,
        );

        // Dispatch orders to members if not already own it
        for (member_i, orders) in distribute {
            let member = self.world.individual(member_i);
            if member.orders != orders {
                let update = Update::SetOrders(orders);
                let update = runner::update::Update::UpdateIndividual(member_i, update);
                updates.push(update);
            }
        }

        updates.extend(updates_);

        macro_rules! push_update {
            ($self:expr, $updates:expr, $individual:expr, $( $field:ident => $variant:ident ),+ $(,)?) => {
                $(
                    if $field != $individual.$field {
                        $updates.push(runner::update::Update::UpdateIndividual(
                            $self.i,
                            Update::$variant($field),
                        ));
                    }
                )+
            };
        }

        push_update!(self, updates, individual,
            intent => SetIntent,
            behavior => SetBehavior,
            gesture => SetGesture,
            forces => SetForces,
        );

        tracing::trace!(name = "individual-step-updates", i=?self.i, updates=?updates);
        updates
    }

    fn suppress(&self, updates: &mut Vec<runner::update::Update>, individual: &Individual) {
        let decrease = self.world.w.individual_tick_decrease_suppress;
        let suppress_ = individual.suppress;
        let suppress = individual.suppress.decrease(decrease);
        if suppress_ != suppress {
            updates.push(runner::update::Update::UpdateIndividual(
                self.i,
                oc_individual::Update::SetSuppress(suppress),
            ));
        }
    }

    /// Determine if current order (or order step) is accomplished
    fn accomplished(&self, situation: &Situation) -> Option<Vec<runner::update::Update>> {
        let individual = self.world.individual(self.i);
        let Some(order) = individual.orders.first() else {
            tracing::trace!(name = "individual-step-accomplished-no-order", i=?self.i);
            return None;
        };
        let squad_i = self.index.individual_squad(self.i);
        let squad = self.world.squad(squad_i);
        let is_squad_leader = self.i == squad.leader();
        let mut updates = None;
        let direction = individual.gesture.direction();

        // Builds the "accomplished" update batch for given individual, appending squad-level
        // updates when this individual is a squad leader.
        let accomplished_updates =
            |i: IndividualIndex, direction: Direction| -> Vec<runner::update::Update> {
                let i_accomplish = Update::Accomplished;
                let i_accomplish = runner::update::Update::UpdateIndividual(i, i_accomplish);

                let i_idle = Update::SetIntent(Intent::Idle(direction));
                let i_idle = runner::update::Update::UpdateIndividual(i, i_idle);

                let mut updates = vec![i_accomplish, i_idle];

                if is_squad_leader {
                    // FIXME: Must wait all members finished associated order (only relevant for MoveTo,
                    // kept here since Idle currently shares the same behavior).
                    let accomplish = oc_individual::squad::Update::Accomplished;
                    let accomplish = runner::update::Update::UpdateSquad(squad_i, accomplish);

                    let orders = squad.orders.clone().into_iter().skip(1).collect();
                    let orders = oc_individual::squad::Update::SetOrders(orders);
                    let orders = runner::update::Update::UpdateSquad(squad_i, orders);

                    updates.extend(vec![accomplish, orders]);
                }

                updates
            };

        match order {
            Order::Idle => {
                tracing::trace!(name = "individual-step-accomplished-idle-finished", i = ?self.i);
                updates = Some(accomplished_updates(self.i, direction));
            }
            Order::MoveTo(position) | Order::MoveFastTo(position) | Order::SneakTo(position) => {
                if position.almost_equal(individual.position, POSITION_TOLERANCE) {
                    tracing::trace!(name = "individual-step-accomplished-move-to-finished", i = ?self.i);
                    updates = Some(accomplished_updates(self.i, direction));
                }
            }
            // Defend and hide never finish
            Order::Defend(_) | Order::Hide(_) => {}
            // FIXME: must finish when ammo low
            Order::Suppress(_) => {}
            Order::Engage(individual) => {
                if situation.visible(*individual).is_none() {
                    tracing::trace!(name = "individual-step-accomplished-engage-finished", i=?self.i, i2=?individual);
                    updates = Some(accomplished_updates(self.i, direction));
                }
            }
        };

        if updates.is_some() {
            return updates;
        }

        // Test move intents to trigger "next step" when step reached
        match &individual.intent {
            // Idle/Defend/Hide/Engage/Suppress never finish
            Intent::Idle(_)
            | Intent::Defend(_)
            | Intent::Hide(_)
            | Intent::Engage(_)
            | Intent::Suppress(_) => {}
            Intent::MoveTo(_, move_path)
            | Intent::MoveFastTo(_, move_path)
            | Intent::SneakTo(_, move_path) => {
                let Some(next) = move_path.iter().next() else {
                    tracing::trace!(name = "individual-step-accomplished-intent-move-to-no-next", i=?self.i);
                    return updates;
                };

                if next.almost_equal(individual.position, POSITION_TOLERANCE) {
                    let update = Update::MoveStepAccomplished;
                    let update = runner::update::Update::UpdateIndividual(self.i, update);
                    updates.get_or_insert(vec![]).push(update);
                }
            }
        }

        updates
    }

    /// Build updates to ensure each member of squad receive order according to situation.
    fn distribute(&self) -> Vec<(IndividualIndex, Vec<Order>)> {
        let squad_i = self.index.individual_squad(self.i);
        let squad = self.world.squad(squad_i);
        let order = squad.orders.first();
        let mut distribution = Vec::with_capacity(squad.members.len());
        let is_squad_leader = self.i == squad.leader();

        if !is_squad_leader {
            tracing::trace!(name="individual-step-distribute-not-leader", i=?self.i);
            return distribution;
        }

        if let Some(order) = order {
            // Squad leader own the squad order
            distribution.push((squad.leader(), vec![order.clone()]));
        };

        let leader = self.world.individual(squad.leader());
        let gesture = &leader.gesture;
        let reference = Vec2::new(leader.position.x, leader.position.y);
        let count = squad.actives as usize;
        let direction = gesture.direction();
        let angle = direction.angle(V::Server);

        let positions = squad
            .formation
            .positions(&self.world.w, V::Server, reference, angle, count)
            .into_iter()
            .map(|p| WorldVec2::new(p.x, p.y));
        tracing::trace!(name="individual-step-distribute-formation", i=?self.i, squad_i=?squad_i, reference=?reference, direction=?direction, angle=?angle, positions=?positions);

        for (member, position) in squad.members.iter().zip(positions).skip(1) {
            let individual = self.world.individual(*member);
            if position.almost_equal(individual.position, POSITION_TOLERANCE) {
                tracing::trace!(name="individual-step-distribute-already-on-position", i=?self.i, squad_i=?squad_i, member=?member, position=?position);
                continue;
            }

            // According to order, choose appropriate order to distribute (move if move, move fast if move fast, etc.)
            let orders = match order {
                Some(order) => match order {
                    Order::Idle => vec![Order::MoveTo(position.into())],
                    Order::MoveTo(_) => vec![Order::MoveTo(position.into())],
                    Order::MoveFastTo(_) => vec![Order::MoveFastTo(position.into())],
                    Order::SneakTo(_) => vec![Order::SneakTo(position.into())],
                    Order::Defend(_) => vec![Order::MoveFastTo(position.into())],
                    Order::Hide(_) => vec![Order::SneakTo(position.into())],
                    Order::Engage(_) => vec![Order::MoveFastTo(position.into())],
                    Order::Suppress(_) => vec![Order::MoveFastTo(position.into())],
                },
                None => {
                    vec![Order::MoveTo(position.into())]
                }
            };
            tracing::trace!(name="individual-step-distribute-to", i=?self.i, squad_i=?squad_i, order=?order, member=?member, orders=?orders);
            distribution.push((*member, orders))
        }

        tracing::trace!(name="individual-step-distribution", i=?self.i, squad_i=?squad_i, order=?order, distribution=?distribution);
        distribution
    }

    /// Build object which reflect individual situation against environment
    fn situation(&self, individual: &Individual) -> Situation<'_> {
        let reference = individual.position;

        let mut visibles: Vec<Visible> = self
            .world
            .visibilities
            .for_(self.i)
            .iter()
            .enumerate()
            .filter_map(|(i, v)| {
                v.visible.then(|| {
                    let target = IndividualIndex(i as u64);
                    let target_ = self.world.individual(target);
                    let distance = reference.distance(target_.position);
                    let distance = Meters(distance / self.world.w.geo_pixels_per_meters);
                    Visible::new(target, v, distance)
                })
            })
            .collect();
        visibles.sort_unstable_by(|a, b| a.distance.0.total_cmp(&b.distance.0));

        Situation { visibles }
    }

    /// Decide the individual's intent for this tick. Will be influenced by situation.
    fn decide(&self, situation: &Situation) -> Intent {
        let individual = self.world.individual(self.i);
        let order = individual.orders.first();

        let intent = match individual.can_follow_orders() {
            // FIXME BS NOW: suppress ?
            true => match order {
                None | Some(Order::Idle) => self.resolve_idle_order(individual, situation),
                Some(Order::Defend(direction)) => {
                    self.resolve_defend_order(individual, situation, *direction)
                }
                Some(Order::Hide(direction)) => {
                    self.resolve_hide_order(individual, situation, *direction)
                }
                Some(Order::MoveTo(position)) => {
                    self.resolve_move_to_order(individual, situation, *position)
                }
                Some(Order::MoveFastTo(position)) => {
                    self.resolve_move_fast_to_order(individual, situation, *position)
                }
                Some(Order::SneakTo(position)) => {
                    self.resolve_sneak_to_order(individual, situation, *position)
                }
                Some(Order::Engage(target)) => {
                    self.resolve_engage_order(individual, situation, *target)
                }
                Some(Order::Suppress(target)) => {
                    self.resolve_suppress_order(individual, situation, *target)
                }
            },
            false => individual.intent.clone(),
        };

        tracing::trace!(name="individual-step-decide", i=?self.i, order=?order, intent=?intent);
        intent
    }

    // TODO: Ex. idle -> Behavior::TakingCover when under fire
    fn act(&self, _situation: &Situation, intent: &Intent) -> Behavior {
        let individual = self.world.individual(self.i);

        match intent {
            Intent::Idle(direction) => Behavior::Idle(*direction),
            Intent::MoveTo(_, path) => match self.direction_to_next(individual, path, "move") {
                Some(direction) => Behavior::Walk(direction),
                None => Behavior::Idle(Direction::NORTH),
            },
            Intent::MoveFastTo(_, path) => {
                match self.direction_to_next(individual, path, "move-fast") {
                    Some(direction) => Behavior::Run(direction),
                    None => Behavior::Idle(Direction::NORTH),
                }
            }
            Intent::SneakTo(_, path) => match self.direction_to_next(individual, path, "sneak") {
                Some(direction) => Behavior::Crawl(direction),
                None => Behavior::Idle(Direction::NORTH),
            },
            Intent::Defend(direction) => Behavior::Defend(*direction),
            Intent::Hide(direction) => Behavior::Hide(*direction),
            Intent::Engage(individual) => Behavior::Engage(*individual),
            Intent::Suppress(position) => Behavior::Suppress(*position),
        }
    }

    fn gesture(
        &self,
        individual: &Individual,
        situation: &Situation,
        behavior: &Behavior,
    ) -> (Gesture, Vec<runner::update::Update>) {
        match behavior {
            Behavior::Idle(direction) => match situation.imply_hide() {
                true => (Gesture::body(BodyGesture::Prone(*direction)), vec![]),
                false => (Gesture::body(BodyGesture::StandUp(*direction)), vec![]),
            },
            Behavior::Walk(direction) => (Gesture::body(BodyGesture::Walking(*direction)), vec![]),
            Behavior::Run(direction) => (Gesture::body(BodyGesture::Running(*direction)), vec![]),
            Behavior::Crawl(direction) => {
                (Gesture::body(BodyGesture::Crawling(*direction)), vec![])
            }
            Behavior::Defend(direction) => (Gesture::body(BodyGesture::Prone(*direction)), vec![]),
            Behavior::Hide(direction) => (Gesture::body(BodyGesture::Prone(*direction)), vec![]),
            Behavior::Engage(target) => self.engage_gesture(individual, situation, *target),
            Behavior::Suppress(target) => self.suppress_gesture(individual, situation, *target),
        }
    }

    // TODO: a lot of similar code with engage_gesture
    fn engage_gesture(
        &self,
        individual: &Individual,
        situation: &Situation,
        target: IndividualIndex,
    ) -> (Gesture, Vec<runner::update::Update>) {
        let interval = self.world.w.individual_tick_interval_us;
        let target_ = self.world.individual(target);
        let reference = individual.position.into();
        let direction = Direction::from_points3d(reference, target_.position.into());
        let_some!(
            visible = situation.visible(target),
            // Should not happen as engage gesture is used when target is visible
            return (Gesture::body(BodyGesture::Prone(direction)), vec![])
        );

        let (hands, updates) = match self.weapon(individual, target_) {
            // Individual own weapon adapted to this target
            Some((weapon_kind, weapon)) => {
                tracing::trace!(name="individual-processor-engage-gesture-weapon-found", i=?self.i, weapon_kind=?weapon_kind, weapon=?weapon);

                match weapon.filled {
                    // The weapon is filled and ready to fire
                    Some((magazine, ammunition)) => {
                        tracing::trace!(name="individual-processor-engage-gesture-weapon-filled", i=?self.i, magazine=?magazine, ammunition=?ammunition);

                        match individual.gesture.hands {
                            // All other than aiming imply to aim
                            oc_individual::HandsGesture::Idle
                            | oc_individual::HandsGesture::Reloading(_) => {
                                tracing::trace!(name="individual-processor-engage-gesture-not-aiming", i=?self.i);

                                (
                                    oc_individual::HandsGesture::Aiming(U8Progress::zero()),
                                    vec![],
                                )
                            }
                            // If already aiming, continue
                            oc_individual::HandsGesture::Aiming(progress) => {
                                let weapon = self.world.mod_.weapon(weapon.i);
                                let progress = progress.tick(interval, weapon.aim());
                                tracing::trace!(name="individual-processor-engage-gesture-aiming", i=?self.i, progress=progress.0);

                                match progress.finished() {
                                    // If aiming is finished, spawn projectile
                                    true => {
                                        tracing::trace!(name="individual-processor-engage-gesture-aiming-progress-finished", i=?self.i);

                                        let plus_z =
                                            target_.gesture.body.target_z().pixels(&self.world.w);
                                        let target = WorldVec3 {
                                            x: target_.position.x,
                                            y: target_.position.y,
                                            z: target_.position.z + plus_z,
                                        };
                                        (
                                            oc_individual::HandsGesture::Idle,
                                            self.spawn_projectile(
                                                individual,
                                                weapon,
                                                ammunition,
                                                weapon_kind,
                                                target,
                                                visible.visibility.opacity,
                                            ),
                                        )
                                    }
                                    // Else continue aiming
                                    false => {
                                        tracing::trace!(name="individual-processor-engage-gesture-aiming-continue", i=?self.i);

                                        (oc_individual::HandsGesture::Aiming(progress), vec![])
                                    }
                                }
                            }
                        }
                    }
                    // The weapon is not ready to fire
                    None => {
                        tracing::trace!(name="individual-processor-engage-gesture-weapon-not-ready-to-fire", i=?self.i);

                        match individual.gesture.hands {
                            // All other than reloading imply start reloading
                            oc_individual::HandsGesture::Idle
                            | oc_individual::HandsGesture::Aiming(_) => {
                                tracing::trace!(name="individual-processor-engage-gesture-weapon-not-reloading", i=?self.i);

                                match self.can_reload(individual, weapon) {
                                    true => (
                                        oc_individual::HandsGesture::Reloading(U8Progress::zero()),
                                        vec![],
                                    ),
                                    // This weapon can't be reloaded, next individual tick will choose another weapon if any
                                    false => (oc_individual::HandsGesture::Idle, vec![]),
                                }
                            }
                            // If already reloading, continue
                            oc_individual::HandsGesture::Reloading(progress) => {
                                let weapon = self.world.mod_.weapon(weapon.i);
                                let progress = progress.tick(interval, weapon.reload());
                                tracing::trace!(name="individual-processor-engage-gesture-weapon-reloading", i=?self.i, progress=progress.0);

                                match progress.finished() {
                                    // If reloading is finished, update weapons and start to aim
                                    true => {
                                        tracing::trace!(name="individual-processor-engage-gesture-weapon-reloading-finished", i=?self.i);

                                        (
                                            oc_individual::HandsGesture::Aiming(U8Progress::zero()),
                                            vec![self.reloaded(individual, weapon_kind, weapon)],
                                        )
                                    }
                                    // Else continue reloading
                                    false => {
                                        tracing::trace!(name="individual-processor-engage-gesture-weapon-reloading-not-finished", i=?self.i);

                                        (oc_individual::HandsGesture::Reloading(progress), vec![])
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // No weapon adapted to the target
            None => {
                tracing::trace!(name="individual-processor-engage-gesture-no-weapon-found", i=?self.i, target=?target);

                (oc_individual::HandsGesture::Idle, vec![])
            } // FIXME: choose/try another target where weapon match ? do it where behavior chosen ?
        };

        (
            Gesture::body(BodyGesture::Prone(direction)).with_hands(hands),
            updates,
        )
    }

    // TODO: a lot of similar code with engage_gesture
    fn suppress_gesture(
        &self,
        individual: &Individual,
        _situation: &Situation,
        target: WorldVec2,
    ) -> (Gesture, Vec<runner::update::Update>) {
        let w = &self.world.w;
        let mod_ = &self.world.mod_;
        let interval = self.world.w.individual_tick_interval_us;
        let reference = Vec2::new(individual.position.x, individual.position.y);
        let direction = Direction::from_points2d(reference, Vec2::new(target.x, target.y));
        // TODO: mechanism to cache (perf) ?
        let tile_i = WorldTileIndex::from_(TileXy::from_(target, w), w);
        let_some!(
            target_tile = self.world.tile(tile_i),
            return (Gesture::body(BodyGesture::Prone(direction)), vec![])
        );
        let tile_z = target_tile.z_pixels(w);
        let at = |xy, z| path_objects_at(w, mod_, self.world, xy, z);
        let target__ = target.extend(tile_z);
        let visibility = visibility(w, at, individual.position, target__);
        tracing::trace!(name="individual-processor-suppress-gesture-visibility", i=?self.i, target=?target, tile_z=tile_z, visibility=?visibility);

        let (hands, updates) = match self.primary_weapon(individual, target) {
            // Individual own weapon adapted to this target
            Some(weapon) => {
                tracing::trace!(name="individual-processor-suppress-gesture-weapon-found", i=?self.i, weapon=?weapon);

                match weapon.filled {
                    // The weapon is filled and ready to fire
                    Some((magazine, ammunition)) => {
                        tracing::trace!(name="individual-processor-suppress-gesture-weapon-filled", i=?self.i, magazine=?magazine, ammunition=?ammunition);

                        match individual.gesture.hands {
                            // All other than aiming imply to aim
                            oc_individual::HandsGesture::Idle
                            | oc_individual::HandsGesture::Reloading(_) => {
                                tracing::trace!(name="individual-processor-suppress-gesture-not-aiming", i=?self.i);

                                (
                                    oc_individual::HandsGesture::Aiming(U8Progress::zero()),
                                    vec![],
                                )
                            }
                            // If already aiming, continue
                            oc_individual::HandsGesture::Aiming(progress) => {
                                let weapon = self.world.mod_.weapon(weapon.i);
                                let progress = progress.tick(interval, weapon.aim());
                                tracing::trace!(name="individual-processor-suppress-gesture-aiming", i=?self.i, progress=progress.0);

                                match progress.finished() {
                                    // If aiming is finished, spawn projectile
                                    true => {
                                        tracing::trace!(name="individual-processor-suppress-gesture-aiming-progress-finished", i=?self.i);

                                        let tile = TileXy::from_(target, &self.world.w);
                                        let tile = WorldTileIndex::from_(tile, &self.world.w);
                                        match self.world.tile(tile) {
                                            Some(tile) => {
                                                let z = tile.z_pixels(&self.world.w);
                                                let target = WorldVec3::new(target.x, target.y, z);
                                                (
                                                    oc_individual::HandsGesture::Idle,
                                                    self.spawn_projectile(
                                                        individual,
                                                        weapon,
                                                        ammunition,
                                                        WeaponKind::Primary,
                                                        target,
                                                        visibility.opacity,
                                                    ),
                                                )
                                            }
                                            // No tile on given coordinates (should not happen)
                                            None => (oc_individual::HandsGesture::Idle, vec![]),
                                        }
                                    }
                                    // Else continue aiming
                                    false => {
                                        tracing::trace!(name="individual-processor-suppress-gesture-aiming-continue", i=?self.i);

                                        (oc_individual::HandsGesture::Aiming(progress), vec![])
                                    }
                                }
                            }
                        }
                    }
                    // The weapon is not ready to fire
                    None => {
                        tracing::trace!(name="individual-processor-suppress-gesture-weapon-not-ready-to-fire", i=?self.i);

                        match individual.gesture.hands {
                            // All other than reloading imply start reloading
                            oc_individual::HandsGesture::Idle
                            | oc_individual::HandsGesture::Aiming(_) => {
                                match self.can_reload(individual, weapon) {
                                    true => (
                                        oc_individual::HandsGesture::Reloading(U8Progress::zero()),
                                        vec![],
                                    ),
                                    // This weapon can't be reloaded, next individual tick will choose another weapon if any
                                    false => (oc_individual::HandsGesture::Idle, vec![]),
                                }
                            }
                            // If already reloading, continue
                            oc_individual::HandsGesture::Reloading(progress) => {
                                let weapon = self.world.mod_.weapon(weapon.i);
                                let progress = progress.tick(interval, weapon.reload());
                                tracing::trace!(name="individual-processor-suppress-gesture-weapon-reloading", i=?self.i, progress=progress.0);

                                match progress.finished() {
                                    // If reloading is finished, update weapons and start to aim
                                    true => {
                                        tracing::trace!(name="individual-processor-suppress-gesture-weapon-reloading-finished", i=?self.i);

                                        (
                                            oc_individual::HandsGesture::Aiming(U8Progress::zero()),
                                            vec![self.reloaded(
                                                individual,
                                                WeaponKind::Primary,
                                                weapon,
                                            )],
                                        )
                                    }
                                    // Else continue reloading
                                    false => {
                                        tracing::trace!(name="individual-processor-suppress-gesture-weapon-reloading-not-finished", i=?self.i);

                                        (oc_individual::HandsGesture::Reloading(progress), vec![])
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // No weapon adapted to the target
            None => {
                tracing::trace!(name="individual-processor-suppress-gesture-no-weapon-found", i=?self.i, target=?target);
                (oc_individual::HandsGesture::Idle, vec![])
            }
        };

        (
            Gesture::body(BodyGesture::Prone(direction)).with_hands(hands),
            updates,
        )
    }

    fn can_reload(&self, _individual: &Individual, _weapon: &oc_individual::Weapon) -> bool {
        // FIXME BS NOW: must be according to carried magazines
        true
    }

    fn reloaded(
        &self,
        individual: &Individual,
        kind: WeaponKind,
        weapon: &oc_mod::weapons::Weapon,
    ) -> runner::update::Update {
        let mut weapons = individual.weapons.clone();
        match kind {
            WeaponKind::Primary => {
                if let Some(weapon_) = &mut weapons.primary {
                    // FIXME BS NOW: According to transported magazines / etc
                    if let Some(magazine) = weapon.magazines().first() {
                        // FIXME BS NOW: According to transported magazines / etc
                        if let Some(ammunition) = weapon.ammunitions().first() {
                            weapon_.filled = Some((magazine.index(), ammunition.index()));
                            weapon_.filled_count = magazine.capacity();
                        }
                    }
                }
            }
        };
        runner::update::Update::UpdateIndividual(self.i, oc_individual::Update::SetWeapons(weapons))
    }

    fn spawn_projectile(
        &self,
        individual: &Individual,
        weapon: &oc_mod::weapons::IndexedWeapon,
        ammunition: oc_mod::ammunition::AmmunitionIndex,
        kind: WeaponKind,
        target: WorldVec3,
        opacity: CumulatedOpacity,
    ) -> Vec<runner::update::Update> {
        // FIXME BS NOW: how choose mode ?
        let_some!(shot = weapon.shots().first(), return vec![]);
        let repeat = 1;

        let mut weapons = individual.weapons.clone();
        match kind {
            WeaponKind::Primary => {
                if let Some(weapon) = &mut weapons.primary {
                    weapon.filled = None;
                    weapon.filled_count = 0;
                }
            }
        };

        // FIXME BS NOW: must ensure the individual z is updated according to tile z
        let plus_z = individual.gesture.body.weapon_z().pixels(&self.world.w);
        let from = WorldVec3 {
            x: individual.position.x,
            y: individual.position.y,
            z: individual.position.z + plus_z,
        };

        let direction = (target - from).normalize_or_zero();
        let direction = Vec3::new(direction.x, direction.y, direction.z);
        let directions = (0..(repeat * shot.rounds()))
            .map(|_| {
                let spread = match WorldConfig::inaccuracy_spread_enabled() {
                    true => WorldConfig::inaccuracy_spread(),
                    false => self.inaccuracy(individual, weapon, opacity),
                };
                let direction = direction_with_inaccuracy(direction, spread);
                WorldVec3::new(direction.x, direction.y, direction.z)
            })
            .collect::<Vec<_>>();

        let spawn = SpawnProjectiles {
            weapon: weapon.index(),
            ammunition: ammunition,
            shot: shot.index(),
            repeat: repeat as u8,
            from,
            // FIXME BS NOW: berk, create Direction3d
            directions,
            side: individual.side,
        };
        tracing::trace!(name="individual-processor-spawn-projectiles", i=?self.i, spawn=?spawn, weapons=?weapons);

        vec![
            runner::update::Update::SpawnProjectiles(spawn),
            runner::update::Update::UpdateIndividual(
                self.i,
                oc_individual::Update::SetWeapons(weapons),
            ),
        ]
    }

    // TODO: introduce inaccuracy with recoil ?
    fn inaccuracy(
        &self,
        individual: &Individual,
        weapon: &oc_mod::weapons::IndexedWeapon,
        opacity: CumulatedOpacity,
    ) -> f32 {
        // TODO: Add time to aim factor ?
        let w = &self.world.w;
        let individual_xp_inaccuracy = individual.xp_inaccuracy(w);
        let individual_gesture_inaccuracy = individual.gesture.inaccuracy(w);
        let weapon_inaccuracy = weapon.inaccuracy(w);
        let individual_suppress_inaccuracy = individual.suppress_inaccuracy(w);
        let opacity_inaccuracy = opacity.inaccuracy(w);

        tracing::trace!(name="individual-processor-inaccuracy", i=?self.i,
            individual_xp_inaccuracy=individual_xp_inaccuracy,
            individual_gesture_inaccuracy=individual_gesture_inaccuracy,
            weapon_inaccuracy=weapon_inaccuracy,
            individual_suppress_inaccuracy=individual_suppress_inaccuracy,
            opacity_inaccuracy=opacity_inaccuracy,
        );

        w.base_inaccuracy
            + individual_xp_inaccuracy
            + individual_gesture_inaccuracy
            + weapon_inaccuracy
            + individual_suppress_inaccuracy
            + opacity_inaccuracy
    }

    fn weapon<'w>(
        &self,
        individual: &'w Individual,
        _target: &'w Individual,
    ) -> Option<(WeaponKind, &'w oc_individual::Weapon)> {
        // TODO: several things can change (knife when contact, secondary weapon when no more bullet on primary, etc)
        // TODO: choose of this weapon must imply it can be reloaded, else, choose another
        individual
            .weapons
            .primary
            .as_ref()
            .map(|w| (WeaponKind::Primary, w))
    }

    fn primary_weapon<'w>(
        &self,
        individual: &'w Individual,
        _target: WorldVec2,
    ) -> Option<&'w oc_individual::Weapon> {
        individual.weapons.primary.as_ref().map(|w| w)
    }

    /// Compute forces for the current behavior.
    fn forces(&self, individual: &Individual, behavior: &Behavior, intent: &Intent) -> Vec<Force> {
        match behavior {
            Behavior::Idle(_) => vec![],
            Behavior::Walk(direction) | Behavior::Run(direction) | Behavior::Crawl(direction) => {
                let nominal_speed = behavior.nominal_speed();
                let move_disability_factor = self.move_disability_factor(individual, behavior);
                let arrival_factor = self.arrival_factor(intent);
                let direction = WorldVec3::new(direction.x, direction.y, 0.);
                vec![Force::Translation(
                    direction.into(),
                    nominal_speed * move_disability_factor * arrival_factor,
                )]
            }
            Behavior::Defend(_) => vec![],
            Behavior::Hide(_) => vec![],
            Behavior::Engage(_) => vec![],
            Behavior::Suppress(_) => vec![],
        }
    }

    fn move_disability_factor(&self, _individual: &Individual, _behavior: &Behavior) -> f32 {
        // TODO: here use fatigue, injures, etc
        1.0
    }

    /// Returns a value in [MIN_FORCE_FACTOR, 1.0] based on distance to the
    /// current waypoint: 1.0 far away, tapering down to MIN_FORCE_FACTOR
    /// within ARRIVAL_RADIUS. We compute it to ensure arrival near the targeted
    /// pixel and avoid turning around target.
    fn arrival_factor(&self, intent: &Intent) -> f32 {
        let path = match intent {
            Intent::Idle(_)
            | Intent::Defend(_)
            | Intent::Hide(_)
            | Intent::Engage(_)
            | Intent::Suppress(_) => return 1.0, // fallback (should not happens)
            Intent::MoveTo(_, path) => path,
            Intent::MoveFastTo(_, path) => path,
            Intent::SneakTo(_, path) => path,
        };
        let_some!(next = path.iter().next(), return 1.0); // 1.0 is fallback (should not happens)

        let individual = self.world.individual(self.i);
        let from = WorldVec2::from(individual.position);
        let distance = *next - from;
        let distance = Vec2::new(distance.x, distance.y).length();

        // No chance to miss the target, return full speed
        if distance >= ARRIVAL_RADIUS {
            1.0
        } else {
            // FIXME: 0.5 is a hack to reduce "missing target".
            // This arrival factor should probably consider the tick delay between each compute
            ((distance / ARRIVAL_RADIUS) * 0.5).max(MIN_FORCE_FACTOR)
        }
    }

    // TODO: can change to fire target if any
    fn resolve_idle_order(&self, individual: &Individual, situation: &Situation) -> Intent {
        tracing::trace!(name="individual-processor-resolve-idle-order", i=?self.i);
        let direction = individual.gesture.direction();

        if individual.suppress >= self.world.w.individual_suppress_limit_hide {
            tracing::trace!(name="individual-processor-resolve-idle-order-suppressed", i=?self.i);
            return Intent::Hide(direction);
        }

        // FIXME: introduce suppressed, etc (refactored way!)
        match &individual.intent {
            // If already engaging, continue try to engage this target
            Intent::Engage(target) => {
                tracing::trace!(name="individual-processor-resolve-idle-order-already-engaging", i=?self.i);
                self.engage_intent(individual, situation, *target, direction)
            }
            // TODO: something somewhere to prevent all squad members fire same individual (because nearest) ?
            _ => match situation.visibles.first() {
                Some(visible) => {
                    tracing::trace!(name="individual-processor-resolve-idle-order-target", i=?self.i);
                    self.engage_intent(individual, situation, visible.individual, direction)
                }
                None => {
                    tracing::trace!(name="individual-processor-resolve-idle-order-no-target", i=?self.i);
                    Intent::Idle(direction)
                }
            },
        }
    }

    fn engage_intent(
        &self,
        _individual: &Individual,
        situation: &Situation,
        target: IndividualIndex,
        direction: Direction,
    ) -> Intent {
        match situation.visible(target).is_some() {
            true => {
                tracing::trace!(name="individual-processor-engage-intent-visible", i=?self.i, target=?target);
                Intent::Engage(target)
            }
            false => {
                tracing::trace!(name="individual-processor-engage-intent-not-visible", i=?self.i, target=?target);
                Intent::Idle(direction)
            }
        }
    }

    // TODO: can change to fire target if any
    fn resolve_defend_order(
        &self,
        individual: &Individual,
        situation: &Situation,
        direction: Direction,
    ) -> Intent {
        tracing::trace!(name="individual-processor-resolve-defend-order", i=?self.i);

        if individual.suppress >= self.world.w.individual_suppress_limit_hide {
            tracing::trace!(name="individual-processor-resolve-defend-order-suppressed", i=?self.i);
            return Intent::Hide(direction);
        }

        // FIXME: introduce suppressed, etc (refactored way!)
        match &individual.intent {
            Intent::Engage(target) => Intent::Engage(*target),
            _ => match situation.visibles.first() {
                Some(visible) => Intent::Engage(visible.individual),
                None => Intent::Defend(direction),
            },
        }
    }

    // TODO: can change to fire target if any and near individual
    fn resolve_hide_order(
        &self,
        individual: &Individual,
        situation: &Situation,
        direction: Direction,
    ) -> Intent {
        tracing::trace!(name="individual-processor-resolve-hide-order", i=?self.i);

        if individual.suppress >= self.world.w.individual_suppress_limit_hide {
            tracing::trace!(name="individual-processor-resolve-hide-order-suppressed", i=?self.i);
            return Intent::Hide(direction);
        }

        // FIXME: introduce suppressed, etc (refactored way!)
        match &individual.intent {
            // FIXME: maybe stop engage if enemy is not anymore near ?
            Intent::Engage(target) => Intent::Engage(*target),
            _ => match situation
                .visibles
                .iter()
                // FIXME BS NOW: must engage if squad already engaging near (to avoid not engage "just" near)
                // maybe a second distance ?
                .filter(|v| v.distance <= HIDE_ENGAGE_DISTANCE)
                .next()
            {
                Some(visible) => Intent::Engage(visible.individual),
                None => Intent::Hide(direction),
            },
        }
    }

    fn resolve_move_to_order(
        &self,
        individual: &Individual,
        situation: &Situation,
        position: WorldVec2,
    ) -> Intent {
        tracing::trace!(name="individual-processor-resolve-move-to-order", i=?self.i);

        if individual.suppress >= self.world.w.individual_suppress_limit_hide {
            tracing::trace!(name="individual-processor-resolve-move-to-order-suppressed", i=?self.i);
            let direction = individual.gesture.direction();
            return Intent::Hide(direction);
        }

        // FIXME BS NOW: sneak if tired (and can't run)
        // FIXME BS NOW: impact of suppress ?
        match self.resolve_path(individual, position) {
            Some(path) => match !situation.visibles.is_empty() {
                // Move fast as enemy can fire on them
                true => {
                    tracing::trace!(name="individual-processor-resolve-move-to-order-path-move-fast", i=?self.i);
                    Intent::MoveFastTo(position, path)
                }
                // Move normally
                false => {
                    tracing::trace!(name="individual-processor-resolve-move-to-order-path-move", i=?self.i);
                    Intent::MoveTo(position, path)
                }
            },
            None => {
                tracing::trace!(name="individual-processor-resolve-move-to-order-no-path", i=?self.i);
                let reference = individual.position.into();
                let target: Vec2 = position.into();
                let direction = Direction::from_points3d(reference, target.extend(0.));
                Intent::Hide(direction)
            }
        }
    }

    fn resolve_move_fast_to_order(
        &self,
        individual: &Individual,
        _situation: &Situation,
        position: WorldVec2,
    ) -> Intent {
        tracing::trace!(name="individual-processor-resolve-move-fast-to-order", i=?self.i);

        if individual.suppress >= self.world.w.individual_suppress_limit_hide {
            tracing::trace!(name="individual-processor-resolve-move-fast-to-order-suppressed", i=?self.i);
            let direction = individual.gesture.direction();
            return Intent::Hide(direction);
        }

        // FIXME BS NOW: sneak if tired (and can't run)
        // FIXME BS NOW: impact of suppress ?
        match self.resolve_path(individual, position) {
            Some(path) => {
                tracing::trace!(name="individual-processor-resolve-move-fast-to-order-path", i=?self.i, position=?position, path=?path);
                Intent::MoveFastTo(position, path)
            }
            None => {
                tracing::trace!(name="individual-processor-resolve-move-fast-to-order-no-path", i=?self.i);
                let reference = individual.position.into();
                let target: Vec2 = position.into();
                let direction = Direction::from_points3d(reference, target.extend(0.));
                Intent::Hide(direction)
            }
        }
    }

    // FIXME BS NOW: impact of suppress ?
    fn resolve_sneak_to_order(
        &self,
        individual: &Individual,
        _situation: &Situation,
        position: WorldVec2,
    ) -> Intent {
        tracing::trace!(name="individual-processor-resolve-sneak-to-order", i=?self.i);

        if individual.suppress >= self.world.w.individual_suppress_limit_hide {
            tracing::trace!(name="individual-processor-resolve-sneak-to-order-suppressed", i=?self.i);
            let direction = individual.gesture.direction();
            return Intent::Hide(direction);
        }

        match self.resolve_path(individual, position) {
            Some(path) => {
                tracing::trace!(name="individual-processor-resolve-sneak-to-order-path", i=?self.i, position=?position, path=?path);
                Intent::SneakTo(position, path)
            }
            None => {
                tracing::trace!(name="individual-processor-resolve-sneak-to-order-no-path", i=?self.i);
                let reference = individual.position.into();
                let target: Vec2 = position.into();
                let direction = Direction::from_points3d(reference, target.extend(0.));
                Intent::Hide(direction)
            }
        }
    }

    // FIXME BS NOW: impact of suppress ?
    // FIXME BS NOW: impact of no ammo / weapon for target ?
    fn resolve_engage_order(
        &self,
        individual: &Individual,
        situation: &Situation,
        target: IndividualIndex,
    ) -> Intent {
        tracing::trace!(name="individual-processor-resolve-engage-order", i=?self.i, target=?target);

        if individual.suppress >= self.world.w.individual_suppress_limit_hide {
            tracing::trace!(name="individual-processor-resolve-engage-order-suppressed", i=?self.i, target=?target);
            let direction = individual.gesture.direction();
            return Intent::Hide(direction);
        }

        match situation.visible(target) {
            // Target is visible, engage it
            Some(_) => {
                tracing::trace!(name="individual-processor-resolve-engage-order-visible", i=?self.i, target=?target);
                Intent::Engage(target)
            }
            // Target is not visible
            None => {
                tracing::trace!(name="individual-processor-resolve-engage-order-not-visible", i=?self.i, target=?target);
                // Try to find another
                match situation.visibles.first() {
                    // Engage this one
                    Some(target) => {
                        tracing::trace!(name="individual-processor-resolve-engage-order-other-visible", i=?self.i, target=?target);
                        Intent::Engage(target.individual)
                    }
                    // No target possible, hide
                    None => {
                        tracing::trace!(name="individual-processor-resolve-engage-order-none", i=?self.i);
                        // And use original target as direction
                        let target = self.world.individual(target);
                        let reference = individual.position.into();
                        let target = target.position.into();
                        let direction = Direction::from_points3d(reference, target);
                        Intent::Hide(direction)
                    }
                }
            }
        }
    }

    // FIXME BS NOW: impact of suppress ?
    fn resolve_suppress_order(
        &self,
        individual: &Individual,
        _situation: &Situation,
        target: WorldVec2,
    ) -> Intent {
        tracing::trace!(name="individual-processor-resolve-suppress-order", i=?self.i);

        if individual.suppress >= self.world.w.individual_suppress_limit_hide {
            tracing::trace!(name="individual-processor-resolve-suppress-order-suppressed", i=?self.i);
            let direction = individual.gesture.direction();
            return Intent::Hide(direction);
        }

        Intent::Suppress(target)
    }

    /// Compute path to target, or reuse current if already known path
    fn resolve_path(&self, individual: &Individual, target: WorldVec2) -> Option<MovePath> {
        if let Some((current_target, current_path)) = individual.intent.path() {
            if current_target == target && current_path.iter().next().is_some() {
                return Some(current_path.clone());
            }
        }

        let from = (individual.position.x, individual.position.y);
        let to = (target.x, target.y);
        self.world.navmesh.path(from, to).map(MovePath::from)
    }

    fn direction_to_next(
        &self,
        individual: &Individual,
        path: &MovePath,
        trace_name: &'static str,
    ) -> Option<Direction> {
        let Some(next) = path.iter().next() else {
            tracing::trace!(name = format!("individual-step-act-{trace_name}-no-next"), i=?self.i);
            return None;
        };

        let from = WorldVec2::from(individual.position);
        let direction = (*next - from).normalize_or_zero();
        Some(Direction::from(direction))
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use oc_individual::{
        BodyGesture, Gesture, HandsGesture, Individual, IndividualIndex, Weapon, Weapons,
        behavior::{Behavior, Intent},
        order::Order,
        squad::SquadIndex,
    };
    use oc_mod::{Mod, ammunition::AmmunitionIndex, magazine::MagazineIndex, weapons::WeaponIndex};
    use oc_root::{
        Suppress, U8Progress, WorldConfig,
        geo::{WorldVec2, WorldVec3},
        opacity::CumulatedOpacity,
        physics::Meters,
    };
    use oc_utils::d2::Direction;
    use oc_world::{World, visibility::Visibility};

    use crate::{index::Indexes, individual::Processor, runner::update::Update};
    use tests::{
        individual::TestIndividual,
        squad::TestSquad,
        utils::workspace_root,
        weapons::{TestWeapon, TestWeapons},
        world::TestWorld,
    };

    const MOD: &str = "mods/tests1";

    // Test orders distribution when squad own move to order
    #[test]
    fn test_distribute_move() {
        // Given
        let w = WorldConfig::new(100, 100, Meters(0.1))
            .formation_tiles_between_positions(2)
            .geo_pixels_per_tile(5);
        // test parameters (assume individual are all Idle in EST direction)
        let individual_1_position = WorldVec3::new(100., 100., 0.);
        let individual_2_position = WorldVec3::new(90., 110., 0.);
        let squad_position = WorldVec2::new(individual_1_position.x, individual_1_position.y);
        let move_to_position = WorldVec2::new(150., 100.);
        let move_to_order = Order::MoveTo(move_to_position);
        // expected
        let expected_individual_1_move_to_position = WorldVec2::new(150., 100.);
        let expected_individual_2_move_to_position = WorldVec2::new(100., 110.);

        let world = two_individuals_world(
            &w,
            individual_1_position,
            individual_2_position,
            squad_position,
            vec![move_to_order],
        );
        let index = Indexes::new(&world, &w);
        let processor = Processor::new(&world, &index, 0.into());

        // When
        let updates = processor.step();

        // Then
        assert!(updates.contains(&Update::UpdateIndividual(
            IndividualIndex(0),
            oc_individual::Update::SetOrders(vec![Order::MoveTo(
                expected_individual_1_move_to_position
            )])
        )));
        assert!(updates.contains(&Update::UpdateIndividual(
            IndividualIndex(1),
            oc_individual::Update::SetOrders(vec![Order::MoveTo(
                expected_individual_2_move_to_position
            )])
        )));
    }

    // Test orders distribution when squad have no order and squad member not at correct place
    #[test]
    fn test_distribute_idle() {
        // Given
        let w = WorldConfig::new(100, 100, Meters(0.1))
            .formation_tiles_between_positions(2)
            .geo_pixels_per_tile(5);
        // test parameters (assume individual are all Idle in EST direction)
        let individual_1_position = WorldVec3::new(100., 100., 0.);
        let individual_2_position = WorldVec3::new(90., 110., 0.);
        let squad_position = WorldVec2::new(individual_1_position.x, individual_1_position.y);
        // expected
        let expected_individual_2_move_to_position = WorldVec2::new(100., 110.);

        let world = two_individuals_world(
            &w,
            individual_1_position,
            individual_2_position,
            squad_position,
            vec![],
        );
        let index = Indexes::new(&world, &w);
        let processor = Processor::new(&world, &index, 0.into());

        // When
        let updates = processor.step();

        // Then
        assert!(updates.contains(&Update::UpdateIndividual(
            IndividualIndex(1),
            oc_individual::Update::SetOrders(vec![Order::MoveTo(
                expected_individual_2_move_to_position
            )])
        )),);
    }

    // Test update when individual have no order
    #[test]
    fn test_idle() {
        // Given
        let w = WorldConfig::new(100, 100, Meters(0.1))
            .formation_tiles_between_positions(2)
            .geo_pixels_per_tile(5);
        // test parameters (assume individual are all Idle in EST direction)
        let position = WorldVec3::new(100., 100., 0.);

        let world = one_individual_world(&w, position, vec![]);
        let index = Indexes::new(&world, &w);
        let processor = Processor::new(&world, &index, 0.into());

        // When
        let updates = processor.step();

        // Then
        assert_eq!(
            updates,
            vec![Update::UpdateIndividual(
                IndividualIndex(0),
                oc_individual::Update::SetSuppress(Suppress(0))
            )]
        );
    }

    // Test update when individual have idle order in other direction than current
    #[test]
    fn test_idle_order() {
        // Given
        let w = WorldConfig::new(100, 100, Meters(0.1))
            .formation_tiles_between_positions(2)
            .geo_pixels_per_tile(5);
        // test parameters (assume individual are all Idle in EST direction)
        let position = WorldVec3::new(100., 100., 0.);

        let mut world = one_individual_world(&w, position, vec![Order::Idle]);
        let index = Indexes::new(&world, &w);

        // When-Then
        {
            let processor = Processor::new(&world, &index, 0.into());
            let updates = processor.step();
            assert!(updates.contains(&Update::UpdateIndividual(
                IndividualIndex(0),
                oc_individual::Update::SetOrders(vec![Order::Idle])
            )));
        }

        // When-Then
        world.individuals[0].orders = vec![Order::Idle];
        let processor = Processor::new(&world, &index, 0.into());
        let updates = processor.step();
        assert!(updates.contains(&Update::UpdateIndividual(
            IndividualIndex(0),
            oc_individual::Update::Accomplished
        )));
        assert!(updates.contains(&Update::UpdateIndividual(
            IndividualIndex(0),
            oc_individual::Update::SetIntent(Intent::Idle(Direction::EST))
        )));
        assert!(updates.contains(&Update::UpdateSquad(
            SquadIndex(0),
            oc_individual::squad::Update::Accomplished
        )));
        assert!(updates.contains(&Update::UpdateSquad(
            SquadIndex(0),
            oc_individual::squad::Update::SetOrders(vec![])
        )));
    }

    fn engage_test_gesture(
        w: WorldConfig,
        setup_individual: impl FnOnce(&mut Individual),
    ) -> (Gesture, Vec<Update>) {
        // Given
        let position1 = WorldVec3::new(100., 100., 0.);
        let position2 = WorldVec3::new(100., 200., 0.);
        let mut world = one_vs_one_individual_world(
            &w,
            position1,
            position2,
            vec![Order::Idle],
            vec![Order::Idle],
        );
        setup_individual(world.individual_mut(IndividualIndex(0)));
        *world.visibilities.values_mut() = vec![
            vec![
                Visibility::new(true, CumulatedOpacity(0.)),
                Visibility::new(true, CumulatedOpacity(0.)),
            ],
            vec![
                Visibility::new(true, CumulatedOpacity(0.)),
                Visibility::new(true, CumulatedOpacity(0.)),
            ],
        ];
        let index = Indexes::new(&world, &w);
        let processor = Processor::new(&world, &index, 0.into());
        let individual1 = world.individual(IndividualIndex(0));
        let situation = processor.situation(individual1).clone();
        // When
        processor.engage_gesture(individual1, &situation, IndividualIndex(1))
    }

    #[test]
    fn test_engage_begin() {
        // When-Then
        let mod_ = Mod::load(&workspace_root().join(MOD), None).unwrap();
        let w = WorldConfig::new(100, 100, Meters(0.1));
        let gesture = engage_test_gesture(w, |individual1| {
            *individual1 = individual1.clone().with_weapons(
                TestWeapons::builder()
                    .primary(TestWeapon::filled(&mod_, "Weapon1").make())
                    .build()
                    .make(),
            );
        });

        // Then
        assert_eq!(gesture.0.hands, HandsGesture::Aiming(U8Progress::zero()));
        assert_eq!(gesture.1, vec![]);
    }

    #[test]
    fn test_engage_aiming_progress() {
        // When-Then
        let mod_ = Mod::load(&workspace_root().join(MOD), None).unwrap();
        let w = WorldConfig::new(100, 100, Meters(0.1))
            // 10 tick per seconds
            // (must be configured as it) 1.0 seconds to aim "Weapon1"
            // So, one tick -> 255 / 10 = 25 u8
            .individual_tick_interval_us(1_000_000 / 10);
        let gesture = engage_test_gesture(w, |individual1| {
            individual1.gesture = individual1
                .gesture
                .clone()
                .with_hands(HandsGesture::Aiming(U8Progress(0)));
            *individual1 = individual1.clone().with_weapons(
                TestWeapons::builder()
                    .primary(TestWeapon::filled(&mod_, "Weapon1").make())
                    .build()
                    .make(),
            );
        });

        // Then
        assert_eq!(gesture.0.hands, HandsGesture::Aiming(U8Progress(25)));
        assert_eq!(gesture.1, vec![]);
    }

    #[test]
    fn test_engage_aiming_finished() {
        // When-Then
        let mod_ = Mod::load(&workspace_root().join(MOD), None).unwrap();
        let w = WorldConfig::new(100, 100, Meters(0.1));
        let gesture = engage_test_gesture(w, |individual1| {
            individual1.gesture = individual1
                .gesture
                .clone()
                .with_hands(HandsGesture::Aiming(U8Progress(254)));
            *individual1 = individual1.clone().with_weapons(
                TestWeapons::builder()
                    .primary(TestWeapon::filled(&mod_, "Weapon1").make())
                    .build()
                    .make(),
            );
        });

        // Then
        assert_eq!(gesture.0.hands, HandsGesture::Idle);
        assert_eq!(gesture.1.len(), 2);
        assert_matches!(gesture.1[0], Update::SpawnProjectiles(_));
        assert_eq!(
            gesture.1[1],
            Update::UpdateIndividual(
                IndividualIndex(0),
                oc_individual::Update::SetWeapons(Weapons {
                    primary: Some(Weapon {
                        i: WeaponIndex(0),
                        filled: None,
                        filled_count: 0
                    })
                })
            )
        );
    }

    #[test]
    fn test_engage_reload_finished() {
        // When-Then
        let mod_ = Mod::load(&workspace_root().join(MOD), None).unwrap();
        let w = WorldConfig::new(100, 100, Meters(0.1));
        let gesture = engage_test_gesture(w, |individual1| {
            individual1.gesture = individual1
                .gesture
                .clone()
                .with_hands(HandsGesture::Reloading(U8Progress(254)));
            *individual1 = individual1.clone().with_weapons(
                TestWeapons::builder()
                    .primary(TestWeapon::not_filled(&mod_, "Weapon1").make())
                    .build()
                    .make(),
            );
        });

        // Then
        assert_eq!(gesture.0.hands, HandsGesture::Aiming(U8Progress(0)));
        assert_eq!(
            gesture.1,
            vec![Update::UpdateIndividual(
                IndividualIndex(0),
                oc_individual::Update::SetWeapons(Weapons {
                    primary: Some(Weapon {
                        i: WeaponIndex(0),
                        filled: Some((MagazineIndex(0), AmmunitionIndex(0))),
                        filled_count: 5
                    })
                })
            )]
        );
    }

    // Refactored function which generate a world with one squad composed of two members.
    // Both individuals Idle in EST direction.
    fn two_individuals_world(
        w: &WorldConfig,
        individual_1_position: WorldVec3,
        individual_2_position: WorldVec3,
        squad_position: WorldVec2,
        squad_orders: Vec<Order>,
    ) -> World {
        let individual1 = TestIndividual::builder();
        let individual1 = individual1.position(individual_1_position);
        let individual1 = individual1
            .gesture(Gesture::body(BodyGesture::StandUp(Direction::EST))) // Gesture & Behavior & Intent are important
            .behavior(Behavior::Idle(Direction::EST)) // to conditionate the .step() response
            .intent(Intent::Idle(Direction::EST));
        let individual1 = individual1.build().make(w);
        let individual2 = TestIndividual::builder();
        let individual2 = individual2.position(individual_2_position);
        let individual2 = individual2
            .gesture(Gesture::body(BodyGesture::StandUp(Direction::EST))) // Gesture & Behavior & Intent are important
            .behavior(Behavior::Idle(Direction::EST)) // to conditionate the .step() response
            .intent(Intent::Idle(Direction::EST));
        let individual2 = individual2.build().make(w);

        let squad = TestSquad::builder();
        let squad = squad.position(squad_position);
        let squad = squad.members(vec![0.into(), 1.into()]);
        let squad = squad.orders(squad_orders);
        let squad = squad.build().make();

        let world = TestWorld::builder();
        let world = world.individuals(vec![individual1, individual2]);
        let world = world.squads(vec![squad]);
        let world = world.build().make(w);

        world
    }

    // Refactored function which generate a world with one squad composed of one member.
    // Individuals Idle in EST direction.
    fn one_individual_world(w: &WorldConfig, position: WorldVec3, orders: Vec<Order>) -> World {
        let individual = TestIndividual::builder();
        let individual = individual.position(position);
        let individual = individual
            .gesture(Gesture::body(BodyGesture::StandUp(Direction::EST))) // Gesture & Behavior & Intent are important
            .behavior(Behavior::Idle(Direction::EST)) // to conditionate the .step() response
            .intent(Intent::Idle(Direction::EST));
        let individual = individual.build().make(w);

        let squad = TestSquad::builder();
        let squad = squad.position(WorldVec2::new(position.x, position.y));
        let squad = squad.members(vec![0.into()]);
        let squad = squad.orders(orders);
        let squad = squad.build().make();

        let world = TestWorld::builder();
        let world = world.individuals(vec![individual]);
        let world = world.squads(vec![squad]);
        let world = world.build().make(&w);

        world
    }

    // Refactored function which generate a world with one squad composed of one member.
    // Individuals Idle in EST direction.
    fn one_vs_one_individual_world(
        w: &WorldConfig,
        position1: WorldVec3,
        position2: WorldVec3,
        orders1: Vec<Order>,
        orders2: Vec<Order>,
    ) -> World {
        let individual1 = TestIndividual::builder();
        let individual1 = individual1.position(position1);
        let individual1 = individual1
            .gesture(Gesture::body(BodyGesture::StandUp(Direction::EST))) // Gesture & Behavior & Intent are important
            .behavior(Behavior::Idle(Direction::EST)) // to conditionate the .step() response
            .intent(Intent::Idle(Direction::EST));
        let individual1 = individual1.build().make(w);

        let squad1 = TestSquad::builder();
        let squad1 = squad1.position(WorldVec2::new(position1.x, position1.y));
        let squad1 = squad1.members(vec![0.into()]);
        let squad1 = squad1.orders(orders1);
        let squad1 = squad1.build().make();

        let individual2 = TestIndividual::builder();
        let individual2 = individual2.position(position2);
        let individual2 = individual2
            .gesture(Gesture::body(BodyGesture::StandUp(Direction::EST))) // Gesture & Behavior & Intent are important
            .behavior(Behavior::Idle(Direction::EST)) // to conditionate the .step() response
            .intent(Intent::Idle(Direction::EST));
        let individual2 = individual2.build().make(w);

        let squad2 = TestSquad::builder();
        let squad2 = squad2.position(WorldVec2::new(position2.x, position2.y));
        let squad2 = squad2.members(vec![1.into()]);
        let squad2 = squad2.orders(orders2);
        let squad2 = squad2.build().make();

        let world = TestWorld::builder();
        let world = world.individuals(vec![individual1, individual2]);
        let world = world.squads(vec![squad1, squad2]);
        let world = world.build().make(&w);

        world
    }
}
