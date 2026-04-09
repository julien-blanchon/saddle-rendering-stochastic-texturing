use bevy::prelude::*;

use crate::{
    StochasticBlendMode, StochasticDebugSettings, StochasticQuality, StochasticRotationMode,
    StochasticSamplingMode, StochasticTexturing, StochasticUvSpace, TextureChannel,
};

#[test]
fn stochastic_defaults_match_mvp_direction() {
    let settings = StochasticTexturing::default();

    assert!(settings.enabled);
    assert_eq!(settings.sampling_mode, StochasticSamplingMode::Hex3);
    assert_eq!(settings.blend_mode, StochasticBlendMode::HeightAware);
    assert_eq!(settings.quality, StochasticQuality::Balanced);
    assert_eq!(settings.uv_space, StochasticUvSpace::MeshUv0);
    assert_eq!(settings.rotation_mode, StochasticRotationMode::RotateMirror);
    assert_eq!(settings.base_scale, Vec2::ONE);
    assert!(settings.uses_height_map());
}

#[test]
fn debug_defaults_start_disabled() {
    let settings = StochasticDebugSettings::default();
    assert!(!settings.enabled);
    assert!(settings.show_overlay);
}

#[test]
fn height_map_constructor_uses_luminance_channel() {
    let image = Handle::<Image>::default();
    let map = crate::StochasticHeightMap::new(image);
    assert_eq!(map.channel, TextureChannel::Luminance);
    assert_eq!(map.remap_min, 0.0);
    assert_eq!(map.remap_max, 1.0);
}
