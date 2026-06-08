use bevy::prelude::*;
use clap::{Parser, ValueEnum};
use oc_examples::{logging, tests::behavior};
use oc_individual::order::Order;
use oc_utils::d2::Position;

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
    StraightAhead,
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
        TestCase::StraightAhead => vec![([150., 150.], Order::MoveTo(Position::new(180., 150.)))],
    };

    let behavior = behavior::run().setup(setup);

    #[cfg(not(feature = "test"))]
    let behavior = { behavior.tests(((), ())) };
    #[cfg(feature = "test")]
    let behavior = {
        use oc_battle_gui::{entity::individual::IndividualIndex, states::Game};
        use oc_utils::number::almost_equal;
        use std::sync::Mutex;
        use std::time::Duration;
        use std::time::Instant;

        behavior.tests((
            Box::new(move |app| {
                app.add_systems(
                    Update,
                    |mut commands: Commands,
                     game: Res<Game>,
                     individuals: Query<
                        &oc_physics::update::bevy::Position,
                        With<IndividualIndex>,
                    >| {
                        static MOVE_DONE: Mutex<Option<Instant>> = Mutex::new(None);

                        let timeout = game.started.elapsed() > Duration::from_secs(15);
                        let mut move_done = MOVE_DONE.lock().unwrap();
                        *move_done = match *move_done {
                            None => match args.case {
                                TestCase::StraightAhead => {
                                    individuals.iter().next().and_then(|position| {
                                        (almost_equal(position.0[0], 180., POSITION_TOLERANCE)
                                            && almost_equal(
                                                position.0[1],
                                                150.,
                                                POSITION_TOLERANCE,
                                            ))
                                        .then(|| Instant::now())
                                    })
                                }
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
                    },
                );
            }),
            Box::new(move |tracker| match args.case {
                TestCase::StraightAhead => {
                    let tracker = tracker.take();

                    let accomplished = (
                        oc_individual::IndividualIndex(0),
                        oc_individual::Update::Accomplished,
                    );
                    let accomplished = tracker.individuals.contains(&accomplished);
                    assert!(accomplished);

                    println!("✅ (SERVER) All tests passed");
                }
            }),
        ))
    };

    behavior.call()?;

    Ok(())
}
