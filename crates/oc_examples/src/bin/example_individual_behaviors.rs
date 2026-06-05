use bevy::prelude::*;
use clap::{Parser, ValueEnum};
use oc_examples::{logging, tests::behavior};
use oc_individual::order::Order;
use oc_utils::d2::Position;

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

    let setup = match args.case {
        TestCase::StraightAhead => vec![([150., 150.], Order::MoveTo(Position::new(300., 150.)))],
    };

    behavior::run().setup(setup).test(args.test).call()?;

    Ok(())
}
