use oc_individual::network::IndividualMessage;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum ServerToClientMessage {
    Individual(IndividualMessage),
}

impl From<IndividualMessage> for ServerToClientMessage {
    fn from(value: IndividualMessage) -> Self {
        ServerToClientMessage::Individual(value)
    }
}
