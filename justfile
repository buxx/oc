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
    ./examples.sh --debug walking_squad

example-firing:
    ./examples.sh --debug firing_with_wall

example-wall:
    ./examples.sh --debug wall

world-minidblue *args:
    cargo run --bin world -- examples/minidblue examples/minidblue.snapshot --verbose {{ args }}

world-world1 *args:
    cargo run --bin world -- examples/world1 examples/world1.snapshot --verbose {{ args }}

test:
    cargo test

test-all:
    cargo test && ./examples.sh test_projectile_wall

test-projectile-wall:
    ./examples.sh test_projectile_wall
