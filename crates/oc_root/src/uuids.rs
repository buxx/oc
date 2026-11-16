use derive_more::Constructor;
use uuid::Uuid;

/// Define a typed uuid: serialized (serde and rkyv) like the bare [`Uuid`] it wraps.
macro_rules! uuid_type {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[oc_derive::archivable(bare)]
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord,
            serde::Serialize, serde::Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            /// New random uuid
            pub fn new_v4() -> Self {
                Self(Uuid::new_v4())
            }

            pub fn uuid(&self) -> Uuid {
                self.0
            }
        }

        impl From<Uuid> for $name {
            fn from(value: Uuid) -> Self {
                Self(value)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl std::str::FromStr for $name {
            type Err = uuid::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Uuid::parse_str(s).map(Self)
            }
        }
    };
}

uuid_type! {
    /// Uuid of a squad (stable between battles, see deployments)
    SquadUuid
}

uuid_type! {
    /// Uuid of an individual (stable between battles, see deployments)
    IndividualUuid
}

#[oc_derive::archivable(bare)]
#[derive(Debug, Clone, Default, PartialEq, Constructor)]
pub struct Uuids {
    /// Indexed like world individuals
    pub individuals: Vec<IndividualUuid>,
    /// Indexed like world squads (can be empty when world is not built from deployments)
    pub squads: Vec<SquadUuid>,
}

impl Uuids {
    pub fn individual(&self, i: usize) -> &IndividualUuid {
        &self.individuals[i]
    }

    pub fn squad(&self, i: usize) -> Option<&SquadUuid> {
        self.squads.get(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_as_bare_uuid() {
        let uuid = Uuid::new_v4();
        let squad = SquadUuid(uuid);
        assert_eq!(
            serde_json::to_string(&squad).unwrap(),
            serde_json::to_string(&uuid).unwrap()
        );
        let parsed: SquadUuid = serde_json::from_str(&format!("\"{uuid}\"")).unwrap();
        assert_eq!(parsed, squad);
        assert_eq!(squad.to_string().parse::<SquadUuid>().unwrap(), squad);
    }

    #[test]
    fn rkyv_as_bare_uuid() {
        let uuid = Uuid::new_v4();
        let typed = rkyv::to_bytes::<rkyv::rancor::Error>(&IndividualUuid(uuid)).unwrap();
        let bare = rkyv::to_bytes::<rkyv::rancor::Error>(&uuid).unwrap();
        assert_eq!(typed.as_slice(), bare.as_slice());
    }
}
