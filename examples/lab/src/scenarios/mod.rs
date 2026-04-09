use bevy::{math::Affine2, prelude::*};
use saddle_bevy_e2e::{
    action::Action,
    actions::{assertions, inspect},
    scenario::Scenario,
};
use saddle_rendering_stochastic_texturing_example_common::{
    DemoAssets, ExampleOverlay, ExampleSceneReady, ExampleSceneText,
};
use stochastic_texturing::{
    StochasticHeightMap, StochasticQuality, StochasticSamplingMode, StochasticTexturing,
    StochasticTexturingDiagnostics,
};

pub fn list_scenarios() -> Vec<&'static str> {
    vec![
        "smoke_launch",
        "stochastic_overview",
        "stochastic_before_after_closeup",
        "stochastic_normal_maps",
        "stochastic_height_blend",
        "stochastic_triplanar_slope_transition",
        "stochastic_quality_ladder",
        "stochastic_stress",
    ]
}

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "smoke_launch" => Some(smoke_launch()),
        "stochastic_overview" => Some(stochastic_overview()),
        "stochastic_before_after_closeup" => Some(stochastic_before_after_closeup()),
        "stochastic_normal_maps" => Some(stochastic_normal_maps()),
        "stochastic_height_blend" => Some(stochastic_height_blend()),
        "stochastic_triplanar_slope_transition" => Some(stochastic_triplanar_slope_transition()),
        "stochastic_quality_ladder" => Some(stochastic_quality_ladder()),
        "stochastic_stress" => Some(stochastic_stress()),
        _ => None,
    }
}

fn smoke_launch() -> Scenario {
    Scenario::builder("smoke_launch")
        .description("Boot the lab, verify the diagnostics surface exists, and capture the opening composition.")
        .then(wait_for_scene_ready())
        .then(Action::WaitFrames(6))
        .then(assertions::resource_exists::<StochasticTexturingDiagnostics>(
            "stochastic diagnostics resource exists",
        ))
        .then(assertions::resource_satisfies::<ExampleSceneText>(
            "scene text contains the lab title",
            |text| text.title.contains("Stochastic Texturing Lab"),
        ))
        .then(inspect::log_resource::<StochasticTexturingDiagnostics>(
            "stochastic smoke diagnostics",
        ))
        .then(Action::Screenshot("smoke_launch".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("smoke_launch"))
        .build()
}

fn stochastic_overview() -> Scenario {
    Scenario::builder("stochastic_overview")
        .description(
            "Capture a cleaner hero overview of the integrated lab with diagnostics hidden.",
        )
        .then(wait_for_scene_ready())
        .then(Action::WaitFrames(6))
        .then(set_overlay_visible(false))
        .then(set_camera_pose(
            Vec3::new(0.2, 5.0, 14.2),
            Vec3::new(0.0, 1.8, 1.0),
        ))
        .then(Action::WaitFrames(4))
        .then(assertions::resource_satisfies::<
            StochasticTexturingDiagnostics,
        >("the lab has authored surfaces", |diagnostics| {
            diagnostics.active_surfaces >= 12
        }))
        .then(Action::Screenshot("stochastic_overview".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("stochastic_overview"))
        .build()
}

fn stochastic_before_after_closeup() -> Scenario {
    Scenario::builder("stochastic_before_after_closeup")
        .description("Frame the basic comparison cluster from a shallow angle so the before-and-after repetition breakup reads clearly.")
        .then(wait_for_scene_ready())
        .then(Action::WaitFrames(6))
        .then(show_only_named_clusters(&["Basic"]))
        .then(Action::WaitFrames(2))
        .then(set_overlay_visible(false))
        .then(set_camera_pose(
            Vec3::new(-11.8, 4.3, 1.1),
            Vec3::new(-3.8, 3.35, -3.35),
        ))
        .then(Action::WaitFrames(4))
        .then(Action::Screenshot("stochastic_before_after_establishing".into()))
        .then(set_camera_pose(
            Vec3::new(-9.2, 4.1, 0.1),
            Vec3::new(-4.7, 3.4, -3.35),
        ))
        .then(Action::WaitFrames(4))
        .then(Action::Screenshot("stochastic_before_after_closeup".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("stochastic_before_after_closeup"))
        .build()
}

fn stochastic_normal_maps() -> Scenario {
    Scenario::builder("stochastic_normal_maps")
        .description("Push a grazing key light across the normal-map comparison cluster and capture a close read of tangent detail continuity.")
        .then(wait_for_scene_ready())
        .then(Action::WaitFrames(6))
        .then(show_only_named_clusters(&["Normals"]))
        .then(Action::WaitFrames(2))
        .then(set_overlay_visible(false))
        .then(set_camera_pose(
            Vec3::new(4.7, 3.7, 1.9),
            Vec3::new(5.0, 3.35, -3.4),
        ))
        .then(configure_directional_light(
            "Sun",
            Quat::from_euler(EulerRot::XYZ, -0.38, -0.1, 0.0),
            16_000.0,
            false,
            Color::srgb(1.0, 0.96, 0.92),
        ))
        .then(configure_point_light(
            "Warm Key",
            Vec3::new(6.4, 4.6, 2.6),
            12_000.0,
            Color::srgb(1.0, 0.9, 0.82),
        ))
        .then(Action::WaitFrames(6))
        .then(assertions::resource_satisfies::<StochasticTexturingDiagnostics>(
            "the lab has high-quality surfaces",
            |diagnostics| diagnostics.high_quality_surfaces >= 2,
        ))
        .then(Action::Screenshot("stochastic_normal_maps".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("stochastic_normal_maps"))
        .build()
}

fn stochastic_height_blend() -> Scenario {
    Scenario::builder("stochastic_height_blend")
        .description("Capture both a wider and tighter view of the height-blend cluster so the seam quality is readable.")
        .then(wait_for_scene_ready())
        .then(Action::WaitFrames(6))
        .then(show_only_named_clusters(&["Blend"]))
        .then(Action::WaitFrames(2))
        .then(set_overlay_visible(false))
        .then(set_camera_pose(
            Vec3::new(-7.2, 4.2, 10.3),
            Vec3::new(-6.1, 3.45, 5.55),
        ))
        .then(Action::WaitFrames(4))
        .then(assertions::resource_satisfies::<StochasticTexturingDiagnostics>(
            "the lab has height maps and histogram-preserving surfaces",
            |diagnostics| {
                diagnostics.active_height_maps >= 4
                    && diagnostics.histogram_preserving_surfaces >= 1
            },
        ))
        .then(Action::Screenshot("stochastic_height_blend_wide".into()))
        .then(set_camera_pose(
            Vec3::new(-6.4, 4.05, 9.6),
            Vec3::new(-6.0, 3.45, 5.55),
        ))
        .then(Action::WaitFrames(4))
        .then(Action::Screenshot("stochastic_height_blend_macro".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("stochastic_height_blend"))
        .build()
}

fn stochastic_triplanar_slope_transition() -> Scenario {
    Scenario::builder("stochastic_triplanar_slope_transition")
        .description("Frame the terrain-oriented cluster diagonally so top, slope, and transition planes are visible in one shot.")
        .then(wait_for_scene_ready())
        .then(Action::WaitFrames(6))
        .then(show_only_named_clusters(&["Terrain"]))
        .then(Action::WaitFrames(2))
        .then(set_overlay_visible(false))
        .then(set_camera_pose(
            Vec3::new(8.6, 2.5, 8.8),
            Vec3::new(4.9, 0.95, 6.25),
        ))
        .then(Action::WaitFrames(4))
        .then(assertions::resource_satisfies::<StochasticTexturingDiagnostics>(
            "the lab has world-triplanar surfaces",
            |diagnostics| diagnostics.world_triplanar_surfaces >= 3,
        ))
        .then(Action::Screenshot("stochastic_triplanar_slope_transition".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("stochastic_triplanar_slope_transition"))
        .build()
}

fn stochastic_quality_ladder() -> Scenario {
    Scenario::builder("stochastic_quality_ladder")
        .description("Inject a dedicated fast-balanced-high-quality comparison row and capture it with the current diagnostics totals.")
        .then(wait_for_scene_ready())
        .then(Action::WaitFrames(6))
        .then(spawn_quality_ladder_surfaces())
        .then(set_overlay_visible(false))
        .then(set_camera_pose(
            Vec3::new(0.0, 2.0, 16.2),
            Vec3::new(0.0, 1.75, 11.4),
        ))
        .then(Action::WaitFrames(4))
        .then(assertions::resource_satisfies::<StochasticTexturingDiagnostics>(
            "quality ladder increased the surface count and estimated sample budget",
            |diagnostics| diagnostics.active_surfaces >= 16 && diagnostics.estimated_base_sample_count >= 47,
        ))
        .then(inspect::log_resource::<StochasticTexturingDiagnostics>(
            "stochastic quality ladder diagnostics",
        ))
        .then(Action::Screenshot("stochastic_quality_ladder".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("stochastic_quality_ladder"))
        .build()
}

fn stochastic_stress() -> Scenario {
    Scenario::builder("stochastic_stress")
        .description("Inject extra authored surfaces with real PBR materials into the lab and verify diagnostics grow accordingly.")
        .then(wait_for_scene_ready())
        .then(Action::WaitFrames(6))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let material = world
                .resource_mut::<Assets<StandardMaterial>>()
                .add(StandardMaterial::default());
            for index in 0..8 {
                world.spawn((
                    StochasticTexturing {
                        quality: if index % 2 == 0 {
                            StochasticQuality::HighQuality
                        } else {
                            StochasticQuality::Balanced
                        },
                        seed: 400 + index as u32,
                        ..default()
                    },
                    MeshMaterial3d::<StandardMaterial>(material.clone()),
                    Name::new(format!("Scenario Stress Surface {index}")),
                ));
            }
        })))
        .then(Action::WaitFrames(2))
        .then(assertions::resource_satisfies::<StochasticTexturingDiagnostics>(
            "stress injection increased surface and PBR counts",
            |diagnostics| diagnostics.active_surfaces >= 21 && diagnostics.active_pbr_surfaces >= 21,
        ))
        .then(inspect::log_resource::<StochasticTexturingDiagnostics>(
            "stochastic stress diagnostics",
        ))
        .then(Action::Screenshot("stochastic_stress".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("stochastic_stress"))
        .build()
}

fn wait_for_scene_ready() -> Action {
    Action::WaitUntil {
        label: "example scene ready".into(),
        condition: Box::new(|world: &World| world.resource::<ExampleSceneReady>().0),
        max_frames: 360,
    }
}

fn set_overlay_visible(visible: bool) -> Action {
    Action::Custom(Box::new(move |world: &mut World| {
        let mut overlays = world.query_filtered::<&mut Node, With<ExampleOverlay>>();
        if let Some(mut node) = overlays.iter_mut(world).next() {
            node.display = if visible {
                Display::Flex
            } else {
                Display::None
            };
        }
    }))
}

fn show_only_named_clusters(prefixes: &'static [&'static str]) -> Action {
    Action::Custom(Box::new(move |world: &mut World| {
        let mut entities = world.query::<(&Name, &mut Visibility)>();
        for (name, mut visibility) in entities.iter_mut(world) {
            let surface_cluster = ["Basic ", "Normals ", "Blend ", "Terrain "]
                .into_iter()
                .any(|prefix| name.as_str().starts_with(prefix));
            if !surface_cluster {
                continue;
            }

            let should_show = prefixes
                .iter()
                .any(|prefix| name.as_str().starts_with(prefix));
            *visibility = if should_show {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }))
}

fn set_camera_pose(translation: Vec3, target: Vec3) -> Action {
    Action::Custom(Box::new(move |world: &mut World| {
        let mut cameras = world.query_filtered::<&mut Transform, With<Camera3d>>();
        if let Some(mut transform) = cameras.iter_mut(world).next() {
            *transform = Transform::from_translation(translation).looking_at(target, Vec3::Y);
        }
    }))
}

fn configure_point_light(
    light_name: &'static str,
    translation: Vec3,
    intensity: f32,
    color: Color,
) -> Action {
    Action::Custom(Box::new(move |world: &mut World| {
        let mut lights = world.query::<(&Name, &mut Transform, &mut PointLight)>();
        for (name, mut transform, mut light) in lights.iter_mut(world) {
            if name.as_str() == light_name {
                *transform = Transform::from_translation(translation);
                light.intensity = intensity;
                light.color = color;
            }
        }
    }))
}

fn configure_directional_light(
    light_name: &'static str,
    rotation: Quat,
    illuminance: f32,
    shadows_enabled: bool,
    color: Color,
) -> Action {
    Action::Custom(Box::new(move |world: &mut World| {
        let mut lights = world.query::<(&Name, &mut Transform, &mut DirectionalLight)>();
        for (name, mut transform, mut light) in lights.iter_mut(world) {
            if name.as_str() == light_name {
                *transform = Transform::from_rotation(rotation);
                light.illuminance = illuminance;
                light.shadows_enabled = shadows_enabled;
                light.color = color;
            }
        }
    }))
}

fn spawn_quality_ladder_surfaces() -> Action {
    Action::Custom(Box::new(|world: &mut World| {
        let assets = world.resource::<DemoAssets>().clone();
        let mesh = {
            let mut meshes = world.resource_mut::<Assets<Mesh>>();
            meshes.add(Cuboid::new(2.25, 2.25, 0.18))
        };
        let material = {
            let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
            materials.add(StandardMaterial {
                base_color_texture: Some(assets.albedo_a.clone()),
                normal_map_texture: Some(assets.normal_a.clone()),
                uv_transform: Affine2::from_scale(Vec2::splat(5.0)),
                perceptual_roughness: 0.86,
                reflectance: 0.28,
                ..default()
            })
        };

        for (index, (label, quality)) in [
            ("Fast", StochasticQuality::Fast),
            ("Balanced", StochasticQuality::Balanced),
            ("HighQuality", StochasticQuality::HighQuality),
        ]
        .into_iter()
        .enumerate()
        {
            world.spawn((
                Name::new(format!("Quality Ladder {label}")),
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(Vec3::new(index as f32 * 3.2 - 3.2, 1.7, 11.4)),
                StochasticTexturing {
                    quality,
                    sampling_mode: StochasticSamplingMode::Hex3,
                    seed: 600 + index as u32,
                    ..default()
                },
                StochasticHeightMap::new(assets.height_a.clone()),
            ));
        }
    }))
}
