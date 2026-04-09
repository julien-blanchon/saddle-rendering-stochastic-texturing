use bevy::prelude::*;

/// Runtime counts for the public authoring and adapter surface.
#[derive(Resource, Reflect, Clone, Debug, Default)]
#[reflect(Resource, Default)]
pub struct StochasticTexturingDiagnostics {
    pub active_surfaces: usize,
    pub active_height_maps: usize,
    pub active_pbr_surfaces: usize,
    pub world_triplanar_surfaces: usize,
    pub histogram_preserving_surfaces: usize,
    pub high_quality_surfaces: usize,
    pub debug_views_enabled: usize,
    pub estimated_base_sample_count: u64,
}

impl StochasticTexturingDiagnostics {
    #[must_use]
    pub fn has_debug_view(&self) -> bool {
        self.debug_views_enabled > 0
    }
}
