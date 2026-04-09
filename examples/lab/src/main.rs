#[cfg(feature = "e2e")]
mod e2e;
#[cfg(feature = "e2e")]
mod scenarios;

use bevy::prelude::*;
#[cfg(feature = "dev")]
use bevy::remote::{RemotePlugin, http::RemoteHttpPlugin};
#[cfg(feature = "dev")]
use bevy_brp_extras::BrpExtrasPlugin;
use saddle_rendering_stochastic_texturing_example_common as common;

fn main() {
    let mut app = App::new();
    common::install_common_plugins(&mut app, "Stochastic Texturing Lab");
    common::add_example_systems(&mut app);
    app.insert_resource(common::ExampleSceneMode(common::ExampleMode::Lab));
    app.insert_resource(common::ExampleSceneText {
        title: "Stochastic Texturing Lab".into(),
        subtitle: "Integrated scene covering basic, normal-map, height-blend, and terrain-oriented stochastic texturing.".into(),
    });
    #[cfg(feature = "dev")]
    app.add_plugins((
        RemotePlugin::default(),
        BrpExtrasPlugin::with_http_plugin(RemoteHttpPlugin::default()),
    ));
    #[cfg(feature = "e2e")]
    app.add_plugins(e2e::StochasticTexturingLabE2EPlugin);
    app.add_systems(Startup, common::setup_scene);
    app.run();
}
