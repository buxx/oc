use eframe::egui;
use oc_root::side::Side;

use oc_deployment::deployment::Deployment;

use crate::{map_view, minimap, page::Page};

use super::{PageContext, new_battle};

pub fn show(ui: &mut egui::Ui, ctx: &mut PageContext) -> Option<Page> {
    let Some((battle, scenario)) = ctx.selected() else {
        ui.label("No scenario selected");
        return None;
    };
    let Some((a_name, b_name)) = ctx
        .selected_mod_info()
        .map(|m| (m.a_name.as_str(), m.b_name.as_str()))
    else {
        return None;
    };

    map_view::button(
        ui,
        "prepare",
        &minimap::scenario_path(ctx.app_dir, battle, scenario, ctx.selected_mod),
    );
    ui.separator();

    let mut chosen = None;
    ui.columns(2, |columns| {
        for (side, name) in [(Side::A, a_name), (Side::B, b_name)] {
            let ui = &mut columns[side as usize];
            ui.heading(name);
            if ui.button(format!("Choose {name}")).clicked() {
                chosen = Some(side);
            }
        }
    });

    let side = chosen?;
    let deployment = match load_deployment(ctx, side) {
        Ok(deployment) => deployment,
        Err(error) => {
            ctx.errors.push(format!("Load deployment: {error}"));
            *ctx.show_errors = true;
            return None;
        }
    };

    *ctx.deployment = Some(deployment);
    *ctx.selected_side = Some(side);
    let (count, defaults) = match side {
        Side::A => (scenario.a_squad_count(), scenario.a_default()),
        Side::B => (scenario.b_squad_count(), scenario.b_default()),
    };
    let mut slots = vec![None; count];
    if let Some(deployment) = ctx.deployment.as_ref() {
        let mut taken = vec![false; deployment.squads.len()];
        for (slot, name) in slots.iter_mut().zip(defaults) {
            // First not yet placed squad with this name
            let found = deployment
                .squads
                .iter()
                .enumerate()
                .position(|(i, squad)| !taken[i] && squad.label == *name);
            if let Some(index) = found {
                taken[index] = true;
                *slot = Some(index);
            }
        }
    }
    *ctx.squad_slots = slots;

    Some(Page::ChooseSquads)
}

/// Read deployment file of given side from current game folder
fn load_deployment(ctx: &PageContext, side: Side) -> Result<Deployment, String> {
    let game = ctx.current_game.as_ref().ok_or("No current game")?;
    let path = game
        .dir(ctx.app_dir)
        .join(new_battle::deployment_file(side));
    Deployment::from_file(&path).map_err(|e| format!("{e:#}"))
}
