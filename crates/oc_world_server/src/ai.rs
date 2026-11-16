//! Run an AI ([`oc_ai::Commander`]) playing one side.
use glam::Vec2;
use oc_ai::{
    AiKind, Command, Commander, Perception,
    perception::{Candidate, CandidateKind, centroid},
};
use oc_deployment::deployment::Deployments as Deployment;
use oc_geo::tile::WorldTileIndex;
use oc_individual::squad::SquadIndex;
use oc_root::material::MaterialKind;
use oc_root::uuids::SquadUuid;
use oc_root::{WcfgFrom, battle::BattlePhase, geo::WorldVec2, side::Side};
use oc_utils::{d2::Angle, polygon::Polygon};
use oc_world::World;
use std::f32::consts::FRAC_PI_2;

use crate::runner::{
    input::{request_support, set_leader_position},
    update::Update,
};

/// Distance (meters) between two open deployment candidates
const OPEN_CANDIDATES_STEP: f32 = 10.0;
/// Minimal cover of an open deployment candidate
const OPEN_CANDIDATES_MIN_COVER: f32 = 0.3;
/// Radius (meters) around a position used to compute its cover
const COVER_RADIUS: f32 = 3.0;
/// Minimal opacity of a wall tile
const WALL_MIN_OPACITY: f32 = 0.9;
/// Radius (tiles) around the middle of an interior edge where its wall tile is searched
const WALL_SEARCH_RADIUS: i32 = 3;

/// AI playing one side
#[derive(Debug, Clone)]
pub struct AiConfig {
    pub side: Side,
    pub kind: AiKind,
    /// Squads placed by scenario: AI must not move them
    pub placed: Vec<SquadUuid>,
}

/// AI playing one side: perceives the world, decides commands and converts them into updates
pub struct Ai {
    side: Side,
    commander: Box<dyn Commander>,
    placed: Vec<SquadUuid>,
    /// Computed once, at first deployment tick (squads and map don't change during deployment)
    deployment: Option<DeploymentKnowledge>,
}

/// What AI needs to deploy its squads
struct DeploymentKnowledge {
    /// Squads placed by scenario
    placed: Vec<SquadIndex>,
    /// Positions where squads can be placed
    candidates: Vec<Candidate>,
}

impl Ai {
    /// AI playing `side` with given commander
    pub fn new(side: Side, commander: Box<dyn Commander>) -> Self {
        Self {
            side,
            commander,
            placed: vec![],
            deployment: None,
        }
    }

    /// AI built from its configuration
    pub fn from_config(config: AiConfig) -> Self {
        let mut ai = Self::new(config.side, config.kind.commander());
        ai.placed = config.placed;
        ai
    }

    /// Side played by this AI
    pub fn side(&self) -> Side {
        self.side
    }

    /// What AI side knows about `world` (with deployment knowledge during deployment phase)
    pub fn perceive(&mut self, world: &World) -> Perception {
        let perception = Perception::from_world(world, self.side);
        if !matches!(world.phase, BattlePhase::Deployment) {
            return perception;
        }
        self.deployment_perceive(world, perception)
    }

    /// Add deployment knowledge (computed at first call) to perception
    fn deployment_perceive(&mut self, world: &World, perception: Perception) -> Perception {
        let (side, placed) = (self.side, &self.placed);
        let deployment = self.deployment.get_or_insert_with(|| DeploymentKnowledge {
            placed: placed_squads(world, side, placed),
            candidates: deployment_candidates(world, side),
        });
        perception
            .with_placed(&deployment.placed)
            .with_candidates(deployment.candidates.clone())
    }

    /// Commands decided by the AI commander
    pub fn decide(&mut self, perception: &Perception) -> Vec<Command> {
        self.commander.tick(perception)
    }

    /// World updates of given commands
    pub fn updates(&self, world: &World, commands: Vec<Command>) -> Vec<Update> {
        commands
            .into_iter()
            .flat_map(|command| self.command_updates(world, command))
            .collect()
    }

    /// World updates of one command. Commands not allowed for this AI (foreign squad, leader
    /// position out of deployment phase) produce no update.
    fn command_updates(&self, world: &World, command: Command) -> Vec<Update> {
        tracing::debug!(name="ai-command", side=?self.side, command=?command);

        match command {
            Command::SetOrders(squad, orders) if self.owns(world, squad) => {
                vec![Update::UpdateSquad(
                    squad,
                    oc_individual::squad::Update::SetOrders(orders),
                )]
            }
            Command::SetLeaderPosition(squad, position, angle)
                if self.owns(world, squad) && matches!(world.phase, BattlePhase::Deployment) =>
            {
                set_leader_position(world, squad, position, angle)
            }
            Command::RequestSupport(type_, position) => {
                request_support(world, self.side, type_, position)
            }
            Command::SetOrders(..) | Command::SetLeaderPosition(..) => vec![],
        }
    }

    /// True if `squad` is of AI side (warn if not)
    fn owns(&self, world: &World, squad: SquadIndex) -> bool {
        let own = world.squads().get(squad.0 as usize).map(|s| s.side) == Some(self.side);
        if !own {
            tracing::warn!(name="ai-command-foreign-squad", side=?self.side, squad=?squad);
        }
        own
    }
}

/// Squads of `side` with uuid in `placed`
fn placed_squads(world: &World, side: Side, placed: &[SquadUuid]) -> Vec<SquadIndex> {
    world
        .squads()
        .iter()
        .enumerate()
        .filter(|(i, s)| {
            s.side == side && world.uuids.squad(*i).is_some_and(|u| placed.contains(u))
        })
        .map(|(i, _)| SquadIndex(i as u64))
        .collect()
}

/// Positions in `side` spawn zones where its biggest squad can be placed: interior walls facing
/// the enemy, map places and high opacity positions.
pub fn deployment_candidates(world: &World, side: Side) -> Vec<Candidate> {
    let zones = world.spawn_zones(side);
    let inside = |p: WorldVec2| zones.iter().any(|z| z.contains(p));
    let count = biggest_squad_size(world, side);

    let candidates: Vec<Candidate> = candidate_positions(world, side)
        .into_iter()
        .filter(|(position, _, _)| inside(*position))
        .map(|(position, kind, angle)| Candidate {
            position,
            kind,
            angle,
            cover: cover(world, position),
        })
        .filter(|c| c.kind != CandidateKind::Open || c.cover >= OPEN_CANDIDATES_MIN_COVER)
        .filter(|c| squad_fits(world, side, c.position, c.angle, count))
        .collect();

    tracing::debug!(name="ai-deployment-candidates", side=?side, count=candidates.len());
    candidates
}

/// All possible deployment positions (not yet filtered), with their formation angle: interior
/// walls, map places and a grid over `side` spawn zones
fn candidate_positions(world: &World, side: Side) -> Vec<(WorldVec2, CandidateKind, Angle)> {
    let front = enemy_front(world, side);
    let facing = |(position, kind): (WorldVec2, CandidateKind)| {
        (position, kind, facing_angle(position, front))
    };

    let mut positions = interior_positions(world, front);
    positions.extend(place_positions(world).into_iter().map(facing));
    positions.extend(open_positions(world, side).into_iter().map(facing));
    positions
}

/// Mean position of spawn zones of `side` enemy
fn enemy_front(world: &World, side: Side) -> Option<WorldVec2> {
    let centers: Vec<WorldVec2> = world
        .spawn_zones(side.opposite())
        .iter()
        .filter_map(|zone| centroid(zone.points()))
        .collect();
    centroid(&centers)
}

/// Angle of a line formation at `position` facing `front` (horizontal if no front)
fn facing_angle(position: WorldVec2, front: Option<WorldVec2>) -> Angle {
    let Some(front) = front else {
        return Angle::zero();
    };
    let toward = Vec2::from(front) - Vec2::from(position);
    Angle(toward.y.atan2(toward.x) + FRAC_PI_2)
}

/// Positions on interior walls facing `front` (all walls if no front). Individuals inside an
/// interior only see outside from its walls: formation follows the wall.
fn interior_positions(
    world: &World,
    front: Option<WorldVec2>,
) -> Vec<(WorldVec2, CandidateKind, Angle)> {
    world
        .interiors
        .iter()
        .flat_map(|interior| facing_walls(world, interior.points(), front))
        .collect()
}

/// For each edge of polygon facing `front`: a wall tile near the edge middle and the edge angle
fn facing_walls(
    world: &World,
    points: &[WorldVec2],
    front: Option<WorldVec2>,
) -> Vec<(WorldVec2, CandidateKind, Angle)> {
    let Some(center) = centroid(points) else {
        return vec![];
    };

    edges(points)
        .filter(|(a, b)| front.is_none_or(|front| faces(center, *a, *b, front)))
        .filter_map(|(a, b)| {
            let middle = WorldVec2::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0);
            let position = wall_tile_near(world, middle)?;
            Some((position, CandidateKind::Interior, edge_angle(a, b)))
        })
        .collect()
}

/// Edges (start, end) of polygon
fn edges(points: &[WorldVec2]) -> impl Iterator<Item = (WorldVec2, WorldVec2)> + '_ {
    (0..points.len()).map(|i| (points[i], points[(i + 1) % points.len()]))
}

/// True if outside of edge (`a`, `b`), of a polygon centered on `center`, faces `front`
fn faces(center: WorldVec2, a: WorldVec2, b: WorldVec2, front: WorldVec2) -> bool {
    let (center, a, b, front) = (
        Vec2::from(center),
        Vec2::from(a),
        Vec2::from(b),
        Vec2::from(front),
    );
    let middle = (a + b) / 2.0;
    let mut outside = (b - a).perp();
    if outside.dot(middle - center) < 0.0 {
        outside = -outside;
    }
    outside.dot(front - middle) > 0.0
}

/// Angle of the line from `a` to `b`
fn edge_angle(a: WorldVec2, b: WorldVec2) -> Angle {
    Angle((b.y - a.y).atan2(b.x - a.x))
}

/// Center of the wall tile nearest to `position`, within [`WALL_SEARCH_RADIUS`] tiles
fn wall_tile_near(world: &World, position: WorldVec2) -> Option<WorldVec2> {
    let tile = world.w.geo_pixels_per_tile() as f32;
    let radius = WALL_SEARCH_RADIUS;

    (-radius..=radius)
        .flat_map(|dy| (-radius..=radius).map(move |dx| (dx, dy)))
        .map(|(dx, dy)| {
            WorldVec2::new(position.x + dx as f32 * tile, position.y + dy as f32 * tile)
        })
        .filter_map(|p| tile_center(world, p))
        .filter(|p| is_wall(world, *p))
        .min_by(|a, b| {
            Vec2::from(*a)
                .distance(position.into())
                .total_cmp(&Vec2::from(*b).distance(position.into()))
        })
}

/// Center of the tile at `position` (None if outside the map)
fn tile_center(world: &World, position: WorldVec2) -> Option<WorldVec2> {
    let w = &world.w;
    let tile = w.geo_pixels_per_tile() as f32;
    let (width, height) = (
        w.world_width() as f32 * tile,
        w.world_height() as f32 * tile,
    );
    if position.x < 0.0 || position.y < 0.0 || position.x >= width || position.y >= height {
        return None;
    }

    Some(WorldVec2::new(
        (position.x / tile).floor() * tile + tile / 2.0,
        (position.y / tile).floor() * tile + tile / 2.0,
    ))
}

/// True if tile at `position` is a wall: opaque and individuals can stand on it
fn is_wall(world: &World, position: WorldVec2) -> bool {
    world
        .tile(WorldTileIndex::from_(position, &world.w))
        .is_some_and(|tile| {
            !tile.allow.deny(MaterialKind::Individual)
                && world.mod_.nature(tile.nature).opacity >= WALL_MIN_OPACITY
        })
}

/// Map places
fn place_positions(world: &World) -> Vec<(WorldVec2, CandidateKind)> {
    world
        .places
        .iter()
        .map(|place| (WorldVec2::new(place.x, place.y), CandidateKind::Place))
        .collect()
}

/// Grid (each [`OPEN_CANDIDATES_STEP`] meters) over bounds of `side` spawn zones
fn open_positions(world: &World, side: Side) -> Vec<(WorldVec2, CandidateKind)> {
    let step = OPEN_CANDIDATES_STEP * world.w.geo_pixels_per_meters();
    let mut positions = vec![];

    for zone in world.spawn_zones(side) {
        let (min, max) = bounds(zone.points());
        let mut y = min.y + step / 2.0;
        while y < max.y {
            let mut x = min.x + step / 2.0;
            while x < max.x {
                positions.push((WorldVec2::new(x, y), CandidateKind::Open));
                x += step;
            }
            y += step;
        }
    }

    positions
}

/// Members count of the biggest squad of `side`
fn biggest_squad_size(world: &World, side: Side) -> usize {
    world
        .squads()
        .iter()
        .filter(|s| s.side == side)
        .map(|s| s.members.len())
        .max()
        .unwrap_or(1)
}

/// True if all members of a squad of `count` individuals, led from `position` in a line along
/// `angle`, are placed inside `side` spawn zones
fn squad_fits(world: &World, side: Side, position: WorldVec2, angle: Angle, count: usize) -> bool {
    let zones = world.spawn_zones(side);
    Deployment::members_positions(
        &world.w,
        &world.mod_,
        &world.tiles,
        zones,
        position,
        angle,
        count,
    )
    .into_iter()
    .all(|p| zones.iter().any(|z| z.contains(p)))
}

/// Mean opacity (0.0 to 1.0) of tiles around position
fn cover(world: &World, position: WorldVec2) -> f32 {
    let w = &world.w;
    let tile = w.geo_pixels_per_tile() as f32;
    let radius = (COVER_RADIUS * w.geo_pixels_per_meters() / tile).ceil() as i32;

    let opacities: Vec<f32> = (-radius..=radius)
        .flat_map(|dy| (-radius..=radius).map(move |dx| (dx, dy)))
        .map(|(dx, dy)| {
            WorldVec2::new(position.x + dx as f32 * tile, position.y + dy as f32 * tile)
        })
        .filter(|p| p.x >= 0.0 && p.y >= 0.0 && p.x < w.world_width() as f32 * tile)
        .filter_map(|p| world.tile(WorldTileIndex::from_(p, w)))
        .map(|t| world.mod_.nature(t.nature).opacity)
        .collect();

    if opacities.is_empty() {
        return 0.0;
    }
    opacities.iter().sum::<f32>() / opacities.len() as f32
}

/// Min and max corners of the rectangle containing all points
fn bounds(points: &[WorldVec2]) -> (WorldVec2, WorldVec2) {
    points.iter().fold(
        (
            WorldVec2::new(f32::MAX, f32::MAX),
            WorldVec2::new(f32::MIN, f32::MIN),
        ),
        |(min, max), p| {
            (
                WorldVec2::new(min.x.min(p.x), min.y.min(p.y)),
                WorldVec2::new(max.x.max(p.x), max.y.max(p.y)),
            )
        },
    )
}

#[cfg(test)]
mod tests {
    use std::f32::consts::{FRAC_PI_2, PI};

    use super::*;

    fn p(x: f32, y: f32) -> WorldVec2 {
        WorldVec2::new(x, y)
    }

    /// Square of 10 pixels, clockwise (in server coordinates, y going down)
    fn square() -> Vec<WorldVec2> {
        vec![p(0., 0.), p(10., 0.), p(10., 10.), p(0., 10.)]
    }

    #[test]
    fn edges_close_polygon() {
        let edges: Vec<_> = edges(&square()).collect();
        assert_eq!(edges.len(), 4);
        assert_eq!(edges[3], (p(0., 10.), p(0., 0.)));
    }

    #[test]
    fn only_edges_facing_front() {
        let center = p(5., 5.);
        let facing: Vec<_> = edges(&square())
            .filter(|(a, b)| faces(center, *a, *b, p(5., -100.)))
            .collect();
        // Enemy at north (negative y): only top edge faces it
        assert_eq!(facing, vec![(p(0., 0.), p(10., 0.))]);

        // Enemy at north east: top and right edges face it
        let facing = edges(&square())
            .filter(|(a, b)| faces(center, *a, *b, p(100., -100.)))
            .count();
        assert_eq!(facing, 2);
    }

    #[test]
    fn formation_follows_edge() {
        assert_eq!(edge_angle(p(0., 0.), p(10., 0.)), Angle(0.));
        assert_eq!(edge_angle(p(10., 0.), p(10., 10.)), Angle(FRAC_PI_2));
        assert_eq!(edge_angle(p(0., 10.), p(0., 0.)), Angle(-FRAC_PI_2));
        assert_eq!(edge_angle(p(10., 10.), p(0., 10.)), Angle(PI));
    }

    #[test]
    fn formation_faces_front() {
        // Enemy at north: horizontal line
        let angle = facing_angle(p(0., 0.), Some(p(0., -100.)));
        assert!(angle.0.abs() < f32::EPSILON);
        // Enemy at east: vertical line
        let angle = facing_angle(p(0., 0.), Some(p(100., 0.)));
        assert!((angle.0 - FRAC_PI_2).abs() < f32::EPSILON);
        assert_eq!(facing_angle(p(0., 0.), None), Angle::zero());
    }
}
