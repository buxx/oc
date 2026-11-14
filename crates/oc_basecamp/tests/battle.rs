//! End-to-end: start a battle from basecamp, battle gui ends it by itself and exits.
//!
//! Needs a display and the debug battle gui (searched in `target/debug`) built first.
//! Run with `just test-basecamp-battle-test1` (or as part of `just test-e2e`).
#![cfg(feature = "e2e")]

mod common;

use std::time::{Duration, Instant};

use eframe::egui::accesskit::Role;
use egui_kittest::kittest::Queryable;
use oc_basecamp::page::Page;

/// Maximum time for the battle gui to start, end the battle and exit
const BATTLE_TIMEOUT: Duration = Duration::from_secs(300);
const POLL_INTERVAL: Duration = Duration::from_millis(250);

#[test]
fn solo_battle_test1_starts_and_exits() {
    // Battle gui ends the battle as soon as it receives the game config
    let battle_args = vec!["--debug-auto-end".to_string()];
    let mut basecamp = common::start_with(move |app| app.with_battle_args(battle_args));
    let harness = &mut basecamp.harness;

    // Select mod
    harness.get_by_role(Role::ComboBox).click();
    harness.run_ok();
    harness.get_by_label("tests1").click();
    harness.run_ok();

    for (label, page) in [
        ("Solo", Page::Solo),
        ("Battle", Page::Battle),
        ("Test1", Page::Battle),
        ("Scenario 1", Page::Battle),
        ("New game", Page::NewBattle),
        ("Start", Page::PrepareBattle),
        ("Choose Allied", Page::ChooseSquads),
        ("Start battle", Page::BattleInProgress),
    ] {
        harness.get_by_label(label).click();
        harness.run_ok();
        assert_eq!(
            harness.state().page(),
            page,
            "after clicking '{label}' (errors: {:?})",
            harness.state().error_messages()
        );
    }

    // Wait battle gui ends the battle and exits
    let started = Instant::now();
    while harness.state().page() == Page::BattleInProgress {
        assert!(
            started.elapsed() < BATTLE_TIMEOUT,
            "battle gui did not exit within {BATTLE_TIMEOUT:?}"
        );
        std::thread::sleep(POLL_INTERVAL);
        harness.step();
    }

    // Successful exit goes back to battle page, with deployments updated from world state
    assert_eq!(
        harness.state().error_messages(),
        &[] as &[String],
        "battle errors (is target/debug/oc_battle_gui built with `--features debug`?)"
    );
    assert_eq!(harness.state().page(), Page::Battle);
}
