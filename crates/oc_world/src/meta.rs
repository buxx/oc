use derive_more::Constructor;
use uuid::Uuid;

#[derive(Debug, Constructor)]
pub struct Meta {
    id: Uuid,
    revision: usize,
}
