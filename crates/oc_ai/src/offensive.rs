//! Offensive, rule based, AI. For each squad, during fight:
//!
//! - if it sees an enemy nearby: engage it (see [`crate::combat`])
//! - else if a squad in combat is near: help it
//! - else: move to a flag not controlled by the side (neutral, enemy or contested), squads being
//!   spread over these flags. For a contested flag: move to the nearest enemy inside it (see
//!   [`crate::capture`])
//! - else (all flags controlled): stay idle
//!
//! Orders are only given when they change, to let squads accomplish them.
use oc_individual::order::Order;

use crate::{
    Commander, capture,
    combat::{self, Engagement},
    command::Command,
    perception::{Perception, SquadView},
};

/// Distance (meters) under which a squad keeps its current capture move (avoid path changes
/// when followed enemy moves a little)
pub const CAPTURE_MOVE_TOLERANCE: f32 = 10.0;

/// Offensive AI (see module documentation)
#[derive(Debug, Default)]
pub struct Offensive;

impl Commander for Offensive {
    /// Give orders to all squads, during fight phase only
    fn tick(&mut self, perception: &Perception) -> Vec<Command> {
        if !perception.phase.fight() {
            return vec![];
        }

        let squads: Vec<&SquadView> = perception.squads.iter().collect();
        let engagements = combat::engagements(perception, &squads);

        decide(perception, &squads, &engagements)
            .into_iter()
            .filter_map(|(squad, order)| {
                Command::set_orders_if_changed(squad, order.unwrap_or(Order::Idle))
            })
            .collect()
    }
}

/// Order for each given squad: combat order (engage, help) else capture order. None when squad
/// is not in combat and no flag is left to capture.
pub(crate) fn decide<'a>(
    perception: &Perception,
    squads: &[&'a SquadView],
    engagements: &[Engagement],
) -> Vec<(&'a SquadView, Option<Order>)> {
    let (fighting, free): (Vec<_>, Vec<_>) = squads
        .iter()
        .map(|squad| (*squad, combat::order(perception, squad, engagements)))
        .partition(|(_, order)| order.is_some());
    let free: Vec<&SquadView> = free.into_iter().map(|(squad, _)| squad).collect();

    fighting
        .into_iter()
        .chain(capture_orders(perception, free))
        .collect()
}

/// Capture order for each given squad, spread over flags (None when all flags are controlled)
fn capture_orders<'a>(
    perception: &Perception,
    squads: Vec<&'a SquadView>,
) -> Vec<(&'a SquadView, Option<Order>)> {
    let targets = capture::targets(perception, &squads);

    squads
        .into_iter()
        .zip(targets)
        .map(|(squad, target)| {
            let order = target
                .map(|target| combat::move_to(perception, squad, target, CAPTURE_MOVE_TOLERANCE));
            (squad, order)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use oc_individual::{IndividualIndex, squad::SquadIndex};
    use oc_root::{battle::BattlePhase, geo::WorldVec2, side::Side};
    use oc_world::flag::Ownership;

    use super::*;
    use crate::perception::{EnemyView, FlagView};

    fn perception(phase: BattlePhase) -> Perception {
        Perception {
            side: Side::B,
            pixels_per_meter: 1.0,
            phase,
            squads: vec![SquadView {
                index: SquadIndex(3),
                position: WorldVec2::new(0.0, 0.0),
                actives: 4,
                orders: vec![],
                placed: false,
            }],
            enemies: vec![],
            flags: vec![
                FlagView {
                    center: WorldVec2::new(1000.0, 0.0),
                    ownership: Ownership::A,
                    enemies: vec![],
                },
                FlagView {
                    center: WorldVec2::new(10.0, 0.0),
                    ownership: Ownership::B,
                    enemies: vec![],
                },
            ],
            supports: vec![],
            enemy_spawns: vec![],
            candidates: vec![],
        }
    }

    #[test]
    fn nothing_during_deployment() {
        assert!(
            Offensive
                .tick(&perception(BattlePhase::Deployment))
                .is_empty()
        );
    }

    #[test]
    fn go_to_nearest_not_owned_flag() {
        let commands = Offensive.tick(&perception(BattlePhase::Fight));
        assert_eq!(
            commands,
            vec![Command::SetOrders(
                SquadIndex(3),
                vec![Order::MoveTo(WorldVec2::new(1000.0, 0.0))]
            )]
        );
    }

    #[test]
    fn spread_squads_over_flags() {
        let mut perception = perception(BattlePhase::Fight);
        perception.flags[1].ownership = Ownership::A;
        perception.flags[1].center = WorldVec2::new(1100.0, 0.0);
        let mut second = perception.squads[0].clone();
        second.index = SquadIndex(4);
        perception.squads.push(second);
        assert_eq!(
            Offensive.tick(&perception),
            vec![
                Command::SetOrders(
                    SquadIndex(3),
                    vec![Order::MoveTo(WorldVec2::new(1000.0, 0.0))]
                ),
                Command::SetOrders(
                    SquadIndex(4),
                    vec![Order::MoveTo(WorldVec2::new(1100.0, 0.0))]
                ),
            ]
        );
    }

    #[test]
    fn go_to_contested_flag() {
        let mut perception = perception(BattlePhase::Fight);
        perception.flags[0].ownership = Ownership::Nobody;
        perception.flags[1].ownership = Ownership::Both;
        let commands = Offensive.tick(&perception);
        assert_eq!(
            commands,
            vec![Command::SetOrders(
                SquadIndex(3),
                vec![Order::MoveTo(WorldVec2::new(10.0, 0.0))]
            )]
        );
    }

    #[test]
    fn engage_near_enemy() {
        let mut perception = perception(BattlePhase::Fight);
        perception.enemies = vec![
            EnemyView {
                index: IndividualIndex(7),
                position: WorldVec2::new(100.0, 0.0),
                seen_by: vec![SquadIndex(3)],
            },
            EnemyView {
                index: IndividualIndex(8),
                position: WorldVec2::new(500.0, 0.0),
                seen_by: vec![SquadIndex(3)],
            },
        ];
        let commands = Offensive.tick(&perception);
        assert_eq!(
            commands,
            vec![Command::SetOrders(
                SquadIndex(3),
                vec![Order::Engage(IndividualIndex(7))]
            )]
        );
    }

    #[test]
    fn keep_attacking_when_enemy_not_seen_by_squad() {
        let mut perception = perception(BattlePhase::Fight);
        perception.enemies = vec![EnemyView {
            index: IndividualIndex(7),
            position: WorldVec2::new(100.0, 0.0),
            seen_by: vec![SquadIndex(42)],
        }];
        assert_eq!(
            Offensive.tick(&perception),
            vec![Command::SetOrders(
                SquadIndex(3),
                vec![Order::MoveTo(WorldVec2::new(1000.0, 0.0))]
            )]
        );
    }

    #[test]
    fn contested_flag_goes_to_enemy_inside() {
        let mut perception = perception(BattlePhase::Fight);
        perception.flags[1].ownership = Ownership::Both;
        perception.flags[1].enemies = vec![WorldVec2::new(15.0, 5.0), WorldVec2::new(30.0, 0.0)];
        assert_eq!(
            Offensive.tick(&perception),
            vec![Command::SetOrders(
                SquadIndex(3),
                vec![Order::MoveTo(WorldVec2::new(15.0, 5.0))]
            )]
        );
    }

    #[test]
    fn help_near_squad_in_combat() {
        let mut perception = perception(BattlePhase::Fight);
        let mut fighting = perception.squads[0].clone();
        fighting.index = SquadIndex(4);
        fighting.position = WorldVec2::new(50.0, 0.0);
        perception.squads.push(fighting);
        perception.enemies = vec![EnemyView {
            index: IndividualIndex(7),
            position: WorldVec2::new(150.0, 0.0),
            seen_by: vec![SquadIndex(4)],
        }];

        // Helping squad goes toward the enemy engaged by helped squad
        assert_eq!(
            Offensive.tick(&perception),
            vec![
                Command::SetOrders(
                    SquadIndex(3),
                    vec![Order::MoveTo(WorldVec2::new(150.0, 0.0))]
                ),
                Command::SetOrders(SquadIndex(4), vec![Order::Engage(IndividualIndex(7))]),
            ]
        );

        // Enemy moved a little: current move is kept
        perception.squads[0].orders = vec![Order::MoveTo(WorldVec2::new(150.0, 0.0))];
        perception.squads[1].orders = vec![Order::Engage(IndividualIndex(7))];
        perception.enemies[0].position = WorldVec2::new(155.0, 0.0);
        assert!(Offensive.tick(&perception).is_empty());

        // Helping squad approached and sees the enemy: it engages
        perception.squads[0].position = WorldVec2::new(100.0, 0.0);
        perception.enemies[0].seen_by.push(SquadIndex(3));
        assert_eq!(
            Offensive.tick(&perception),
            vec![Command::SetOrders(
                SquadIndex(3),
                vec![Order::Engage(IndividualIndex(7))]
            )]
        );
    }

    #[test]
    fn no_command_when_order_unchanged() {
        let mut perception = perception(BattlePhase::Fight);
        perception.squads[0].orders = vec![Order::MoveTo(WorldVec2::new(1000.0, 0.0))];
        assert!(Offensive.tick(&perception).is_empty());
    }
}
