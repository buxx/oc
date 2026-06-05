use bevy::prelude::*;
use clap::Parser;
use oc_examples::{logging, tests::behavior};
use oc_individual::order::Order;
use oc_utils::d2::Position;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, action)]
    test: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    logging::setup_logging()?;
    let args = Args::parse();

    behavior::run()
        .setup(vec![
            ([150., 150.], Order::MoveTo(Position::new(300., 150.))),
            // ([150., 160.], Order::MoveFastTo(Position::new(300., 160.))),
        ])
        .test(args.test)
        .call()?;

    Ok(())
}
