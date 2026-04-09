mod components;
mod config;
mod diagnostics;
mod gpu;
mod pbr;
mod shader_library;
mod systems;

pub use components::{StochasticHeightMap, StochasticTexturing};
pub use config::{
    StochasticBlendMode, StochasticDebugSettings, StochasticDebugView, StochasticNormalMapMode,
    StochasticQuality, StochasticRotationMode, StochasticSamplingMode, StochasticUvSpace,
    TextureChannel,
};
pub use diagnostics::StochasticTexturingDiagnostics;
pub use gpu::StochasticShaderUniform;
pub use pbr::{StochasticPbrMaterial, StochasticPbrUniform, StochasticTexturingMaterialExtension};
pub use shader_library::{
    STOCHASTIC_SAMPLING_IMPORT_PATH, STOCHASTIC_TYPES_IMPORT_PATH, StochasticShaderLibraryPlugin,
};

use bevy::{
    app::PostStartup,
    ecs::{intern::Interned, schedule::ScheduleLabel},
    prelude::*,
};

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum StochasticTexturingSystems {
    Prepare,
    Diagnostics,
    Debug,
}

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum StochasticPbrSystems {
    AdaptMaterials,
    SyncUniforms,
    Diagnostics,
}

#[derive(Resource, Default)]
pub(crate) struct StochasticTexturingRuntimeState {
    pub active: bool,
}

#[derive(Resource, Default)]
pub(crate) struct StochasticPbrRuntimeState {
    pub active: bool,
}

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct NeverDeactivateSchedule;

pub struct StochasticTexturingPlugin {
    pub activate_schedule: Interned<dyn ScheduleLabel>,
    pub deactivate_schedule: Interned<dyn ScheduleLabel>,
    pub update_schedule: Interned<dyn ScheduleLabel>,
}

impl StochasticTexturingPlugin {
    #[must_use]
    pub fn new(
        activate_schedule: impl ScheduleLabel,
        deactivate_schedule: impl ScheduleLabel,
        update_schedule: impl ScheduleLabel,
    ) -> Self {
        Self {
            activate_schedule: activate_schedule.intern(),
            deactivate_schedule: deactivate_schedule.intern(),
            update_schedule: update_schedule.intern(),
        }
    }

    #[must_use]
    pub fn always_on(update_schedule: impl ScheduleLabel) -> Self {
        Self::new(PostStartup, NeverDeactivateSchedule, update_schedule)
    }
}

impl Default for StochasticTexturingPlugin {
    fn default() -> Self {
        Self::always_on(Update)
    }
}

impl Plugin for StochasticTexturingPlugin {
    fn build(&self, app: &mut App) {
        if self.deactivate_schedule == NeverDeactivateSchedule.intern() {
            app.init_schedule(NeverDeactivateSchedule);
        }

        app.init_resource::<StochasticTexturingRuntimeState>()
            .init_resource::<StochasticDebugSettings>()
            .init_resource::<StochasticTexturingDiagnostics>()
            .register_type::<StochasticBlendMode>()
            .register_type::<StochasticDebugSettings>()
            .register_type::<StochasticDebugView>()
            .register_type::<StochasticHeightMap>()
            .register_type::<StochasticNormalMapMode>()
            .register_type::<StochasticQuality>()
            .register_type::<StochasticRotationMode>()
            .register_type::<StochasticSamplingMode>()
            .register_type::<StochasticTexturing>()
            .register_type::<StochasticTexturingDiagnostics>()
            .register_type::<StochasticUvSpace>()
            .register_type::<TextureChannel>()
            .add_systems(self.activate_schedule, systems::activate_runtime)
            .add_systems(self.deactivate_schedule, systems::deactivate_runtime)
            .configure_sets(
                self.update_schedule,
                (
                    StochasticTexturingSystems::Prepare,
                    StochasticTexturingSystems::Diagnostics,
                    StochasticTexturingSystems::Debug,
                )
                    .chain(),
            )
            .add_systems(
                self.update_schedule,
                systems::publish_core_diagnostics
                    .in_set(StochasticTexturingSystems::Diagnostics)
                    .run_if(systems::runtime_is_active),
            );
    }
}

pub struct StochasticPbrPlugin {
    pub activate_schedule: Interned<dyn ScheduleLabel>,
    pub deactivate_schedule: Interned<dyn ScheduleLabel>,
    pub update_schedule: Interned<dyn ScheduleLabel>,
}

impl StochasticPbrPlugin {
    #[must_use]
    pub fn new(
        activate_schedule: impl ScheduleLabel,
        deactivate_schedule: impl ScheduleLabel,
        update_schedule: impl ScheduleLabel,
    ) -> Self {
        Self {
            activate_schedule: activate_schedule.intern(),
            deactivate_schedule: deactivate_schedule.intern(),
            update_schedule: update_schedule.intern(),
        }
    }

    #[must_use]
    pub fn always_on(update_schedule: impl ScheduleLabel) -> Self {
        Self::new(PostStartup, NeverDeactivateSchedule, update_schedule)
    }
}

impl Default for StochasticPbrPlugin {
    fn default() -> Self {
        Self::always_on(Update)
    }
}

impl Plugin for StochasticPbrPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<StochasticTexturingPlugin>() {
            app.add_plugins(StochasticTexturingPlugin {
                activate_schedule: self.activate_schedule,
                deactivate_schedule: self.deactivate_schedule,
                update_schedule: self.update_schedule,
            });
        }

        if self.deactivate_schedule == NeverDeactivateSchedule.intern() {
            app.init_schedule(NeverDeactivateSchedule);
        }

        pbr::plugin(app);

        app.init_resource::<StochasticPbrRuntimeState>()
            .add_systems(self.activate_schedule, systems::activate_pbr_runtime)
            .add_systems(
                self.deactivate_schedule,
                (systems::deactivate_pbr_runtime, pbr::reset_materials),
            )
            .configure_sets(
                self.update_schedule,
                (
                    StochasticPbrSystems::AdaptMaterials,
                    StochasticPbrSystems::SyncUniforms,
                    StochasticPbrSystems::Diagnostics,
                )
                    .chain(),
            );

        pbr::schedule_systems(app, self.update_schedule);
        app.add_systems(
            self.update_schedule,
            systems::publish_pbr_diagnostics
                .in_set(StochasticPbrSystems::Diagnostics)
                .run_if(systems::pbr_runtime_is_active),
        );
    }
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod config_tests;

#[cfg(test)]
#[path = "systems_tests.rs"]
mod systems_tests;
