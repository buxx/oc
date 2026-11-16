use glam::Vec2;
use oc_individual::{IndividualIndex, Status, order::Order, squad::SquadIndex};
use oc_root::{battle::BattlePhase, geo::WorldVec2, physics::Meters, side::Side};
use oc_utils::{d2::Angle, polygon::Polygon};
use oc_world::{
    World,
    flag::{Flag, Ownership},
    support::SupportType,
};

/// What one side knows about the battle. Only contains information a human player of this side
/// would have (enemies are only those currently seen by an individual of the side), except
/// [`FlagView::enemies`] (AI cheat).
#[derive(Debug, Clone)]
pub struct Perception {
    pub side: Side,
    /// World distance unit (pixels) for one meter
    pub pixels_per_meter: f32,
    pub phase: BattlePhase,
    /// Squads of the side which still have alive members
    pub squads: Vec<SquadView>,
    /// Enemy individuals currently seen by at least one individual of the side
    pub enemies: Vec<EnemyView>,
    pub flags: Vec<FlagView>,
    /// Available count of each support type
    pub supports: Vec<(SupportType, u8)>,
    /// Center of each enemy spawn zone (where enemy comes from)
    pub enemy_spawns: Vec<WorldVec2>,
    /// Positions where a squad can be deployed (only during deployment phase, see
    /// [`Perception::with_candidates`])
    pub candidates: Vec<Candidate>,
}

/// Position where a squad leader can be placed during deployment
#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    pub position: WorldVec2,
    pub kind: CandidateKind,
    /// Angle of the squad line formation (along the wall for interiors)
    pub angle: Angle,
    /// Mean opacity (0.0 to 1.0) of tiles around the position
    pub cover: f32,
}

/// Kind of deployment position
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateKind {
    /// On a building wall facing the enemy (individuals inside a building only see outside from
    /// its walls)
    Interior,
    /// Map place (precise position chosen by map designer)
    Place,
    /// Any other position (like high opacity tiles)
    Open,
}

/// Squad of the side
#[derive(Debug, Clone, PartialEq)]
pub struct SquadView {
    pub index: SquadIndex,
    pub position: WorldVec2,
    pub actives: u8,
    pub orders: Vec<Order>,
    /// Placed by scenario: must not be moved during deployment
    pub placed: bool,
}

/// Enemy individual currently seen by the side
#[derive(Debug, Clone, PartialEq)]
pub struct EnemyView {
    pub index: IndividualIndex,
    pub position: WorldVec2,
    /// Squads of the side with at least one member seeing this enemy
    pub seen_by: Vec<SquadIndex>,
}

/// Flag and its current ownership
#[derive(Debug, Clone, PartialEq)]
pub struct FlagView {
    pub center: WorldVec2,
    pub ownership: Ownership,
    /// Positions of operational enemies inside the flag. AI cheats here: they are known even if
    /// not seen (used to fight for contested flags).
    pub enemies: Vec<WorldVec2>,
}

impl FlagView {
    /// True if `side` owns (or shares) this flag
    pub fn owned_by(&self, side: Side) -> bool {
        matches!(
            (&self.ownership, side),
            (Ownership::Both, _) | (Ownership::A, Side::A) | (Ownership::B, Side::B)
        )
    }

    /// True if both sides own this flag
    pub fn contested(&self) -> bool {
        self.ownership == Ownership::Both
    }

    /// True if only `side` owns this flag (not contested)
    pub fn controlled_by(&self, side: Side) -> bool {
        matches!(
            (&self.ownership, side),
            (Ownership::A, Side::A) | (Ownership::B, Side::B)
        )
    }
}

impl Perception {
    /// What `side` knows about the battle from `world`. Deployment knowledge (placed squads,
    /// candidates) must be added with [`Perception::with_placed`] and
    /// [`Perception::with_candidates`].
    pub fn from_world(world: &World, side: Side) -> Self {
        Self {
            side,
            pixels_per_meter: Meters(1.0).pixels(&world.w),
            phase: world.phase,
            squads: squads(world, side),
            enemies: enemies(world, side),
            flags: flags(world, side),
            supports: supports(world, side),
            enemy_spawns: enemy_spawns(world, side),
            candidates: vec![],
        }
    }

    /// Mark squads placed by scenario (they must not be moved during deployment)
    pub fn with_placed(mut self, placed: &[SquadIndex]) -> Self {
        for squad in &mut self.squads {
            squad.placed = placed.contains(&squad.index);
        }
        self
    }

    /// Set positions where squads can be deployed
    pub fn with_candidates(mut self, candidates: Vec<Candidate>) -> Self {
        self.candidates = candidates;
        self
    }

    /// Distance in meters between two positions
    pub fn meters(&self, a: WorldVec2, b: WorldVec2) -> f32 {
        Vec2::from(a).distance(b.into()) / self.pixels_per_meter
    }

    /// Item of `items` nearest to `from` (`position` gives the position of an item)
    pub fn nearest<T>(
        &self,
        from: WorldVec2,
        items: impl IntoIterator<Item = T>,
        position: impl Fn(&T) -> WorldVec2,
    ) -> Option<T> {
        items.into_iter().min_by(|a, b| {
            self.meters(from, position(a))
                .total_cmp(&self.meters(from, position(b)))
        })
    }

    /// Mean position of enemy spawn zones
    pub fn enemy_front(&self) -> Option<WorldVec2> {
        centroid(&self.enemy_spawns)
    }

    /// Engage order for given squad, only on enemies seen by its own members (squads not in
    /// contact keep their orders): keep current target while seen within `meters`, else nearest
    /// seen enemy within `meters`
    pub fn engage(&self, squad: &SquadView, meters: f32) -> Option<Order> {
        let in_range: Vec<&EnemyView> = self.enemies_in_range(squad, meters).collect();

        if let Some(Order::Engage(target)) = squad.orders.first()
            && in_range.iter().any(|e| e.index == *target)
        {
            return Some(Order::Engage(*target));
        }

        self.nearest(squad.position, in_range, |e| e.position)
            .map(|e| Order::Engage(e.index))
    }

    /// Enemies seen by members of `squad` and within `meters` of it
    fn enemies_in_range<'a>(
        &'a self,
        squad: &'a SquadView,
        meters: f32,
    ) -> impl Iterator<Item = &'a EnemyView> {
        self.enemies
            .iter()
            .filter(|e| e.seen_by.contains(&squad.index))
            .filter(move |e| self.meters(squad.position, e.position) <= meters)
    }
}

/// Squads of `side` which still have active members
fn squads(world: &World, side: Side) -> Vec<SquadView> {
    world
        .squads()
        .iter()
        .enumerate()
        .filter(|(_, s)| s.side == side && s.actives > 0)
        .map(|(i, s)| SquadView {
            index: SquadIndex(i as u64),
            position: s.position,
            actives: s.actives,
            orders: s.orders.clone(),
            placed: false,
        })
        .collect()
}

/// Individuals of `side` able to see, with their squad (grouped by squad)
fn watchers(world: &World, side: Side) -> Vec<(IndividualIndex, SquadIndex)> {
    let individuals = world.individuals();

    world
        .squads()
        .iter()
        .enumerate()
        .filter(|(_, s)| s.side == side)
        .flat_map(|(squad, s)| {
            s.members
                .iter()
                .map(move |member| (*member, SquadIndex(squad as u64)))
        })
        .filter(|(member, _)| {
            individuals
                .get(member.0 as usize)
                .is_some_and(|i| i.can_lov())
        })
        .collect()
}

/// Alive enemies of `side` seen by at least one of its individuals
fn enemies(world: &World, side: Side) -> Vec<EnemyView> {
    let watchers = watchers(world, side);

    world
        .individuals()
        .iter()
        .enumerate()
        .filter(|(_, i)| i.side != side && !matches!(i.status, Status::Dead))
        .filter_map(|(i, individual)| {
            let index = IndividualIndex(i as u64);
            let seen_by = seen_by(world, &watchers, index);
            (!seen_by.is_empty()).then(|| EnemyView {
                index,
                position: WorldVec2::new(individual.position.x, individual.position.y),
                seen_by,
            })
        })
        .collect()
}

/// Squads of `watchers` (grouped by squad) with at least one member seeing `target`
fn seen_by(
    world: &World,
    watchers: &[(IndividualIndex, SquadIndex)],
    target: IndividualIndex,
) -> Vec<SquadIndex> {
    let mut squads: Vec<SquadIndex> = watchers
        .iter()
        .filter(|(watcher, _)| {
            world
                .visibilities
                .for_(*watcher)
                .get(target.0 as usize)
                .is_some_and(|v| v.visible)
        })
        .map(|(_, squad)| *squad)
        .collect();
    squads.dedup();
    squads
}

/// All flags with their current ownership and enemies of `side` inside them
fn flags(world: &World, side: Side) -> Vec<FlagView> {
    world
        .flags
        .iter()
        .map(|(flag, ownership)| FlagView {
            center: flag.center(),
            ownership: ownership.clone(),
            enemies: enemies_in_flag(world, side, flag),
        })
        .collect()
}

/// Positions of operational enemies of `side` inside `flag` (seen or not)
fn enemies_in_flag(world: &World, side: Side, flag: &Flag) -> Vec<WorldVec2> {
    world
        .individuals()
        .iter()
        .filter(|i| i.side != side && matches!(i.status, Status::Operational))
        .map(|i| WorldVec2::new(i.position.x, i.position.y))
        .filter(|position| flag.contains(*position))
        .collect()
}

/// Available count of each support type of `side`
fn supports(world: &World, side: Side) -> Vec<(SupportType, u8)> {
    world.supports[side]
        .0
        .iter()
        .map(|(t, _)| (t.clone(), world.available_support(side, t)))
        .collect()
}

/// Center of each spawn zone of `side` enemy
fn enemy_spawns(world: &World, side: Side) -> Vec<WorldVec2> {
    world
        .spawn_zones(side.opposite())
        .iter()
        .filter_map(|zone| centroid(zone.points()))
        .collect()
}

/// Mean of given points
pub fn centroid(points: &[WorldVec2]) -> Option<WorldVec2> {
    if points.is_empty() {
        return None;
    }
    let sum = points
        .iter()
        .fold(Vec2::ZERO, |sum, p| sum + Vec2::from(*p));
    let mean = sum / points.len() as f32;
    Some(WorldVec2::new(mean.x, mean.y))
}
