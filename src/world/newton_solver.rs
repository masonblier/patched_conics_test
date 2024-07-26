const EPSILON: f32 = 0.000001;
const MAX_ITERATIONS: u32 = 20;

// approximate solution of given f,df using newton's method
pub fn newton_solver<F,G>(
    f: F,
    df: G,
    x0: f32,
) -> f32 where
    F: Fn(f32) -> f32,
    G: Fn(f32) -> f32,
{
    let mut x = x0;
    let mut delta = (0. - f(x)).abs();
    let mut iterations = 0;
    while delta > EPSILON && iterations < 10 {
        x = x - f(x) / df(x);
        delta = (0. - f(x)).abs();
        iterations += 1;
    }
    if iterations == MAX_ITERATIONS {
        panic!("newton solver failed to converge");
    }
    x
}


#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_f {
        ($x:expr, $y:expr) => {
            if !(($x - $y) < EPSILON && ($y - $x) < EPSILON) {
                panic!("assert_f failed: {} !=> {}", $x, $y);
            }
        }
    }

    #[test]
    fn test_newton_solver() {
        let f = {|x: f32|
            6. * x.powi(5) - 5. * x.powi(4) - 4. * x.powi(3) + 3. * x.powi(2)
        };
        let df = {|x: f32|
            30. * x.powi(4) - 20. * x.powi(3) - 12. * x.powi(2) + 6. * x
        };

        // first root
        let x = newton_solver(f, df, 0.);
        assert_f!(x, 0.);
        assert_f!(f(x), 0.);

        // second root
        let x = newton_solver(f, df, 0.5);
        assert_f!(x, 0.628667);
        assert_f!(f(x), 0.);

        // third root
        let x = newton_solver(f, df, 1.);
        assert_f!(x, 1.);
        assert_f!(f(x), 0.);
    }
}
