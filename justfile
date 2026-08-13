setup:
    # cargo install cargo-all-features
    cargo binstall cargo-nextest --secure

check:
    cargo check
    cargo check --tests
    cargo check --features debug,perfs,tracker,test

make-caches:
    cargo build
    cargo build --features debug,perfs
    cargo build --features test
    cargo build --tests
    cargo build --release
    just check

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

example-regions-projectile-move-out *args:
    cargo run --bin example_regions --features debug -- projectile-move-out {{ args }}

example-regions-projectile-move-in *args:
    cargo run --bin example_regions --features debug -- projectile-move-in {{ args }}

example-regions-individual-move-out *args:
    cargo run --bin example_regions --features debug -- individual-move-out {{ args }}

example-regions-individual-move-in *args:
    cargo run --bin example_regions --features debug -- individual-move-in {{ args }}

example-individual-behaviors *args:
    cargo run --bin example_individual_behaviors --features debug {{ args }}

example-stress-gui-projectiles:
    cargo run --bin example_stress_projectiles --features debug,perfs --release

example-stress-server-projectiles:
    cargo run --bin example_stress_projectiles_server --features perfs --release

[working-directory: 'crates/oc_battle_gui']
test-battle_gui:
    if grep -rqE '#\[(test|rstest)\]' src tests 2>/dev/null; then cargo nextest run; fi

[working-directory: 'crates/oc_geo']
test-geo:
    if grep -rqE '#\[(test|rstest)\]' src tests 2>/dev/null; then cargo nextest run; fi

[working-directory: 'crates/oc_individual']
test-individual:
    if grep -rqE '#\[(test|rstest)\]' src tests 2>/dev/null; then cargo nextest run; fi

[working-directory: 'crates/oc_lov']
test-lov:
    if grep -rqE '#\[(test|rstest)\]' src tests 2>/dev/null; then cargo nextest run; fi

[working-directory: 'crates/oc_mod']
test-mod:
    if grep -rqE '#\[(test|rstest)\]' src tests 2>/dev/null; then cargo nextest run; fi

[working-directory: 'crates/oc_network']
test-network:
    if grep -rqE '#\[(test|rstest)\]' src tests 2>/dev/null; then cargo nextest run; fi

[working-directory: 'crates/oc_physics']
test-physics:
    if grep -rqE '#\[(test|rstest)\]' src tests 2>/dev/null; then cargo nextest run; fi

[working-directory: 'crates/oc_projectile']
test-projectile:
    if grep -rqE '#\[(test|rstest)\]' src tests 2>/dev/null; then cargo nextest run; fi

[working-directory: 'crates/oc_root']
test-root:
    if grep -rqE '#\[(test|rstest)\]' src tests 2>/dev/null; then cargo nextest run; fi

[working-directory: 'crates/oc_utils']
test-utils:
    if grep -rqE '#\[(test|rstest)\]' src tests 2>/dev/null; then cargo nextest run; fi

[working-directory: 'crates/oc_world']
test-world:
    if grep -rqE '#\[(test|rstest)\]' src tests 2>/dev/null; then cargo nextest run; fi

[working-directory: 'crates/oc_world_generator']
test-world_generator:
    if grep -rqE '#\[(test|rstest)\]' src tests 2>/dev/null; then cargo nextest run; fi

[working-directory: 'crates/oc_world_server']
test-world_server:
    if grep -rqE '#\[(test|rstest)\]' src tests 2>/dev/null; then cargo nextest run; fi

[working-directory: 'crates/tests']
test-tests:
    if grep -rqE '#\[(test|rstest)\]' src tests 2>/dev/null; then cargo nextest run; fi

# When run tests from workspace root, it trigger a massive parallelized compile works which need
# a lot of disk simultaneous read/write. One of my computer can't follow ... So, a simple solution
# is here is run test from each separated crate.
test:
    just test-battle_gui
    just test-geo
    just test-individual
    just test-lov
    just test-mod
    just test-network
    just test-physics
    just test-projectile
    just test-root
    just test-utils
    just test-world
    just test-world_generator
    just test-world_server
    just test-tests

test-e2e:
    just test-projectiles-obstacles-one-wall
    just test-projectiles-obstacles-multiple-wall
    just test-projectiles-obstacles-one-hill
    just test-projectiles-obstacles-multiple-hill
    just test-individual-shots-same-pixel
    just test-individual-shots-in-volume
    just test-individual-shots-different-tile
    just test-individual-behaviors-move-straight-ahead1
    just test-individual-behaviors-move-straight-ahead-obstacle1
    just test-individual-behaviors-move-straight-ahead2
    just test-individual-behaviors-move-straight-ahead-obstacle2
    just test-regions-projectile-move-out
    just test-regions-projectile-move-in
    just test-regions-individual-move-out
    just test-regions-individual-move-in

test-projectiles-obstacles-one-wall:
    RUST_LOG=ERROR cargo run --bin example_projectiles_obstacles --features test -- one-against-wall --test

test-projectiles-obstacles-multiple-wall:
    RUST_LOG=ERROR cargo run --bin example_projectiles_obstacles --features test -- multiple-against-wall --test

test-projectiles-obstacles-one-hill:
    RUST_LOG=ERROR cargo run --bin example_projectiles_obstacles --features test -- one-against-hill --test

test-projectiles-obstacles-multiple-hill:
    RUST_LOG=ERROR cargo run --bin example_projectiles_obstacles --features test -- multiple-against-hill --test

test-individual-shots-same-pixel:
    RUST_LOG=ERROR cargo run --bin example_individual_shots --features test -- same-pixel --test

test-individual-shots-in-volume:
    RUST_LOG=ERROR cargo run --bin example_individual_shots --features test -- in-volume --test

test-individual-shots-different-tile:
    RUST_LOG=ERROR cargo run --bin example_individual_shots --features test -- different-tile --test

test-individual-behaviors-move-straight-ahead1:
    RUST_LOG=ERROR cargo run --bin example_individual_behaviors --features test -- move-straight-ahead --test --count 1

test-individual-behaviors-move-straight-ahead-obstacle1:
    RUST_LOG=ERROR cargo run --bin example_individual_behaviors --features test -- move-straight-ahead-obstacle --test --count 1

test-individual-behaviors-move-straight-ahead2:
    RUST_LOG=ERROR cargo run --bin example_individual_behaviors --features test -- move-straight-ahead --test --count 2

test-individual-behaviors-move-straight-ahead-obstacle2:
    RUST_LOG=ERROR cargo run --bin example_individual_behaviors --features test -- move-straight-ahead-obstacle --test --count 2

test-regions-projectile-move-out:
    RUST_LOG=ERROR cargo run --bin example_regions -- projectile-move-out --test

test-regions-projectile-move-in:
    RUST_LOG=ERROR cargo run --bin example_regions -- projectile-move-in --test

test-regions-individual-move-out:
    RUST_LOG=ERROR cargo run --bin example_regions -- individual-move-out --test

test-regions-individual-move-in:
    RUST_LOG=ERROR cargo run --bin example_regions -- individual-move-in --test

test-all:
    just test
    just test-e2e

list:
    just --list
