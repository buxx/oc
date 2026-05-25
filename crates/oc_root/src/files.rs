use std::{fmt::Display, net::SocketAddr, path::PathBuf};

use derive_more::Constructor;

use crate::static_::StaticSource;

#[derive(Debug, Copy, Clone)]
pub enum File {
    Mod,
    World,
    Minimap,
    Region(u64),
}

#[derive(Debug, Clone)]
pub enum Connection {
    Network(SocketAddr),
    Embedded,
}

#[derive(Debug, Clone)]
pub enum Sync {
    DirectDownload(String),
    ArchiveDownload(String),
}

#[derive(Debug, Clone, Constructor)]
pub struct Files {
    mod_: String,
    world: String,
}

pub struct FilesAsServer {
    cache: PathBuf,
    mod_: String,
    world: String,
}

pub struct FilesAsGui {
    connection: Connection,
    sources: StaticSource,
    mod_: String,
    world: String,
}

impl Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            File::Mod => f.write_str("Mod"),
            File::World => f.write_str("World"),
            File::Minimap => f.write_str("Minimap"),
            File::Region(i) => f.write_str(&format!("Region({i})")),
        }
    }
}

impl Display for Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sync::DirectDownload(path) => f.write_str(&format!("DirectDownload({path})")),
            Sync::ArchiveDownload(path) => f.write_str(&format!("ArchiveDownload({path})")),
        }
    }
}

impl Files {
    pub fn into_server(self, cache: PathBuf) -> FilesAsServer {
        FilesAsServer {
            cache,
            mod_: self.mod_,
            world: self.world,
        }
    }

    pub fn into_gui(self, sources: StaticSource, connection: Connection) -> FilesAsGui {
        FilesAsGui {
            connection,
            sources,
            mod_: self.mod_,
            world: self.world,
        }
    }
}

impl FilesAsServer {
    pub fn mods(&self) -> PathBuf {
        self.cache.join("mods")
    }

    pub fn worlds(&self) -> PathBuf {
        self.cache.join("worlds")
    }

    pub fn world(&self) -> PathBuf {
        self.cache.join(format!("worlds/{}", self.world))
    }

    pub fn minimap(&self) -> PathBuf {
        self.cache
            .join(format!("worlds/{}/minimap.png", self.world))
    }

    pub fn region(&self, region: u64) -> PathBuf {
        self.cache
            .join(format!("worlds/{}/region{}.png", self.world, region))
    }

    pub fn world_archive(&self) -> PathBuf {
        self.cache
            .join("worlds")
            .join(format!("{}.tar.gz", self.world))
    }

    pub fn mod_archive(&self) -> PathBuf {
        self.cache
            .join("mods")
            .join(format!("{}.tar.gz", self.mod_))
    }
}

impl FilesAsGui {
    pub fn mod_(&self) -> PathBuf {
        match &self.sources {
            StaticSource::Remote(_) => PathBuf::from(format!("cache/mods/{}", &self.mod_)),
            StaticSource::Local { mod_: _, world: _ } => PathBuf::from("mods_").join(&self.mod_),
        }
    }

    pub fn world_(&self) -> PathBuf {
        match &self.sources {
            StaticSource::Remote(_) => PathBuf::from(format!("cache/worlds/{}", &self.world)),
            // FIXME BS NOW: not used ?
            StaticSource::Local { mod_: _, world } => PathBuf::from(world).join("UNUSED"),
        }
    }

    pub fn terrain_png(&self) -> PathBuf {
        match &self.sources {
            StaticSource::Remote(_) => {
                PathBuf::from(format!("cache/worlds/{}/terrain.png", &self.world))
            }
            StaticSource::Local { mod_: _, world } => {
                PathBuf::from("worlds_").join(world).join("terrain.png")
            }
        }
    }

    pub fn height_png(&self) -> PathBuf {
        match &self.sources {
            StaticSource::Remote(_) => {
                PathBuf::from(format!("cache/worlds/{}/height.png", &self.world))
            }
            StaticSource::Local { mod_: _, world } => {
                PathBuf::from("worlds_").join(world).join("height.png")
            }
        }
    }

    pub fn terrain_tsx(&self) -> PathBuf {
        match &self.sources {
            StaticSource::Remote(_) => {
                PathBuf::from(format!("cache/worlds/{}/terrain.tsx", &self.world))
            }
            StaticSource::Local { mod_: _, world } => PathBuf::from("assets")
                .join("worlds_")
                .join(world)
                .join("terrain.tsx"),
        }
    }

    pub fn minimap(&self) -> PathBuf {
        self.sources
            .cache()
            .join(format!("worlds/{}/minimap.png", &self.world))
    }

    pub fn region(&self, region: u64) -> PathBuf {
        self.sources
            .cache()
            .join(format!("worlds/{}/region{}.png", &self.world, region))
    }

    pub fn method(&self, file: File) -> Option<(Sync, PathBuf)> {
        match (&self.connection, &self.sources) {
            (Connection::Network(server), StaticSource::Remote(port)) => {
                let base_url = format!("http://{}:{}", server.ip(), port);
                let base_target = PathBuf::from("assets");

                Some(match file {
                    File::Mod => (
                        Sync::ArchiveDownload(format!("{base_url}/mod")),
                        base_target.join(self.mod_()),
                    ),
                    File::World => (
                        Sync::ArchiveDownload(format!("{base_url}/world")),
                        base_target.join(self.world_()),
                    ),
                    File::Minimap => (
                        Sync::DirectDownload(format!("{base_url}/minimap")),
                        base_target.join(self.minimap()),
                    ),
                    File::Region(i) => (
                        Sync::DirectDownload(format!("{base_url}/region/{i}")),
                        base_target.join(self.region(i)),
                    ),
                })
            }
            _ => None,
        }
    }
}
