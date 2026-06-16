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

gui *args:
    cargo run --bin oc_battle_gui {{ args }} -- --autoconnect 127.0.0.1:6589

example-world1 *args:
    cargo run --bin example_world1 --features debug {{ args }}

example-minidblue *args:
    cargo run --bin example_minidblue --features debug {{ args }}

example-height *args:
    cargo run --bin example_height --features debug {{ args }}

example-projectiles-obstacles *args:
    cargo run --bin example_projectiles_obstacles --features debug {{ args }}

example-individual-shots *args:
    cargo run --bin example_individual_shots --features debug {{ args }}

example-individual-behaviors *args:
    cargo run --bin example_individual_behaviors --features debug {{ args }}

example-stress-gui-projectiles:
    cargo run --bin example_stress_projectiles --features debug,perfs --release

example-stress-server-projectiles:
    cargo run --bin example_stress_projectiles_server --features perfs --release

test:
    cargo nextest run

test-e2e:
    just test-projectiles-obstacles-one-wall
    just test-projectiles-obstacles-multiple-wall
    just test-projectiles-obstacles-one-hill
    just test-projectiles-obstacles-multiple-hill
    just test-individual-shots
    just test-individual-behaviors-move-straight-ahead
    just test-individual-behaviors-move-straight-ahead-obstacle

test-projectiles-obstacles-one-wall:
    RUST_LOG=WARN cargo run --bin example_projectiles_obstacles --features test -- one-against-wall --test

test-projectiles-obstacles-multiple-wall:
    RUST_LOG=WARN cargo run --bin example_projectiles_obstacles --features test -- multiple-against-wall --test

test-projectiles-obstacles-one-hill:
    RUST_LOG=WARN cargo run --bin example_projectiles_obstacles --features test -- one-against-hill --test

test-projectiles-obstacles-multiple-hill:
    RUST_LOG=WARN cargo run --bin example_projectiles_obstacles --features test -- multiple-against-hill --test

test-individual-shots:
    RUST_LOG=WARN cargo run --bin example_individual_shots --features test -- --test

test-individual-behaviors-move-straight-ahead:
    RUST_LOG=WARN cargo run --bin example_individual_behaviors --features test -- move-straight-ahead --test

test-individual-behaviors-move-straight-ahead-obstacle:
    RUST_LOG=WARN cargo run --bin example_individual_behaviors --features test -- move-straight-ahead-obstacle --test

test-all:
    just test
    just test-e2e

list:
    just --list
