setup:
    # cargo install cargo-all-features
    cargo binstall cargo-nextest --secure

check:
    cargo check && cargo check --features debug && cargo check --tests && cargo check --features perfs

serve-minidblue:
    RUST_LOG=DEBUG WORLD_WIDTH=200 WORLD_HEIGHT=200 \
      cargo run --bin oc_world_server --features debug -- \
      ./mods/std1 ./examples/minidblue ./examples/minidblue.snapshot

serve-world1:
    RUST_LOG=DEBUG WORLD_WIDTH=1000 WORLD_HEIGHT=1000 \
      cargo run --bin oc_world_server --features debug -- \
      ./mods/std1 ./examples/world1 ./examples/world1.snapshot

gui:
    cargo run --bin oc_battle_gui

example-world1 *args:
    cargo run --bin example_world1 --features debug {{ args }}

example-minidblue *args:
    cargo run --bin example_minidblue --features debug {{ args }}

example-wall *args:
    cargo run --bin example_wall --features debug {{ args }}

example-height *args:
    cargo run --bin example_height --features debug {{ args }}

example-projectiles-wall *args:
    cargo run --bin example_projectiles_wall --features test {{ args }}

example-stress-gui-projectiles:
    cargo run --bin example_stress_projectiles --features debug,perfs --release

example-stress-server-projectiles:
    cargo run --bin example_stress_projectiles_server --features perfs --release

test:
    cargo nextest run
