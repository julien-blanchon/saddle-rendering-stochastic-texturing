use bevy::prelude::*;

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum StochasticSamplingMode {
    Off,
    Checker2,
    #[default]
    Hex3,
    HistogramPreserving,
    TextureBombing,
    Wang,
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum StochasticBlendMode {
    #[default]
    HeightAware,
    Linear,
    HistogramPreserving,
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum StochasticQuality {
    Fast,
    #[default]
    Balanced,
    HighQuality,
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum StochasticUvSpace {
    #[default]
    MeshUv0,
    MeshUv1,
    WorldTriplanar,
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum StochasticRotationMode {
    None,
    Rotate60,
    #[default]
    RotateMirror,
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum StochasticNormalMapMode {
    Disabled,
    #[default]
    RotateTangentSpace,
    DerivativeReconstruction,
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TextureChannel {
    Red,
    Green,
    Blue,
    Alpha,
    #[default]
    Luminance,
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum StochasticDebugView {
    #[default]
    Off,
    BlendWeights,
    CellIds,
    SampleTransforms,
    HeightMask,
    SampleCount,
}

#[derive(Resource, Reflect, Clone, Debug, PartialEq, Eq)]
#[reflect(Resource, Default)]
pub struct StochasticDebugSettings {
    pub enabled: bool,
    pub view: StochasticDebugView,
    pub show_surface_bounds: bool,
    pub freeze_seed: bool,
    pub show_overlay: bool,
}

impl Default for StochasticDebugSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            view: StochasticDebugView::Off,
            show_surface_bounds: false,
            freeze_seed: false,
            show_overlay: true,
        }
    }
}
