use bevy::prelude::*;
use saddle_rendering_stochastic_texturing_example_common as common;

fn main() {
    let mut app = App::new();
    common::install_common_plugins(&mut app, "Stochastic Texturing - Terrain Bridge");
    common::install_pane(&mut app);
    common::install_auto_exit(&mut app, "STOCHASTIC_TEXTURING_EXAMPLE_SECONDS");
    common::add_example_systems(&mut app);
    app.insert_resource(common::ExampleSceneMode(common::ExampleMode::TerrainBridge));
    app.insert_resource(common::ExampleSceneText {
        title: "Stochastic Texturing".into(),
        subtitle: "World-triplanar and terrain-oriented authoring for future adapter work.".into(),
    });
    app.add_systems(Startup, common::setup_scene);
    app.run();
}
