# Configuration

## Current Surface

The public authoring components are now consumed by the bundled PBR adapter and by the lower-level shader-library path. A few authoring variants still map to approximated or shared behavior, but the practical runtime backend is now larger than a single fallback path.

## `StochasticTexturing`

Attach to a 3D surface that will eventually opt into anti-repetition sampling.

| Field | Type | Default | Intended Effect |
|------|------|---------|-----------------|
| `enabled` | `bool` | `true` | Master toggle for the surface |
| `sampling_mode` | `StochasticSamplingMode` | `Hex3` | Selects the current runtime path: `Off`, `Checker2`, `Hex3`, the histogram-preserving approximation path, the distinct texture-bombing path, or the current Wang fallback label |
| `blend_mode` | `StochasticBlendMode` | `HeightAware` | Chooses linear weighting or height-aware weighting; `HistogramPreserving` currently routes to a luminance-aware height-backed approximation |
| `quality` | `StochasticQuality` | `Balanced` | `Fast` drops to a two-sample blend; `Balanced` and `HighQuality` keep the three-sample path |
| `uv_space` | `StochasticUvSpace` | `MeshUv0` | Uses UV0, UV1, or world-triplanar projection; the bundled PBR adapter reconstructs triplanar normals |
| `rotation_mode` | `StochasticRotationMode` | `RotateMirror` | Controls per-cell rotation and mirroring inside the stochastic sampler |
| `normal_map_mode` | `StochasticNormalMapMode` | `RotateTangentSpace` | Enables rotated tangent-space correction for mesh-UV normal maps; `DerivativeReconstruction` currently still aliases this correction path |
| `base_scale` | `Vec2` | `Vec2::ONE` | Additional authoring-side scale multiplied on top of the base material UV transform |
| `variation_strength` | `f32` | `1.0` | Controls per-cell translation / rotation variation amplitude |
| `blend_softness` | `f32` | `0.35` | Controls how sharp or soft the lattice blend weights are |
| `height_blend_strength` | `f32` | `0.65` | Controls the emphasis of the optional height signal when height-aware blending is active |
| `sample_cull_threshold` | `f32` | `0.08` | Zeroes very low weights before normalization |
| `mip_bias` | `f32` | `0.0` | Biases effective sampling derivatives inside the stochastic path |
| `seed` | `u32` | `1` | Per-surface deterministic variation seed |

## `StochasticHeightMap`

Optional metadata for height-aware or histogram-preserving blending.

| Field | Type | Default | Intended Effect |
|------|------|---------|-----------------|
| `image` | `Handle<Image>` | required by user | Height texture source |
| `channel` | `TextureChannel` | `Luminance` | Channel used as the height signal |
| `amplitude` | `f32` | `1.0` | Planned height contrast multiplier |
| `remap_min` | `f32` | `0.0` | Input-floor remap before height weighting |
| `remap_max` | `f32` | `1.0` | Input-ceiling remap before height weighting |

## Enums

### `StochasticSamplingMode`

Current public variants:

- `Off`
- `Checker2`
- `Hex3`
- `HistogramPreserving`
- `TextureBombing`
- `Wang`

Current backend note:

- `Hex3` is the main v1 path
- `TextureBombing` now uses a distinct square-cell runtime layout
- `HistogramPreserving` still uses a shared practical backend with luminance-aware reweighting instead of a baked histogram-preserving workflow
- `Wang` still routes through the shared hex-like path

### `StochasticBlendMode`

Current public variants:

- `Linear`
- `HeightAware`
- `HistogramPreserving`

### `StochasticUvSpace`

Current public variants:

- `MeshUv0`
- `MeshUv1`
- `WorldTriplanar`

### `StochasticRotationMode`

Current public variants:

- `None`
- `Rotate60`
- `RotateMirror`

### `StochasticNormalMapMode`

Current public variants:

- `Disabled`
- `RotateTangentSpace`
- `DerivativeReconstruction`

Current backend note:

- `RotateTangentSpace` is the main shipping path
- `DerivativeReconstruction` currently maps to the same rotated tangent correction and remains an open quality upgrade

## Lower-Level Shader Use

Custom material or terrain users can now consume the crate without going through the bundled `StandardMaterial` adapter:

- load `StochasticShaderLibraryPlugin`
- pack per-surface settings with `StochasticShaderUniform`
- import `stochastic_texturing::types` and `stochastic_texturing::sampling` from WGSL
- keep bind-group ownership and texture policy inside the downstream material

## `StochasticDebugSettings`

| Field | Type | Default | Intended Effect |
|------|------|---------|-----------------|
| `enabled` | `bool` | `false` | Master debug toggle |
| `view` | `StochasticDebugView` | `Off` | Planned debug visualization mode |
| `show_surface_bounds` | `bool` | `false` | Planned bounds overlay |
| `freeze_seed` | `bool` | `false` | Planned deterministic debug freeze |
| `show_overlay` | `bool` | `true` | Example / lab overlay toggle |

## `StochasticTexturingDiagnostics`

Tracks:

- authored surfaces
- optional height maps
- surfaces that already match the planned PBR bridge shape
- world-triplanar users
- histogram-preserving users
- high-quality users
- whether debug views are active
- estimated baseline sample pressure

This is currently the main runtime summary surface for both examples and lab verification.
