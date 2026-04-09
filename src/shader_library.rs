use bevy::{
    asset::{AssetApp, Assets, Handle, load_internal_asset, uuid_handle},
    prelude::*,
    shader::Shader,
};

pub const STOCHASTIC_TYPES_IMPORT_PATH: &str = "stochastic_texturing::types";
pub const STOCHASTIC_SAMPLING_IMPORT_PATH: &str = "stochastic_texturing::sampling";

pub(crate) const STOCHASTIC_TYPES_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("1b999ebe-2ce5-44f2-9ff4-ce85f49c6476");
pub(crate) const STOCHASTIC_SAMPLING_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("f6f5ebba-c17f-4e02-8f84-d4d6ed93b565");

pub struct StochasticShaderLibraryPlugin;

impl Plugin for StochasticShaderLibraryPlugin {
    fn build(&self, app: &mut App) {
        plugin(app);
    }
}

pub(crate) fn plugin(app: &mut App) {
    if app.world().contains_resource::<AssetServer>() {
        if !app.world().contains_resource::<Assets<Shader>>() {
            app.init_asset::<Shader>();
        }

        load_internal_asset!(
            app,
            STOCHASTIC_TYPES_SHADER_HANDLE,
            "shaders/stochastic_types.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            STOCHASTIC_SAMPLING_SHADER_HANDLE,
            "shaders/stochastic_sampling.wgsl",
            Shader::from_wgsl
        );
    }
}
