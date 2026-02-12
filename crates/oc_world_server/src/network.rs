use derive_more::Constructor;
use oc_network::ServerToClientMessage;
use rkyv::rancor::Error;

#[derive(Constructor)]
pub struct Network {}

impl Network {
    pub fn broadcast(&self, msg: ServerToClientMessage) {
        let _bytes = rkyv::to_bytes::<Error>(&msg).unwrap();
    }
}
