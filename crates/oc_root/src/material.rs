use rkyv::Archive;

// FIXME BS NOW: bug; on utilise le volume pour Collision;
// mais le volume existe par ex. pour le buisson
// Il faut introduire la "nature" / "materialité" des objets
// pour laisser passer une balle à travers un buisson; mais pas a travers un mur
// pour bloquer un individu et une balle à travers un rocher, etc.
#[derive(
    Debug,
    Clone,
    Copy,
    Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum MaterialKind {
    Individual,
    Projectile,
}

impl MaterialKind {}
