use bevy::{pbr::StandardMaterial, prelude::*};

use crate::{
    StochasticHeightMap, StochasticPbrMaterial, StochasticPbrPlugin, StochasticTexturing,
    StochasticTexturingDiagnostics,
};

#[test]
fn diagnostics_count_public_authoring_and_pbr_surfaces() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StochasticPbrPlugin::default());

    let height = Handle::<Image>::default();
    let material = app
        .world_mut()
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial::default());

    app.world_mut().spawn((
        Name::new("Hex Wall"),
        StochasticTexturing::default(),
        MeshMaterial3d::<StandardMaterial>(material),
    ));
    app.world_mut().spawn((
        Name::new("Height Surface"),
        StochasticTexturing::default(),
        StochasticHeightMap::new(height),
    ));

    app.update();
    app.update();

    let diagnostics = app.world().resource::<StochasticTexturingDiagnostics>();
    assert_eq!(diagnostics.active_surfaces, 2);
    assert_eq!(diagnostics.active_height_maps, 1);
    assert_eq!(diagnostics.active_pbr_surfaces, 1);
    assert!(diagnostics.estimated_base_sample_count >= 6);
}

#[test]
fn removing_stochastic_texturing_restores_standard_material_slot() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StochasticPbrPlugin::default());

    let material = app
        .world_mut()
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial::default());

    let entity = app
        .world_mut()
        .spawn((
            Name::new("Restored Wall"),
            StochasticTexturing::default(),
            MeshMaterial3d::<StandardMaterial>(material.clone()),
        ))
        .id();

    app.update();
    app.update();

    assert!(
        app.world()
            .entity(entity)
            .contains::<MeshMaterial3d<StochasticPbrMaterial>>()
    );

    app.world_mut()
        .entity_mut(entity)
        .remove::<StochasticTexturing>();
    app.update();
    app.update();

    let entity_ref = app.world().entity(entity);
    assert!(entity_ref.contains::<MeshMaterial3d<StandardMaterial>>());
    assert!(!entity_ref.contains::<MeshMaterial3d<StochasticPbrMaterial>>());
}
