use crate::GameState;
use crate::loading::{SettingsConfigAsset,SettingsConfigAssets};

use bevy::{
    prelude::*,
    window::CursorGrabMode,
};

pub struct CameraPlugin;

/// This plugin is responsible for the game camera
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Menu), setup_camera);
        app.add_systems(OnEnter(GameState::Playing), setup_sun);
        app.add_systems(Update, (
            grab_mouse.run_if(in_state(GameState::Playing)),
        ));
    }
}

#[derive(Component)]
pub struct GameCamera;

fn setup_camera(
    mut commands: Commands,
    config_handles: Res<SettingsConfigAssets>,
    config_assets: Res<Assets<SettingsConfigAsset>>,
) {
    let settings = config_assets.get(config_handles.settings.clone()).unwrap();

    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(settings.camera_pos)
                        .looking_at(settings.camera_look_at, Vec3::Y),
            ..Default::default()
        },
        GameCamera { },
    ));    
}

#[derive(Component)]
pub struct GameSunLight;

fn setup_sun(mut commands: Commands) {
    // light
    commands.spawn((DirectionalLightBundle {
        transform: Transform::from_xyz(30.0, 20.0, 10.0)
                    .looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    },
    GameSunLight { }));
}

// This system grabs the mouse when the left mouse button is pressed
// and releases it when the escape key is pressed
fn grab_mouse(
    mut windows: Query<&mut Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
) {
    let mut window = windows.single_mut();

    if mouse.just_pressed(MouseButton::Left) {
        window.cursor.visible = false;
        window.cursor.grab_mode = CursorGrabMode::Locked;
    }

    if key.just_pressed(KeyCode::Escape) {
        window.cursor.visible = true;
        window.cursor.grab_mode = CursorGrabMode::None;
    }
}
