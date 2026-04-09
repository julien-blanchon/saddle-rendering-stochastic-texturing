# Architecture

## Current State

This crate now has a real first backend:

- public authoring components exist
- a core plugin and a working PBR adapter plugin exist
- the adapter performs entity-local `StandardMaterial` to `ExtendedMaterial` adaptation
- a generic GPU settings payload exists for adapter or custom-material users
- a reusable internal WGSL library exists for shared stochastic sampling logic
- diagnostics exist
- examples and lab exist
- the example common crate now loads bundled CC0 texture assets through `AssetServer` and gates scene spawn on asset readiness
- the example common crate now also owns a shared `saddle-pane` control surface that can override authored presets across the showcase apps
- the shader backend is implemented as a practical stochastic hex-style sampler plus a distinct texture-bombing branch

The reusable product shape still matters, but the crate has moved past pure scaffolding.

## Planned Core vs Adapter Split

The target architecture is:

- `StochasticTexturingPlugin`: owns shared config, diagnostics, asset-prep hooks, and future renderer-neutral helpers
- `StochasticShaderLibraryPlugin`: owns reusable GPU packing support and internal WGSL module loading
- `StochasticPbrPlugin`: owns the `StandardMaterial` bridge and the first production backend
- future adapters: terrain or custom-material integrations built on the same sampling concepts, not hard-coded into the core crate

This mirrors the repo's broader pattern where the core owns meaning and adapters own render policy.

## Planned Data Flow

```text
mesh + StandardMaterial + StochasticTexturing + optional StochasticHeightMap
    ->
core diagnostics / prep / future asset validation
    ->
generic shader uniform packing + reusable WGSL helpers
    ->
PBR adapter clones StandardMaterial state into a per-entity ExtendedMaterial
    ->
anti-repetition shader sampling
    ->
optional future terrain / custom material adapters
```

## Why This Shape

### Stable authoring layer

Most downstream games should only need:

- `StochasticTexturing`
- `StochasticHeightMap`
- debug and diagnostics resources
- the bundled PBR adapter plugin when they use `StandardMaterial`

### Swappable internal implementation

We want to keep room for:

- practical hex tiling
- a cheaper checker / two-sample path
- optional histogram-preserving blending
- future Wang-tiling or bombing-inspired adapters

without forcing users to rewrite gameplay-side or authoring-side code.

## Current Internal Modules

- `config`: reflectable enums and debug settings
- `components`: per-surface authoring data
- `diagnostics`: stable runtime counts and budget hints
- `gpu`: reusable `StochasticShaderUniform` packing
- `systems`: activation, diagnostics, and core runtime state
- `shader_library`: reusable shader handles and import-path loading
- `pbr`: bundled PBR adapter, material sync, and restoration flow
- `shaders/stochastic_types.wgsl`: shared structs, constants, and utility helpers
- `shaders/stochastic_sampling.wgsl`: shared lattice, bombing, weighting, height, and triplanar helpers
- `shaders/stochastic_pbr.wgsl`: first production fragment backend for the adapter

## Backend Sequencing

1. Practical hex tiling on `ExtendedMaterial<StandardMaterial, _>`
2. Normal-map correctness and explicit-gradient hardening
3. Height-aware blend polish and debug views
4. Lower-level shader-library extraction for custom materials and terrain
5. Terrain / custom material integration guidance
6. Optional offline histogram-preserving path

## Current Limitations

- `HistogramPreserving` still approximates the Heitz-style workflow through luminance-aware reweighting plus height emphasis instead of a baked histogram-preserving asset path
- `TextureBombing` now uses a distinct runtime layout, while `Wang` still routes through the shared hex-like path
- `DerivativeReconstruction` still aliases the rotated tangent-space path; a stronger derivative-aware implementation is still open
