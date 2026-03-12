use std::path::PathBuf;

use oc_mod::MOD_DIR;
use oc_world::meta::Meta;

pub trait OcPaths {
    fn assets() -> PathBuf;
    fn mods() -> PathBuf;
    fn maps() -> PathBuf;
    fn minimap(meta: &Meta) -> PathBuf;
}

impl OcPaths for PathBuf {
    fn assets() -> PathBuf {
        Self::from("./assets")
    }

    fn mods() -> PathBuf {
        let path = PathBuf::from(".cache");
        path.join(MOD_DIR)
    }

    fn maps() -> PathBuf {
        let path = Self::assets();
        path.join(".cache").join("maps")
    }

    fn minimap(meta: &Meta) -> PathBuf {
        let path = Self::maps().join(meta.folder_name());
        path.join("minimap.png")
    }
}

#[macro_export]
macro_rules! http_to_file {
    ($url:expr, $path:expr) => {
        if let Some(parent) = $path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut resp = reqwest::blocking::get($url)?;
        let mut file = std::fs::File::create($path)?;
        std::io::copy(&mut resp, &mut file)?;
    };
}
