use bevy::prelude::*;
use clap::{Parser, ValueEnum};
use oc_battle_gui::world::InsertedTiles;
use oc_examples::{logging, tests::behavior};
use oc_individual::order::Order;
use oc_utils::d2::Position;

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
    MoveStraightAhead,
    MoveStraightAheadObstacle,
}

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
        TestCase::MoveStraightAhead => {
            vec![([150., 150.], Order::MoveTo(Position::new(180., 150.)))]
        }
        TestCase::MoveStraightAheadObstacle => {
            vec![([160., 415.], Order::MoveTo(Position::new(235., 415.)))]
        }
    };

    let run = behavior::run().setup(setup);

    let run = {
        let test_track = {
            #[cfg(feature = "test")]
            {
                use oc_world_server::tracker::Tracker;

                Box::new(move |tracker: Tracker| match args.case {
                    TestCase::MoveStraightAhead => {
                        let tracker = tracker.take();

                        let accomplished = (
                            oc_individual::IndividualIndex(0),
                            oc_individual::Update::Accomplished,
                        );
                        let accomplished = tracker.individuals.contains(&accomplished);
                        assert!(accomplished);

                        println!("✅ (SERVER) All tests passed");
                    }
                    TestCase::MoveStraightAheadObstacle => {}
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
                    #[cfg(feature = "test")]
                    app.add_systems(Update, end_when_success_or_timeout);
                    app.add_observer(on_inserted_tiles);
                })
            },
            test_track,
        ))
    };

    run.call()?;

    Ok(())
}

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

    let timeout = game.started.elapsed() > Duration::from_secs(15);
    let mut move_done = MOVE_DONE.lock().unwrap();
    *move_done = match *move_done {
        None => match args.0.case {
            TestCase::MoveStraightAhead => individuals.iter().next().and_then(|position| {
                (almost_equal(position.0[0], 180., POSITION_TOLERANCE)
                    && almost_equal(position.0[1], 150., POSITION_TOLERANCE))
                .then(|| Instant::now())
            }),
            TestCase::MoveStraightAheadObstacle => None,
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
        // FIXME BS NOW: test must check this code !
        commands.write_message(bevy::app::AppExit::from_code(1));
    }
}

fn on_inserted_tiles(_: On<InsertedTiles>, mut commands: Commands) {
    // commands.trigger(ToggleShowTiles);
}
