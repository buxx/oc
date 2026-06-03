setup:
    # cargo install cargo-all-features
    cargo binstall cargo-nextest --secure

check:
    cargo check
    cargo check --tests
    cargo check --features debug,perfs,tracker,test

serve-minidblue:
    cargo run --bin oc_world_server --features debug -- \
      ./mods/std1 ./examples/minidblue ./examples/minidblue.snapshot

serve-world1:
    cargo run --bin oc_world_server --features debug -- \
      ./mods/std1 ./examples/world1 ./examples/world1.snapshot

gui:
    cargo run --bin oc_battle_gui -- --autoconnect 127.0.0.1:6589

example-world1 *args:
    cargo run --bin example_world1 --features debug {{ args }}

example-minidblue *args:
    cargo run --bin example_minidblue --features debug {{ args }}

example-wall *args:
    cargo run --bin example_wall --features debug {{ args }}

example-height *args:
    cargo run --bin example_height --features debug {{ args }}

example-projectiles-wall *args:
    cargo run --bin example_projectiles_wall {{ args }}

example-individual-shots *args:
    cargo run --bin example_individual_shots --features debug {{ args }}

example-stress-gui-projectiles:
    cargo run --bin example_stress_projectiles --features debug,perfs --release

example-stress-server-projectiles:
    cargo run --bin example_stress_projectiles_server --features perfs --release

test:
    cargo nextest run

test-e2e:
    RUST_LOG=WARN cargo run --bin example_projectiles_wall --features test -- --test
    RUST_LOG=WARN cargo run --bin example_individual_shots --features test -- --test

test-all:
    just test
    just test-e2e
