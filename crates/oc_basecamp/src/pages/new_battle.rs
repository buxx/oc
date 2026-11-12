use std::path::Path;

use eframe::egui;
use oc_deployment::generator::Generator;
use oc_game::{BattleGame, Game};
use oc_mod::Mod;
use oc_root::side::Side;

use crate::{
    battle::{Battle, Scenario},
    mods::MODS_FOLDER,
    page::Page,
};

const A_DEPLOYMENT_FILE: &str = "a_deployment.yml";
const B_DEPLOYMENT_FILE: &str = "b_deployment.yml";

/// Deployment file name (in game folder) of given side
pub fn deployment_file(side: Side) -> &'static str {
    match side {
        Side::A => A_DEPLOYMENT_FILE,
        Side::B => B_DEPLOYMENT_FILE,
    }
}

use super::PageContext;

pub fn show(ui: &mut egui::Ui, ctx: &mut PageContext) -> Option<Page> {
    let Some((battle, scenario)) = ctx.selected() else {
        ui.label("No scenario selected");
        return None;
    };

    ui.heading(format!("{}: {}", battle.name(), scenario.name()));
    ui.separator();
    ui.label("todo");
    ui.add_space(8.0);

    if ui.button("Start").clicked() {
        match start(ctx.app_dir, battle, scenario, ctx.selected_mod) {
            Ok(game) => {
                ctx.games.insert(0, game.clone());
                *ctx.current_game = Some(game);
                return Some(Page::PrepareBattle);
            }
            Err(error) => {
                ctx.errors.push(format!("Start game: {error}"));
                // Show errors popin now
                *ctx.show_errors = true;
            }
        }
    }

    None
}

/// Create the game, and generate side A and B deployments files
fn start(
    app_dir: &Path,
    battle: &Battle,
    scenario: &Scenario,
    mod_name: &str,
) -> Result<Game, String> {
    let mod_ = Mod::load(&Path::new(MODS_FOLDER).join(mod_name), None)
        .map_err(|e| format!("Load mod '{mod_name}': {e}"))?;

    let generator = Generator::new();
    let mut deployments = vec![];
    for (file, profile) in [
        (A_DEPLOYMENT_FILE, scenario.deploy_a()),
        (B_DEPLOYMENT_FILE, scenario.deploy_b()),
    ] {
        let deployment = generator
            .generate(&mod_, profile)
            .map_err(|e| format!("Generate {file}: {e}"))?;
        let yaml = serde_yaml::to_string(&deployment).map_err(|e| format!("{file}: {e}"))?;
        deployments.push((file, yaml));
    }

    let game = Game::Battle(BattleGame::new(mod_name, battle.name(), scenario.name()));
    let game_path = game.save(app_dir).map_err(|e| e.to_string())?;
    let dir = game_path.parent().ok_or("Invalid game path")?;
    if let Err(error) = write_deployments(dir, deployments) {
        // Do not leave a game without its deployments (folder was just created for this game)
        let _ = std::fs::remove_dir_all(dir);
        return Err(error);
    }

    Ok(game)
}

fn write_deployments(dir: &Path, deployments: Vec<(&str, String)>) -> Result<(), String> {
    for (file, yaml) in deployments {
        let path = dir.join(file);
        std::fs::write(&path, yaml).map_err(|e| format!("Write '{}': {e}", path.display()))?;
    }
    Ok(())
}
