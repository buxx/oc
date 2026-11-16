//! Combat rules shared by AIs: a squad is in combat when it engages an enemy seen by its own
//! members. A squad not in combat near a squad in combat goes to help it: it moves toward the
//! enemy engaged by this squad, until its own members see an enemy (then it engages).
use oc_individual::{IndividualIndex, order::Order, squad::SquadIndex};
use oc_root::geo::WorldVec2;

use crate::perception::{Perception, SquadView};

/// Distance (meters) under which a squad engages an enemy seen by its members
pub const ENGAGE_DISTANCE: f32 = 150.0;
/// Distance (meters) under which a squad not in combat goes to help a squad in combat
pub const HELP_DISTANCE: f32 = 100.0;
/// Distance (meters) under which a helping squad keeps its current move (avoid path changes each
/// time the followed enemy moves a little)
pub const HELP_MOVE_TOLERANCE: f32 = 10.0;

/// Squad in combat
#[derive(Debug, Clone, PartialEq)]
pub struct Engagement {
    pub squad: SquadIndex,
    pub position: WorldVec2,
    /// Engaged enemy individual
    pub target: IndividualIndex,
}

/// Squads, among given ones, in combat
pub fn engagements(perception: &Perception, squads: &[&SquadView]) -> Vec<Engagement> {
    squads
        .iter()
        .filter_map(|squad| engagement(perception, squad))
        .collect()
}

/// Engagement of squad if it sees an enemy within [`ENGAGE_DISTANCE`]
fn engagement(perception: &Perception, squad: &SquadView) -> Option<Engagement> {
    match perception.engage(squad, ENGAGE_DISTANCE)? {
        Order::Engage(target) => Some(Engagement {
            squad: squad.index,
            position: squad.position,
            target,
        }),
        _ => None,
    }
}

/// Combat order of squad: engage when in combat, else help the nearest squad in combat. None if
/// squad is not concerned by combat.
pub fn order(
    perception: &Perception,
    squad: &SquadView,
    engagements: &[Engagement],
) -> Option<Order> {
    engage_order(squad, engagements).or_else(|| help_order(perception, squad, engagements))
}

/// Engage order of squad when it is in combat
pub fn engage_order(squad: &SquadView, engagements: &[Engagement]) -> Option<Order> {
    engagements
        .iter()
        .find(|e| e.squad == squad.index)
        .map(|e| Order::Engage(e.target))
}

/// Order to help the nearest other squad in combat within [`HELP_DISTANCE`]: move toward the
/// enemy it engages
fn help_order(
    perception: &Perception,
    squad: &SquadView,
    engagements: &[Engagement],
) -> Option<Order> {
    let helped = helped(perception, squad, engagements)?;
    let target = target_position(perception, helped);

    Some(move_to(perception, squad, target, HELP_MOVE_TOLERANCE))
}

/// Position of the enemy engaged by squad in combat (its own position if enemy is not seen
/// anymore)
fn target_position(perception: &Perception, engagement: &Engagement) -> WorldVec2 {
    perception
        .enemies
        .iter()
        .find(|e| e.index == engagement.target)
        .map(|e| e.position)
        .unwrap_or(engagement.position)
}

/// Nearest other squad in combat within [`HELP_DISTANCE`] of squad
fn helped<'a>(
    perception: &Perception,
    squad: &SquadView,
    engagements: &'a [Engagement],
) -> Option<&'a Engagement> {
    let near = engagements.iter().filter(|e| {
        e.squad != squad.index && perception.meters(squad.position, e.position) <= HELP_DISTANCE
    });

    perception.nearest(squad.position, near, |e| e.position)
}

/// Move order to `position`. Current move is kept when it already goes within `tolerance` meters
/// of `position` (avoid path changes each time a followed position moves a little).
pub fn move_to(
    perception: &Perception,
    squad: &SquadView,
    position: WorldVec2,
    tolerance: f32,
) -> Order {
    match squad.orders.first() {
        Some(Order::MoveTo(current)) if perception.meters(*current, position) <= tolerance => {
            Order::MoveTo(*current)
        }
        _ => Order::MoveTo(position),
    }
}
