use std::path::PathBuf;

use bon::Builder;
use derive_more::Constructor;
use oc_root::static_::StaticSource;

#[derive(Debug, Clone, Builder, Constructor)]
pub struct ServerConfig {
    pub mod_: PathBuf,
    pub world: PathBuf,
    pub snapshot: PathBuf,
    pub cache: PathBuf,
    pub static_: StaticSource,
    pub region_width: u64,
    pub region_height: u64,
}
