use std::time::Instant;

use oc_geo::region::WorldRegionIndex;
use oc_geo::tile::{TileXy, WorldTileIndex};
use oc_individual::{INDIVIDUAL_PRONE_VOLUME_WIDTH, IndividualIndex, Status};
use oc_lov::{Modifier, PathBuilder};
use oc_mod::Mod;
use oc_physics::fx;
use oc_root::explosion::Explosion;
use oc_root::geo::{WorldVec2, WorldVec3};
use oc_root::{Client, WcfgFrom, WcfgInto};
use oc_utils::let_some;
use oc_world::World;
use rustc_hash::FxHashSet;

use super::update::ExplosionOrigin;
use crate::individual::death::die;
use crate::routing::Listening;
use crate::visibility::path_objects_at;
use oc_network::ToClient;

impl<E: Client> super::State<E> {
    pub fn spawn_explosion(
        &self,
        explosion: Explosion,
        position: WorldVec2,
        origin: ExplosionOrigin,
    ) -> Vec<(Listening, Vec<ToClient>)> {
        let (updates, region, debug_fx, sound) = {
            let world = self.world();
            let indexes = self.indexes();
            let w = &world.w;
            let mod_ = world.mod_();

            let kill_rayon = explosion.kill_rayon.pixels(w);
            let stunned_rayon = explosion.stunned_rayon.pixels(w);
            let shrapnel_rayon = explosion.shrapnel_rayon.pixels(w);
            let largest_rayon = kill_rayon.max(stunned_rayon).max(shrapnel_rayon);

            // Tiles concerned by largest rayon
            let tile = WorldTileIndex::from_(TileXy::from_(position, w), w);
            let_some!(tile_ = world.tile(tile), return vec![]);
            let region: WorldRegionIndex = tile.into_(w);
            let tiles_rayon = (largest_rayon / w.geo_pixels_per_tile() as f32).ceil() as u64;
            let tiles = crate::utils::d2::shape_from_tile(tile, tiles_rayon, w);

            tracing::trace!(name="runner-explosion", position=?position, largest_rayon=largest_rayon);

            // Not dead individuals in these tiles
            let mut seen = FxHashSet::default();
            let individuals: Vec<IndividualIndex> = tiles
                .into_iter()
                .flat_map(|tile| indexes.tile_individuals(tile).iter().copied())
                .filter(|i| seen.insert(*i))
                .filter(|i| !matches!(world.individual(*i).status, Status::Dead))
                .collect();

            let distance = |i: IndividualIndex| {
                let p = world.individual(i).position;
                ((p.x - position.x).powi(2) + (p.y - position.y).powi(2)).sqrt()
            };

            let mut killed = FxHashSet::default();
            let mut updates = vec![];

            // Individuals in kill rayon die
            for i in &individuals {
                if distance(*i) <= kill_rayon {
                    killed.insert(*i);
                    updates.extend(die(&world, mod_, *i));
                }
            }

            // Shrapnels
            let start = WorldVec3::new(position.x, position.y, tile_.z_pixels(w));
            let candidates: Vec<IndividualIndex> = individuals
                .iter()
                .copied()
                .filter(|i| !killed.contains(i))
                .collect();
            let (shrapnel_killed, _shrapnel_lines) =
                shrapnels(&world, mod_, &explosion, start, &candidates);
            for i in shrapnel_killed {
                updates.extend(die(&world, mod_, i));
            }

            #[cfg(feature = "debug")]
            let debug_fx = fx::Fx::Visual(fx::Visual::ExplosionDebug(fx::ExplosionDebug {
                position,
                kill_rayon: explosion.kill_rayon,
                stunned_rayon: explosion.stunned_rayon,
                shrapnel_rayon: explosion.shrapnel_rayon,
                shrapnels: _shrapnel_lines,
            }));
            #[cfg(not(feature = "debug"))]
            let debug_fx = None::<fx::Fx>;

            let sound = match origin {
                ExplosionOrigin::Shell(shell) => mod_.shell(shell).pick_sound(),
            }
            .map(|sound| {
                fx::Fx::Audio(fx::Audio::PlayWithGeo(
                    sound,
                    start,
                    fx::SoundBus::Explosion,
                ))
            });

            (updates, region, debug_fx, sound)
        };

        let now = Instant::now();
        for update in updates {
            self.schedule(now, update);
        }

        let fx = fx::Fx::Visual(fx::Visual::Explosion(position, explosion.stunned_rayon));
        let mut messages = vec![ToClient::Fx(fx)];
        messages.extend(sound.map(ToClient::Fx));
        #[cfg(feature = "debug")]
        messages.push(ToClient::Fx(debug_fx));
        #[cfg(not(feature = "debug"))]
        let _ = debug_fx;

        vec![(Listening::Regions(vec![region]), messages)]
    }
}

// WARNING: AI generated
/// Individuals killed by explosion's shrapnels
fn shrapnels(
    world: &World,
    mod_: &Mod,
    explosion: &Explosion,
    start: WorldVec3,
    candidates: &[IndividualIndex],
) -> (FxHashSet<IndividualIndex>, Vec<(WorldVec2, WorldVec2)>) {
    let w = &world.w;
    let shrapnel_rayon = explosion.shrapnel_rayon.pixels(w);
    let hit_radius = INDIVIDUAL_PRONE_VOLUME_WIDTH.pixels(w) / 2.; // Arbitrary choice to simplify
    let at = |xy, z| path_objects_at(w, mod_, world, xy, z);
    let builder = PathBuilder::new(w, at);
    let distance = |i: IndividualIndex| {
        let p = world.individual(i).position;
        ((p.x - start.x).powi(2) + (p.y - start.y).powi(2)).sqrt()
    };
    let candidates: Vec<IndividualIndex> = candidates
        .iter()
        .copied()
        .filter(|i| distance(*i) <= shrapnel_rayon + hit_radius)
        .collect();
    let mut killed = FxHashSet::default();
    let mut lines = Vec::with_capacity(explosion.shrapnel_quantity as usize);

    for _ in 0..explosion.shrapnel_quantity {
        let angle = fastrand::f32() * std::f32::consts::TAU;
        let length = shrapnel_rayon * fastrand::f32();
        let end = WorldVec3::new(
            start.x + length * angle.cos(),
            start.y + length * angle.sin(),
            start.z,
        );

        lines.push((
            WorldVec2::new(start.x, start.y),
            WorldVec2::new(end.x, end.y),
        ));

        for i in &candidates {
            let p = world.individual(*i).position;
            if distance_to_segment(p, start, end) > hit_radius {
                continue;
            }

            // Shrapnel reach individual if nothing stop it on the way
            let path = builder.build(start, p, 0, Modifier::None(true));
            let opacity = path.sections.last().map(|s| s.opacity.0).unwrap_or(0.);
            if opacity < 1.0 {
                killed.insert(*i);
            }
        }
    }

    (killed, lines)
}

// WARNING: AI generated
/// Distance (2d) between point and segment
fn distance_to_segment(p: WorldVec3, a: WorldVec3, b: WorldVec3) -> f32 {
    let (abx, aby) = (b.x - a.x, b.y - a.y);
    let length2 = abx * abx + aby * aby;
    let t = if length2 == 0. {
        0.
    } else {
        (((p.x - a.x) * abx + (p.y - a.y) * aby) / length2).clamp(0., 1.)
    };
    let (cx, cy) = (a.x + t * abx, a.y + t * aby);
    ((p.x - cx).powi(2) + (p.y - cy).powi(2)).sqrt()
}
