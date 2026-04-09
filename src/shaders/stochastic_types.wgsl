#define_import_path stochastic_texturing::types

struct StochasticUniform {
    primary: vec4<f32>,
    secondary: vec4<f32>,
    remap: vec4<f32>,
    state: vec4<u32>,
    space: vec4<u32>,
    seed: vec4<u32>,
}

struct SampleFrame {
    uv: vec2<f32>,
    ddx_uv: vec2<f32>,
    ddy_uv: vec2<f32>,
}

struct StochasticLayout {
    cells: array<vec2<i32>, 3>,
    weights: vec3<f32>,
}

struct TextureBombLayout {
    cells: array<vec2<i32>, 4>,
    weights: vec4<f32>,
}

const STATE_ENABLED: u32 = 1u;
const STATE_HEIGHT_PRESENT: u32 = 2u;

const SAMPLING_OFF: u32 = 0u;
const SAMPLING_CHECKER2: u32 = 1u;
const SAMPLING_HEX3: u32 = 2u;
const SAMPLING_HISTOGRAM: u32 = 3u;
const SAMPLING_TEXTURE_BOMBING: u32 = 4u;
const SAMPLING_WANG: u32 = 5u;

const BLEND_LINEAR: u32 = 0u;
const BLEND_HEIGHT_AWARE: u32 = 1u;
const BLEND_HISTOGRAM_APPROX: u32 = 2u;

const QUALITY_FAST: u32 = 0u;
const QUALITY_BALANCED: u32 = 1u;
const QUALITY_HIGH: u32 = 2u;

const UV_MESH_UV0: u32 = 0u;
const UV_MESH_UV1: u32 = 1u;
const UV_WORLD_TRIPLANAR: u32 = 2u;

const ROTATION_NONE: u32 = 0u;
const ROTATION_ROTATE60: u32 = 1u;
const ROTATION_ROTATE_MIRROR: u32 = 2u;

const NORMAL_DISABLED: u32 = 0u;
const NORMAL_ROTATE_TANGENT: u32 = 1u;
const NORMAL_DERIVATIVE: u32 = 2u;

const HEIGHT_RED: u32 = 0u;
const HEIGHT_GREEN: u32 = 1u;
const HEIGHT_BLUE: u32 = 2u;
const HEIGHT_ALPHA: u32 = 3u;
const HEIGHT_LUMINANCE: u32 = 4u;

const PI: f32 = 3.141592653589793;
const INV_SQRT3: f32 = 0.5773502691896258;
const TWO_INV_SQRT3: f32 = 1.1547005383792515;

fn saturate(value: f32) -> f32 {
    return clamp(value, 0.0, 1.0);
}

fn normalize_weights(weights: vec3<f32>) -> vec3<f32> {
    let total = max(weights.x + weights.y + weights.z, 1e-5);
    return weights / total;
}

fn normalize_weights4(weights: vec4<f32>) -> vec4<f32> {
    let total = max(weights.x + weights.y + weights.z + weights.w, 1e-5);
    return weights / total;
}
