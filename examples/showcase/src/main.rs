use bevy::prelude::*;
use saddle_rendering_stochastic_texturing_example_common as common;

fn main() {
    let mut app = App::new();
    common::install_common_plugins(&mut app, "Stochastic Texturing - Showcase");
    common::install_pane(&mut app);
    common::install_auto_exit(&mut app, "STOCHASTIC_TEXTURING_EXAMPLE_SECONDS");
    common::add_example_systems(&mut app);
    app.insert_resource(common::ExampleSceneMode(common::ExampleMode::Lab));
    app.insert_resource(common::ExampleSceneText {
        title: "Stochastic Texturing".into(),
        subtitle:
            "Reference-style multi-demo gallery with live saddle-pane controls and bundled CC0 materials."
                .into(),
    });
    app.add_systems(Startup, common::setup_scene);
    app.run();
}
