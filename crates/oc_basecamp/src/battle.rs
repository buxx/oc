use std::path::Path;

use oc_battle::check::check_battle;
use serde::Deserialize;

const BATTLES_FOLDER: &str = "battles";
const MAPS_FOLDER: &str = "maps";

#[derive(Debug, Clone)]
pub struct Battle {
    pub name: String,
    pub description: String,
    pub map: String,
    pub scenarios: Vec<Scenario>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Scenario {
    pub name: String,
    pub description: String,
}

#[derive(Deserialize)]
struct BattleRaw {
    name: String,
    description: String,
    map: String,
    #[serde(default)]
    scenarios: Vec<String>,
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
        match load(&folder.join(format!("{name}.ron")), &mut errors) {
            Ok(battle) => battles.push(battle),
            Err(error) => errors.push(error),
        }
    }

    (battles, errors)
}

fn load(path: &Path, errors: &mut Vec<String>) -> Result<Battle, String> {
    let raw: BattleRaw = read_ron(path, "battle")?;
    let dir = path.parent().unwrap_or(Path::new("."));

    let mut scenarios = vec![];
    for file in &raw.scenarios {
        match read_ron(&dir.join(file), "scenario") {
            Ok(scenario) => scenarios.push(scenario),
            Err(error) => errors.push(error),
        }
    }

    let map_path = Path::new(MAPS_FOLDER).join(&raw.map);
    errors.extend(
        check_battle(path, &map_path)
            .iter()
            .map(|e| format!("Battle '{}': {e}", raw.name)),
    );

    Ok(Battle {
        name: raw.name,
        description: raw.description,
        map: raw.map,
        scenarios,
    })
}

fn read_ron<T: serde::de::DeserializeOwned>(path: &Path, kind: &str) -> Result<T, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Read {kind} '{}': {e}", path.display()))?;
    ron::from_str(&content).map_err(|e| format!("Parse {kind} '{}': {e}", path.display()))
}
