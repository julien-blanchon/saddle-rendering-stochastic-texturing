use bevy::app::AppExit;
use bevy::asset::AssetPlugin;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};
use bevy::light::GlobalAmbientLight;
use bevy::math::Affine2;
use bevy::prelude::*;
use bevy::text::TextColor;
use bevy_flair::prelude::InlineStyle;
use saddle_pane::prelude::*;
use stochastic_texturing::{
    StochasticBlendMode, StochasticDebugSettings, StochasticDebugView, StochasticHeightMap,
    StochasticNormalMapMode, StochasticPbrPlugin, StochasticPbrSystems, StochasticQuality,
    StochasticRotationMode, StochasticSamplingMode, StochasticTexturing,
    StochasticTexturingDiagnostics, StochasticTexturingPlugin, StochasticTexturingSystems,
    StochasticUvSpace,
};

const PANE_THEME_VARS: &[(&str, &str)] = &[
    ("--pane-elevation-1", "#25221f"),
    ("--pane-elevation-2", "#1d1b19"),
    ("--pane-elevation-3", "rgba(233, 225, 212, 0.10)"),
    ("--pane-border", "#54483d"),
    ("--pane-border-focus", "#bf7d4e"),
    ("--pane-border-subtle", "#3a322c"),
    ("--pane-text-primary", "#e8e1d4"),
    ("--pane-text-secondary", "#b7ac9f"),
    ("--pane-text-muted", "#8c8479"),
    ("--pane-text-on-accent", "#ffffff"),
    ("--pane-text-brighter", "#f4ecde"),
    ("--pane-text-monitor", "#d2c5b6"),
    ("--pane-text-log", "#c4b9ab"),
    ("--pane-accent", "#bf7d4e"),
    ("--pane-accent-hover", "#d89160"),
    ("--pane-accent-active", "#a9683d"),
    ("--pane-accent-subtle", "rgba(191, 125, 78, 0.16)"),
    ("--pane-accent-fill", "rgba(191, 125, 78, 0.62)"),
    ("--pane-accent-fill-hover", "rgba(216, 145, 96, 0.72)"),
    ("--pane-accent-fill-active", "rgba(216, 145, 96, 0.82)"),
    ("--pane-accent-checked", "rgba(191, 125, 78, 0.24)"),
    ("--pane-accent-checked-hover", "rgba(191, 125, 78, 0.34)"),
    ("--pane-accent-indicator", "rgba(216, 145, 96, 0.82)"),
    ("--pane-accent-knob", "#efb07f"),
    ("--pane-widget-bg", "rgba(233, 225, 212, 0.10)"),
    ("--pane-widget-hover", "rgba(233, 225, 212, 0.15)"),
    ("--pane-widget-focus", "rgba(233, 225, 212, 0.20)"),
    ("--pane-widget-active", "rgba(233, 225, 212, 0.25)"),
    ("--pane-widget-bg-muted", "rgba(233, 225, 212, 0.06)"),
    ("--pane-tab-hover-bg", "rgba(233, 225, 212, 0.06)"),
    ("--pane-hover-bg", "rgba(255, 255, 255, 0.03)"),
    ("--pane-active-bg", "rgba(255, 255, 255, 0.05)"),
    ("--pane-popup-bg", "#181614"),
    ("--pane-bg-dark", "rgba(0, 0, 0, 0.24)"),
];

#[derive(Resource)]
struct AutoExitAfter(Timer);

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExampleSceneMode(pub ExampleMode);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExampleMode {
    Basic,
    NormalMaps,
    HeightBlend,
    TerrainBridge,
    Stress,
    Lab,
}

#[derive(Resource, Clone)]
pub struct ExampleSceneText {
    pub title: String,
    pub subtitle: String,
}

#[derive(Component)]
pub struct ExampleOverlay;

#[derive(Component, Clone)]
struct ExampleAuthoredStochastic(pub StochasticTexturing);

#[derive(Resource, Clone)]
pub struct DemoAssets {
    pub albedo_a: Handle<Image>,
    pub normal_a: Handle<Image>,
    pub height_a: Handle<Image>,
    pub albedo_b: Handle<Image>,
    pub normal_b: Handle<Image>,
    pub height_b: Handle<Image>,
    pub albedo_c: Handle<Image>,
    pub normal_c: Handle<Image>,
    pub height_c: Handle<Image>,
}

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ExampleSceneReady(pub bool);

#[derive(Resource, Default)]
struct ExampleSceneSpawnState {
    spawned: bool,
}

#[derive(Resource, Debug, Clone, PartialEq, Pane)]
#[pane(title = "Stochastic Texturing", position = "top-right")]
pub struct ExampleStochasticPane {
    #[pane(toggle)]
    pub override_authored: bool,
    #[pane(toggle)]
    pub enabled: bool,
    #[pane(select(options = ["Off", "Checker 2", "Hex 3", "Histogram", "Bombing", "Wang"]))]
    pub sampling_mode: usize,
    #[pane(select(options = ["Linear", "Height", "Histogram"]))]
    pub blend_mode: usize,
    #[pane(select(options = ["Fast", "Balanced", "High Quality"]))]
    pub quality: usize,
    #[pane(select(options = ["UV0", "UV1", "World Triplanar"]))]
    pub uv_space: usize,
    #[pane(select(options = ["None", "Rotate 60", "Rotate Mirror"]))]
    pub rotation_mode: usize,
    #[pane(select(options = ["Disabled", "Rotate Tangent", "Derivative"]))]
    pub normal_map_mode: usize,
    #[pane(select(options = ["Off", "Blend Weights", "Cell IDs", "Transforms", "Height Mask", "Sample Count"]))]
    pub debug_view: usize,
    #[pane(toggle)]
    pub debug_enabled: bool,
    #[pane(toggle)]
    pub show_overlay: bool,
    #[pane(slider, min = 0.1, max = 6.0, step = 0.05)]
    pub base_scale: f32,
    #[pane(slider, min = 0.0, max = 2.0, step = 0.01)]
    pub variation_strength: f32,
    #[pane(slider, min = 0.0, max = 1.0, step = 0.01)]
    pub blend_softness: f32,
    #[pane(slider, min = 0.0, max = 1.5, step = 0.01)]
    pub height_blend_strength: f32,
    #[pane(slider, min = 0.0, max = 0.35, step = 0.01)]
    pub sample_cull_threshold: f32,
    #[pane(slider, min = -2.0, max = 2.0, step = 0.05)]
    pub mip_bias: f32,
    #[pane(slider, min = 0, max = 255, step = 1)]
    pub seed: u32,
    #[pane(monitor, label = "Active Surfaces")]
    pub active_surfaces: f32,
    #[pane(monitor, label = "PBR Targets")]
    pub active_pbr_surfaces: f32,
    #[pane(monitor, label = "Est. Samples")]
    pub estimated_base_samples: f32,
}

impl Default for ExampleStochasticPane {
    fn default() -> Self {
        Self {
            override_authored: false,
            enabled: true,
            sampling_mode: 2,
            blend_mode: 1,
            quality: 1,
            uv_space: 0,
            rotation_mode: 2,
            normal_map_mode: 1,
            debug_view: 0,
            debug_enabled: false,
            show_overlay: true,
            base_scale: 1.0,
            variation_strength: 1.0,
            blend_softness: 0.35,
            height_blend_strength: 0.65,
            sample_cull_threshold: 0.08,
            mip_bias: 0.0,
            seed: 13,
            active_surfaces: 0.0,
            active_pbr_surfaces: 0.0,
            estimated_base_samples: 0.0,
        }
    }
}

pub fn install_common_plugins(app: &mut App, window_title: &str) {
    app.insert_resource(ClearColor(Color::srgb(0.06, 0.07, 0.09)));
    app.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.66, 0.71, 0.78),
        brightness: 160.0,
        ..default()
    });
    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                file_path: "../assets".into(),
                ..default()
            })
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: window_title.into(),
                    resolution: (1440, 900).into(),
                    ..default()
                }),
                ..default()
            }),
    );
    app.add_plugins((
        StochasticTexturingPlugin::default(),
        StochasticPbrPlugin::default(),
    ));
}

pub fn install_pane(app: &mut App) {
    if !app.is_plugin_added::<PanePlugin>() {
        app.add_plugins((
            bevy_flair::FlairPlugin,
            bevy_input_focus::InputDispatchPlugin,
            bevy_ui_widgets::UiWidgetsPlugins,
            bevy_input_focus::tab_navigation::TabNavigationPlugin,
            PanePlugin,
        ));
    }

    app.register_pane::<ExampleStochasticPane>()
        .add_systems(PreUpdate, prime_pane_theme_vars);
}

pub fn install_auto_exit(app: &mut App, env_var: &str) {
    let timer = std::env::var(env_var)
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
        .map(|seconds| AutoExitAfter(Timer::from_seconds(seconds.max(0.1), TimerMode::Once)));

    if let Some(timer) = timer {
        app.insert_resource(timer);
        app.add_systems(Update, auto_exit_after);
    }
}

pub fn add_example_systems(app: &mut App) {
    app.init_resource::<ExampleSceneReady>();
    app.init_resource::<ExampleSceneSpawnState>();
    app.add_systems(Update, finish_scene_setup);
    app.add_systems(
        Update,
        (
            apply_pane_overrides_to_surfaces,
            sync_debug_settings_from_pane,
        )
            .chain()
            .after(finish_scene_setup)
            .before(StochasticTexturingSystems::Diagnostics)
            .before(StochasticPbrSystems::Diagnostics),
    );
    app.add_systems(Update, sync_overlay_visibility.after(finish_scene_setup));
    app.add_systems(
        Update,
        (update_overlay, reflect_pane_monitors)
            .chain()
            .after(StochasticTexturingSystems::Diagnostics)
            .after(StochasticPbrSystems::Diagnostics),
    );
}

pub fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mode: Res<ExampleSceneMode>,
    text: Res<ExampleSceneText>,
) {
    let assets = DemoAssets::load(&asset_server);
    commands.insert_resource(assets);

    let (camera_position, camera_target) = camera_preset(mode.0);
    spawn_camera(&mut commands, camera_position, camera_target);
    spawn_lighting(&mut commands);
    spawn_overlay(&mut commands, &text.title, &text.subtitle);
}

pub fn update_overlay(
    diagnostics: Res<StochasticTexturingDiagnostics>,
    text: Res<ExampleSceneText>,
    mode: Res<ExampleSceneMode>,
    ready: Res<ExampleSceneReady>,
    pane: Option<Res<ExampleStochasticPane>>,
    mut overlays: Query<&mut Text, With<ExampleOverlay>>,
) {
    let Ok(mut overlay) = overlays.single_mut() else {
        return;
    };

    let pane_summary = pane.as_ref().map_or_else(
        || "Pane: not installed in this app".to_string(),
        |pane| {
            format!(
                "Pane: live controls active ({})",
                if pane.override_authored {
                    "overriding authored presets"
                } else {
                    "authored presets preserved"
                }
            )
        },
    );

    if !ready.0 {
        *overlay = Text::new(format!(
            "{}\n{}\n\nMode: {:?}\n{}\nScene ready: loading bundled CC0 showcase textures\nTexture set A: Poly Haven rock_face_03\nTexture set B: Poly Haven coast_sand_rocks_02",
            text.title, text.subtitle, mode.0, pane_summary,
        ));
        return;
    }

    *overlay = Text::new(format!(
        "{}\n{}\n\nMode: {:?}\n{}\nScene ready: yes\nSurfaces: {}\nPBR bridge targets: {}\nHeight maps: {}\nWorld triplanar: {}\nHistogram preserving: {}\nHigh quality: {}\nEstimated base samples: {}\n\nShowcase textures: Poly Haven rock_face_03 + coast_sand_rocks_02 (CC0).\nBackend: practical stochastic hex-style sampling with a distinct texture-bombing path.\nReference-style coverage: basic, normals, blending, terrain, stress, plus the new showcase gallery.\nOpen work: true histogram-preserving blending, Wang tiling, and stronger derivative normal reconstruction.",
        text.title,
        text.subtitle,
        mode.0,
        pane_summary,
        diagnostics.active_surfaces,
        diagnostics.active_pbr_surfaces,
        diagnostics.active_height_maps,
        diagnostics.world_triplanar_surfaces,
        diagnostics.histogram_preserving_surfaces,
        diagnostics.high_quality_surfaces,
        diagnostics.estimated_base_sample_count,
    ));
}

fn finish_scene_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    assets: Res<DemoAssets>,
    mode: Res<ExampleSceneMode>,
    mut ready: ResMut<ExampleSceneReady>,
    mut spawn_state: ResMut<ExampleSceneSpawnState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if spawn_state.spawned {
        ready.0 = true;
        return;
    }

    if !assets.is_loaded(&asset_server) {
        ready.0 = false;
        return;
    }

    spawn_ground(&mut commands, &mut meshes, &mut materials, &assets);
    spawn_mode_scene(&mut commands, &mut meshes, &mut materials, &assets, mode.0);
    spawn_state.spawned = true;
    ready.0 = true;
}

fn prime_pane_theme_vars(mut panes: Query<&mut InlineStyle, Added<PaneRoot>>) {
    for mut style in &mut panes {
        for &(key, value) in PANE_THEME_VARS {
            style.set(key, value.to_owned());
        }
    }
}

fn apply_pane_overrides_to_surfaces(
    pane: Option<Res<ExampleStochasticPane>>,
    mut surfaces: Query<(&ExampleAuthoredStochastic, &mut StochasticTexturing)>,
) {
    let Some(pane) = pane else {
        return;
    };

    for (authored, mut surface) in &mut surfaces {
        let desired = if pane.override_authored {
            surface_from_pane(&pane, &authored.0)
        } else {
            authored.0.clone()
        };

        if *surface != desired {
            *surface = desired;
        }
    }
}

fn sync_debug_settings_from_pane(
    pane: Option<Res<ExampleStochasticPane>>,
    mut debug_settings: ResMut<StochasticDebugSettings>,
) {
    let Some(pane) = pane else {
        return;
    };

    let desired = StochasticDebugSettings {
        enabled: pane.debug_enabled,
        view: debug_view_from_index(pane.debug_view),
        show_surface_bounds: false,
        freeze_seed: false,
        show_overlay: pane.show_overlay,
    };

    if *debug_settings != desired {
        *debug_settings = desired;
    }
}

fn sync_overlay_visibility(
    pane: Option<Res<ExampleStochasticPane>>,
    mut overlays: Query<&mut Visibility, With<ExampleOverlay>>,
) {
    let Some(pane) = pane else {
        return;
    };

    for mut visibility in &mut overlays {
        *visibility = if pane.show_overlay {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

fn reflect_pane_monitors(
    diagnostics: Res<StochasticTexturingDiagnostics>,
    pane: Option<ResMut<ExampleStochasticPane>>,
) {
    let Some(mut pane) = pane else {
        return;
    };

    pane.active_surfaces = diagnostics.active_surfaces as f32;
    pane.active_pbr_surfaces = diagnostics.active_pbr_surfaces as f32;
    pane.estimated_base_samples = diagnostics.estimated_base_sample_count as f32;
}

fn surface_from_pane(
    pane: &ExampleStochasticPane,
    authored: &StochasticTexturing,
) -> StochasticTexturing {
    let mut settings = authored.clone();
    settings.enabled = pane.enabled;
    settings.sampling_mode = sampling_mode_from_index(pane.sampling_mode);
    settings.blend_mode = blend_mode_from_index(pane.blend_mode);
    settings.quality = quality_from_index(pane.quality);
    settings.uv_space = uv_space_from_index(pane.uv_space);
    settings.rotation_mode = rotation_mode_from_index(pane.rotation_mode);
    settings.normal_map_mode = normal_map_mode_from_index(pane.normal_map_mode);
    settings.base_scale = Vec2::splat(pane.base_scale.max(0.1));
    settings.variation_strength = pane.variation_strength.max(0.0);
    settings.blend_softness = pane.blend_softness.clamp(0.0, 1.0);
    settings.height_blend_strength = pane.height_blend_strength.max(0.0);
    settings.sample_cull_threshold = pane.sample_cull_threshold.clamp(0.0, 0.35);
    settings.mip_bias = pane.mip_bias;
    settings.seed = pane.seed;
    settings
}

fn sampling_mode_from_index(index: usize) -> StochasticSamplingMode {
    match index {
        0 => StochasticSamplingMode::Off,
        1 => StochasticSamplingMode::Checker2,
        2 => StochasticSamplingMode::Hex3,
        3 => StochasticSamplingMode::HistogramPreserving,
        4 => StochasticSamplingMode::TextureBombing,
        _ => StochasticSamplingMode::Wang,
    }
}

fn blend_mode_from_index(index: usize) -> StochasticBlendMode {
    match index {
        0 => StochasticBlendMode::Linear,
        1 => StochasticBlendMode::HeightAware,
        _ => StochasticBlendMode::HistogramPreserving,
    }
}

fn quality_from_index(index: usize) -> StochasticQuality {
    match index {
        0 => StochasticQuality::Fast,
        1 => StochasticQuality::Balanced,
        _ => StochasticQuality::HighQuality,
    }
}

fn uv_space_from_index(index: usize) -> StochasticUvSpace {
    match index {
        0 => StochasticUvSpace::MeshUv0,
        1 => StochasticUvSpace::MeshUv1,
        _ => StochasticUvSpace::WorldTriplanar,
    }
}

fn rotation_mode_from_index(index: usize) -> StochasticRotationMode {
    match index {
        0 => StochasticRotationMode::None,
        1 => StochasticRotationMode::Rotate60,
        _ => StochasticRotationMode::RotateMirror,
    }
}

fn normal_map_mode_from_index(index: usize) -> StochasticNormalMapMode {
    match index {
        0 => StochasticNormalMapMode::Disabled,
        1 => StochasticNormalMapMode::RotateTangentSpace,
        _ => StochasticNormalMapMode::DerivativeReconstruction,
    }
}

fn debug_view_from_index(index: usize) -> StochasticDebugView {
    match index {
        0 => StochasticDebugView::Off,
        1 => StochasticDebugView::BlendWeights,
        2 => StochasticDebugView::CellIds,
        3 => StochasticDebugView::SampleTransforms,
        4 => StochasticDebugView::HeightMask,
        _ => StochasticDebugView::SampleCount,
    }
}

fn camera_preset(mode: ExampleMode) -> (Vec3, Vec3) {
    match mode {
        ExampleMode::Basic => (Vec3::new(-3.4, 2.6, 10.8), Vec3::new(1.1, 1.65, 0.05)),
        ExampleMode::NormalMaps => (Vec3::new(-1.3, 1.55, 5.4), Vec3::new(0.9, 1.75, 0.0)),
        ExampleMode::HeightBlend => (Vec3::new(0.1, 1.9, 5.3), Vec3::new(0.0, 1.75, 0.0)),
        ExampleMode::TerrainBridge => (Vec3::new(3.8, 2.6, 7.5), Vec3::new(-0.5, 1.05, 0.8)),
        ExampleMode::Stress => (Vec3::new(0.0, 5.8, 13.0), Vec3::new(0.0, 2.0, 0.0)),
        ExampleMode::Lab => (Vec3::new(-2.2, 4.1, 11.8), Vec3::new(-0.3, 1.55, 1.2)),
    }
}

pub fn spawn_camera(commands: &mut Commands, position: Vec3, target: Vec3) -> Entity {
    commands
        .spawn((
            Name::new("Main Camera"),
            Camera3d::default(),
            Tonemapping::AcesFitted,
            Projection::from(PerspectiveProjection {
                fov: std::f32::consts::FRAC_PI_4,
                ..default()
            }),
            Transform::from_translation(position).looking_at(target, Vec3::Y),
        ))
        .id()
}

pub fn spawn_lighting(commands: &mut Commands) {
    commands.spawn((
        Name::new("Sun"),
        DirectionalLight {
            color: Color::srgb(1.0, 0.94, 0.86),
            illuminance: 28_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.65, 0.55, 0.0)),
    ));

    commands.spawn((
        Name::new("Warm Key"),
        PointLight {
            color: Color::srgb(1.0, 0.86, 0.72),
            intensity: 7_500.0,
            range: 22.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(5.5, 5.0, 5.0),
    ));

    commands.spawn((
        Name::new("Cool Rim"),
        PointLight {
            color: Color::srgb(0.68, 0.83, 1.0),
            intensity: 3_000.0,
            range: 20.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-7.0, 3.5, -2.0),
    ));
}

pub fn spawn_ground(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    assets: &DemoAssets,
) {
    commands.spawn((
        Name::new("Stage Deck"),
        Mesh3d(meshes.add(Cuboid::new(18.0, 0.4, 8.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.12, 0.13, 0.16),
            perceptual_roughness: 0.95,
            metallic: 0.04,
            ..default()
        })),
        Transform::from_xyz(0.0, -0.82, 1.0),
    ));

    commands.spawn((
        Name::new("Ground"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(18.0, 18.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(assets.albedo_b.clone()),
            uv_transform: Affine2::from_scale(Vec2::splat(10.0)),
            perceptual_roughness: 0.98,
            normal_map_texture: Some(assets.normal_b.clone()),
            base_color: Color::WHITE,
            reflectance: 0.05,
            ..default()
        })),
        Transform::from_xyz(0.0, -0.6, 0.0),
    ));

    commands.spawn((
        Name::new("Backdrop"),
        Mesh3d(meshes.add(Cuboid::new(22.0, 10.0, 0.6))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.02, 0.03, 0.05),
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(0.0, 4.2, -5.8),
    ));

    commands.spawn((
        Name::new("Left Wing"),
        Mesh3d(meshes.add(Cuboid::new(0.6, 8.0, 8.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.03, 0.04, 0.06),
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(-10.4, 3.0, -2.0).with_rotation(Quat::from_rotation_y(0.35)),
    ));

    commands.spawn((
        Name::new("Right Wing"),
        Mesh3d(meshes.add(Cuboid::new(0.6, 8.0, 8.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.03, 0.04, 0.06),
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(10.4, 3.0, -2.0).with_rotation(Quat::from_rotation_y(-0.35)),
    ));
}

pub fn spawn_overlay(commands: &mut Commands, title: &str, subtitle: &str) -> Entity {
    commands
        .spawn((
            Name::new("Example Overlay"),
            ExampleOverlay,
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(18.0),
                bottom: Val::Px(18.0),
                width: Val::Px(300.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.04, 0.06, 0.6)),
            Text::new(format!("{title}\n{subtitle}")),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ))
        .id()
}

fn spawn_mode_scene(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    assets: &DemoAssets,
    mode: ExampleMode,
) {
    match mode {
        ExampleMode::Basic => spawn_basic(commands, meshes, materials, assets, Vec3::ZERO),
        ExampleMode::NormalMaps => {
            spawn_normal_maps(commands, meshes, materials, assets, Vec3::ZERO)
        }
        ExampleMode::HeightBlend => {
            spawn_height_blend(commands, meshes, materials, assets, Vec3::ZERO)
        }
        ExampleMode::TerrainBridge => {
            spawn_terrain_bridge(commands, meshes, materials, assets, Vec3::ZERO)
        }
        ExampleMode::Stress => spawn_stress(commands, meshes, materials, assets, Vec3::ZERO),
        ExampleMode::Lab => {
            spawn_basic(
                commands,
                meshes,
                materials,
                assets,
                Vec3::new(-6.0, 1.6, -3.5),
            );
            spawn_normal_maps(
                commands,
                meshes,
                materials,
                assets,
                Vec3::new(5.0, 1.6, -3.5),
            );
            spawn_height_blend(
                commands,
                meshes,
                materials,
                assets,
                Vec3::new(-6.0, 1.6, 5.5),
            );
            spawn_terrain_bridge(
                commands,
                meshes,
                materials,
                assets,
                Vec3::new(5.5, 1.0, 5.5),
            );
        }
    }
}

fn spawn_basic(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    assets: &DemoAssets,
    offset: Vec3,
) {
    spawn_surface(
        commands,
        meshes,
        materials,
        "Basic Off",
        offset + Vec3::new(-4.8, 1.8, 0.0),
        assets.albedo_b.clone(),
        Some(assets.normal_b.clone()),
        Affine2::from_scale(Vec2::splat(14.0)),
        StochasticTexturing {
            sampling_mode: StochasticSamplingMode::Off,
            blend_mode: StochasticBlendMode::Linear,
            ..default()
        },
        None,
    );
    spawn_surface(
        commands,
        meshes,
        materials,
        "Basic Checker",
        offset + Vec3::new(-1.6, 1.8, 0.0),
        assets.albedo_b.clone(),
        Some(assets.normal_b.clone()),
        Affine2::from_scale(Vec2::splat(14.0)),
        StochasticTexturing {
            sampling_mode: StochasticSamplingMode::Checker2,
            quality: StochasticQuality::Fast,
            rotation_mode: StochasticRotationMode::Rotate60,
            seed: 7,
            ..default()
        },
        None,
    );
    spawn_surface(
        commands,
        meshes,
        materials,
        "Basic Hex",
        offset + Vec3::new(1.6, 1.8, 0.0),
        assets.albedo_b.clone(),
        Some(assets.normal_b.clone()),
        Affine2::from_scale(Vec2::splat(14.0)),
        StochasticTexturing {
            sampling_mode: StochasticSamplingMode::Hex3,
            quality: StochasticQuality::Balanced,
            seed: 13,
            ..default()
        },
        None,
    );
    spawn_surface(
        commands,
        meshes,
        materials,
        "Basic Bombing",
        offset + Vec3::new(4.8, 1.8, 0.0),
        assets.albedo_b.clone(),
        Some(assets.normal_b.clone()),
        Affine2::from_scale(Vec2::splat(14.0)),
        StochasticTexturing {
            sampling_mode: StochasticSamplingMode::TextureBombing,
            quality: StochasticQuality::Balanced,
            rotation_mode: StochasticRotationMode::RotateMirror,
            seed: 17,
            ..default()
        },
        None,
    );
}

fn spawn_normal_maps(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    assets: &DemoAssets,
    offset: Vec3,
) {
    spawn_surface(
        commands,
        meshes,
        materials,
        "Normals Disabled",
        offset + Vec3::new(-3.2, 1.8, 0.0),
        assets.albedo_a.clone(),
        Some(assets.normal_a.clone()),
        Affine2::from_scale(Vec2::splat(10.0)),
        StochasticTexturing {
            normal_map_mode: StochasticNormalMapMode::Disabled,
            quality: StochasticQuality::Fast,
            seed: 2,
            ..default()
        },
        None,
    );
    spawn_surface(
        commands,
        meshes,
        materials,
        "Normals Rotated",
        offset + Vec3::new(0.0, 1.8, 0.0),
        assets.albedo_a.clone(),
        Some(assets.normal_a.clone()),
        Affine2::from_scale(Vec2::splat(10.0)),
        StochasticTexturing {
            normal_map_mode: StochasticNormalMapMode::RotateTangentSpace,
            seed: 5,
            ..default()
        },
        None,
    );
    spawn_surface(
        commands,
        meshes,
        materials,
        "Normals Derivative",
        offset + Vec3::new(3.2, 1.8, 0.0),
        assets.albedo_a.clone(),
        Some(assets.normal_a.clone()),
        Affine2::from_scale(Vec2::splat(10.0)),
        StochasticTexturing {
            normal_map_mode: StochasticNormalMapMode::DerivativeReconstruction,
            quality: StochasticQuality::HighQuality,
            sampling_mode: StochasticSamplingMode::TextureBombing,
            seed: 9,
            ..default()
        },
        None,
    );
}

fn spawn_height_blend(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    assets: &DemoAssets,
    offset: Vec3,
) {
    spawn_surface(
        commands,
        meshes,
        materials,
        "Blend Linear",
        offset + Vec3::new(-3.2, 1.8, 0.0),
        assets.albedo_a.clone(),
        Some(assets.normal_a.clone()),
        Affine2::from_scale(Vec2::splat(10.0)),
        StochasticTexturing {
            blend_mode: StochasticBlendMode::Linear,
            sampling_mode: StochasticSamplingMode::Checker2,
            seed: 11,
            ..default()
        },
        Some(StochasticHeightMap::new(assets.height_a.clone())),
    );
    spawn_surface(
        commands,
        meshes,
        materials,
        "Blend Height",
        offset + Vec3::new(0.0, 1.8, 0.0),
        assets.albedo_a.clone(),
        Some(assets.normal_a.clone()),
        Affine2::from_scale(Vec2::splat(10.0)),
        StochasticTexturing {
            blend_mode: StochasticBlendMode::HeightAware,
            seed: 15,
            ..default()
        },
        Some(StochasticHeightMap::new(assets.height_a.clone())),
    );
    spawn_surface(
        commands,
        meshes,
        materials,
        "Blend Histogram",
        offset + Vec3::new(3.2, 1.8, 0.0),
        assets.albedo_a.clone(),
        Some(assets.normal_a.clone()),
        Affine2::from_scale(Vec2::splat(10.0)),
        StochasticTexturing {
            blend_mode: StochasticBlendMode::HistogramPreserving,
            sampling_mode: StochasticSamplingMode::HistogramPreserving,
            quality: StochasticQuality::HighQuality,
            seed: 21,
            ..default()
        },
        Some(StochasticHeightMap::new(assets.height_a.clone())),
    );
}

fn spawn_terrain_bridge(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    assets: &DemoAssets,
    offset: Vec3,
) {
    spawn_surface(
        commands,
        meshes,
        materials,
        "Terrain Hex Slope",
        offset + Vec3::new(-2.8, 1.25, 0.0),
        assets.albedo_b.clone(),
        Some(assets.normal_b.clone()),
        Affine2::from_scale(Vec2::splat(12.0)),
        StochasticTexturing {
            uv_space: StochasticUvSpace::WorldTriplanar,
            sampling_mode: StochasticSamplingMode::Hex3,
            seed: 31,
            ..default()
        },
        Some(StochasticHeightMap::new(assets.height_b.clone())),
    );
    spawn_surface(
        commands,
        meshes,
        materials,
        "Terrain HQ Slope",
        offset + Vec3::new(1.2, 1.0, -0.6),
        assets.albedo_b.clone(),
        Some(assets.normal_b.clone()),
        Affine2::from_scale(Vec2::splat(12.0)),
        StochasticTexturing {
            uv_space: StochasticUvSpace::WorldTriplanar,
            sampling_mode: StochasticSamplingMode::Hex3,
            quality: StochasticQuality::HighQuality,
            seed: 33,
            ..default()
        },
        Some(StochasticHeightMap::new(assets.height_b.clone())),
    );

    commands.spawn((
        Name::new("Terrain Bombing Ramp"),
        Mesh3d(meshes.add(Cuboid::new(7.2, 0.8, 3.6))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(assets.albedo_b.clone()),
            normal_map_texture: Some(assets.normal_b.clone()),
            uv_transform: Affine2::from_scale(Vec2::splat(12.0)),
            perceptual_roughness: 0.98,
            base_color: Color::WHITE,
            reflectance: 0.05,
            ..default()
        })),
        Transform::from_xyz(offset.x - 0.8, offset.y - 0.6, offset.z + 1.4)
            .with_rotation(Quat::from_rotation_x(-0.4)),
        ExampleAuthoredStochastic(StochasticTexturing {
            uv_space: StochasticUvSpace::WorldTriplanar,
            sampling_mode: StochasticSamplingMode::TextureBombing,
            quality: StochasticQuality::Balanced,
            seed: 41,
            ..default()
        }),
        StochasticTexturing {
            uv_space: StochasticUvSpace::WorldTriplanar,
            sampling_mode: StochasticSamplingMode::TextureBombing,
            quality: StochasticQuality::Balanced,
            seed: 41,
            ..default()
        },
        StochasticHeightMap::new(assets.height_b.clone()),
    ));
}

fn spawn_stress(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    assets: &DemoAssets,
    offset: Vec3,
) {
    for row in 0..3 {
        for column in 0..5 {
            let index = row * 5 + column;
            let x = column as f32 * 1.9 - 3.8;
            let y = row as f32 * 1.5 + 0.8;
            let seed = 100 + index as u32;
            spawn_surface(
                commands,
                meshes,
                materials,
                &format!("Stress Surface {index}"),
                offset + Vec3::new(x, y, 0.0),
                if index % 2 == 0 {
                    assets.albedo_a.clone()
                } else {
                    assets.albedo_b.clone()
                },
                Some(if index % 2 == 0 {
                    assets.normal_a.clone()
                } else {
                    assets.normal_b.clone()
                }),
                Affine2::from_scale(Vec2::splat(5.5 + row as f32)),
                StochasticTexturing {
                    sampling_mode: match index % 4 {
                        0 => StochasticSamplingMode::Checker2,
                        1 => StochasticSamplingMode::Hex3,
                        2 => StochasticSamplingMode::TextureBombing,
                        _ => StochasticSamplingMode::HistogramPreserving,
                    },
                    quality: if index % 4 == 0 {
                        StochasticQuality::HighQuality
                    } else {
                        StochasticQuality::Balanced
                    },
                    seed,
                    ..default()
                },
                Some(StochasticHeightMap::new(if index % 2 == 0 {
                    assets.height_a.clone()
                } else {
                    assets.height_b.clone()
                })),
            );
        }
    }
}

fn spawn_surface(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: &str,
    translation: Vec3,
    texture: Handle<Image>,
    normal_map: Option<Handle<Image>>,
    uv_transform: Affine2,
    stochastic: StochasticTexturing,
    height_map: Option<StochasticHeightMap>,
) -> Entity {
    let yaw = (-translation.x * 0.035).clamp(-0.22, 0.22);
    let rotation = Quat::from_euler(EulerRot::XYZ, -0.04, yaw, 0.0);

    commands.spawn((
        Name::new(format!("{name} Backplate")),
        Mesh3d(meshes.add(Cuboid::new(2.55, 2.55, 0.12))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.08, 0.1, 0.13),
            emissive: LinearRgba::rgb(0.012, 0.016, 0.02),
            perceptual_roughness: 0.98,
            ..default()
        })),
        Transform::from_translation(translation + Vec3::new(0.0, 0.0, -0.13))
            .with_rotation(rotation),
    ));

    commands.spawn((
        Name::new(format!("{name} Pedestal")),
        Mesh3d(meshes.add(Cuboid::new(2.9, 0.35, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.18, 0.2, 0.24),
            perceptual_roughness: 0.9,
            metallic: 0.05,
            ..default()
        })),
        Transform::from_translation(translation + Vec3::new(0.0, -1.28, 0.34)),
    ));

    let mut entity = commands.spawn((
        Name::new(name.to_string()),
        Mesh3d(meshes.add(Cuboid::new(2.35, 2.35, 0.18))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(texture),
            normal_map_texture: normal_map,
            base_color: Color::srgb(0.8, 0.78, 0.75),
            uv_transform,
            perceptual_roughness: 0.98,
            reflectance: 0.06,
            ..default()
        })),
        Transform::from_translation(translation).with_rotation(rotation),
        ExampleAuthoredStochastic(stochastic.clone()),
        stochastic,
    ));

    if let Some(height_map) = height_map {
        entity.insert(height_map);
    }

    entity.id()
}

fn auto_exit_after(
    time: Res<Time>,
    mut timer: ResMut<AutoExitAfter>,
    mut exit: MessageWriter<AppExit>,
) {
    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        exit.write(AppExit::Success);
    }
}

impl DemoAssets {
    pub fn load(asset_server: &AssetServer) -> Self {
        Self {
            albedo_a: load_demo_texture(
                asset_server,
                "textures/stochastic/rock_face_03/rock_face_03_diff_1k.png",
                true,
            ),
            normal_a: load_demo_texture(
                asset_server,
                "textures/stochastic/rock_face_03/rock_face_03_nor_gl_1k.png",
                false,
            ),
            height_a: load_demo_texture(
                asset_server,
                "textures/stochastic/rock_face_03/rock_face_03_disp_1k.png",
                false,
            ),
            albedo_b: load_demo_texture(
                asset_server,
                "textures/stochastic/coast_sand_rocks_02/coast_sand_rocks_02_diff_1k.png",
                true,
            ),
            normal_b: load_demo_texture(
                asset_server,
                "textures/stochastic/coast_sand_rocks_02/coast_sand_rocks_02_nor_gl_1k.png",
                false,
            ),
            height_b: load_demo_texture(
                asset_server,
                "textures/stochastic/coast_sand_rocks_02/coast_sand_rocks_02_disp_1k.png",
                false,
            ),
            albedo_c: load_demo_texture(
                asset_server,
                "textures/stochastic/grassy_cobblestone/grassy_cobblestone_diff_1k.png",
                true,
            ),
            normal_c: load_demo_texture(
                asset_server,
                "textures/stochastic/grassy_cobblestone/grassy_cobblestone_nor_gl_1k.png",
                false,
            ),
            height_c: load_demo_texture(
                asset_server,
                "textures/stochastic/grassy_cobblestone/grassy_cobblestone_disp_1k.png",
                false,
            ),
        }
    }

    pub fn is_loaded(&self, asset_server: &AssetServer) -> bool {
        [
            self.albedo_a.id(),
            self.normal_a.id(),
            self.height_a.id(),
            self.albedo_b.id(),
            self.normal_b.id(),
            self.height_b.id(),
            self.albedo_c.id(),
            self.normal_c.id(),
            self.height_c.id(),
        ]
        .into_iter()
        .all(|handle| asset_server.is_loaded_with_dependencies(handle))
    }
}

fn load_demo_texture(
    asset_server: &AssetServer,
    path: &'static str,
    is_srgb: bool,
) -> Handle<Image> {
    asset_server.load_with_settings(path, move |settings: &mut ImageLoaderSettings| {
        settings.is_srgb = is_srgb;
        settings.sampler = repeating_linear_sampler();
    })
}

fn repeating_linear_sampler() -> ImageSampler {
    let mut sampler = ImageSamplerDescriptor::linear();
    sampler.set_address_mode(ImageAddressMode::Repeat);
    sampler.anisotropy_clamp = 8;
    ImageSampler::Descriptor(sampler)
}
