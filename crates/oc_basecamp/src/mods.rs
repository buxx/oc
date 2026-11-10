use std::path::PathBuf;

pub const MODS_FOLDER: &str = "mods";

pub fn list() -> Vec<String> {
    let mut mods: Vec<String> = std::fs::read_dir(PathBuf::from(MODS_FOLDER))
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter(|e| e.path().is_dir())
                .filter_map(|e| e.file_name().into_string().ok())
                .collect()
        })
        .unwrap_or_default();
    mods.sort();
    mods
}
