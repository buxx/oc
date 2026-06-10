use oc_geo::tile::{TileXy, WorldTileIndex};
use oc_root::{WcfgFrom, WorldConfig};
use oc_utils::d2::Xy;
use polyanya::*;

// function built by AI (claude), then modified by myself
pub fn navmesh(w: &WorldConfig, grid: &[bool]) -> Mesh {
    // 1. Define the outer boundary (the full walkable world)
    //    Points go counter-clockwise around the perimeter.
    let width = w.world_width_pixels as f32;
    let cols = w.world_width;
    let height = w.world_height_pixels as f32;
    let rows = w.world_width;
    let tile_size = w.geo_pixels_per_tile as f32;

    let mut triangulation = Triangulation::from_outer_edges(&[
        [0.0, 0.0].into(),
        [width, 0.0].into(),
        [width, height].into(),
        [0.0, height].into(),
    ]);

    // TODO: values
    // Keeps the path center at least 2.5px away from any wall edge
    triangulation.set_agent_radius(2.5);
    // For tile walls, rounded corners aren't needed — fewer segments = faster
    triangulation.set_agent_radius_segments(1);
    // merge nearly-collinear points, epsilon in pixels²
    triangulation.simplify(0.1);

    // 2. Add each blocked tile as a rectangular obstacle.
    //    Points must be in clockwise order for obstacles.
    for row in 0..rows {
        for col in 0..cols {
            let tile_xy = TileXy(Xy(col, row));
            let tile_i = WorldTileIndex::from_(tile_xy, w);
            if grid[tile_i.0 as usize]
            /* is wall/blocked */
            {
                let x = col as f32 * tile_size;
                let y = row as f32 * tile_size;

                // dbg!([
                //     [x, y],
                //     [x, y + tile_size],
                //     [x + tile_size, y + tile_size],
                //     [x + tile_size, y],
                // ]);
                triangulation.add_obstacle([
                    [x, y].into(),
                    [x, y + tile_size].into(),
                    [x + tile_size, y + tile_size].into(),
                    [x + tile_size, y].into(),
                ]);
            }
        }
    }

    // 3. Convert triangulation → Mesh, merge + bake for efficiency
    let mut mesh = triangulation.as_navmesh();
    mesh.merge_polygons(); // collapses triangles into bigger convex polygons
    mesh.bake(); // builds internal spatial index for fast queries

    mesh
}
