use bevy::prelude::*;

// calculate distance from body (anomoly?) for given angle from periapsis
pub fn r_at_theta(
    fac: f32,
    ecc: f32,
    theta: f32,
) -> f32{
    fac / (1. + ecc * f32::cos(theta))
}

// calculate change in velocity for given position relative to body center
pub fn dv_at_pos(
    dt: f32,
    body_fac: f32,
    rel_pos: Vec3,
) -> Vec3 {
    body_fac * 16. * dt * -rel_pos.normalize() / rel_pos.length_squared()
}
