use bevy::prelude::*;
use clap::{Parser, ValueEnum};
#[cfg(feature = "debug")]
use oc_battle_gui::ingame::camera::squad::ToggleShowFormationPositions;
use oc_examples::{logging, tests::behavior};
use oc_individual::order::Order;

#[cfg(feature = "test")]
const POSITION_TOLERANCE: f32 = 3.0;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg()]
    case: TestCase,

    #[arg(long, action)]
    test: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum TestCase {
    Idle,
    MoveStraightAhead,
    MoveStraightAheadObstacle,
}

const IDLE_LEADER_POS: [f32; 2] = [150., 150.];

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
    };

    let run = behavior::run().test(args.test).setup(setup);

    let run = {
        let test_track = {
            #[cfg(feature = "test")]
            {
                use oc_world_server::tracker::Tracker;

                Box::new(move |tracker: Tracker| match args.case {
                    // TODO: test if squad member reach expected position
                    TestCase::Idle => {}
                    // TODO: test if leader accomplished twice (two orders), reached expected position
                    //       + squad members
                    TestCase::MoveStraightAhead | TestCase::MoveStraightAheadObstacle => {
                        let tracker = tracker.take();

                        let accomplished = (
                            oc_individual::IndividualIndex(0),
                            oc_individual::Update::Accomplished,
                        );
                        let accomplished = tracker.individuals.contains(&accomplished);
                        assert!(accomplished);

                        println!("✅ (SERVER) All tests passed");
                    }
                })
            }
            #[cfg(not(feature = "test"))]
            {
                ()
            }
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
                    #[cfg(feature = "test")]
                    app.add_systems(Update, end_when_success_or_timeout);
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
use {
    oc_battle_gui::{entity::individual::IndividualIndex, states::Game},
    oc_utils::number::almost_equal,
    std::sync::Mutex,
    std::time::Duration,
    std::time::Instant,
};

#[cfg(feature = "test")]
fn end_when_success_or_timeout(
    mut commands: Commands,
    game: Res<Game>,
    args: Res<Args_>,
    individuals: Query<&oc_physics::update::bevy::Position, With<IndividualIndex>>,
) {
    static MOVE_DONE: Mutex<Option<Instant>> = Mutex::new(None);
    let timeout = match args.0.case {
        TestCase::Idle => Duration::from_secs(10),
        TestCase::MoveStraightAhead => Duration::from_secs(20),
        TestCase::MoveStraightAheadObstacle => Duration::from_secs(40),
    };

    let timeout = game.started.elapsed() > timeout;
    let mut move_done = MOVE_DONE.lock().unwrap();
    *move_done = match *move_done {
        None => match args.0.case {
            // TODO: Test squad member position
            TestCase::Idle => None,
            // TODO: Test squad members positions
            TestCase::MoveStraightAhead => individuals.iter().next().and_then(|position| {
                (almost_equal(position.0[0], MSA_POS2[0], POSITION_TOLERANCE)
                    && almost_equal(position.0[1], MSA_POS2[1], POSITION_TOLERANCE))
                .then(|| Instant::now())
            }),
            TestCase::MoveStraightAheadObstacle => individuals.iter().next().and_then(|position| {
                (almost_equal(position.0[0], MSAO_POS2[0], POSITION_TOLERANCE)
                    && almost_equal(position.0[1], MSAO_POS2[1], POSITION_TOLERANCE))
                .then(|| Instant::now())
            }),
        },
        Some(value) => {
            if value.elapsed().as_secs() > 1 {
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
