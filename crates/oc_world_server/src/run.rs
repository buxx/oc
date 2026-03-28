use std::sync::{
    Arc,
    mpsc::{Receiver, Sender},
};

use crate::{bridge::Event, config::ServerConfig, runner::Runner, state::State};
use oc_network::ToClient;
use oc_root::Client;

// TODO: useless function
pub fn run<E: Client>(
    config: ServerConfig,
    state: Arc<State<E>>,
    input: Receiver<Event<E>>,
    output: Sender<(E, ToClient)>,
    ready: Sender<Result<(), String>>,
) {
    Runner::new(config, state, output).run(input, ready);
}
