check:
    cargo check && cargo check --features debug && cargo check --tests

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

example-projectiles-wall:
    cargo run --bin projectiles_wall --features test

test:
    cargo test bevy_heightmap
    cargo test oc_battle_gui
    cargo test oc_examples
    cargo test oc_geo
    cargo test oc_individual
    cargo test oc_lov
    cargo test oc_mod
    cargo test oc_network
    cargo test oc_physics
    cargo test oc_projectile
    cargo test oc_root
    cargo test oc_utils
    cargo test oc_world
    cargo test oc_world_generator
    cargo test oc_world_server
    cargo test tests
