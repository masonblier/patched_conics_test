// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::DefaultPlugins;
use patched_conics_test::GamePlugin;

fn main() {
    App::new()
        .insert_resource(Msaa::Off)
        .insert_resource(AssetMetaCheck::Never)
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Patched_conics_test".to_string(),
                // Bind to canvas included in `index.html`
                canvas: Some("#bevy".to_owned()),
                // Tells wasm not to override default event handling, like F5 and Ctrl+R
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(GamePlugin)
        .run();
}
