#define_import_path stochastic_texturing::sampling

#import bevy_pbr::pbr_types
#import stochastic_texturing::types::{
    BLEND_HISTOGRAM_APPROX,
    BLEND_LINEAR,
    HEIGHT_ALPHA,
    HEIGHT_BLUE,
    HEIGHT_GREEN,
    HEIGHT_LUMINANCE,
    HEIGHT_RED,
    NORMAL_DISABLED,
    NORMAL_ROTATE_TANGENT,
    PI,
    QUALITY_BALANCED,
    QUALITY_FAST,
    QUALITY_HIGH,
    ROTATION_NONE,
    ROTATION_ROTATE_MIRROR,
    ROTATION_ROTATE60,
    SAMPLING_CHECKER2,
    SAMPLING_HISTOGRAM,
    SAMPLING_OFF,
    SAMPLING_TEXTURE_BOMBING,
    SampleFrame,
    StochasticLayout,
    StochasticUniform,
    TextureBombLayout,
    TWO_INV_SQRT3,
    UV_WORLD_TRIPLANAR,
    INV_SQRT3,
    normalize_weights,
    normalize_weights4,
    saturate,
}

fn state_enabled(settings: StochasticUniform) -> bool {
    return (settings.state.x & 1u) != 0u;
}

fn state_has_height(settings: StochasticUniform) -> bool {
    return (settings.state.x & 2u) != 0u;
}

fn hash11(value: f32) -> f32 {
    return fract(sin(value) * 43758.5453123);
}

fn hash12(value: vec2<f32>) -> f32 {
    return hash11(dot(value, vec2<f32>(127.1, 311.7)));
}

fn hash22(value: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(
        hash12(value + vec2<f32>(1.0, 17.0)),
        hash12(value + vec2<f32>(29.0, 53.0)),
    );
}

fn gain3(weights: vec3<f32>, bias: f32) -> vec3<f32> {
    let clamped_bias = clamp(bias, 0.001, 0.999);
    let k = log(1.0 - clamped_bias) / log(0.5);
    let selection = 2.0 * select(vec3<f32>(0.0), vec3<f32>(1.0), weights >= vec3<f32>(0.5));
    let mirror = 2.0 * (vec3<f32>(1.0) - selection);
    let remapped = 0.5 * selection +
        0.25 * mirror * pow(max(vec3<f32>(0.0), selection + weights * mirror), vec3<f32>(k));
    return normalize_weights(remapped);
}

fn lattice_layout(base_uv: vec2<f32>) -> StochasticLayout {
    let grid = vec2<f32>(
        base_uv.x - base_uv.y * INV_SQRT3,
        base_uv.y * TWO_INV_SQRT3,
    );
    let cell = vec2<i32>(floor(grid));
    let local = fract(grid);

    var lattice: StochasticLayout;
    if local.x + local.y < 1.0 {
        lattice.cells[0] = cell;
        lattice.cells[1] = cell + vec2<i32>(1, 0);
        lattice.cells[2] = cell + vec2<i32>(0, 1);
        lattice.weights = vec3<f32>(1.0 - local.x - local.y, local.x, local.y);
    } else {
        lattice.cells[0] = cell + vec2<i32>(1, 1);
        lattice.cells[1] = cell + vec2<i32>(0, 1);
        lattice.cells[2] = cell + vec2<i32>(1, 0);
        lattice.weights = vec3<f32>(local.x + local.y - 1.0, 1.0 - local.x, 1.0 - local.y);
    }

    return lattice;
}

fn texture_bomb_layout(settings: StochasticUniform, base_uv: vec2<f32>) -> TextureBombLayout {
    let cell = vec2<i32>(floor(base_uv));

    var bombing: TextureBombLayout;
    bombing.cells[0] = cell + vec2<i32>(0, 0);
    bombing.cells[1] = cell + vec2<i32>(1, 0);
    bombing.cells[2] = cell + vec2<i32>(0, 1);
    bombing.cells[3] = cell + vec2<i32>(1, 1);

    let feature_0 = vec2<f32>(bombing.cells[0]) + hash22(vec2<f32>(bombing.cells[0]) + vec2<f32>(f32(settings.seed.x), 0.0));
    let feature_1 = vec2<f32>(bombing.cells[1]) + hash22(vec2<f32>(bombing.cells[1]) + vec2<f32>(f32(settings.seed.x), 0.0));
    let feature_2 = vec2<f32>(bombing.cells[2]) + hash22(vec2<f32>(bombing.cells[2]) + vec2<f32>(f32(settings.seed.x), 0.0));
    let feature_3 = vec2<f32>(bombing.cells[3]) + hash22(vec2<f32>(bombing.cells[3]) + vec2<f32>(f32(settings.seed.x), 0.0));

    let strength = mix(4.0, 9.0, saturate(settings.primary.z));
    bombing.weights = vec4<f32>(
        exp(-distance(base_uv, feature_0) * strength),
        exp(-distance(base_uv, feature_1) * strength),
        exp(-distance(base_uv, feature_2) * strength),
        exp(-distance(base_uv, feature_3) * strength),
    );

    if settings.state.w == QUALITY_FAST {
        var smallest = bombing.weights.x;
        var second_smallest = bombing.weights.y;
        var smallest_index = 0u;
        var second_index = 1u;
        if smallest > second_smallest {
            let temp = smallest;
            smallest = second_smallest;
            second_smallest = temp;
            smallest_index = 1u;
            second_index = 0u;
        }

        if bombing.weights.z < smallest {
            second_smallest = smallest;
            second_index = smallest_index;
            smallest = bombing.weights.z;
            smallest_index = 2u;
        } else if bombing.weights.z < second_smallest {
            second_smallest = bombing.weights.z;
            second_index = 2u;
        }

        if bombing.weights.w < smallest {
            second_smallest = smallest;
            second_index = smallest_index;
            smallest_index = 3u;
        } else if bombing.weights.w < second_smallest {
            second_index = 3u;
        }

        if smallest_index == 0u || second_index == 0u { bombing.weights.x = 0.0; }
        if smallest_index == 1u || second_index == 1u { bombing.weights.y = 0.0; }
        if smallest_index == 2u || second_index == 2u { bombing.weights.z = 0.0; }
        if smallest_index == 3u || second_index == 3u { bombing.weights.w = 0.0; }
    } else if settings.state.w == QUALITY_BALANCED {
        let threshold = saturate(settings.secondary.y);
        bombing.weights = vec4<f32>(
            select(0.0, bombing.weights.x, bombing.weights.x >= threshold),
            select(0.0, bombing.weights.y, bombing.weights.y >= threshold),
            select(0.0, bombing.weights.z, bombing.weights.z >= threshold),
            select(0.0, bombing.weights.w, bombing.weights.w >= threshold),
        );
    }

    if bombing.weights.x + bombing.weights.y + bombing.weights.z + bombing.weights.w < 1e-5 {
        bombing.weights = vec4<f32>(1.0, 0.0, 0.0, 0.0);
    }

    bombing.weights = normalize_weights4(bombing.weights);
    return bombing;
}

fn weight_rules(settings: StochasticUniform, base_weights: vec3<f32>) -> vec3<f32> {
    var weights = base_weights;
    let softness = saturate(settings.primary.w);
    let exponent = mix(7.0, 1.35, softness);
    weights = vec3<f32>(
        pow(max(weights.x, 1e-4), exponent),
        pow(max(weights.y, 1e-4), exponent),
        pow(max(weights.z, 1e-4), exponent),
    );

    if settings.state.w == QUALITY_FAST || settings.state.y == SAMPLING_CHECKER2 {
        if weights.x <= weights.y && weights.x <= weights.z {
            weights.x = 0.0;
        } else if weights.y <= weights.x && weights.y <= weights.z {
            weights.y = 0.0;
        } else {
            weights.z = 0.0;
        }
    }

    if settings.state.y == SAMPLING_OFF {
        weights = vec3<f32>(1.0, 0.0, 0.0);
    }

    if settings.state.w == QUALITY_BALANCED {
        let threshold = saturate(settings.secondary.y);
        weights = vec3<f32>(
            select(0.0, weights.x, weights.x >= threshold),
            select(0.0, weights.y, weights.y >= threshold),
            select(0.0, weights.z, weights.z >= threshold),
        );
    }

    if weights.x + weights.y + weights.z < 1e-5 {
        return vec3<f32>(1.0, 0.0, 0.0);
    }
    return normalize_weights(weights);
}

fn rotation_matrix(settings: StochasticUniform, cell: vec2<i32>) -> mat2x2<f32> {
    var angle = 0.0;
    if settings.space.y == ROTATION_ROTATE60 || settings.space.y == ROTATION_ROTATE_MIRROR {
        let turns = floor(hash12(vec2<f32>(cell) + vec2<f32>(11.0, 37.0) + vec2<f32>(f32(settings.seed.x), 0.0)) * 6.0);
        angle = turns * (PI / 3.0) * saturate(settings.primary.z);
    }

    let c = cos(angle);
    let s = sin(angle);
    var rotation = mat2x2<f32>(vec2<f32>(c, s), vec2<f32>(-s, c));

    if settings.space.y == ROTATION_ROTATE_MIRROR {
        let mirror = select(1.0, -1.0, hash12(vec2<f32>(cell) + vec2<f32>(19.0, 43.0)) > 0.5);
        rotation = rotation * mat2x2<f32>(vec2<f32>(mirror, 0.0), vec2<f32>(0.0, 1.0));
    }

    return rotation;
}

fn transformed_frame(settings: StochasticUniform, frame: SampleFrame, cell: vec2<i32>) -> SampleFrame {
    let variation = saturate(settings.primary.z);
    let offset_scale = select(0.35, 0.7, settings.state.y == SAMPLING_TEXTURE_BOMBING);
    let offset = (hash22(vec2<f32>(cell) + vec2<f32>(5.0, 13.0) + vec2<f32>(f32(settings.seed.x), 0.0)) * 2.0 - 1.0) *
        offset_scale *
        variation;
    var rotation = mat2x2<f32>(vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0));
    if settings.space.y != ROTATION_NONE {
        rotation = rotation_matrix(settings, cell);
    }
    let mip_scale = exp2(settings.secondary.z);

    return SampleFrame(
        rotation * (frame.uv + offset),
        rotation * frame.ddx_uv * mip_scale,
        rotation * frame.ddy_uv * mip_scale,
    );
}

fn projected_world_scale(settings: StochasticUniform, uv_transform: mat3x3<f32>) -> vec2<f32> {
    let scale_x = max(length(uv_transform[0].xy), 1e-4);
    let scale_y = max(length(uv_transform[1].xy), 1e-4);
    return vec2<f32>(scale_x, scale_y) * settings.primary.xy;
}

fn should_use_histogram_weights(settings: StochasticUniform) -> bool {
    return settings.state.y == SAMPLING_HISTOGRAM || settings.state.z == BLEND_HISTOGRAM_APPROX;
}

fn triplanar_weights(normal: vec3<f32>) -> vec3<f32> {
    let blend = pow(abs(normalize(normal)), vec3<f32>(4.0));
    return blend / max(dot(blend, vec3<f32>(1.0)), 1e-4);
}

fn remap_height(settings: StochasticUniform, sampled: vec4<f32>) -> f32 {
    var value = 0.0;
    if settings.space.w == HEIGHT_RED {
        value = sampled.r;
    } else if settings.space.w == HEIGHT_GREEN {
        value = sampled.g;
    } else if settings.space.w == HEIGHT_BLUE {
        value = sampled.b;
    } else if settings.space.w == HEIGHT_ALPHA {
        value = sampled.a;
    } else {
        value = dot(sampled.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    }

    let range = max(settings.remap.y - settings.remap.x, 1e-4);
    return saturate((value - settings.remap.x) / range) * settings.secondary.w;
}

fn height_texture(
    settings: StochasticUniform,
    height_map: texture_2d<f32>,
    height_sampler: sampler,
    frame: SampleFrame,
) -> f32 {
    if !state_has_height(settings) {
        return 1.0;
    }
    return remap_height(
        settings,
        textureSampleGrad(
            height_map,
            height_sampler,
            frame.uv,
            frame.ddx_uv,
            frame.ddy_uv,
        ),
    );
}

fn decode_tangent_normal(flags: u32, encoded: vec3<f32>) -> vec3<f32> {
    var tangent_normal = vec3<f32>(0.0, 0.0, 1.0);
    if (flags & pbr_types::STANDARD_MATERIAL_FLAGS_TWO_COMPONENT_NORMAL_MAP) != 0u {
        tangent_normal = vec3<f32>(encoded.xy * 2.0 - 1.0, 0.0);
        tangent_normal.z = sqrt(max(1.0 - tangent_normal.x * tangent_normal.x - tangent_normal.y * tangent_normal.y, 0.0));
    } else {
        tangent_normal = encoded * 2.0 - 1.0;
    }

    if (flags & pbr_types::STANDARD_MATERIAL_FLAGS_FLIP_NORMAL_MAP_Y) != 0u {
        tangent_normal.y = -tangent_normal.y;
    }

    return normalize(tangent_normal);
}

fn tangent_to_world(TBN: mat3x3<f32>, tangent_normal: vec3<f32>, double_sided: bool, is_front: bool) -> vec3<f32> {
    var Nt = tangent_normal;
    if double_sided && !is_front {
        Nt = -Nt;
    }

    let tangent = TBN[0];
    let bitangent = TBN[1];
    let normal = TBN[2];
    return normalize(Nt.x * tangent + Nt.y * bitangent + Nt.z * normal);
}

fn triplanar_world_normal_x(tangent_normal: vec3<f32>, sign_x: f32) -> vec3<f32> {
    let tangent = vec3<f32>(0.0, 0.0, sign_x);
    let bitangent = vec3<f32>(0.0, 1.0, 0.0);
    let normal = vec3<f32>(sign_x, 0.0, 0.0);
    return normalize(tangent * tangent_normal.x + bitangent * tangent_normal.y + normal * tangent_normal.z);
}

fn triplanar_world_normal_y(tangent_normal: vec3<f32>, sign_y: f32) -> vec3<f32> {
    let tangent = vec3<f32>(1.0, 0.0, 0.0);
    let bitangent = vec3<f32>(0.0, 0.0, sign_y);
    let normal = vec3<f32>(0.0, sign_y, 0.0);
    return normalize(tangent * tangent_normal.x + bitangent * tangent_normal.y + normal * tangent_normal.z);
}

fn triplanar_world_normal_z(tangent_normal: vec3<f32>, sign_z: f32) -> vec3<f32> {
    let tangent = vec3<f32>(sign_z, 0.0, 0.0);
    let bitangent = vec3<f32>(0.0, 1.0, 0.0);
    let normal = vec3<f32>(0.0, 0.0, sign_z);
    return normalize(tangent * tangent_normal.x + bitangent * tangent_normal.y + normal * tangent_normal.z);
}

fn histogram_reweight(settings: StochasticUniform, weights: vec3<f32>, color_a: vec4<f32>, color_b: vec4<f32>, color_c: vec4<f32>) -> vec3<f32> {
    if !should_use_histogram_weights(settings) && settings.state.w != QUALITY_HIGH {
        return weights;
    }

    let luminance_weights = vec3<f32>(
        max(dot(color_a.rgb, vec3<f32>(0.299, 0.587, 0.114)), 1e-3),
        max(dot(color_b.rgb, vec3<f32>(0.299, 0.587, 0.114)), 1e-3),
        max(dot(color_c.rgb, vec3<f32>(0.299, 0.587, 0.114)), 1e-3),
    );
    let falloff = mix(vec3<f32>(1.0), luminance_weights, vec3<f32>(0.6));
    var reweighted = normalize_weights(falloff * pow(max(weights, vec3<f32>(1e-4)), vec3<f32>(7.0)));

    if should_use_histogram_weights(settings) {
        reweighted = gain3(reweighted, mix(0.58, 0.84, saturate(settings.primary.w)));
    }

    return reweighted;
}

fn normal_reweight(settings: StochasticUniform, weights: vec3<f32>, normal_a: vec3<f32>, normal_b: vec3<f32>, normal_c: vec3<f32>) -> vec3<f32> {
    if settings.state.w != QUALITY_HIGH && settings.space.z == NORMAL_ROTATE_TANGENT {
        return weights;
    }

    let slope = vec3<f32>(
        sqrt(dot(normal_a.xy, normal_a.xy) / max(1.0 + dot(normal_a.xy, normal_a.xy), 1e-4)),
        sqrt(dot(normal_b.xy, normal_b.xy) / max(1.0 + dot(normal_b.xy, normal_b.xy), 1e-4)),
        sqrt(dot(normal_c.xy, normal_c.xy) / max(1.0 + dot(normal_c.xy, normal_c.xy), 1e-4)),
    );
    let falloff = mix(vec3<f32>(1.0), slope, vec3<f32>(0.6));
    return normalize_weights(falloff * pow(max(weights, vec3<f32>(1e-4)), vec3<f32>(7.0)));
}

fn final_weights(
    settings: StochasticUniform,
    height_map: texture_2d<f32>,
    height_sampler: sampler,
    lattice: StochasticLayout,
    frame: SampleFrame,
) -> vec3<f32> {
    var weights = weight_rules(settings, lattice.weights);
    if !state_has_height(settings) || (settings.state.z == BLEND_LINEAR) {
        return weights;
    }

    let frame_a = transformed_frame(settings, frame, lattice.cells[0]);
    let frame_b = transformed_frame(settings, frame, lattice.cells[1]);
    let frame_c = transformed_frame(settings, frame, lattice.cells[2]);
    let heights = vec3<f32>(
        height_texture(settings, height_map, height_sampler, frame_a),
        height_texture(settings, height_map, height_sampler, frame_b),
        height_texture(settings, height_map, height_sampler, frame_c),
    );
    let highest = max(max(heights.x, heights.y), heights.z);
    let emphasis = mix(0.0, select(5.0, 7.0, settings.state.z == BLEND_HISTOGRAM_APPROX), settings.secondary.x);
    let bias = vec3<f32>(
        exp2((heights.x - highest) * emphasis),
        exp2((heights.y - highest) * emphasis),
        exp2((heights.z - highest) * emphasis),
    );
    weights *= bias;
    return normalize_weights(weights);
}
