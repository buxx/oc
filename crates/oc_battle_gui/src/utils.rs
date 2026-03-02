use std::path::PathBuf;

use oc_world::meta::Meta;

pub trait OcPaths {
    fn assets() -> PathBuf;
    fn maps() -> PathBuf;
    fn minimap(meta: &Meta) -> PathBuf;
}

impl OcPaths for PathBuf {
    fn assets() -> PathBuf {
        Self::from("./assets")
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
        let mut resp = reqwest::blocking::get($url)?;
        let mut file = std::fs::File::create($path)?;
        std::io::copy(&mut resp, &mut file)?;
    };
}
