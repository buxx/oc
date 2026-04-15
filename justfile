check:
    cargo check && cargo check --features debug

serve-minidblue:
    RUST_LOG=DEBUG WORLD_WIDTH=200 WORLD_HEIGHT=200 \
      cargo run --bin oc_world_server --features debug -- \
      ./mods/std1 ./examples/minidblue ./examples/minidblue.snapshot

serve-world1:
    RUST_LOG=DEBUG WORLD_WIDTH=1000 WORLD_HEIGHT=1000 \
      cargo run --bin oc_world_server --features debug -- \
      ./mods/std1 ./examples/world1 ./examples/world1.snapshot

example1:
    ./examples.sh --release --debug walking_squad

example2:
    ./examples.sh --release --debug firing_with_wall

world-minidblue *args:
    cargo run --bin world -- examples/minidblue examples/minidblue.snapshot --verbose {{ args }}

world-world1 *args:
    cargo run --bin world -- examples/world1 examples/world1.snapshot --verbose {{ args }}
