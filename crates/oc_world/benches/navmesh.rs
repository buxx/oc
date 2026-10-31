use gungraun::prelude::*;
use oc_root::WorldConfig;
use oc_world::navmesh::navmesh;
use std::hint::black_box;

const SIZE: u64 = 32;

fn world() -> WorldConfig {
    WorldConfig::new(SIZE, SIZE).with_geo_pixels_per_tile(5)
}

fn empty_grid() -> (WorldConfig, Vec<bool>) {
    (world(), vec![false; (SIZE * SIZE) as usize])
}

/// Scattered blocked tiles (about 1 on 7)
fn scattered_grid() -> (WorldConfig, Vec<bool>) {
    let grid = (0..SIZE * SIZE).map(|i| i % 7 == 0 && i % 5 != 0).collect();
    (world(), grid)
}

#[library_benchmark]
#[bench::empty(setup = empty_grid)]
#[bench::scattered(setup = scattered_grid)]
fn bench_navmesh((w, grid): (WorldConfig, Vec<bool>)) {
    black_box(navmesh(black_box(&w), black_box(&grid)));
}

library_benchmark_group!(
    name = bench_navmesh_group,
    benchmarks = [bench_navmesh]
);

main!(library_benchmark_groups = bench_navmesh_group);
