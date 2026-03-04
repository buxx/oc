use std::sync::{Arc, mpsc::Sender};

use message_io::network::Endpoint;
use oc_network::ToClient;

use crate::{routing::Listening, state::State};

#[derive(Clone)]
pub struct Context {
    pub cpus: usize,
    pub state: Arc<State>,
    pub output: Sender<(Endpoint, ToClient)>,
}

impl Context {
    pub fn new(state: Arc<State>, output: Sender<(Endpoint, ToClient)>) -> Self {
        let cpus = num_cpus::get();

        Self {
            cpus,
            state,
            output,
        }
    }

    pub fn broadcast<T>(&self, filter: Listening, messages: Vec<T>)
    where
        T: Clone + Into<ToClient>,
    {
        let listeners = self.state.listeners();

        for listener in listeners.find(filter) {
            for message in &messages {
                let pkg = (listener, message.clone().into());
                self.output.send(pkg).unwrap() // TODO
            }
        }
    }
}
