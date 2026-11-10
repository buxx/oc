use std::path::Path;

use serde::Deserialize;

const BATTLES_FOLDER: &str = "battles";

#[derive(Debug, Clone, Deserialize)]
pub struct Battle {
    pub name: String,
    pub description: String,
    pub map: String,
}

pub fn load_all() -> (Vec<Battle>, Vec<String>) {
    let mut battles = vec![];
    let mut errors = vec![];

    let entries = match std::fs::read_dir(BATTLES_FOLDER) {
        Ok(entries) => entries,
        Err(error) => {
            errors.push(format!("Read folder '{BATTLES_FOLDER}': {error}"));
            return (battles, errors);
        }
    };

    let mut folders: Vec<_> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    folders.sort();

    for folder in folders {
        let Some(name) = folder.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        match load(&folder.join(format!("{name}.ron"))) {
            Ok(battle) => battles.push(battle),
            Err(error) => errors.push(error),
        }
    }

    (battles, errors)
}

fn load(path: &Path) -> Result<Battle, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Read battle '{}': {e}", path.display()))?;
    ron::from_str(&content).map_err(|e| format!("Parse battle '{}': {e}", path.display()))
}
