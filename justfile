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

example-walking:
    cargo run --bin example_walking_squad --features debug

example-firing:
    cargo run --bin example_firing_with_wall --features debug

example-wall:
    cargo run --bin example_wall --features debug

world-minidblue *args:
    cargo run --bin world -- examples/minidblue examples/minidblue.snapshot --verbose {{ args }}

world-world1 *args:
    cargo run --bin world -- examples/world1 examples/world1.snapshot --verbose {{ args }}

test:
    cargo test

test-all:
    cargo test && just test-projectile-wall

test-projectile-wall:
    cargo run --bin test_projectile_wall --features test
