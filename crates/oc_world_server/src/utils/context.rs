use std::sync::{Arc, mpsc::Sender};

use oc_network::ToClient;
use oc_root::Client;

use crate::{routing::Listening, state::State};

#[derive(Clone)]
pub struct Context<E: Client> {
    pub cpus: usize,
    pub state: Arc<State<E>>,
    pub output: Sender<(E, ToClient)>,
}

impl<E: Client> Context<E> {
    pub fn new(state: Arc<State<E>>, output: Sender<(E, ToClient)>) -> Self {
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
                let pkg = (listener.clone(), message.clone().into());
                self.output.send(pkg).unwrap() // TODO
            }
        }
    }
}
