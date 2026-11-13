use eframe::egui;
use oc_root::side::Side;

use oc_deployment::deployment::Deployment;

use crate::{launch, map_view, minimap, page::Page};

use super::PageContext;

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

    // If chosen, prepare battle files
    let side = chosen?;
    prepare_battle(ctx, scenario, side)
}

fn prepare_battle(
    ctx: &mut PageContext<'_>,
    scenario: &oc_battle::Scenario,
    side: Side,
) -> Option<Page> {
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
    *ctx.squad_slots = match ctx.deployment.as_ref() {
        Some(deployment) => launch::default_slots(deployment, defaults, count),
        None => vec![None; count],
    };

    Some(Page::ChooseSquads)
}

/// Read deployment file of given side from current game folder
fn load_deployment(ctx: &PageContext, side: Side) -> Result<Deployment, String> {
    let game = ctx.current_game.as_ref().ok_or("No current game")?;
    launch::load_deployment(&game.dir(ctx.app_dir), side)
}
