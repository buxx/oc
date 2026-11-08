setup:
    # cargo install cargo-all-features
    cargo binstall cargo-nextest --secure

check:
    cargo check
    cargo check --tests
    cargo check --features debug,perfs,tracker,test
    cargo clippy

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
    cargo run --bin example_minidblue {{ args }}

example-height *args:
    cargo run --bin example_height --features debug {{ args }}

example-projectiles-obstacles *args:
    cargo run --bin example_projectiles_obstacles --features debug {{ args }}

example-individual-shots *args:
    cargo run --bin example_individual_shots --features debug {{ args }}

example-regions *args:
    cargo run --bin example_regions --features debug -- {{ args }}

example-individual-behaviors *args:
    cargo run --bin example_individual_behaviors --features debug {{ args }}

example-visibilities *args:
    cargo run --bin example_visibilities --features debug {{ args }}

example-engaging *args:
    cargo run --bin example_engaging --features debug {{ args }}

example-stress-gui-projectiles:
    cargo run --bin example_stress_projectiles --features debug,perfs --release

example-stress-server-projectiles:
    cargo run --bin example_stress_projectiles_server --features perfs --release

example-weapons *args:
    cargo run --bin example_weapons --features debug {{ args }}

example-world *args:
    cargo run --bin example_world --features debug -- {{ args }}

example-interiors *args:
    cargo run --bin example_interiors --features debug {{ args }}

example-stress *args:
    cargo run --bin example_stress --features debug --release {{ args }}

test:
    cargo nextest run

# speed: when not empty, run tests with OC_SPEED=100. jobs: number of tests run in parallel
test-e2e speed="" jobs="4":
    #!/usr/bin/env bash
    if [ -n "{{ speed }}" ]; then
        export OC_SPEED=100
    fi
    tests=(
        test-projectiles-obstacles-one-wall
        test-projectiles-obstacles-multiple-wall
        test-projectiles-obstacles-one-hill
        test-projectiles-obstacles-multiple-hill
        test-projectiles-obstacles-one-traverse-volumes
        test-projectiles-obstacles-one-start-inside-wall
        test-projectiles-obstacles-individual-shot-traverse-volume
        test-projectiles-obstacles-individual-shot-blocked-by-volume
        test-individual-shots-same-pixel
        test-individual-shots-in-volume
        test-individual-shots-different-tile
        test-individual-shots-above
        test-individual-shots-above-prone
        test-individual-shots-near-rotated-prone
        test-individual-behaviors-move-straight-ahead1
        test-individual-behaviors-move-straight-ahead-obstacle1
        test-individual-behaviors-move-straight-ahead2
        test-individual-behaviors-move-straight-ahead-obstacle2
        test-individual-behaviors-move-fast-straight-ahead1
        test-individual-behaviors-move-fast-straight-ahead-obstacle1
        test-individual-behaviors-move-fast-straight-ahead2
        test-individual-behaviors-move-fast-straight-ahead-obstacle2
        test-individual-behaviors-stunned
        test-individual-behaviors-throw-grenade
        test-individual-behaviors-throw-grenade-multiple
        test-individual-behaviors-throw-grenade-after-move
        test-individual-behaviors-throw-grenade-after-move-multiple
        test-individual-behaviors-throw-grenade-when-ennemy-near
        test-regions-projectile-move-out
        test-regions-projectile-move-in
        test-regions-individual-move-out
        test-regions-individual-move-in
        test-visibilities-direct
        test-visibilities-through
        test-visibilities-not-visible
        test-visibilities-hidden
        test-visibilities-discover
        test-visibilities-move-then-enemy-visible
        test-visibilities-altered-wall
        test-engaging-direct
        test-engaging-direct2
        test-engaging-far-machine-gun
        test-engaging-hedge
        test-engaging-suppressed
        test-engaging-move-to-hiding
        test-engaging-move-to-hiding-then-discover
        test-engaging-engage-one-by-one
        test-engaging-suppress-fire
        test-height-climb
        test-interiors-enter
        test-interiors-out
        test-interiors-outside-explosion-protected
        test-interiors-outside-explosion-not-protected
        test-interiors-on-the-top-explosion-protected
        test-interiors-on-the-top-explosion-not-protected-because-broken
        test-interiors-on-the-top-explosion-not-protected
        test-interiors-near-explosion-protected-in-bunker
        test-world-support
        test-world-end-time
        test-world-end-morale
        test-world-explosion-alter-wall
        test-world-smoke-grenade
    )
    # Build once, so parallel runs below don't compete on cargo build lock
    cargo build --bins --features test || exit 1
    logs=$(mktemp -d)
    trap 'rm -rf "$logs"' EXIT
    run() {
        if just "$1" >"$logs/$1.log" 2>&1; then
            echo "✅ $1"
        else
            echo "❌ $1"
            cat "$logs/$1.log"
            return 1
        fi
    }
    export -f run
    export logs
    printf '%s\n' "${tests[@]}" | xargs -P {{ jobs }} -I{} bash -c 'run {}'

test-projectiles-obstacles-one-wall:
    RUST_LOG=ERROR cargo run --bin example_projectiles_obstacles --features test -- one-against-wall --test

test-projectiles-obstacles-multiple-wall:
    RUST_LOG=ERROR cargo run --bin example_projectiles_obstacles --features test -- multiple-against-wall --test

test-projectiles-obstacles-one-hill:
    RUST_LOG=ERROR cargo run --bin example_projectiles_obstacles --features test -- one-against-hill --test

test-projectiles-obstacles-multiple-hill:
    RUST_LOG=ERROR cargo run --bin example_projectiles_obstacles --features test -- multiple-against-hill --test

test-projectiles-obstacles-one-traverse-volumes:
    RUST_LOG=ERROR cargo run --bin example_projectiles_obstacles --features test -- one-traverse-volumes --test

test-projectiles-obstacles-one-start-inside-wall:
    RUST_LOG=ERROR cargo run --bin example_projectiles_obstacles --features test -- one-start-inside-wall --test

test-projectiles-obstacles-individual-shot-traverse-volume:
    RUST_LOG=ERROR cargo run --bin example_projectiles_obstacles --features test -- individual-shot-traverse-volume --test

test-projectiles-obstacles-individual-shot-blocked-by-volume:
    RUST_LOG=ERROR cargo run --bin example_projectiles_obstacles --features test -- individual-shot-blocked-by-volume --test

test-individual-shots-same-pixel:
    RUST_LOG=ERROR cargo run --bin example_individual_shots --features test -- same-pixel --test

test-individual-shots-in-volume:
    RUST_LOG=ERROR cargo run --bin example_individual_shots --features test -- in-volume --test

test-individual-shots-different-tile:
    RUST_LOG=ERROR cargo run --bin example_individual_shots --features test -- different-tile --test

test-individual-shots-above:
    RUST_LOG=ERROR cargo run --bin example_individual_shots --features test -- above --test

test-individual-shots-above-prone:
    RUST_LOG=ERROR cargo run --bin example_individual_shots --features test -- above-prone --test

test-individual-shots-near-rotated-prone:
    RUST_LOG=ERROR cargo run --bin example_individual_shots --features test -- near-rotated-prone --test

test-individual-behaviors-move-straight-ahead1:
    RUST_LOG=ERROR cargo run --bin example_individual_behaviors --features test -- move-straight-ahead --test --individual-count 1

test-individual-behaviors-move-straight-ahead-obstacle1:
    RUST_LOG=ERROR cargo run --bin example_individual_behaviors --features test -- move-straight-ahead-obstacle --test --individual-count 1

test-individual-behaviors-move-straight-ahead2:
    RUST_LOG=ERROR cargo run --bin example_individual_behaviors --features test -- move-straight-ahead --test --individual-count 2

test-individual-behaviors-move-straight-ahead-obstacle2:
    RUST_LOG=ERROR cargo run --bin example_individual_behaviors --features test -- move-straight-ahead-obstacle --test --individual-count 2

test-individual-behaviors-move-fast-straight-ahead1:
    RUST_LOG=ERROR cargo run --bin example_individual_behaviors --features test -- move-fast-straight-ahead --test --individual-count 1

test-individual-behaviors-move-fast-straight-ahead-obstacle1:
    RUST_LOG=ERROR cargo run --bin example_individual_behaviors --features test -- move-fast-straight-ahead-obstacle --test --individual-count 1

test-individual-behaviors-move-fast-straight-ahead2:
    RUST_LOG=ERROR cargo run --bin example_individual_behaviors --features test -- move-fast-straight-ahead --test --individual-count 2

test-individual-behaviors-move-fast-straight-ahead-obstacle2:
    RUST_LOG=ERROR cargo run --bin example_individual_behaviors --features test -- move-fast-straight-ahead-obstacle --test --individual-count 2

test-individual-behaviors-stunned:
    RUST_LOG=ERROR cargo run --bin example_individual_behaviors --features test -- stunned --test --individual-count 1

test-individual-behaviors-throw-grenade:
    RUST_LOG=ERROR cargo run --bin example_individual_behaviors --features test -- throw-grenade --individual-count 1 --grenade-count 1 --test

test-individual-behaviors-throw-grenade-multiple:
    RUST_LOG=ERROR cargo run --bin example_individual_behaviors --features test -- throw-grenade --individual-count 2 --grenade-count 2 --test

test-individual-behaviors-throw-grenade-after-move:
    RUST_LOG=ERROR cargo run --bin example_individual_behaviors --features test -- throw-grenade-after-move --individual-count 1 --grenade-count 1 --test

test-individual-behaviors-throw-grenade-after-move-multiple:
    RUST_LOG=ERROR cargo run --bin example_individual_behaviors --features test -- throw-grenade-after-move --individual-count 2 --grenade-count 2 --test

test-individual-behaviors-throw-grenade-when-ennemy-near:
    RUST_LOG=ERROR cargo run --bin example_individual_behaviors --features test -- throw-grenade-when-ennemy-near --individual-count 1 --grenade-count 1 --test

test-regions-projectile-move-out:
    RUST_LOG=ERROR cargo run --bin example_regions --features test -- projectile-move-out --test

test-regions-projectile-move-in:
    RUST_LOG=ERROR cargo run --bin example_regions --features test -- projectile-move-in --test

test-regions-individual-move-out:
    RUST_LOG=ERROR cargo run --bin example_regions --features test -- individual-move-out --test

test-regions-individual-move-in:
    RUST_LOG=ERROR cargo run --bin example_regions --features test -- individual-move-in --test

test-visibilities-direct:
    RUST_LOG=ERROR cargo run --bin example_visibilities --features test -- direct --test

test-visibilities-through:
    RUST_LOG=ERROR cargo run --bin example_visibilities --features test -- through --test

test-visibilities-not-visible:
    RUST_LOG=ERROR cargo run --bin example_visibilities --features test -- not-visible --test

test-visibilities-hidden:
    RUST_LOG=ERROR cargo run --bin example_visibilities --features test -- hidden --test

test-visibilities-discover:
    RUST_LOG=ERROR cargo run --bin example_visibilities --features test -- discover --test

test-visibilities-move-then-enemy-visible:
    RUST_LOG=ERROR cargo run --bin example_visibilities --features test -- move-then-enemy-visible --test

test-visibilities-altered-wall:
    RUST_LOG=ERROR cargo run --bin example_visibilities --features test -- altered-wall --test

test-engaging-direct:
    RUST_LOG=ERROR cargo run --bin example_engaging --features test,debug -- direct --test

test-engaging-direct2:
    RUST_LOG=ERROR cargo run --bin example_engaging --features test,debug -- direct2 --test

test-engaging-far-machine-gun:
    RUST_LOG=ERROR cargo run --bin example_engaging --features test,debug -- far-machine-gun --test

test-engaging-hedge:
    RUST_LOG=ERROR cargo run --bin example_engaging --features test,debug -- hedge --test

test-engaging-suppressed:
    RUST_LOG=ERROR cargo run --bin example_engaging --features test,debug -- suppressed --test

test-engaging-move-to-hiding:
    RUST_LOG=ERROR cargo run --bin example_engaging --features test,debug -- move-to-hiding --test

test-engaging-move-to-hiding-then-discover:
    RUST_LOG=ERROR cargo run --bin example_engaging --features test,debug -- move-to-hiding-then-discover --test

test-engaging-engage-one-by-one:
    RUST_LOG=ERROR cargo run --bin example_engaging --features test,debug -- engage-one-by-one --test

test-engaging-suppress-fire:
    RUST_LOG=ERROR cargo run --bin example_engaging --features test,debug -- suppress-fire --test

test-height-climb:
    RUST_LOG=ERROR cargo run --bin example_height --features test -- climb --test

test-interiors-enter:
    RUST_LOG=ERROR cargo run --bin example_interiors --features test,debug -- enter --test

test-interiors-out:
    RUST_LOG=ERROR cargo run --bin example_interiors --features test,debug -- out --test

test-interiors-outside-explosion-protected:
    RUST_LOG=ERROR cargo run --bin example_interiors --features test,debug -- outside-explosion-protected --test

test-interiors-outside-explosion-not-protected:
    RUST_LOG=ERROR cargo run --bin example_interiors --features test,debug -- outside-explosion-not-protected --test

test-interiors-on-the-top-explosion-protected:
    RUST_LOG=ERROR cargo run --bin example_interiors --features test,debug -- on-the-top-explosion-protected --test

test-interiors-on-the-top-explosion-not-protected-because-broken:
    RUST_LOG=ERROR cargo run --bin example_interiors --features test,debug -- on-the-top-explosion-not-protected-because-broken --test

test-interiors-on-the-top-explosion-not-protected:
    RUST_LOG=ERROR cargo run --bin example_interiors --features test,debug -- on-the-top-explosion-not-protected --test

test-interiors-near-explosion-protected-in-bunker:
    RUST_LOG=ERROR cargo run --bin example_interiors --features test,debug -- near-explosion-protected-in-bunker --test

test-world-support:
    RUST_LOG=ERROR cargo run --bin example_world --features test -- support --test

test-world-end-time:
    RUST_LOG=ERROR cargo run --bin example_world --features test -- end-time --test

test-world-end-morale:
    RUST_LOG=ERROR cargo run --bin example_world --features test -- end-morale --test

test-world-explosion-alter-wall:
    RUST_LOG=ERROR cargo run --bin example_world --features test -- explosion-alter-wall --test

test-world-smoke-grenade:
    RUST_LOG=ERROR cargo run --bin example_world --features test -- smoke-grenade --test

test-all:
    just test
    just test-e2e fast

list:
    just --list
