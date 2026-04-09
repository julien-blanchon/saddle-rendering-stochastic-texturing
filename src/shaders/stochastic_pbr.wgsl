#import bevy_pbr::{
    decal::clustered::apply_decals,
    pbr_bindings,
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{
        alpha_discard,
        apply_normal_mapping,
        apply_pbr_lighting,
        calculate_tbn_mikktspace,
        main_pass_post_lighting_processing,
    },
    pbr_types,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    pbr_deferred_functions::deferred_output,
    prepass_io::{FragmentOutput, VertexOutput},
}
#else
#import bevy_pbr::{
    forward_io::{FragmentOutput, VertexOutput},
    pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT,
}
#endif

#ifdef VISIBILITY_RANGE_DITHER
#import bevy_pbr::pbr_functions::visibility_range_dither
#endif

#ifdef MESHLET_MESH_MATERIAL_PASS
#import bevy_pbr::meshlet_visibility_buffer_resolve::resolve_vertex_output
#endif

#ifdef OIT_ENABLED
#import bevy_core_pipeline::oit::oit_draw
#endif

#ifdef FORWARD_DECAL
#import bevy_pbr::decal::forward::get_forward_decal_info
#endif

#ifdef BINDLESS
#import bevy_render::bindless::{bindless_samplers_filtering, bindless_textures_2d}
#import bevy_pbr::{mesh_bindings::mesh, pbr_bindings::material_indices}
#endif

#import stochastic_texturing::types::{
    SampleFrame,
    StochasticLayout,
    StochasticUniform,
    TextureBombLayout,
}

#import stochastic_texturing::sampling::{
    decode_tangent_normal,
    final_weights,
    histogram_reweight,
    lattice_layout,
    normal_reweight,
    projected_world_scale,
    rotation_matrix,
    state_enabled,
    state_has_height,
    tangent_to_world,
    texture_bomb_layout,
    transformed_frame,
    triplanar_weights,
    triplanar_world_normal_x,
    triplanar_world_normal_y,
    triplanar_world_normal_z,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(100) var<uniform> stochastic: StochasticUniform;
@group(#{MATERIAL_BIND_GROUP}) @binding(101) var stochastic_height_map: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(102) var stochastic_height_sampler: sampler;

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

const HEIGHT_RED: u32 = 0u;
const HEIGHT_GREEN: u32 = 1u;
const HEIGHT_BLUE: u32 = 2u;
const HEIGHT_ALPHA: u32 = 3u;
const HEIGHT_LUMINANCE: u32 = 4u;

const PI: f32 = 3.141592653589793;

fn material_slot(in: VertexOutput) -> u32 {
#ifdef BINDLESS
#ifdef MESHLET_MESH_MATERIAL_PASS
    return in.material_bind_group_slot;
#else
    return mesh[in.instance_index].material_and_lightmap_bind_group_slot & 0xffffu;
#endif
#else
    return 0u;
#endif
}

fn base_color_factor(slot: u32) -> vec4<f32> {
#ifdef BINDLESS
    return pbr_bindings::material_array[material_indices[slot].material].base_color;
#else
    return pbr_bindings::material.base_color;
#endif
}

fn emissive_factor(slot: u32) -> vec4<f32> {
#ifdef BINDLESS
    return pbr_bindings::material_array[material_indices[slot].material].emissive;
#else
    return pbr_bindings::material.emissive;
#endif
}

fn metallic_factor(slot: u32) -> f32 {
#ifdef BINDLESS
    return pbr_bindings::material_array[material_indices[slot].material].metallic;
#else
    return pbr_bindings::material.metallic;
#endif
}

fn roughness_factor(slot: u32) -> f32 {
#ifdef BINDLESS
    return pbr_bindings::material_array[material_indices[slot].material].perceptual_roughness;
#else
    return pbr_bindings::material.perceptual_roughness;
#endif
}

fn base_color_texture(flags: u32, slot: u32, uv: vec2<f32>, ddx_uv: vec2<f32>, ddy_uv: vec2<f32>) -> vec4<f32> {
    if (flags & pbr_types::STANDARD_MATERIAL_FLAGS_BASE_COLOR_TEXTURE_BIT) == 0u {
        return vec4<f32>(1.0);
    }
#ifdef BINDLESS
    return textureSampleGrad(
        bindless_textures_2d[material_indices[slot].base_color_texture],
        bindless_samplers_filtering[material_indices[slot].base_color_sampler],
        uv,
        ddx_uv,
        ddy_uv,
    );
#else
    return textureSampleGrad(
        pbr_bindings::base_color_texture,
        pbr_bindings::base_color_sampler,
        uv,
        ddx_uv,
        ddy_uv,
    );
#endif
}

fn emissive_texture(flags: u32, slot: u32, uv: vec2<f32>, ddx_uv: vec2<f32>, ddy_uv: vec2<f32>) -> vec3<f32> {
    if (flags & pbr_types::STANDARD_MATERIAL_FLAGS_EMISSIVE_TEXTURE_BIT) == 0u {
        return vec3<f32>(1.0);
    }
#ifdef BINDLESS
    return textureSampleGrad(
        bindless_textures_2d[material_indices[slot].emissive_texture],
        bindless_samplers_filtering[material_indices[slot].emissive_sampler],
        uv,
        ddx_uv,
        ddy_uv,
    ).rgb;
#else
    return textureSampleGrad(
        pbr_bindings::emissive_texture,
        pbr_bindings::emissive_sampler,
        uv,
        ddx_uv,
        ddy_uv,
    ).rgb;
#endif
}

fn orm_texture(flags: u32, slot: u32, uv: vec2<f32>, ddx_uv: vec2<f32>, ddy_uv: vec2<f32>) -> vec4<f32> {
    if (flags & pbr_types::STANDARD_MATERIAL_FLAGS_METALLIC_ROUGHNESS_TEXTURE_BIT) == 0u {
        return vec4<f32>(1.0);
    }
#ifdef BINDLESS
    return textureSampleGrad(
        bindless_textures_2d[material_indices[slot].metallic_roughness_texture],
        bindless_samplers_filtering[material_indices[slot].metallic_roughness_sampler],
        uv,
        ddx_uv,
        ddy_uv,
    );
#else
    return textureSampleGrad(
        pbr_bindings::metallic_roughness_texture,
        pbr_bindings::metallic_roughness_sampler,
        uv,
        ddx_uv,
        ddy_uv,
    );
#endif
}

fn normal_texture(flags: u32, slot: u32, uv: vec2<f32>, ddx_uv: vec2<f32>, ddy_uv: vec2<f32>) -> vec3<f32> {
    if (flags & pbr_types::STANDARD_MATERIAL_FLAGS_TWO_COMPONENT_NORMAL_MAP) == 0u &&
        (flags & pbr_types::STANDARD_MATERIAL_FLAGS_FLIP_NORMAL_MAP_Y) == 0u &&
        (flags & pbr_types::STANDARD_MATERIAL_FLAGS_BASE_COLOR_TEXTURE_BIT) == 0u &&
        (flags & pbr_types::STANDARD_MATERIAL_FLAGS_OCCLUSION_TEXTURE_BIT) == 0u {
        // no-op placeholder branch so the compiler keeps the flags live across all paths
    }
#ifdef BINDLESS
    return textureSampleGrad(
        bindless_textures_2d[material_indices[slot].normal_map_texture],
        bindless_samplers_filtering[material_indices[slot].normal_map_sampler],
        uv,
        ddx_uv,
        ddy_uv,
    ).xyz;
#else
    return textureSampleGrad(
        pbr_bindings::normal_map_texture,
        pbr_bindings::normal_map_sampler,
        uv,
        ddx_uv,
        ddy_uv,
    ).xyz;
#endif
}

fn occlusion_texture(flags: u32, slot: u32, uv: vec2<f32>, ddx_uv: vec2<f32>, ddy_uv: vec2<f32>) -> f32 {
    if (flags & pbr_types::STANDARD_MATERIAL_FLAGS_OCCLUSION_TEXTURE_BIT) == 0u {
        return 1.0;
    }
#ifdef BINDLESS
    return textureSampleGrad(
        bindless_textures_2d[material_indices[slot].occlusion_texture],
        bindless_samplers_filtering[material_indices[slot].occlusion_sampler],
        uv,
        ddx_uv,
        ddy_uv,
    ).r;
#else
    return textureSampleGrad(
        pbr_bindings::occlusion_texture,
        pbr_bindings::occlusion_sampler,
        uv,
        ddx_uv,
        ddy_uv,
    ).r;
#endif
}

fn mesh_frame(in: VertexOutput, transformed_uv: vec2<f32>) -> SampleFrame {
    return SampleFrame(transformed_uv, dpdx(transformed_uv), dpdy(transformed_uv));
}

fn sample_stochastic_base(frame: SampleFrame, flags: u32, slot: u32) -> vec4<f32> {
    if stochastic.state.y == SAMPLING_TEXTURE_BOMBING {
        let bombing = texture_bomb_layout(stochastic, frame.uv);
        let sample_a = transformed_frame(stochastic, frame, bombing.cells[0]);
        let sample_b = transformed_frame(stochastic, frame, bombing.cells[1]);
        let sample_c = transformed_frame(stochastic, frame, bombing.cells[2]);
        let sample_d = transformed_frame(stochastic, frame, bombing.cells[3]);
        var color = vec4<f32>(0.0);
        if bombing.weights.x > 0.0 { color += base_color_texture(flags, slot, sample_a.uv, sample_a.ddx_uv, sample_a.ddy_uv) * bombing.weights.x; }
        if bombing.weights.y > 0.0 { color += base_color_texture(flags, slot, sample_b.uv, sample_b.ddx_uv, sample_b.ddy_uv) * bombing.weights.y; }
        if bombing.weights.z > 0.0 { color += base_color_texture(flags, slot, sample_c.uv, sample_c.ddx_uv, sample_c.ddy_uv) * bombing.weights.z; }
        if bombing.weights.w > 0.0 { color += base_color_texture(flags, slot, sample_d.uv, sample_d.ddx_uv, sample_d.ddy_uv) * bombing.weights.w; }
        return color;
    }

    let lattice = lattice_layout(frame.uv);
    let base_weights = final_weights(stochastic, stochastic_height_map, stochastic_height_sampler, lattice, frame);
    let sample_a = transformed_frame(stochastic, frame, lattice.cells[0]);
    let sample_b = transformed_frame(stochastic, frame, lattice.cells[1]);
    let sample_c = transformed_frame(stochastic, frame, lattice.cells[2]);
    let color_a = base_color_texture(flags, slot, sample_a.uv, sample_a.ddx_uv, sample_a.ddy_uv);
    let color_b = base_color_texture(flags, slot, sample_b.uv, sample_b.ddx_uv, sample_b.ddy_uv);
    let color_c = base_color_texture(flags, slot, sample_c.uv, sample_c.ddx_uv, sample_c.ddy_uv);
    let weights = histogram_reweight(stochastic, base_weights, color_a, color_b, color_c);
    return color_a * weights.x + color_b * weights.y + color_c * weights.z;
}

fn sample_stochastic_emissive(frame: SampleFrame, flags: u32, slot: u32) -> vec3<f32> {
    if stochastic.state.y == SAMPLING_TEXTURE_BOMBING {
        let bombing = texture_bomb_layout(stochastic, frame.uv);
        let sample_a = transformed_frame(stochastic, frame, bombing.cells[0]);
        let sample_b = transformed_frame(stochastic, frame, bombing.cells[1]);
        let sample_c = transformed_frame(stochastic, frame, bombing.cells[2]);
        let sample_d = transformed_frame(stochastic, frame, bombing.cells[3]);
        var emissive = vec3<f32>(0.0);
        if bombing.weights.x > 0.0 { emissive += emissive_texture(flags, slot, sample_a.uv, sample_a.ddx_uv, sample_a.ddy_uv) * bombing.weights.x; }
        if bombing.weights.y > 0.0 { emissive += emissive_texture(flags, slot, sample_b.uv, sample_b.ddx_uv, sample_b.ddy_uv) * bombing.weights.y; }
        if bombing.weights.z > 0.0 { emissive += emissive_texture(flags, slot, sample_c.uv, sample_c.ddx_uv, sample_c.ddy_uv) * bombing.weights.z; }
        if bombing.weights.w > 0.0 { emissive += emissive_texture(flags, slot, sample_d.uv, sample_d.ddx_uv, sample_d.ddy_uv) * bombing.weights.w; }
        return emissive;
    }

    let lattice = lattice_layout(frame.uv);
    let weights = final_weights(stochastic, stochastic_height_map, stochastic_height_sampler, lattice, frame);
    let sample_a = transformed_frame(stochastic, frame, lattice.cells[0]);
    let sample_b = transformed_frame(stochastic, frame, lattice.cells[1]);
    let sample_c = transformed_frame(stochastic, frame, lattice.cells[2]);
    var emissive = vec3<f32>(0.0);
    if weights.x > 0.0 { emissive += emissive_texture(flags, slot, sample_a.uv, sample_a.ddx_uv, sample_a.ddy_uv) * weights.x; }
    if weights.y > 0.0 { emissive += emissive_texture(flags, slot, sample_b.uv, sample_b.ddx_uv, sample_b.ddy_uv) * weights.y; }
    if weights.z > 0.0 { emissive += emissive_texture(flags, slot, sample_c.uv, sample_c.ddx_uv, sample_c.ddy_uv) * weights.z; }
    return emissive;
}

fn sample_stochastic_orm(frame: SampleFrame, flags: u32, slot: u32) -> vec4<f32> {
    if stochastic.state.y == SAMPLING_TEXTURE_BOMBING {
        let bombing = texture_bomb_layout(stochastic, frame.uv);
        let sample_a = transformed_frame(stochastic, frame, bombing.cells[0]);
        let sample_b = transformed_frame(stochastic, frame, bombing.cells[1]);
        let sample_c = transformed_frame(stochastic, frame, bombing.cells[2]);
        let sample_d = transformed_frame(stochastic, frame, bombing.cells[3]);
        var orm = vec4<f32>(0.0);
        if bombing.weights.x > 0.0 { orm += orm_texture(flags, slot, sample_a.uv, sample_a.ddx_uv, sample_a.ddy_uv) * bombing.weights.x; }
        if bombing.weights.y > 0.0 { orm += orm_texture(flags, slot, sample_b.uv, sample_b.ddx_uv, sample_b.ddy_uv) * bombing.weights.y; }
        if bombing.weights.z > 0.0 { orm += orm_texture(flags, slot, sample_c.uv, sample_c.ddx_uv, sample_c.ddy_uv) * bombing.weights.z; }
        if bombing.weights.w > 0.0 { orm += orm_texture(flags, slot, sample_d.uv, sample_d.ddx_uv, sample_d.ddy_uv) * bombing.weights.w; }
        return orm;
    }

    let lattice = lattice_layout(frame.uv);
    let weights = final_weights(stochastic, stochastic_height_map, stochastic_height_sampler, lattice, frame);
    let sample_a = transformed_frame(stochastic, frame, lattice.cells[0]);
    let sample_b = transformed_frame(stochastic, frame, lattice.cells[1]);
    let sample_c = transformed_frame(stochastic, frame, lattice.cells[2]);
    var orm = vec4<f32>(0.0);
    if weights.x > 0.0 { orm += orm_texture(flags, slot, sample_a.uv, sample_a.ddx_uv, sample_a.ddy_uv) * weights.x; }
    if weights.y > 0.0 { orm += orm_texture(flags, slot, sample_b.uv, sample_b.ddx_uv, sample_b.ddy_uv) * weights.y; }
    if weights.z > 0.0 { orm += orm_texture(flags, slot, sample_c.uv, sample_c.ddx_uv, sample_c.ddy_uv) * weights.z; }
    return orm;
}

fn sample_stochastic_occlusion(frame: SampleFrame, flags: u32, slot: u32) -> f32 {
    if stochastic.state.y == SAMPLING_TEXTURE_BOMBING {
        let bombing = texture_bomb_layout(stochastic, frame.uv);
        let sample_a = transformed_frame(stochastic, frame, bombing.cells[0]);
        let sample_b = transformed_frame(stochastic, frame, bombing.cells[1]);
        let sample_c = transformed_frame(stochastic, frame, bombing.cells[2]);
        let sample_d = transformed_frame(stochastic, frame, bombing.cells[3]);
        var occlusion = 0.0;
        if bombing.weights.x > 0.0 { occlusion += occlusion_texture(flags, slot, sample_a.uv, sample_a.ddx_uv, sample_a.ddy_uv) * bombing.weights.x; }
        if bombing.weights.y > 0.0 { occlusion += occlusion_texture(flags, slot, sample_b.uv, sample_b.ddx_uv, sample_b.ddy_uv) * bombing.weights.y; }
        if bombing.weights.z > 0.0 { occlusion += occlusion_texture(flags, slot, sample_c.uv, sample_c.ddx_uv, sample_c.ddy_uv) * bombing.weights.z; }
        if bombing.weights.w > 0.0 { occlusion += occlusion_texture(flags, slot, sample_d.uv, sample_d.ddx_uv, sample_d.ddy_uv) * bombing.weights.w; }
        return occlusion;
    }

    let lattice = lattice_layout(frame.uv);
    let weights = final_weights(stochastic, stochastic_height_map, stochastic_height_sampler, lattice, frame);
    let sample_a = transformed_frame(stochastic, frame, lattice.cells[0]);
    let sample_b = transformed_frame(stochastic, frame, lattice.cells[1]);
    let sample_c = transformed_frame(stochastic, frame, lattice.cells[2]);
    var occlusion = 0.0;
    if weights.x > 0.0 { occlusion += occlusion_texture(flags, slot, sample_a.uv, sample_a.ddx_uv, sample_a.ddy_uv) * weights.x; }
    if weights.y > 0.0 { occlusion += occlusion_texture(flags, slot, sample_b.uv, sample_b.ddx_uv, sample_b.ddy_uv) * weights.y; }
    if weights.z > 0.0 { occlusion += occlusion_texture(flags, slot, sample_c.uv, sample_c.ddx_uv, sample_c.ddy_uv) * weights.z; }
    return occlusion;
}

fn sample_stochastic_normal_tangent(frame: SampleFrame, flags: u32, slot: u32) -> vec3<f32> {
    if stochastic.state.y == SAMPLING_TEXTURE_BOMBING {
        let bombing = texture_bomb_layout(stochastic, frame.uv);
        let sample_a = transformed_frame(stochastic, frame, bombing.cells[0]);
        let sample_b = transformed_frame(stochastic, frame, bombing.cells[1]);
        let sample_c = transformed_frame(stochastic, frame, bombing.cells[2]);
        let sample_d = transformed_frame(stochastic, frame, bombing.cells[3]);

        var normal_a = decode_tangent_normal(flags, normal_texture(flags, slot, sample_a.uv, sample_a.ddx_uv, sample_a.ddy_uv));
        var normal_b = decode_tangent_normal(flags, normal_texture(flags, slot, sample_b.uv, sample_b.ddx_uv, sample_b.ddy_uv));
        var normal_c = decode_tangent_normal(flags, normal_texture(flags, slot, sample_c.uv, sample_c.ddx_uv, sample_c.ddy_uv));
        var normal_d = decode_tangent_normal(flags, normal_texture(flags, slot, sample_d.uv, sample_d.ddx_uv, sample_d.ddy_uv));

        if stochastic.space.z != NORMAL_DISABLED {
            normal_a = vec3<f32>(rotation_matrix(stochastic, bombing.cells[0]) * normal_a.xy, normal_a.z);
            normal_b = vec3<f32>(rotation_matrix(stochastic, bombing.cells[1]) * normal_b.xy, normal_b.z);
            normal_c = vec3<f32>(rotation_matrix(stochastic, bombing.cells[2]) * normal_c.xy, normal_c.z);
            normal_d = vec3<f32>(rotation_matrix(stochastic, bombing.cells[3]) * normal_d.xy, normal_d.z);
        }

        var tangent_normal = vec3<f32>(0.0);
        if bombing.weights.x > 0.0 { tangent_normal += normal_a * bombing.weights.x; }
        if bombing.weights.y > 0.0 { tangent_normal += normal_b * bombing.weights.y; }
        if bombing.weights.z > 0.0 { tangent_normal += normal_c * bombing.weights.z; }
        if bombing.weights.w > 0.0 { tangent_normal += normal_d * bombing.weights.w; }
        return normalize(tangent_normal);
    }

    let lattice = lattice_layout(frame.uv);
    let base_weights = final_weights(stochastic, stochastic_height_map, stochastic_height_sampler, lattice, frame);
    let sample_a = transformed_frame(stochastic, frame, lattice.cells[0]);
    let sample_b = transformed_frame(stochastic, frame, lattice.cells[1]);
    let sample_c = transformed_frame(stochastic, frame, lattice.cells[2]);

    var normal_a = decode_tangent_normal(flags, normal_texture(flags, slot, sample_a.uv, sample_a.ddx_uv, sample_a.ddy_uv));
    var normal_b = decode_tangent_normal(flags, normal_texture(flags, slot, sample_b.uv, sample_b.ddx_uv, sample_b.ddy_uv));
    var normal_c = decode_tangent_normal(flags, normal_texture(flags, slot, sample_c.uv, sample_c.ddx_uv, sample_c.ddy_uv));

    if stochastic.space.z != NORMAL_DISABLED {
        normal_a = vec3<f32>(rotation_matrix(stochastic, lattice.cells[0]) * normal_a.xy, normal_a.z);
        normal_b = vec3<f32>(rotation_matrix(stochastic, lattice.cells[1]) * normal_b.xy, normal_b.z);
        normal_c = vec3<f32>(rotation_matrix(stochastic, lattice.cells[2]) * normal_c.xy, normal_c.z);
    }

    let weights = normal_reweight(stochastic, base_weights, normal_a, normal_b, normal_c);
    return normalize(normal_a * weights.x + normal_b * weights.y + normal_c * weights.z);
}

fn apply_stochastic_texturing(in: VertexOutput, is_front: bool, pbr_input: ptr<function, pbr_types::PbrInput>) {
    if !state_enabled(stochastic) || stochastic.state.y == SAMPLING_OFF {
        return;
    }

#ifndef VERTEX_UVS
    return;
#endif

    let slot = material_slot(in);
    let flags = (*pbr_input).material.flags;
    let uv_transform = (*pbr_input).material.uv_transform;

    var mesh_uv = (uv_transform * vec3<f32>(in.uv, 1.0)).xy * stochastic.primary.xy;
    var mesh_uv_b = mesh_uv;
#ifdef VERTEX_UVS_B
    mesh_uv_b = (uv_transform * vec3<f32>(in.uv_b, 1.0)).xy * stochastic.primary.xy;
#endif

    let frame = mesh_frame(in, select(mesh_uv, mesh_uv_b, stochastic.space.x == UV_MESH_UV1));

    var base_sample = sample_stochastic_base(frame, flags, slot);
    var emissive_sample = sample_stochastic_emissive(frame, flags, slot);
    var orm_sample = sample_stochastic_orm(frame, flags, slot);
    var occlusion_sample = sample_stochastic_occlusion(frame, flags, slot);

    if stochastic.space.x == UV_WORLD_TRIPLANAR {
        let world_scale = projected_world_scale(stochastic, uv_transform);
        let normal = normalize((*pbr_input).world_normal);
        let blend = triplanar_weights(normal);
        let sign_x = select(-1.0, 1.0, normal.x >= 0.0);
        let sign_y = select(-1.0, 1.0, normal.y >= 0.0);
        let sign_z = select(-1.0, 1.0, normal.z >= 0.0);

        let frame_x = mesh_frame(in, vec2<f32>((*pbr_input).world_position.z * sign_x, (*pbr_input).world_position.y) * world_scale);
        let frame_y = mesh_frame(in, vec2<f32>((*pbr_input).world_position.x, (*pbr_input).world_position.z * sign_y) * world_scale);
        let frame_z = mesh_frame(in, vec2<f32>((*pbr_input).world_position.x * sign_z, (*pbr_input).world_position.y) * world_scale);

        base_sample =
            sample_stochastic_base(frame_x, flags, slot) * blend.x +
            sample_stochastic_base(frame_y, flags, slot) * blend.y +
            sample_stochastic_base(frame_z, flags, slot) * blend.z;
        emissive_sample =
            sample_stochastic_emissive(frame_x, flags, slot) * blend.x +
            sample_stochastic_emissive(frame_y, flags, slot) * blend.y +
            sample_stochastic_emissive(frame_z, flags, slot) * blend.z;
        orm_sample =
            sample_stochastic_orm(frame_x, flags, slot) * blend.x +
            sample_stochastic_orm(frame_y, flags, slot) * blend.y +
            sample_stochastic_orm(frame_z, flags, slot) * blend.z;
        occlusion_sample =
            sample_stochastic_occlusion(frame_x, flags, slot) * blend.x +
            sample_stochastic_occlusion(frame_y, flags, slot) * blend.y +
            sample_stochastic_occlusion(frame_z, flags, slot) * blend.z;
    }

#ifdef VERTEX_COLORS
    (*pbr_input).material.base_color = base_color_factor(slot) * base_sample * in.color;
#else
    (*pbr_input).material.base_color = base_color_factor(slot) * base_sample;
#endif
    (*pbr_input).material.emissive = vec4<f32>(emissive_factor(slot).rgb * emissive_sample, emissive_factor(slot).a);
    (*pbr_input).material.metallic = metallic_factor(slot) * orm_sample.b;
    (*pbr_input).material.perceptual_roughness = roughness_factor(slot) * orm_sample.g;
    (*pbr_input).diffuse_occlusion *= vec3<f32>(occlusion_sample);
    (*pbr_input).specular_occlusion *= occlusion_sample;

#ifdef VERTEX_TANGENTS
    if stochastic.space.x != UV_WORLD_TRIPLANAR &&
        stochastic.space.z != NORMAL_DISABLED &&
        (flags & pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u
    {
        let TBN = calculate_tbn_mikktspace(in.world_normal, in.world_tangent);
        let tangent_space_normal = sample_stochastic_normal_tangent(frame, flags, slot);
        (*pbr_input).N = tangent_to_world(
            TBN,
            tangent_space_normal,
            (flags & pbr_types::STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u,
            is_front,
        );
    } else if stochastic.space.x == UV_WORLD_TRIPLANAR &&
        stochastic.space.z != NORMAL_DISABLED &&
        (flags & pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u
    {
        let triplanar_normal_x = triplanar_world_normal_x(
            sample_stochastic_normal_tangent(frame_x, flags, slot),
            sign_x,
        );
        let triplanar_normal_y = triplanar_world_normal_y(
            sample_stochastic_normal_tangent(frame_y, flags, slot),
            sign_y,
        );
        let triplanar_normal_z = triplanar_world_normal_z(
            sample_stochastic_normal_tangent(frame_z, flags, slot),
            sign_z,
        );
        (*pbr_input).N = normalize(
            triplanar_normal_x * blend.x +
            triplanar_normal_y * blend.y +
            triplanar_normal_z * blend.z
        );
    } else if stochastic.space.x == UV_WORLD_TRIPLANAR {
        (*pbr_input).N = normalize((*pbr_input).world_normal);
    }
#endif
}

@fragment
fn fragment(
#ifdef MESHLET_MESH_MATERIAL_PASS
    @builtin(position) frag_coord: vec4<f32>,
#else
    vertex_output: VertexOutput,
    @builtin(front_facing) is_front: bool,
#endif
) -> FragmentOutput {
#ifdef MESHLET_MESH_MATERIAL_PASS
    let vertex_output = resolve_vertex_output(frag_coord);
    let is_front = true;
#endif

    var in = vertex_output;

#ifdef VISIBILITY_RANGE_DITHER
    visibility_range_dither(in.position, in.visibility_range_dither);
#endif

#ifdef FORWARD_DECAL
    let forward_decal_info = get_forward_decal_info(in);
    in.world_position = forward_decal_info.world_position;
    in.uv = forward_decal_info.uv;
#endif

    var pbr_input = pbr_input_from_standard_material(in, is_front);
    apply_stochastic_texturing(in, is_front, &pbr_input);

    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);
    apply_decals(&pbr_input);

#ifdef PREPASS_PIPELINE
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    if (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u {
        out.color = apply_pbr_lighting(pbr_input);
    } else {
        out.color = pbr_input.material.base_color;
    }
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif

#ifdef OIT_ENABLED
    let alpha_mode = pbr_input.material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_RESERVED_BITS;
    if alpha_mode != pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_OPAQUE {
        oit_draw(in.position, out.color);
        discard;
    }
#endif

#ifdef FORWARD_DECAL
    out.color.a = min(forward_decal_info.alpha, out.color.a);
#endif

    return out;
}
