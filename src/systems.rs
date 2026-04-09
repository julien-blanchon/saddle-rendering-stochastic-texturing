use bevy::prelude::*;

use crate::{
    StochasticPbrRuntimeState, StochasticTexturingRuntimeState,
    components::{StochasticHeightMap, StochasticTexturing},
    config::{
        StochasticBlendMode, StochasticDebugSettings, StochasticQuality, StochasticSamplingMode,
        StochasticUvSpace,
    },
    diagnostics::StochasticTexturingDiagnostics,
};

pub fn activate_runtime(mut runtime: ResMut<StochasticTexturingRuntimeState>) {
    runtime.active = true;
}

pub fn deactivate_runtime(mut runtime: ResMut<StochasticTexturingRuntimeState>) {
    runtime.active = false;
}

pub fn activate_pbr_runtime(mut runtime: ResMut<StochasticPbrRuntimeState>) {
    runtime.active = true;
}

pub fn deactivate_pbr_runtime(mut runtime: ResMut<StochasticPbrRuntimeState>) {
    runtime.active = false;
}

pub fn runtime_is_active(runtime: Res<StochasticTexturingRuntimeState>) -> bool {
    runtime.active
}

pub fn pbr_runtime_is_active(runtime: Res<StochasticPbrRuntimeState>) -> bool {
    runtime.active
}

pub fn publish_core_diagnostics(
    surfaces: Query<&StochasticTexturing>,
    height_maps: Query<(), With<StochasticHeightMap>>,
    debug: Res<StochasticDebugSettings>,
    mut diagnostics: ResMut<StochasticTexturingDiagnostics>,
) {
    diagnostics.active_surfaces = 0;
    diagnostics.active_height_maps = height_maps.iter().count();
    diagnostics.world_triplanar_surfaces = 0;
    diagnostics.histogram_preserving_surfaces = 0;
    diagnostics.high_quality_surfaces = 0;
    diagnostics.debug_views_enabled = usize::from(debug.enabled);
    diagnostics.estimated_base_sample_count = 0;

    for surface in &surfaces {
        diagnostics.active_surfaces += 1;
        diagnostics.estimated_base_sample_count +=
            u64::from(surface.expected_sample_count_per_map());

        if surface.uv_space == StochasticUvSpace::WorldTriplanar {
            diagnostics.world_triplanar_surfaces += 1;
        }

        if surface.quality == StochasticQuality::HighQuality {
            diagnostics.high_quality_surfaces += 1;
        }

        if surface.sampling_mode == StochasticSamplingMode::HistogramPreserving
            || surface.blend_mode == StochasticBlendMode::HistogramPreserving
        {
            diagnostics.histogram_preserving_surfaces += 1;
        }
    }
}

pub fn publish_pbr_diagnostics(
    surfaces: Query<(), With<crate::pbr::StochasticPbrMaterialBinding>>,
    mut diagnostics: ResMut<StochasticTexturingDiagnostics>,
) {
    diagnostics.active_pbr_surfaces = surfaces.iter().count();
}
