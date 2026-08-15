use derive_more::Constructor;
use oc_geo::tile::{TileXy, WorldTileIndex};
use oc_individual::IndividualIndex;
use oc_root::{WcfgFrom, WorldConfig, opacity::CumulatedOpacity};
use oc_utils::d2::Xy;
use oc_world::{World, visibility::Visibility};

use crate::index::Indexes;

#[derive(Constructor)]
pub struct Processor<'a> {
    world: &'a World,
    index: &'a Indexes,
}

impl<'a> Processor<'a> {
    pub fn compute(&self) -> Vec<(IndividualIndex, IndividualIndex, Visibility)> {
        let w = &self.world.w;
        let mod_ = &self.world.mod_;
        let count = self.world.individuals().len();
        let mut visibilities = Vec::with_capacity(count * 2);
        let at = |xy, z| path_objects_at(w, mod_, self.world, xy, z);

        for i1 in self.index.side_a_individuals() {
            tracing::trace!(name="visibility-processor-compute", i1=?i1);

            let individual1 = self.world.individual(*i1);
            if !individual1.can_lov() {
                tracing::trace!(name="visibility-processor-compute-cant-lov", i1=?i1);
                continue;
            };

            for i2 in self.index.side_b_individuals() {
                let individual2 = self.world.individual(*i2);
                if !individual2.is_lov() {
                    tracing::trace!(name="visibility-processor-compute-cant-be-lov", i1=?i1, i2=?i2);
                    continue;
                };

                let p1 = individual1.position;
                let p2 = individual2.position;
                let lov = oc_lov::PathBuilder::new(w, at).build_(p1, p2);

                let opacity = lov
                    .sections
                    .last()
                    .map(|s| s.opacity)
                    .unwrap_or(CumulatedOpacity(1.0));
                let visible = opacity <= w.individual_visibility_until;

                tracing::trace!(name="visibility-processor-compute-result", i1=?i1, i2=?i2, visible=visible, opacity=?opacity);
                visibilities.push((*i1, *i2, Visibility::new(visible, opacity)));
            }
        }

        for i1 in self.index.side_b_individuals() {
            tracing::trace!(name="visibility-processor-compute", i1=?i1);

            let individual1 = self.world.individual(*i1);
            if !individual1.can_lov() {
                tracing::trace!(name="visibility-processor-compute-cant-lov", i1=?i1);
                continue;
            };

            for i2 in self.index.side_a_individuals() {
                let individual2 = self.world.individual(*i2);
                if !individual2.is_lov() {
                    tracing::trace!(name="visibility-processor-compute-cant-be-lov", i1=?i1, i2=?i2);
                    continue;
                };

                let p1 = individual1.position;
                let p2 = individual2.position;
                let lov = oc_lov::PathBuilder::new(w, at).build_(p1, p2);

                let opacity = lov
                    .sections
                    .last()
                    .map(|s| s.opacity)
                    .unwrap_or(CumulatedOpacity(1.0));
                let visible = opacity <= w.individual_visibility_until;

                tracing::trace!(name="visibility-processor-compute-result", i1=?i1, i2=?i2, visible=visible, opacity=?opacity);
                visibilities.push((*i1, *i2, Visibility::new(visible, opacity)));
            }
        }

        visibilities
    }
}

fn path_objects_at(
    w: &WorldConfig,
    mod_: &oc_mod::Mod,
    world: &World,
    at: Xy,
    z: f32,
) -> Vec<oc_lov::Step> {
    world
        .tile(WorldTileIndex::from_(TileXy(at), w))
        .map(|t| {
            let tile_z = t.z as f32 * w.geo_meters_per_z.0 * w.geo_pixels_per_meters;
            let relative_z = z - tile_z;
            let opacity = mod_.nature(t.nature).opacity(w, relative_z);
            vec![oc_lov::Step { opacity }]
        })
        .unwrap_or(vec![])
}
