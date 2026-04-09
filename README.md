# Saddle Rendering Stochastic Texturing

Reusable anti-repetition toolkit for Bevy focused on stochastic texturing, practical hex tiling, strong documentation, crate-local examples, and lab-first verification.

## Status

This crate now ships a **working practical backend**:

What exists today:

- the public authoring surface we want to stabilize
- a core plugin plus a working `StandardMaterial` adapter plugin
- a practical stochastic hex-style sampler implemented through `ExtendedMaterial<StandardMaterial, _>`
- a distinct texture-bombing runtime path
- full world-triplanar normal reconstruction for the bundled PBR adapter
- per-entity material adaptation and restoration so downstream games can keep authoring `MeshMaterial3d<StandardMaterial>`
- a lower-level shader-library path for custom materials and terrain-oriented integrations
- diagnostics for authored surfaces and adapted PBR surfaces
- standalone example workspace and crate-local lab

What is still to come:

- a true histogram-preserving mode instead of the current height-aware approximation
- a dedicated Wang / structured-material backend instead of the current hex-like fallback
- a stronger derivative-reconstruction normal path instead of the current rotated-tangent alias

The goal of the crate is now broader than a first release: keep the ergonomic `StandardMaterial` path easy while also exposing the reusable GPU and WGSL pieces that future terrain or custom-material users can build on.

## Quick Start

```toml
saddle-rendering-stochastic-texturing = { git = "https://github.com/julien-blanchon/saddle-rendering-stochastic-texturing" }
```

```rust,no_run
use bevy::prelude::*;
use stochastic_texturing::{
    StochasticBlendMode, StochasticPbrPlugin, StochasticSamplingMode, StochasticTexturing,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(StochasticPbrPlugin::default())
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 3.5, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        PointLight {
            intensity: 1_800_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(6.0, 8.0, 6.0),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(4.0, 4.0, 0.25))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.88, 0.86, 0.81),
            ..default()
        })),
        Transform::from_xyz(0.0, 1.8, 0.0),
        StochasticTexturing {
            sampling_mode: StochasticSamplingMode::Hex3,
            blend_mode: StochasticBlendMode::HeightAware,
            ..default()
        },
        Name::new("Demo Surface"),
    ));
}
```

## Public API

| Type | Purpose |
|------|---------|
| `StochasticTexturingPlugin` | Core plugin for shared config, diagnostics, and future asset prep |
| `StochasticPbrPlugin` | Working `StandardMaterial` bridge with per-entity material adaptation |
| `StochasticShaderLibraryPlugin` | Loads the reusable internal WGSL modules for custom material or terrain integrations |
| `StochasticTexturingSystems` | Public ordering hooks for prepare, diagnostics, and debug |
| `StochasticPbrSystems` | Public ordering hooks for material adaptation, uniform sync, and adapter diagnostics |
| `StochasticTexturing` | Per-surface anti-repetition authoring component |
| `StochasticHeightMap` | Optional height-map metadata for height-aware and histogram-preserving blend paths |
| `StochasticShaderUniform` | Generic GPU-packed settings payload for adapters or custom material users |
| `StochasticPbrMaterial` | The adapter material alias: `ExtendedMaterial<StandardMaterial, _>` |
| `StochasticTexturingMaterialExtension` | The extension asset used by the bundled PBR adapter |
| `STOCHASTIC_TYPES_IMPORT_PATH` / `STOCHASTIC_SAMPLING_IMPORT_PATH` | Stable WGSL import paths for lower-level integrations |
| `StochasticSamplingMode` | Runtime sampling families: `Off`, `Checker2`, `Hex3`, `HistogramPreserving`, `TextureBombing`, `Wang` |
| `StochasticBlendMode` | Blend families: linear, height-aware, histogram-preserving |
| `StochasticQuality` | Quality tiers: `Fast`, `Balanced`, `HighQuality` |
| `StochasticUvSpace` | UV projection families: mesh UV0, mesh UV1, world triplanar |
| `StochasticRotationMode` | Transform family for per-cell rotation / mirroring |
| `StochasticNormalMapMode` | Normal-map handling strategy |
| `StochasticDebugSettings` / `StochasticDebugView` | Debug visualization controls |
| `StochasticTexturingDiagnostics` | Runtime counts for authored surfaces, height maps, PBR targets, and estimated sample pressure |

## Direction

The planned product direction is:

- one core crate with renderer-agnostic authoring data
- one ergonomic bundled PBR adapter for `StandardMaterial`
- a practical hex-tiling backend first
- optional higher-cost histogram-preserving mode later
- a shipped shader-library path for terrain and custom material integration
- examples that are visually instructive and easy to hack on

The public implementation direction is summarized in this README and the crate docs so the published repo stays self-contained.

## Examples

The example workspace is now the main showcase for the implemented v1 backend.

The shared `examples/assets/textures/stochastic` bundle now includes CC0 Poly Haven materials so the examples and lab validate the runtime against real scanned surface data instead of only procedural placeholders.

All user-facing examples now ship with a `saddle-pane` control panel in the top-right. By default the authored comparison presets stay intact, and users can opt into a shared live override set to probe sampling mode, blend mode, UV projection, rotation policy, debug views, and quality without editing code.

| Example | Purpose | Run |
|---------|---------|-----|
| `showcase` | Reference-style all-in-one gallery that combines the main feature slices into a live comparison app | `cd examples && cargo run -p saddle-rendering-stochastic-texturing-example-showcase` |
| `basic` | Side-by-side repeated surfaces with off, checker, hex, and texture-bombing sampling | `cd examples && cargo run -p saddle-rendering-stochastic-texturing-example-basic` |
| `normal_maps` | Normal-map-aware stochastic sampling and rotation-policy comparison | `cd examples && cargo run -p saddle-rendering-stochastic-texturing-example-normal-maps` |
| `height_blend` | Linear vs height-aware vs histogram-preserving-approximation blend comparison | `cd examples && cargo run -p saddle-rendering-stochastic-texturing-example-height-blend` |
| `terrain_bridge` | World-triplanar and terrain-oriented authoring with reconstructed normals and texture-bombing coverage | `cd examples && cargo run -p saddle-rendering-stochastic-texturing-example-terrain-bridge` |
| `stress` | Many authored surfaces for diagnostics and perf-budget planning | `cd examples && cargo run -p saddle-rendering-stochastic-texturing-example-stress` |

## Crate-Local Lab

The richer verification app lives at `examples/lab`:

```bash
cd examples && cargo run -p saddle-rendering-stochastic-texturing-lab
```

Current scenarios:

```bash
cd examples && cargo run -p saddle-rendering-stochastic-texturing-lab --features e2e -- smoke_launch
cd examples && cargo run -p saddle-rendering-stochastic-texturing-lab --features e2e -- stochastic_overview
cd examples && cargo run -p saddle-rendering-stochastic-texturing-lab --features e2e -- stochastic_before_after_closeup
cd examples && cargo run -p saddle-rendering-stochastic-texturing-lab --features e2e -- stochastic_normal_maps
cd examples && cargo run -p saddle-rendering-stochastic-texturing-lab --features e2e -- stochastic_height_blend
cd examples && cargo run -p saddle-rendering-stochastic-texturing-lab --features e2e -- stochastic_triplanar_slope_transition
cd examples && cargo run -p saddle-rendering-stochastic-texturing-lab --features e2e -- stochastic_quality_ladder
cd examples && cargo run -p saddle-rendering-stochastic-texturing-lab --features e2e -- stochastic_stress
```

## Lower-Level Integration

For custom materials or terrain adapters, add the shader-library plugin and pack the same per-surface settings into `StochasticShaderUniform`. The bundled PBR adapter is now just one consumer of that generic layer.

The reusable WGSL modules are loaded under:

- `stochastic_texturing::types`
- `stochastic_texturing::sampling`

## Design Notes

- The public API is intentionally data-first and adapter-friendly.
- The current backend is practical stochastic hex-style sampling plus a distinct texture-bombing path on top of `ExtendedMaterial<StandardMaterial, _>`.
- World-triplanar projection now stochasticizes albedo, emissive, ORM, AO, and normal maps in the bundled PBR adapter.
- `HistogramPreserving` currently uses a luminance-aware, height-backed approximation instead of the full offline Heitz / Deliot workflow.
- `Wang` currently remains an authoring-side fallback label and still routes through the shared hex-like runtime path.
- Terrain and custom material integration are consumers of this crate's shared GPU and WGSL sampling logic, not responsibilities of the core crate.

More detail lives in [`docs/architecture.md`](docs/architecture.md) and [`docs/configuration.md`](docs/configuration.md).
