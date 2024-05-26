use std::f32::consts::PI;

use crate::GameState;
use crate::loading::{SettingsConfigAsset,SettingsConfigAssets};
use crate::world::{dv_at_pos, r_at_theta};

use bevy::prelude::*;


// This plugin renders demo entities
pub struct OrbitsDemoPlugin;
impl Plugin for OrbitsDemoPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_gizmo_group::<MyRoundGizmos>()
            .add_systems(OnEnter(GameState::Playing), setup_demo)
            .add_systems(Update, update_demo.run_if(in_state(GameState::Playing)));
    }
}

// We can create our own gizmo config group!
#[derive(Default, Reflect, GizmoConfigGroup)]
struct MyRoundGizmos {}

#[derive(Component)]
pub struct SatEntity {
    pub vel: Vec3,
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

    // o0
    let o0_initial_pos = Vec3::Z * r_at_theta(settings.body_fac, settings.o0_ecc, 0.);
    let o0_mesh = Sphere::default().mesh().ico(5).unwrap();
    let o0_mesh_handle = meshes.add(o0_mesh);
    let o0_mat = materials.add(StandardMaterial {
        base_color: Color::YELLOW,
        ..default()
    });
    commands.spawn((MaterialMeshBundle {
        mesh: o0_mesh_handle.clone(),
        material: o0_mat.clone(),
        transform: Transform::from_translation(o0_initial_pos).with_scale(Vec3::splat(settings.sat_scale)),
        ..Default::default()
    }, SatEntity { vel: settings.o0_initial_vel }));

    // o1
    let o1_initial_pos = Vec3::Z * r_at_theta(settings.body_fac, settings.o1_ecc, 0.);
    let o1_mesh = Sphere::default().mesh().ico(5).unwrap();
    let o1_mesh_handle = meshes.add(o1_mesh);
    let o1_mat = materials.add(StandardMaterial {
        base_color: Color::BLUE,
        ..default()
    });
    commands.spawn((MaterialMeshBundle {
        mesh: o1_mesh_handle.clone(),
        material: o1_mat.clone(),
        transform: Transform::from_translation(o1_initial_pos).with_scale(Vec3::splat(settings.sat_scale)),
        ..Default::default()
    }, SatEntity { vel: settings.o1_initial_vel }));


    // o2
    let o2_initial_pos = Vec3::Z * r_at_theta(settings.body_fac, settings.o2_ecc, 0.);
    let o2_mesh = Sphere::default().mesh().ico(5).unwrap();
    let o2_mesh_handle = meshes.add(o2_mesh);
    let o2_mat = materials.add(StandardMaterial {
        base_color: Color::GREEN,
        ..default()
    });
    commands.spawn((MaterialMeshBundle {
        mesh: o2_mesh_handle.clone(),
        material: o2_mat.clone(),
        transform: Transform::from_translation(o2_initial_pos).with_scale(Vec3::splat(settings.sat_scale)),
        ..Default::default()
    }, SatEntity { vel: settings.o2_initial_vel }));


    // o3
    let o3_initial_pos = Vec3::Z * r_at_theta(settings.body_fac, settings.o3_ecc, 0.);
    let o3_mesh = Sphere::default().mesh().ico(5).unwrap();
    let o3_mesh_handle = meshes.add(o3_mesh);
    let o3_mat = materials.add(StandardMaterial {
        base_color: Color::RED,
        ..default()
    });
    commands.spawn((MaterialMeshBundle {
        mesh: o3_mesh_handle.clone(),
        material: o3_mat.clone(),
        transform: Transform::from_translation(o3_initial_pos).with_scale(Vec3::splat(settings.sat_scale)),
        ..Default::default()
    }, SatEntity { vel: settings.o3_initial_vel }));
}


fn update_demo(
    time: Res<Time>,
    mut gizmos: Gizmos,
    mut sat_query: Query<(&mut Transform, &mut SatEntity)>,
    config_handles: Res<SettingsConfigAssets>,
    config_assets: Res<Assets<SettingsConfigAsset>>,
) {
    let settings = config_assets.get(config_handles.settings.clone()).unwrap();

    // update sat entity
    for (mut sat_transform, mut sat_query) in &mut sat_query {
        sat_transform.translation += sat_query.vel * time.delta_seconds();
        sat_query.vel += dv_at_pos(time.delta_seconds(), settings.body_fac, sat_transform.translation);

        // reset position of sat on parabolic trajectory when out-of-bounds
        if sat_transform.translation.length() > 20. {
            sat_transform.translation.x = -sat_transform.translation.x;
            sat_query.vel.z = -sat_query.vel.z;
        }
    }
    
    // draw conics
    draw_conic_path(settings.body_fac, settings.o0_ecc, &mut gizmos, Color::YELLOW);
    draw_conic_path(settings.body_fac, settings.o1_ecc, &mut gizmos, Color::BLUE);
    draw_conic_path(settings.body_fac, settings.o2_ecc, &mut gizmos, Color::GREEN);
    draw_conic_path(settings.body_fac, settings.o3_ecc, &mut gizmos, Color::RED);
}

fn draw_conic_path(
    fac: f32,
    ecc: f32,
    gizmos: &mut Gizmos,
    color: Color,
) {
    const STEPS: i32 = 128;
    for n in 0..STEPS {
        let theta1 = (n as f32 - 0.5) * 2. * PI / (STEPS as f32);
        let theta2 = (n as f32 + 0.5) * 2. * PI / (STEPS as f32);
        let r1 = r_at_theta(fac, ecc, theta1);
        if r1 > 30. {
            continue;
        }
        let r2 = r_at_theta(fac, ecc, theta2);
        const NUDGE: f32 = 0.02; // todo wtf
        let d1 = Vec3::new(f32::sin(theta1 + NUDGE), 0., f32::cos(theta1 + NUDGE));
        let d2 = Vec3::new(f32::sin(theta2 + NUDGE), 0., f32::cos(theta2 + NUDGE));
        gizmos.ray(
            d1 * r2,
            d2 * r2 - d1 * r1,
            color,
        );
    }
}
