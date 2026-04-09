use bevy::prelude::*;
use bevy::render::render_resource::ShaderType;

use crate::{
    StochasticBlendMode, StochasticHeightMap, StochasticNormalMapMode, StochasticQuality,
    StochasticRotationMode, StochasticSamplingMode, StochasticTexturing, StochasticUvSpace,
    TextureChannel,
};

const STATE_ENABLED: u32 = 1 << 0;
const STATE_HEIGHT_PRESENT: u32 = 1 << 1;

#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct StochasticShaderUniform {
    pub primary: Vec4,
    pub secondary: Vec4,
    pub remap: Vec4,
    pub state: UVec4,
    pub space: UVec4,
    pub seed: UVec4,
}

impl StochasticShaderUniform {
    #[must_use]
    pub fn from_surface(
        surface: &StochasticTexturing,
        height_map: Option<&StochasticHeightMap>,
    ) -> Self {
        let mut flags = 0;
        if surface.enabled {
            flags |= STATE_ENABLED;
        }
        if height_map.is_some() {
            flags |= STATE_HEIGHT_PRESENT;
        }

        let height_map = height_map.cloned().unwrap_or_else(|| StochasticHeightMap {
            image: Handle::default(),
            channel: TextureChannel::Luminance,
            amplitude: 1.0,
            remap_min: 0.0,
            remap_max: 1.0,
        });

        Self {
            primary: Vec4::new(
                surface.base_scale.x.max(0.0001),
                surface.base_scale.y.max(0.0001),
                surface.variation_strength.max(0.0),
                surface.blend_softness.clamp(0.0, 1.0),
            ),
            secondary: Vec4::new(
                surface.height_blend_strength.clamp(0.0, 1.0),
                surface.sample_cull_threshold.clamp(0.0, 1.0),
                surface.mip_bias,
                height_map.amplitude.max(0.0),
            ),
            remap: Vec4::new(height_map.remap_min, height_map.remap_max, 0.0, 0.0),
            state: UVec4::new(
                flags,
                sampling_mode_to_u32(surface.sampling_mode),
                blend_mode_to_u32(surface.blend_mode),
                quality_to_u32(surface.quality),
            ),
            space: UVec4::new(
                uv_space_to_u32(surface.uv_space),
                rotation_mode_to_u32(surface.rotation_mode),
                normal_map_mode_to_u32(surface.normal_map_mode),
                texture_channel_to_u32(height_map.channel),
            ),
            seed: UVec4::new(surface.seed, 0, 0, 0),
        }
    }

    #[must_use]
    pub fn enabled(&self) -> bool {
        (self.state.x & STATE_ENABLED) != 0
    }
}

fn sampling_mode_to_u32(mode: StochasticSamplingMode) -> u32 {
    match mode {
        StochasticSamplingMode::Off => 0,
        StochasticSamplingMode::Checker2 => 1,
        StochasticSamplingMode::Hex3 => 2,
        StochasticSamplingMode::HistogramPreserving => 3,
        StochasticSamplingMode::TextureBombing => 4,
        StochasticSamplingMode::Wang => 5,
    }
}

fn blend_mode_to_u32(mode: StochasticBlendMode) -> u32 {
    match mode {
        StochasticBlendMode::Linear => 0,
        StochasticBlendMode::HeightAware => 1,
        StochasticBlendMode::HistogramPreserving => 2,
    }
}

fn quality_to_u32(mode: StochasticQuality) -> u32 {
    match mode {
        StochasticQuality::Fast => 0,
        StochasticQuality::Balanced => 1,
        StochasticQuality::HighQuality => 2,
    }
}

fn uv_space_to_u32(mode: StochasticUvSpace) -> u32 {
    match mode {
        StochasticUvSpace::MeshUv0 => 0,
        StochasticUvSpace::MeshUv1 => 1,
        StochasticUvSpace::WorldTriplanar => 2,
    }
}

fn rotation_mode_to_u32(mode: StochasticRotationMode) -> u32 {
    match mode {
        StochasticRotationMode::None => 0,
        StochasticRotationMode::Rotate60 => 1,
        StochasticRotationMode::RotateMirror => 2,
    }
}

fn normal_map_mode_to_u32(mode: StochasticNormalMapMode) -> u32 {
    match mode {
        StochasticNormalMapMode::Disabled => 0,
        StochasticNormalMapMode::RotateTangentSpace => 1,
        StochasticNormalMapMode::DerivativeReconstruction => 2,
    }
}

fn texture_channel_to_u32(channel: TextureChannel) -> u32 {
    match channel {
        TextureChannel::Red => 0,
        TextureChannel::Green => 1,
        TextureChannel::Blue => 2,
        TextureChannel::Alpha => 3,
        TextureChannel::Luminance => 4,
    }
}
