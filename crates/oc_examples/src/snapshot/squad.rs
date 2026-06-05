use oc_individual::{Individual, squad::Squad};
use oc_root::WorldConfig;

pub trait SquadsGenerator {
    fn squads(&self, w: &WorldConfig, individuals: &Vec<Individual>) -> Vec<Squad>;
}
