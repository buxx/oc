#![allow(clippy::unwrap_used)]

use bevy::prelude::*;
use clap::{Parser, ValueEnum};
#[cfg(feature = "debug")]
use oc_battle_gui::ingame::camera::squad::ToggleShowFormationPositions;
#[cfg(feature = "test")]
use oc_battle_gui::ingame::individual::{AccomplishedEvent, MoveStepAccomplishedEvent};
use oc_examples::{logging, tests::behavior};
use oc_individual::order::Order;
use oc_mod::shell::ShellIndex;
#[cfg(feature = "debug")]
use oc_utils::let_some;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg()]
    case: TestCase,

    /// Number of squad members
    #[arg(long)]
    count: usize,

    #[arg(long, action)]
    test: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum TestCase {
    Idle,
    MoveStraightAhead,
    MoveStraightAheadObstacle,
    MoveFastStraightAhead,
    MoveFastStraightAheadObstacle,
    Stunned,
}

const IDLE_LEADER_POS: [f32; 2] = [150., 150.];

const STUNNED_POS: [f32; 2] = [150., 150.];
const STUNNED_MOVE_TARGET: [f32; 2] = [180., 150.];
const STUNNED_EXPLOSION_DISTANCE: f32 = 5.;
const STUNNED_SHELL_INDEX: ShellIndex = ShellIndex(1);

const MSA_LEADER_POS: [f32; 2] = [150., 150.];
const MSA_POS1: [f32; 2] = [180., 150.];
const MSA_POS2: [f32; 2] = [210., 150.];

const MSAO_LEADER_POS: [f32; 2] = [160., 415.];
const MSAO_POS1: [f32; 2] = [235., 415.];
const MSAO_POS2: [f32; 2] = [250., 415.];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    logging::setup_logging()?;

    let args = Args::parse();

    #[cfg(not(feature = "test"))]
    {
        if args.test {
            panic!("Using --test imply enable test feature")
        }
    }

    let setup = match args.case {
        TestCase::Idle => {
            vec![(IDLE_LEADER_POS, vec![])]
        }
        TestCase::Stunned => {
            vec![(STUNNED_POS, vec![])]
        }
        TestCase::MoveStraightAhead => {
            vec![(
                MSA_LEADER_POS,
                vec![
                    Order::MoveTo(MSA_POS1.into()),
                    Order::MoveTo(MSA_POS2.into()),
                ],
            )]
        }
        TestCase::MoveStraightAheadObstacle => {
            vec![(
                MSAO_LEADER_POS,
                vec![
                    Order::MoveTo(MSAO_POS1.into()),
                    Order::MoveTo(MSAO_POS2.into()),
                ],
            )]
        }
        TestCase::MoveFastStraightAhead => {
            vec![(
                MSA_LEADER_POS,
                vec![
                    Order::MoveFastTo(MSA_POS1.into()),
                    Order::MoveFastTo(MSA_POS2.into()),
                ],
            )]
        }
        TestCase::MoveFastStraightAheadObstacle => {
            vec![(
                MSAO_LEADER_POS,
                vec![
                    Order::MoveFastTo(MSAO_POS1.into()),
                    Order::MoveFastTo(MSAO_POS2.into()),
                ],
            )]
        }
    };

    let run = behavior::run()
        .test(args.test)
        .setup(setup)
        .count(args.count);

    let run = {
        #[cfg_attr(not(feature = "test"), allow(clippy::let_unit_value))]
        let test_track = {
            #[cfg(feature = "test")]
            {
                use oc_world_server::tracker::Tracker;

                let count = args.count;
                Box::new(move |tracker: Tracker| match args.case {
                    // FIXME: test if squad member reach expected position
                    TestCase::Idle | TestCase::Stunned => {}
                    // FIXME: test if leader accomplished twice (two orders), reached expected position
                    //       + squad members
                    TestCase::MoveStraightAhead
                    | TestCase::MoveStraightAheadObstacle
                    | TestCase::MoveFastStraightAhead
                    | TestCase::MoveFastStraightAheadObstacle => {
                        let tracker = tracker.take();

                        for i in 0..count as u64 {
                            let accomplished = (
                                oc_individual::IndividualIndex(i),
                                oc_individual::Update::Accomplished,
                            );
                            assert!(
                                tracker.individuals.contains(&accomplished),
                                "Individual {i} did not accomplish its orders"
                            );
                        }

                        println!("✅ (SERVER) All tests passed");
                    }
                })
            }
            #[cfg(not(feature = "test"))]
            {}
        };

        run.tests((
            {
                let args_ = args.clone();
                Box::new(move |app| {
                    app.insert_resource(Args_(args_.clone()));

                    #[cfg(feature = "debug")]
                    app.add_systems(Startup, |mut commands: Commands| {
                        commands.trigger(ToggleShowFormationPositions)
                    });
                    #[cfg(feature = "debug")]
                    if matches!(args.case, TestCase::Stunned) {
                        app.add_systems(Update, stunned_fire);
                    }
                    #[cfg(feature = "test")]
                    {
                        match args.case {
                            TestCase::Idle => {}
                            TestCase::Stunned => {
                                app.init_resource::<StunnedTest>()
                                    .add_systems(Update, stunned_test);
                            }
                            TestCase::MoveStraightAhead
                            | TestCase::MoveStraightAheadObstacle
                            | TestCase::MoveFastStraightAhead
                            | TestCase::MoveFastStraightAheadObstacle => {
                                app.init_resource::<Tracking>()
                                    .add_observer(on_move_step_accomplished_event)
                                    .add_observer(on_accomplished_event);
                            }
                        }
                        if !matches!(args.case, TestCase::Stunned) {
                            app.add_systems(Update, end_when_success_or_timeout);
                        }
                    }
                })
            },
            test_track,
        ))
    };

    run.call()?;

    Ok(())
}

#[allow(unused)]
#[derive(Debug, Resource)]
struct Args_(Args);

#[cfg(feature = "test")]
#[derive(Debug, Resource, Default)]
struct Tracking {
    move_step_accomplished: Vec<MoveStepAccomplishedEvent>,
    accomplished: Vec<AccomplishedEvent>,
}

#[cfg(feature = "test")]
use {
    oc_battle_gui::states::Game, oc_root::Wcfg, std::sync::Mutex, std::time::Duration,
    std::time::Instant,
};

#[cfg(feature = "test")]
fn end_when_success_or_timeout(
    mut commands: Commands,
    game: Res<Game>,
    args: Res<Args_>,
    tracking: Res<Tracking>,
    w: Res<Wcfg>,
) {
    let_some!(w = &w.0, return);
    static MOVE_DONE: Mutex<Option<Instant>> = Mutex::new(None);
    let timeout = match args.0.case {
        TestCase::Idle | TestCase::Stunned => Duration::from_secs(20),
        TestCase::MoveStraightAhead => Duration::from_secs(40),
        TestCase::MoveStraightAheadObstacle => Duration::from_secs(50),
        TestCase::MoveFastStraightAhead => Duration::from_secs(30),
        TestCase::MoveFastStraightAheadObstacle => Duration::from_secs(40),
    };
    let speed = w.speed();
    // Ensure a minimal timeout to let time to bevy to make "network" exchange with server
    let timeout = timeout.div_f32(speed).max(Duration::from_secs(2));

    let timeout = game.started.elapsed() > timeout;
    let mut move_done = MOVE_DONE.lock().unwrap();
    *move_done = match *move_done {
        None => match args.0.case {
            TestCase::Idle | TestCase::Stunned => None,
            TestCase::MoveStraightAhead
            | TestCase::MoveStraightAheadObstacle
            | TestCase::MoveFastStraightAhead
            | TestCase::MoveFastStraightAheadObstacle => {
                {
                    // Leader accomplish its two orders; members' orders are replaced by the
                    // leader's current one, so they must accomplish at least one.
                    let accomplished = |i: u64| {
                        tracking
                            .accomplished
                            .iter()
                            .filter(|e| ***e == oc_individual::IndividualIndex(i))
                            .count()
                    };
                    (accomplished(0) >= 2 && (1..args.0.count as u64).all(|i| accomplished(i) >= 1))
                        .then(Instant::now)
                }
            }
        },
        Some(value) => {
            if value.elapsed().as_secs_f32() > 1. / speed {
                println!("✅ (GUI) Individual reached target");
                commands.write_message(bevy::app::AppExit::from_code(0));
            };
            Some(value)
        }
    };

    if timeout {
        eprintln!("❌ Timeout reached ! Individual didn't reached target");
        commands.write_message(bevy::app::AppExit::from_code(1));
    }
}

#[cfg(feature = "test")]
fn on_move_step_accomplished_event(
    event: On<MoveStepAccomplishedEvent>,
    mut tracking: ResMut<Tracking>,
) {
    tracking.move_step_accomplished.push(event.clone());
}

#[cfg(feature = "test")]
fn on_accomplished_event(event: On<AccomplishedEvent>, mut tracking: ResMut<Tracking>) {
    tracking.accomplished.push(event.clone());
}

#[cfg(feature = "test")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum StunnedStep {
    /// Waiting explosion effect
    #[default]
    Stunned,
    /// Waiting stun end
    Operational,
    /// Waiting the move order to be followed
    Moving,
}

#[cfg(feature = "test")]
#[derive(Debug, Resource, Default)]
struct StunnedTest {
    step: StunnedStep,
}

/// Explosion and move order are sent at the same time, once individual is in game.
#[cfg(feature = "debug")]
fn stunned_fire(
    mut commands: Commands,
    mut fired: Local<bool>,
    w: Res<oc_root::Wcfg>,
    individuals: Query<&oc_battle_gui::entity::individual::IndividualIndex>,
) {
    use oc_battle_gui::network::output::ToServerEvent;
    use oc_individual::squad::SquadIndex;
    use oc_network::{SquadMessage, ToServer};
    use oc_root::geo::WorldVec2;

    let_some!(w = &w.0, return);
    if *fired
        || !individuals
            .iter()
            .any(|i| i.0 == oc_individual::IndividualIndex(0))
    {
        return;
    }
    *fired = true;

    let position = WorldVec2::new(
        STUNNED_POS[0] + oc_root::physics::Meters(STUNNED_EXPLOSION_DISTANCE).pixels(w),
        STUNNED_POS[1],
    );
    commands.trigger(ToServerEvent(ToServer::DebugExplosion(
        STUNNED_SHELL_INDEX,
        position,
    )));
    commands.trigger(ToServerEvent(ToServer::Squad(
        SquadIndex(0),
        SquadMessage::SetOrders(vec![Order::MoveTo(STUNNED_MOVE_TARGET.into())]),
    )));
}

/// Individual must be stunned, then operational, then moving.
#[cfg(feature = "test")]
fn stunned_test(
    mut commands: Commands,
    mut test: ResMut<StunnedTest>,
    game: Res<Game>,
    w: Res<Wcfg>,
    individuals: Query<(
        &oc_battle_gui::entity::individual::IndividualIndex,
        &oc_battle_gui::ingame::individual::Status,
        &oc_battle_gui::entity::individual::Intent,
    )>,
) {
    use oc_individual::behavior::Intent;

    let_some!(w = &w.0, return);
    let timeout = Duration::from_secs(20)
        .div_f32(w.speed())
        .max(Duration::from_secs(2));
    if game.started.elapsed() > timeout {
        eprintln!("❌ Timeout reached ! Step {:?} not reached", test.step);
        commands.write_message(bevy::app::AppExit::from_code(1));
        return;
    }

    let individual = oc_individual::IndividualIndex(0);
    let Some((_, status, intent)) = individuals.iter().find(|(i, _, _)| i.0 == individual) else {
        return;
    };

    let moving = matches!(intent.0, Intent::MoveTo(..));
    match (test.step, status.0) {
        (StunnedStep::Stunned, oc_individual::Status::Stunned(_)) => {
            println!("✅ (GUI) Individual stunned");
            test.step = StunnedStep::Operational;
        }
        (StunnedStep::Stunned | StunnedStep::Operational, _) if moving => {
            eprintln!("❌ Individual moves before ({:?})", test.step);
            commands.write_message(bevy::app::AppExit::from_code(1));
        }
        (StunnedStep::Operational, oc_individual::Status::Operational) => {
            println!("✅ (GUI) Individual operational again");
            test.step = StunnedStep::Moving;
        }
        (StunnedStep::Moving, oc_individual::Status::Operational) if moving => {
            println!("✅ (GUI) Individual moving");
            commands.write_message(bevy::app::AppExit::from_code(0));
        }
        _ => {}
    }
}
