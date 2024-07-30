use std::f32::consts::PI;

use crate::GameState;
use crate::camera::GameCamera;
use crate::loading::{SettingsConfigAsset,SettingsConfigAssets};
use crate::overlay_ui::{OverlayUiBodyInfo,OverylayUiControls,ViewingBody};
use crate::world::OrbitConic;

use bevy::prelude::*;

// TODO load from config
pub const PLANET_MASS: f32 = 3.1e11;
pub const MOON_MASS: f32 = 3.1e10;
pub const MOON_SOI: f32 = 1.;

// helper macro
macro_rules! deg {
    ($x:expr) => {
        $x * 180. / PI
    }
}

#[derive(Default,Resource)]
pub struct SimulationState {
    simulated_time: f32,
}

// This plugin renders demo entities
pub struct OrbitsDemoPlugin;
impl Plugin for OrbitsDemoPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<SimulationState>()
            .add_systems(OnEnter(GameState::Playing), setup_demo)
            .add_systems(Update, update_demo.run_if(in_state(GameState::Playing)))
            .add_systems(Update, update_demo_controls.run_if(in_state(GameState::Playing)));
    }
}

// colors
const COLORS: [Color; 6] = [Color::GREEN, Color::YELLOW, Color::BLUE, Color::RED, Color::PURPLE, Color::ORANGE];

#[derive(Component)]
pub struct MoonEntity {
    pub idx: usize,
    pub vel: Vec3,
    pub conic: OrbitConic,
    pub color: Color,
}

#[derive(Component)]
pub struct SatEntity {
    pub idx: usize,
    pub vel: Vec3,
    pub conic: OrbitConic,
    pub parent_info: Option<ParentInfo>,
    pub color: Color,
}

#[derive(Clone)]
pub struct ParentInfo {
    pub conic: OrbitConic,
    pub entry_time: f32,
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


    // moons
    for (idx, sat) in settings.moons.iter().enumerate() {

        let conic = OrbitConic::from_initial(
            sat.initial_pos,
            sat.initial_vel,
            settings.body_mass,
            Vec3::Y);
        let mesh = Sphere::default().mesh().ico(5).unwrap();
        let mesh_handle = meshes.add(mesh);
        let color = COLORS[idx % COLORS.len()].with_s(0.3);
        let mat = materials.add(StandardMaterial {
            base_color: color.clone(),
            ..default()
        });
        commands.spawn((MaterialMeshBundle {
            mesh: mesh_handle.clone(),
            material: mat.clone(),
            transform: Transform::from_translation(sat.initial_pos).with_scale(Vec3::splat(sat.scale)),
            ..Default::default()
        }, MoonEntity {
            idx,
            vel: sat.initial_vel,
            conic: conic,
            color,
         }));

    }

    // satellites
    for (idx, sat) in settings.satellites.iter().enumerate() {

        let conic = OrbitConic::from_initial(
            sat.initial_pos,
            sat.initial_vel,
            settings.body_mass,
            Vec3::Y);
        let mesh = Sphere::default().mesh().ico(5).unwrap();
        let mesh_handle = meshes.add(mesh);
        let color = COLORS[(COLORS.len() / 2 + idx) % COLORS.len()];
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
            parent_info: None,
            color,
         }));

    }
}

fn update_demo(
    time: Res<Time>,
    mut simulation_state: ResMut<SimulationState>,
    controls: Res<OverylayUiControls>,
    mut gizmos: Gizmos,
    mut transforms: Query<&mut Transform>,
    mut moons_query: Query<(Entity, &mut MoonEntity)>,
    mut sat_query: Query<(Entity, &mut SatEntity)>,
    mut body_info_query: Query<&mut Text, With<OverlayUiBodyInfo>>,
    camera_query: Query<Entity, With<GameCamera>>,
    config_handles: Res<SettingsConfigAssets>,
    config_assets: Res<Assets<SettingsConfigAsset>>,
) {
    // optionally update camera target
    let mut update_camera_target: Option<Vec3> = None;

    // consume time
    let dt = time.delta_seconds();
    simulation_state.simulated_time += dt;

    // update moon entities
    for (sat_entity, mut sat) in &mut moons_query {
        // simulate physics
        let mut sat_transform = transforms.get_mut(sat_entity).unwrap();
        sat_transform.translation += sat.vel * dt;
        let dv = sat.conic.dv_at_pos(sat_transform.translation);
        sat.vel += dv * dt;

        // draw conic path
        draw_conic_path(sat.conic, None, &mut gizmos, sat.color.clone(), None, 0);

        // update body info ui
        if ViewingBody::Moon(sat.idx) == controls.viewing_body {
            let mut body_info = body_info_query.single_mut();
            body_info.sections[0].value = format_body_info("Moon", sat.idx,
                sat.vel, &sat.conic, &sat_transform);

            // update camera
            update_camera_target = Some(sat_transform.translation);
        }

        // reset position of sat on parabolic trajectory when out-of-bounds
        if sat_transform.translation.length() > 20. {
            sat_transform.translation.x = -sat_transform.translation.x;
            sat.vel.z = -sat.vel.z;
        }
    }

    // update sat entities
    for (sat_entity, mut sat) in &mut sat_query {
        // simulate physics
        let mut sat_transform = transforms.get_mut(sat_entity).unwrap();
        sat_transform.translation += sat.vel * dt;
        let dv = sat.conic.dv_at_pos(sat_transform.translation);
        sat.vel += dv * dt;

        // collect conics of potential soi collisions
        let moon_conics: Vec<&OrbitConic> = moons_query.iter().map(|(_,me)| &me.conic).collect();

        // check soi change
        if let Some(pi) = sat.parent_info.as_ref() {
            // check for exit of current soi
            let parent_soi_r = MOON_SOI; // TODO
            let parent_pos = pi.conic.pos_at_theta(pi.conic.nu_at_t(simulation_state.simulated_time));
            if sat_transform.translation.distance(parent_pos) > parent_soi_r {
                // reset orbit conic to planet (default) soi
                sat.conic = OrbitConic::from_initial(
                    sat_transform.translation,
                    sat.vel,
                    PLANET_MASS, Vec3::Y); // TODO get values from config
                sat.parent_info = None;
            }
        } else {
            // check for entry of moon soi
            for moon_conic in moon_conics.iter() {
                let current_time = simulation_state.simulated_time;
                let moon_nu = moon_conic.nu_at_t(current_time);
                let moon_pos = moon_conic.pos_at_theta(moon_nu);
                let moon_soi_r = MOON_SOI; // TODO

                // TODO remove info
                let mut body_info = body_info_query.single_mut();
                body_info.sections[0].value = format!("Debug:\n\
                    s: {:.2},{:.2},{:.2}\n\
                    m: {:.2},{:.2},{:.2}\n\
                    d: {:.2}, t: {:.2}",
                    sat_transform.translation.x, sat_transform.translation.y, sat_transform.translation.z,
                    moon_pos.x, moon_pos.y, moon_pos.z,
                    sat_transform.translation.distance(moon_pos), current_time,
                );

                if sat_transform.translation.distance(moon_pos) < moon_soi_r {
                    // new orbit conic in moon soi
                    println!("enter soi!");
                    sat.conic = OrbitConic::from_initial(
                        sat_transform.translation,
                        sat.vel,
                        MOON_MASS, Vec3::Y); // TODO get values from config
                    sat.parent_info = Some(ParentInfo { conic: **moon_conic, entry_time: current_time });
                }
            }
        }

        // draw conic path
        draw_conic_path(sat.conic, sat.parent_info.clone(), &mut gizmos, sat.color.clone(), Some(moon_conics), 0);

        // update body info ui
        if ViewingBody::Satellite(sat.idx) == controls.viewing_body {
            let mut body_info = body_info_query.single_mut();
            body_info.sections[0].value = format_body_info("Satellite", sat.idx,
                sat.vel, &sat.conic, &sat_transform);

            // update camera
            update_camera_target = Some(sat_transform.translation);
        }

        // reset position of sat on parabolic trajectory when out-of-bounds
        if sat_transform.translation.length() > 20. {
            sat_transform.translation.x = -sat_transform.translation.x;
            sat.vel.z = -sat.vel.z;
        }
    }

    // update camera
    if let Some(camera_target) = update_camera_target {
        // update camera
        let settings = config_assets.get(config_handles.settings.clone()).unwrap();
        let mut camera_transform = transforms.get_mut(camera_query.single()).unwrap();
        camera_transform.translation = camera_target + settings.camera_pos;
        camera_transform.look_at(camera_target, Vec3::Y);
    }
}

fn format_body_info(
    body_type: &str,
    body_idx: usize,
    body_vel: Vec3,
    conic: &OrbitConic,
    sat_transform: &Transform,
) -> String {
    // current true anomoly
    let t_nu = conic.nu_at_pos(sat_transform.translation);
    let t = conic.t_at_nu(t_nu);

    format!("{} {}:\n\
        p: {:.2},{:.2},{:.2}, v: {:.2},{:.2},{:.2}\n\
        h: {:.2}, i: {:.2}°, e: {:.2}\n\
        Ω: {:.2}°, ω: {:.2}°, ν: {:.2}°\n\
        t_ν: {:.2}°, t: {:.2}",
        body_type, body_idx,
        sat_transform.translation.x, sat_transform.translation.y, sat_transform.translation.z,
        body_vel.x, body_vel.y, body_vel.z,
        conic.h, deg!(conic.i), conic.e,
        deg!(conic.big_omega), deg!(conic.omega), deg!(conic.initial_nu),
        deg!(t_nu), t)
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
        if let ViewingBody::Moon(idx) = controls.viewing_body {
            if idx + 1 == settings.moons.len() {
                controls.viewing_body = ViewingBody::Satellite(0)
            } else {
                controls.viewing_body = ViewingBody::Moon(idx + 1);
            }
        } else if let ViewingBody::Satellite(idx) = controls.viewing_body {
            if idx + 1 == settings.satellites.len() {
                controls.viewing_body = ViewingBody::None;
            } else {
                controls.viewing_body = ViewingBody::Satellite(idx + 1);
            }
        } else {
            if 0 < settings.moons.len() {
                controls.viewing_body = ViewingBody::Moon(0);
            } else if 0 < settings.satellites.len() {
                controls.viewing_body = ViewingBody::Satellite(0);
            }
        }
    }

    // body info if no sat
    if ViewingBody::None == controls.viewing_body {
        let mut body_info = body_info_query.single_mut();
        // body_info.sections[0].value = format!("Body:\n\
        //     p: {:.2},{:.2},{:.2}\n\
        //     m: {:.2}",
        //     0., 0., 0., settings.body_mass);
            // TODO restore body info

        // update camera
        let mut camera_transform = camera_query.single_mut();
        camera_transform.translation = settings.camera_pos;
        camera_transform.look_at(settings.camera_look_at, Vec3::Y);
    }
}

fn draw_conic_path(
    conic: OrbitConic,
    parent_info: Option<ParentInfo>,
    gizmos: &mut Gizmos,
    color: Color,
    other_bodies: Option<Vec<&OrbitConic>>,
    render_depth: i32,
) {
    const STEPS: i32 = 128;
    let mut soi_change: Option<(OrbitConic,Option<ParentInfo>)> = None;
    'conic_loop: for n in 0..STEPS {
        // sweep arc segment
        let theta1 = (n as f32 - 0.5) * 2. * PI / (STEPS as f32);
        let theta2 = (n as f32 + 0.5) * 2. * PI / (STEPS as f32);
        let r1 = conic.r_at_theta(theta1);
        if r1 > 30. || r1 < 0. {
            continue;
        }
        let r2 = conic.r_at_theta(theta2);
        // starting pos to ending pos in space
        const NUDGE: f32 = 0.05; // todo wtf
        let d1 = conic.dir_at_theta(theta1 + NUDGE);
        let d2 = conic.dir_at_theta(theta2 + NUDGE);
        let d2_pos = d2 * r2;

        // orbit center of parent body
        let orbit_center = if let Some(pi) = parent_info.as_ref() {
            let mean_t = conic.t_at_nu((theta1 + theta2) / 2.);
            pi.conic.pos_at_theta(pi.conic.nu_at_t(mean_t + pi.entry_time) - pi.conic.initial_nu) -
                pi.conic.pos_at_theta(pi.conic.nu_at_t(pi.entry_time) - pi.conic.initial_nu)
        } else {
            Vec3::ZERO
        };

        // draw ray
        gizmos.ray(
            orbit_center + d1 * r2,
            d2 * r2 - d1 * r1,
            color,
        );


        // check if d2 intersects other_bodies soi
        if let Some(other_bodies_u) = other_bodies.as_ref() {
            let d2_tu = conic.t_at_nu(theta2);
            let d2_t = if d2_tu < 0. { conic.period + d2_tu } else { d2_tu };

            for other_body in other_bodies_u {
                let o_nuu = other_body.nu_at_t(d2_t);
                let o_nu = if o_nuu < 0. { 2. * PI + o_nuu } else { o_nuu };
                let ob_pos = other_body.pos_at_theta(o_nu);
                let soi_r = MOON_SOI; // TODO
                if d2_pos.distance(ob_pos) < soi_r {
                    let sub_orbit = OrbitConic::from_initial(
                        d2_pos,
                        conic.vel_at_theta(theta2),
                        MOON_MASS, // TODO as property of moon
                        Vec3::Y);
                    soi_change = Some((sub_orbit,Some(ParentInfo { conic: **other_body, entry_time: d2_t })));
                    break 'conic_loop;
                }
            }
        }

        // check if d2 exits current soi
        if parent_info.is_some() {
            let soi_r = MOON_SOI; // TODO
            if r2 > soi_r {
                let out_orbit = OrbitConic::from_initial(
                    d2_pos,
                    conic.vel_at_theta(theta2),
                    PLANET_MASS, // TODO as property of parent body
                    Vec3::Y);
                soi_change = Some((out_orbit,None));
                break 'conic_loop;
            }
        }

    }

    if let Some((conic, parent_info)) = soi_change {
        draw_conic_path(conic, parent_info, gizmos,
            color.with_s(f32::powi(0.5, render_depth + 1)),
            None,
            render_depth + 1);
    }
}
