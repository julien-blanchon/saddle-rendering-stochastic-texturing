use bevy::{prelude::*, render::sync_world::SyncToRenderWorld};

use crate::config::{
    StochasticBlendMode, StochasticNormalMapMode, StochasticQuality, StochasticRotationMode,
    StochasticSamplingMode, StochasticUvSpace, TextureChannel,
};

/// Per-entity anti-repetition authoring surface.
#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component, Default)]
#[require(SyncToRenderWorld)]
pub struct StochasticTexturing {
    pub enabled: bool,
    pub sampling_mode: StochasticSamplingMode,
    pub blend_mode: StochasticBlendMode,
    pub quality: StochasticQuality,
    pub uv_space: StochasticUvSpace,
    pub rotation_mode: StochasticRotationMode,
    pub normal_map_mode: StochasticNormalMapMode,
    pub base_scale: Vec2,
    pub variation_strength: f32,
    pub blend_softness: f32,
    pub height_blend_strength: f32,
    pub sample_cull_threshold: f32,
    pub mip_bias: f32,
    pub seed: u32,
}

impl Default for StochasticTexturing {
    fn default() -> Self {
        Self {
            enabled: true,
            sampling_mode: StochasticSamplingMode::Hex3,
            blend_mode: StochasticBlendMode::HeightAware,
            quality: StochasticQuality::Balanced,
            uv_space: StochasticUvSpace::MeshUv0,
            rotation_mode: StochasticRotationMode::RotateMirror,
            normal_map_mode: StochasticNormalMapMode::RotateTangentSpace,
            base_scale: Vec2::splat(1.0),
            variation_strength: 1.0,
            blend_softness: 0.35,
            height_blend_strength: 0.65,
            sample_cull_threshold: 0.08,
            mip_bias: 0.0,
            seed: 1,
        }
    }
}

impl StochasticTexturing {
    #[must_use]
    pub fn expected_sample_count_per_map(&self) -> u32 {
        match self.sampling_mode {
            StochasticSamplingMode::Off => 1,
            StochasticSamplingMode::Checker2 => 2,
            StochasticSamplingMode::Hex3
            | StochasticSamplingMode::HistogramPreserving
            | StochasticSamplingMode::Wang => 3,
            StochasticSamplingMode::TextureBombing => 4,
        }
    }

    #[must_use]
    pub fn uses_height_map(&self) -> bool {
        matches!(
            self.blend_mode,
            StochasticBlendMode::HeightAware | StochasticBlendMode::HistogramPreserving
        )
    }
}

/// Optional height data for height-aware or histogram-preserving blending.
#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component)]
#[require(SyncToRenderWorld)]
pub struct StochasticHeightMap {
    pub image: Handle<Image>,
    pub channel: TextureChannel,
    pub amplitude: f32,
    pub remap_min: f32,
    pub remap_max: f32,
}

impl StochasticHeightMap {
    #[must_use]
    pub fn new(image: Handle<Image>) -> Self {
        Self {
            image,
            channel: TextureChannel::Luminance,
            amplitude: 1.0,
            remap_min: 0.0,
            remap_max: 1.0,
        }
    }
}
