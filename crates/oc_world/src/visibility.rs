use derive_more::{Constructor, Deref, DerefMut};
use oc_individual::IndividualIndex;
use oc_root::opacity::CumulatedOpacity;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct Visibilities {
    /// Vector containing visibility for each individual for each individual
    values: Vec<Vec<Visibility>>,
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq, Constructor)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Visibility {
    pub visible: bool,
    pub opacity: CumulatedOpacity,
}

impl Default for Visibility {
    fn default() -> Self {
        Self {
            visible: false,
            opacity: CumulatedOpacity(1.0),
        }
    }
}

impl Visibilities {
    pub fn empty(count: usize) -> Self {
        let values = vec![vec![Visibility::default(); count]; count];
        Self { values }
    }

    pub fn for_(&self, i: IndividualIndex) -> &Vec<Visibility> {
        &self.values[i.0 as usize]
    }

    #[cfg(feature = "tests")]
    pub fn values_mut(&mut self) -> &mut Vec<Vec<Visibility>> {
        &mut self.values
    }
}
