use bevy::prelude::*;
use std::f32::consts::PI;

use super::newton_solver;

// gravitational constant
pub const G: f32 = 6.67408e-11;

// struct for classical orbital elements of orbit conic of small satellite
// https://orbital-mechanics.space/classical-orbital-elements/orbital-elements-and-the-state-vector.html
#[derive(Clone, Copy, Debug)]
pub struct OrbitConic {
    pub body_mass: f32, // mass of barycenter/body
    pub initial_r: Vec3, // initial position relative to barycenter
    pub h_vec: Vec3, // vector perpendicular to orbital plane
    pub h: f32, // angular momentum
    pub i: f32, // inclination
    pub big_omega: f32, // right ascension of ascending node
    pub e: f32, // eccentricity
    pub e_vec: Vec3, // eccentricity vector
    pub omega: f32, // argument of periapsis
    pub initial_nu: f32, // initial true anomoly
    pub period: f32, // orbit period
}

impl OrbitConic {

    // initializes conic parameters from initial position, velocity,
    // and body parameters
    pub fn from_initial(
        position: Vec3,
        velocity: Vec3,
        body_mass: f32,
        body_plane_k: Vec3,
    ) -> Self {
        // angular momentum
        let h_vec = position.cross(velocity);
        let h = h_vec.length();

        // inclination relative to body's plane-of-reference
        let i = f32::acos(h_vec.dot(body_plane_k) / h);

        // right ascension of ascending node
        let n_vec = body_plane_k.cross(h_vec);
        let n = n_vec.length();
        let big_omega = 2. * PI - f32::acos(n_vec.x / n);

        // eccentricity
        let mu = G * body_mass;
        let e_vec = velocity.cross(h_vec) / mu - position.normalize();
        let e = e_vec.length();

        // argument of periapsis
        let omega = 2. * PI - f32::acos(n_vec.dot(e_vec) / (n * e));

        // initial true anomaly, from -180. to 180.
        let initial_nu = if e_vec.length() <= 0. {
            0.
        } else if 0. > e_vec.cross(position).dot(h_vec) {
            -f32::acos(position.normalize().dot(e_vec.normalize()))
        } else {
            f32::acos(position.normalize().dot(e_vec.normalize()))
        };

        // period
        let a = h.powi(2) / (mu * (1. - e.powi(2)));
        let period = 2. * PI / mu.sqrt() * a.powf(3. / 2.);

        // return
        OrbitConic {
            body_mass,
            initial_r: position,
            h_vec,
            h,
            i,
            big_omega,
            e,
            e_vec,
            omega,
            initial_nu,
            period,
        }
    }

    // calculate distance from body for given angle from periapsis (true anomaly)
    pub fn r_at_theta(
        &self,
        theta: f32,
    ) -> f32{
        self.h.powi(2) / (G * self.body_mass * (1. + self.e * f32::cos(self.initial_nu + theta)))
    }

    // calculate orientation relative to parent body's center
    pub fn dir_at_theta(
        &self,
        theta: f32,
    ) -> Vec3 {
        let x_vec = self.h_vec.cross(self.initial_r).normalize();
        let z_vec = self.initial_r.normalize();
        x_vec * f32::sin(theta) + z_vec * f32::cos(theta)
    }

    // calculate position relative to body center
    pub fn pos_at_theta(
        &self,
        theta: f32,
    ) -> Vec3 {
        self.r_at_theta(theta) * self.dir_at_theta(theta)
    }

    // calculate velocity for a given anomaly
    pub fn vel_at_theta(
        &self,
        theta: f32,
    ) -> Vec3 {
        // v_w = mu / h * np.array((-np.sin(nu), e + np.cos(nu), 0))
        let t_vel = G * self.body_mass / self.h;
        let nu_r = self.dir_at_theta(-self.initial_nu);
        let z_vec = self.h_vec.cross(nu_r).normalize();
        let x_vec = nu_r.normalize();
        let v_dir = x_vec * -f32::sin(self.initial_nu + theta) + z_vec * (self.e + f32::cos(self.initial_nu + theta));
        t_vel * v_dir
    }

    // calculate change in velocity for given position relative to body center
    pub fn dv_at_pos(
        &self,
        rel_pos: Vec3,
    ) -> Vec3 {
        G * self.body_mass * -rel_pos.normalize() / rel_pos.length_squared()
    }
    // calculate time at given anomaly
    pub fn t_at_nu(
        &self,
        nu: f32,
    ) -> f32 {
        let mu = G * self.body_mass;
        // elliptical
        if self.e < 1. {
            // mean anomoly
            let me_nu = 2. * f32::atan(f32::sqrt((1. - self.e) / (1. + self.e)) * f32::tan(nu / 2.))
                - (self.e * f32::sqrt(1. - self.e.powi(2)) * f32::sin(nu)) / (1. + self.e * f32::cos(nu));
            // t
            me_nu * self.h.powi(3) / (mu.powi(2) * (1. - self.e.powi(2)).powf(3./2.))

        // hyperbolic
        } else {
            let mh_nu = (self.e * f32::sqrt(self.e.powi(2) - 1.) * f32::sin(nu)) / (1. + self.e * f32::cos(nu))
                - f32::ln((f32::sqrt(self.e + 1.) + f32::sqrt(self.e - 1.) * f32::tan(nu/2.))
                    / (f32::sqrt(self.e + 1.) - f32::sqrt(self.e - 1.) * f32::tan(nu/2.)));
            mh_nu * self.h.powi(3) / (mu.powi(2) * (self.e.powi(2) - 1.).powf(3./2.))
        }
    }

    // calculate true anomaly of position
    pub fn nu_at_pos(
        &self,
        position: Vec3,
    ) -> f32 {
        if self.e_vec.length() <= 0. {
            0.
        } else if 0. > self.e_vec.cross(position).dot(self.h_vec) {
            -f32::acos(position.normalize().dot(self.e_vec.normalize()))
        } else {
            f32::acos(position.normalize().dot(self.e_vec.normalize()))
        }
    }

    // calculate true anomaly at time t
    pub fn nu_at_t(
        &self,
        t: f32,
    ) -> f32 {
        let mu = G * self.body_mass;

        // elliptical
        if self.e < 1. {
            let me_nu = 2. * PI * t / self.period;

            // use newton's method to solve for eccentric anomaly
            let e = self.e;
            let f = {|x: f32|
                x - e * x.sin() - me_nu
            };
            let df = {|x: f32|
                1. - e * x.cos()
            };
            let ec_nu = newton_solver(f, df, PI);

            let nu = 2. * (((1. + e) / (1. - e)).sqrt() * (ec_nu / 2.).tan()).atan();
            nu

        // hyperbolic
        } else {
            let me_nu = mu.powi(2) / self.h.powi(3) * (self.e.powi(2) - 1.).powf(3. / 2.) * t;

            // use newton's method to solve for eccentric anomaly
            let e = self.e;
            let f = {|x: f32|
                e * x.sinh() - x - me_nu
            };
            let df = {|x: f32|
                e * x.cosh() - 1.
            };
            let ec_nu = newton_solver(f, df, PI);

            let nu = 2. * (((e + 1.) / (e - 1.)).sqrt() * (ec_nu / 2.).tanh()).atan();
            nu
        }
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 0.000001;
    macro_rules! assert_f {
        ($x:expr, $y:expr) => {
            if !(($x - $y) / $y < EPSILON && ($y - $x) / $y < EPSILON) {
                panic!("assert_f failed: {} !=> {}", $x, $y);
            }
        }
    }
    macro_rules! rad {
        ($x:expr) => {
            $x * PI / 180.
        }
    }

    #[test]
    fn test_example_orbit() {
        let test_oc = OrbitConic::from_initial(
            Vec3::new(1000., 5000., 7000.),
            Vec3::new(3., 4., 5.),
            398600. / G,
            Vec3::Z,
        );

        assert_f!(test_oc.h, 19646.883);
        assert_f!(test_oc.i, rad!(124.0479));
        assert_f!(test_oc.big_omega, rad!(190.6197));
        assert_f!(test_oc.e, 0.94754106);
        assert_f!(test_oc.omega, rad!(303.091));
        assert_f!(test_oc.initial_nu, rad!(159.6116));
    }


    #[test]
    fn test_orbit_fns_circular() {
        let test_oc = OrbitConic::from_initial(
            Vec3::new(0., 0., 1000.),
            Vec3::new(19.96497, 0., 0.),
            398600. / G,
            Vec3::Y,
        );
        assert_f!(test_oc.e + 0.1, 0. + 0.1);
        assert_f!(test_oc.initial_nu + 0.1, 0. + 0.1);
        let pos_0 = test_oc.pos_at_theta(0.);
        let pos_1 = test_oc.pos_at_theta(PI / 2.);
        let pos_2 = test_oc.pos_at_theta(PI);
        let pos_3 = test_oc.pos_at_theta(3. * PI / 2.);
        assert_f!(pos_0.z, 1000.);
        assert_f!(pos_1.x, 1000.);
        assert_f!(pos_2.z, -1000.);
        assert_f!(pos_3.x, -1000.);
        let vel_0 = test_oc.vel_at_theta(0.);
        let vel_1 = test_oc.vel_at_theta(PI / 2.);
        let vel_2 = test_oc.vel_at_theta(PI);
        let vel_3 = test_oc.vel_at_theta(3. * PI / 2.);
        assert_f!(vel_0.x, 19.96497);
        assert_f!(vel_1.z, -19.96497);
        assert_f!(vel_2.x, -19.96497);
        assert_f!(vel_3.z, 19.96497);

        let test_oc2 = OrbitConic::from_initial(
            Vec3::new(1000., 0., 0.),
            Vec3::new(0., 0., -19.96497),
            398600. / G,
            Vec3::Y,
        );
        assert_f!(test_oc2.e + 0.1, 0. + 0.1);
        assert_f!(test_oc2.initial_nu + 0.1, 0. + 0.1);
        let pos_0 = test_oc2.pos_at_theta(0.);
        let pos_1 = test_oc2.pos_at_theta(PI / 2.);
        let pos_2 = test_oc2.pos_at_theta(PI);
        let pos_3 = test_oc2.pos_at_theta(3. * PI / 2.);
        assert_f!(pos_0.x, 1000.);
        assert_f!(pos_1.z, -1000.);
        assert_f!(pos_2.x, -1000.);
        assert_f!(pos_3.z, 1000.);
        let vel_0 = test_oc2.vel_at_theta(0.);
        let vel_1 = test_oc2.vel_at_theta(PI / 2.);
        let vel_2 = test_oc2.vel_at_theta(PI);
        let vel_3 = test_oc2.vel_at_theta(3. * PI / 2.);
        assert_f!(vel_0.z, -19.96497);
        assert_f!(vel_1.x, -19.96497);
        assert_f!(vel_2.z, 19.96497);
        assert_f!(vel_3.x, 19.96497);
    }

    #[test]
    fn test_orbit_fns_ellipse_apoapsis_start() {
        let test_oc3 = OrbitConic::from_initial(
            Vec3::new(1000., 0., 0.),
            Vec3::new(0., 0., -14.),
            398600. / G,
            Vec3::Y,
        );
        assert_f!(test_oc3.e, 0.50827897);
        assert_f!(test_oc3.initial_nu, PI);
        let r_0 = test_oc3.r_at_theta(0.);
        let r_2 = test_oc3.r_at_theta(PI);
        assert_f!(r_0, 1000.);
        assert_f!(r_2, 326.01465);
        let pos_0 = test_oc3.pos_at_theta(0.);
        let pos_1 = test_oc3.pos_at_theta(PI / 2.);
        let pos_2 = test_oc3.pos_at_theta(PI);
        let pos_3 = test_oc3.pos_at_theta(3. * PI / 2.);
        assert_f!(pos_0.x, 1000.);
        assert_f!(pos_1.x, -0.00002149381);
        assert_f!(pos_2.x, -326.01465);
        assert_f!(pos_3.x, 0.0000058637156);
        let vel_0 = test_oc3.vel_at_theta(0.);
        let vel_1 = test_oc3.vel_at_theta(PI / 2.);
        let vel_2 = test_oc3.vel_at_theta(PI);
        let vel_3 = test_oc3.vel_at_theta(3. * PI / 2.);
        assert_f!(vel_0.z, -14.);
        assert_f!(vel_1.x, -28.471428);
        assert_f!(vel_2.z, 42.942856);
        assert_f!(vel_3.x, 28.471428);
    }
}
