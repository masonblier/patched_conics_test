use std::f32::consts::PI;

use crate::GameState;
use crate::camera::GameCamera;
use crate::loading::{SettingsConfigAsset,SettingsConfigAssets};
use crate::overlay_ui::{OverlayUiBodyInfo,OverylayUiControls,ViewingBody};
use crate::world::OrbitConic;

use bevy::prelude::*;

// helper macro
macro_rules! deg {
    ($x:expr) => {
        $x * 180. / PI
    }
}

// This plugin renders demo entities
pub struct OrbitsDemoPlugin;
impl Plugin for OrbitsDemoPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::Playing), setup_demo)
            .add_systems(Update, update_demo.run_if(in_state(GameState::Playing)))
            .add_systems(Update, update_demo_controls.run_if(in_state(GameState::Playing)));
    }
}

// colors
const COLORS: [Color; 6] = [Color::GREEN, Color::YELLOW, Color::BLUE, Color::RED, Color::PURPLE, Color::ORANGE];

#[derive(Component)]
pub struct SatEntity {
    pub idx: usize,
    pub vel: Vec3,
    pub conic: OrbitConic,
    pub color: Color,
}

fn setup_demo(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config_handles: Res<SettingsConfigAssets>,
    config_assets: Res<Assets<SettingsConfigAsset>>,
) {
    let settings = config_assets.get(config_handles.settings.clone()).unwrap();

    // body
    let sphere_mesh = Sphere::default().mesh().ico(5).unwrap();
    let mesh_handle = meshes.add(sphere_mesh);
    let material_handle = materials.add(StandardMaterial {
        base_color: Color::DARK_GRAY,
        ..default()
    });
    commands.spawn(MaterialMeshBundle {
        mesh: mesh_handle.clone(),
        material: material_handle.clone(),
        transform: Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(settings.body_scale)),
        ..Default::default()
    });

    // satellites
    for (idx, sat) in settings.satellites.iter().enumerate() {

        let conic = OrbitConic::from_initial(
            sat.initial_pos,
            sat.initial_vel,
            settings.body_mass,
            Vec3::Y);
        let mesh = Sphere::default().mesh().ico(5).unwrap();
        let mesh_handle = meshes.add(mesh);
        let color = COLORS[idx % COLORS.len()];
        let mat = materials.add(StandardMaterial {
            base_color: color.clone(),
            ..default()
        });
        commands.spawn((MaterialMeshBundle {
            mesh: mesh_handle.clone(),
            material: mat.clone(),
            transform: Transform::from_translation(sat.initial_pos).with_scale(Vec3::splat(sat.scale)),
            ..Default::default()
        }, SatEntity {
            idx,
            vel: sat.initial_vel,
            conic: conic,
            color,
         }));

    }
}

fn update_demo(
    time: Res<Time>,
    controls: Res<OverylayUiControls>,
    mut gizmos: Gizmos,
    mut sat_query: Query<(&mut Transform, &mut SatEntity), Without<GameCamera>>,
    mut body_info_query: Query<&mut Text, With<OverlayUiBodyInfo>>,
    mut camera_query: Query<&mut Transform, With<GameCamera>>,
    config_handles: Res<SettingsConfigAssets>,
    config_assets: Res<Assets<SettingsConfigAsset>>,
) {
    // update sat entity
    for (mut sat_transform, mut sat) in &mut sat_query {
        sat_transform.translation += sat.vel * time.delta_seconds();
        let dv = sat.conic.dv_at_pos(sat_transform.translation);
        sat.vel += dv * time.delta_seconds();

        // draw conic path
        draw_conic_path(sat.conic, &mut gizmos, sat.color.clone());

        // update body info ui
        if ViewingBody::Satellite(sat.idx) == controls.viewing_body {
            // current true anomoly
            let t_nu = sat.conic.nu_at_pos(sat_transform.translation);
            let t = sat.conic.t_at_nu(t_nu);

            let mut body_info = body_info_query.single_mut();
            body_info.sections[0].value = format!("Satellite {}:\n\
                p: {:.2},{:.2},{:.2}, v: {:.2},{:.2},{:.2}\n\
                h: {:.2}, i: {:.2}°, e: {:.2}\n\
                Ω: {:.2}°, ω: {:.2}°, ν: {:.2}°\n\
                t_ν: {:.2}°, t: {:.2}",
                sat.idx,
                sat_transform.translation.x, sat_transform.translation.y, sat_transform.translation.z,
                sat.vel.x, sat.vel.y, sat.vel.z,
                sat.conic.h, deg!(sat.conic.i), sat.conic.e,
                deg!(sat.conic.big_omega), deg!(sat.conic.omega), deg!(sat.conic.nu),
                deg!(t_nu), t);

            // update camera
            let settings = config_assets.get(config_handles.settings.clone()).unwrap();
            let mut camera_transform = camera_query.single_mut();
            camera_transform.translation = sat_transform.translation + settings.camera_pos;
            camera_transform.look_at(sat_transform.translation, Vec3::Y);
        }

        // reset position of sat on parabolic trajectory when out-of-bounds
        if sat_transform.translation.length() > 20. {
            sat_transform.translation.x = -sat_transform.translation.x;
            sat.vel.z = -sat.vel.z;
        }
    }
}

fn update_demo_controls(
    mut controls: ResMut<OverylayUiControls>,
    mut body_info_query: Query<&mut Text, With<OverlayUiBodyInfo>>,
    config_handles: Res<SettingsConfigAssets>,
    config_assets: Res<Assets<SettingsConfigAsset>>,
    mut camera_query: Query<&mut Transform, With<GameCamera>>,
    key: Res<ButtonInput<KeyCode>>,
) {
    let settings = config_assets.get(config_handles.settings.clone()).unwrap();

    // check for tab for next body
    if key.just_pressed(KeyCode::Tab) {
        if let ViewingBody::Satellite(idx) = controls.viewing_body {
            if idx + 1 == settings.satellites.len() {
                controls.viewing_body = ViewingBody::None;
            } else {
                controls.viewing_body = ViewingBody::Satellite(idx + 1);
            }
        } else {
            controls.viewing_body = ViewingBody::Satellite(0);
        }
    }

    // body info if no sat
    if ViewingBody::None == controls.viewing_body {
        let mut body_info = body_info_query.single_mut();
        body_info.sections[0].value = format!("Body:\n\
            p: {:.2},{:.2},{:.2}\n\
            m: {:.2}",
            0., 0., 0., settings.body_mass);

        // update camera
        let mut camera_transform = camera_query.single_mut();
        camera_transform.translation = settings.camera_pos;
        camera_transform.look_at(settings.camera_look_at, Vec3::Y);
    }
}

fn draw_conic_path(
    conic: OrbitConic,
    gizmos: &mut Gizmos,
    color: Color,
) {
    const STEPS: i32 = 128;
    for n in 0..STEPS {
        let theta1 = (n as f32 - 0.5) * 2. * PI / (STEPS as f32);
        let theta2 = (n as f32 + 0.5) * 2. * PI / (STEPS as f32);
        let r1 = conic.r_at_theta(theta1);
        if r1 > 30. || r1 < 0. {
            continue;
        }
        let r2 = conic.r_at_theta(theta2);
        const NUDGE: f32 = 0.05; // todo wtf
        let x_vec = conic.initial_r.cross(conic.h_vec).normalize();
        let z_vec = conic.initial_r.normalize();
        let d1 = x_vec * f32::sin(conic.nu + theta1 + NUDGE) + z_vec * f32::cos(conic.nu + theta1 + NUDGE);
        let d2 = x_vec * f32::sin(conic.nu + theta2 + NUDGE) + z_vec * f32::cos(conic.nu + theta2 + NUDGE);
        gizmos.ray(
            d1 * r2,
            d2 * r2 - d1 * r1,
            color,
        );
    }
}
