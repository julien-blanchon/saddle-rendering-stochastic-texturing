use bevy::{
    asset::{Asset, AssetApp, AssetServer, Assets, Handle, load_internal_asset, uuid_handle},
    pbr::{ExtendedMaterial, MaterialExtension, MaterialPlugin, MeshMaterial3d, StandardMaterial},
    prelude::*,
    reflect::TypePath,
    render::render_resource::AsBindGroup,
    shader::{Shader, ShaderRef},
};

use crate::{
    StochasticHeightMap, StochasticPbrSystems, StochasticShaderUniform, StochasticTexturing,
};

pub type StochasticPbrMaterial =
    ExtendedMaterial<StandardMaterial, StochasticTexturingMaterialExtension>;

pub(crate) const STOCHASTIC_PBR_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("597efee2-cf96-42ad-92ef-db729f2ccdb7");

#[derive(Component, Debug, Clone)]
pub(crate) struct StochasticPbrMaterialBinding {
    pub source_material: Handle<StandardMaterial>,
    pub adapted_material: Handle<StochasticPbrMaterial>,
}

pub type StochasticPbrUniform = StochasticShaderUniform;

#[derive(Asset, AsBindGroup, Debug, Clone, Default, TypePath)]
pub struct StochasticTexturingMaterialExtension {
    #[uniform(100)]
    pub uniform: StochasticShaderUniform,
    #[texture(101)]
    #[sampler(102)]
    pub height_map: Option<Handle<Image>>,
}

impl StochasticTexturingMaterialExtension {
    #[must_use]
    pub fn from_surface(
        surface: &StochasticTexturing,
        height_map: Option<&StochasticHeightMap>,
    ) -> Self {
        Self {
            uniform: StochasticShaderUniform::from_surface(surface, height_map),
            height_map: height_map.map(|map| map.image.clone()),
        }
    }
}

impl MaterialExtension for StochasticTexturingMaterialExtension {
    fn fragment_shader() -> ShaderRef {
        STOCHASTIC_PBR_SHADER_HANDLE.into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        STOCHASTIC_PBR_SHADER_HANDLE.into()
    }
}

pub(crate) fn plugin(app: &mut App) {
    crate::shader_library::plugin(app);

    if app.world().contains_resource::<AssetServer>() {
        if !app.world().contains_resource::<Assets<Image>>() {
            app.init_asset::<Image>();
        }
        if !app.world().contains_resource::<Assets<Shader>>() {
            app.init_asset::<Shader>();
        }
        if !app.world().contains_resource::<Assets<StandardMaterial>>() {
            app.init_asset::<StandardMaterial>();
        }
        load_internal_asset!(
            app,
            STOCHASTIC_PBR_SHADER_HANDLE,
            "shaders/stochastic_pbr.wgsl",
            Shader::from_wgsl
        );
        app.add_plugins(MaterialPlugin::<StochasticPbrMaterial>::default());
    } else {
        if !app.world().contains_resource::<Assets<Image>>() {
            app.insert_resource(Assets::<Image>::default());
        }
        if !app.world().contains_resource::<Assets<StandardMaterial>>() {
            app.insert_resource(Assets::<StandardMaterial>::default());
        }
        if !app
            .world()
            .contains_resource::<Assets<StochasticPbrMaterial>>()
        {
            app.insert_resource(Assets::<StochasticPbrMaterial>::default());
        }
    }
}

pub(crate) fn adapt_standard_materials(
    mut commands: Commands,
    materials: Res<Assets<StandardMaterial>>,
    mut stochastic_materials: ResMut<Assets<StochasticPbrMaterial>>,
    surfaces: Query<
        (
            Entity,
            &StochasticTexturing,
            Option<&StochasticHeightMap>,
            &MeshMaterial3d<StandardMaterial>,
        ),
        Without<StochasticPbrMaterialBinding>,
    >,
) {
    for (entity, surface, height_map, material_slot) in &surfaces {
        let Some(source_material) = materials.get(&material_slot.0).cloned() else {
            continue;
        };

        let adapted_material = stochastic_materials.add(StochasticPbrMaterial {
            base: source_material,
            extension: StochasticTexturingMaterialExtension::from_surface(surface, height_map),
        });

        commands
            .entity(entity)
            .remove::<MeshMaterial3d<StandardMaterial>>();
        commands.entity(entity).insert((
            MeshMaterial3d(adapted_material.clone()),
            StochasticPbrMaterialBinding {
                source_material: material_slot.0.clone(),
                adapted_material,
            },
        ));
    }
}

pub(crate) fn sync_adapted_materials(
    source_materials: Res<Assets<StandardMaterial>>,
    mut stochastic_materials: ResMut<Assets<StochasticPbrMaterial>>,
    surfaces: Query<
        (
            &StochasticTexturing,
            Option<&StochasticHeightMap>,
            &MeshMaterial3d<StochasticPbrMaterial>,
            &StochasticPbrMaterialBinding,
        ),
        With<StochasticTexturing>,
    >,
) {
    for (surface, height_map, slot, binding) in &surfaces {
        if slot.0 != binding.adapted_material {
            continue;
        }

        let Some(stochastic_material) = stochastic_materials.get_mut(&binding.adapted_material)
        else {
            continue;
        };

        if let Some(source_material) = source_materials.get(&binding.source_material) {
            stochastic_material.base = source_material.clone();
        }

        stochastic_material.extension =
            StochasticTexturingMaterialExtension::from_surface(surface, height_map);
    }
}

pub(crate) fn restore_removed_surfaces(
    mut commands: Commands,
    surfaces: Query<
        (
            Entity,
            &StochasticPbrMaterialBinding,
            &MeshMaterial3d<StochasticPbrMaterial>,
        ),
        Without<StochasticTexturing>,
    >,
) {
    for (entity, binding, _) in &surfaces {
        commands
            .entity(entity)
            .remove::<MeshMaterial3d<StochasticPbrMaterial>>()
            .insert(MeshMaterial3d(binding.source_material.clone()))
            .remove::<StochasticPbrMaterialBinding>();
    }
}

pub(crate) fn reset_materials(
    mut commands: Commands,
    surfaces: Query<(
        Entity,
        &StochasticPbrMaterialBinding,
        &MeshMaterial3d<StochasticPbrMaterial>,
    )>,
) {
    for (entity, binding, _) in &surfaces {
        commands
            .entity(entity)
            .remove::<MeshMaterial3d<StochasticPbrMaterial>>()
            .insert(MeshMaterial3d(binding.source_material.clone()))
            .remove::<StochasticPbrMaterialBinding>();
    }
}

pub(crate) fn material_binding_count(
    surfaces: Query<
        (),
        (
            With<StochasticPbrMaterialBinding>,
            With<StochasticTexturing>,
        ),
    >,
    mut diagnostics: ResMut<crate::StochasticTexturingDiagnostics>,
) {
    diagnostics.active_pbr_surfaces = surfaces.iter().count();
}

pub(crate) fn schedule_systems(
    app: &mut App,
    update_schedule: bevy::ecs::intern::Interned<dyn bevy::ecs::schedule::ScheduleLabel>,
) {
    app.add_systems(
        update_schedule,
        (
            restore_removed_surfaces.in_set(StochasticPbrSystems::AdaptMaterials),
            adapt_standard_materials.in_set(StochasticPbrSystems::AdaptMaterials),
            sync_adapted_materials.in_set(StochasticPbrSystems::SyncUniforms),
            material_binding_count.in_set(StochasticPbrSystems::Diagnostics),
        ),
    );
}
