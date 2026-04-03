use {
    crate::utils::{
        matrix, min_max,
        point::Point,
        sequence::{MappedVecDeque, Sequence},
        sliding3::Sliding3,
    },
    std::collections::VecDeque,
};

#[cfg(test)]
use {
    super::tests::{add_gold, build_bank, build_factory, build_furnace, create_miner, create_pts},
    crate::ui::actor::Msg,
};

const TWO_PI: f64 = 2.0 * std::f64::consts::PI;
const ROOT_3_OVER_2: f64 = 0.86602540378443864676372317075294;

pub fn wave_goal_checker(pts: &VecDeque<(f64, f64)>) -> bool {
    let Some((x0, y0)) = pts.front() else {
        return false;
    };

    if pts.len() < 500 {
        return false;
    }

    let (min, max) = min_max(y0, pts.iter().map(|(_, y)| y));

    let amplitude = (max - min) / 2.0;
    let average = (max + min) / 2.0;

    let mut local_max =
        Sliding3::new(pts.iter()).filter_map(
            |[(_, y0), (x1, y1), (_, y2)]| {
                if y0 < y1 && y2 < y1 { Some(x1) } else { None }
            },
        );

    let Some(p1) = local_max.next() else {
        return false;
    };

    let Some(p2) = local_max.next() else {
        return false;
    };

    let period = p2 - p1;
    let phase = f64::asin((y0 - average) / amplitude) - TWO_PI * x0 / period;

    let total_error: f64 = pts
        .iter()
        .map(|(x, y)| {
            let y_calc = amplitude * f64::sin(TWO_PI * x / period + phase) + average;
            let dy = y - y_calc;
            dy * dy
        })
        .sum();

    // Pass condition is based on the mean-square error being small.
    let mse = total_error / (pts.len() as f64);

    mse < 0.5
}

pub fn circle_goal_checker(pts: &VecDeque<(f64, f64)>) -> bool {
    inner_circle_goal_checker(pts)
}

pub fn knot_goal_checker(pts: &VecDeque<(f64, f64)>) -> bool {
    if pts.len() < 500 {
        return false;
    }

    let mut local_min = Sliding3::new(pts.iter().map(|(x, _)| x).enumerate()).filter_map(
        |[(_, x0), (t1, x1), (_, x2)]| {
            if x1 < x0 && x1 < x2 {
                Some((t1, x1))
            } else {
                None
            }
        },
    );

    let Some((t0, x0)) = local_min.next() else {
        return false;
    };

    let Some((t1, x1)) = local_min.next() else {
        return false;
    };

    // The x coordinate changes as a wave plus a line.
    // We compute the slope of the line by comparing two consecutive minima of the wave.
    let slope = (x1 - x0) / ((t1 - t0) as f64);

    // We need a non-trivial slope, or else it is just a circle.
    if slope < 0.005 {
        return false;
    }

    // Map the x coordinate to subtract the linear part
    let mapped_pts = MappedVecDeque {
        inner: pts,
        f: |enumerated_point: (usize, &(f64, f64))| {
            let (i, (x, y)) = enumerated_point;
            let t = i as f64;
            (x - slope * t, *y)
        },
    };

    // Now we have two normal waves, so it should pass the circle check.
    inner_circle_goal_checker(mapped_pts)
}

pub fn bend_goal_checker(pts: &VecDeque<(f64, f64)>) -> bool {
    let Some((x0, y0)) = pts.front() else {
        return false;
    };

    let Some((xn, yn)) = pts.back() else {
        return false;
    };

    if pts.len() < 600 || x0 == xn {
        return false;
    }

    // Starts with a negatively sloped line (at least first 100 pts)
    let Some((x1, y1)) = pts.get(100) else {
        return false;
    };
    let [slope_start, intercept_start] =
        matrix::multiply_col_vector(matrix::inverse_2x2([*x0, 1.0, *x1, 1.0]), [*y0, *y1]);

    if slope_start > -0.01 {
        return false;
    }

    for (x, y) in pts.iter().take(100) {
        let y_calc = x * slope_start + intercept_start;
        if (y - y_calc).abs() > 0.01 {
            return false;
        }
    }

    // The same slope exists after the bend (last 100 pts)
    let intercept_end = yn - xn * slope_start;
    for (x, y) in pts.iter().rev().take(100) {
        let y_calc = x * slope_start + intercept_end;
        if (y - y_calc).abs() > 0.01 {
            return false;
        }
    }

    // Find the bounds of the bend by looking for where
    // the pts deviate from the start and end lines.
    let Some(idx_start) = pts.iter().position(|(x, y)| {
        let y_calc = x * slope_start + intercept_start;
        (y - y_calc).abs() > 0.01
    }) else {
        return false;
    };

    let Some(idx_end) = pts.iter().skip(idx_start).position(|(x, y)| {
        let y_calc = x * slope_start + intercept_end;
        (y - y_calc).abs() <= 0.01
    }) else {
        return false;
    };

    // Use points from the curve to fit the coefficients of the functional form
    let Some((x1, y1)) = pts.get(idx_start) else {
        return false;
    };

    let Some((x2, y2)) = pts.get((idx_start + idx_end) / 2) else {
        return false;
    };

    let Some((x3, y3)) = pts.get(idx_end) else {
        return false;
    };

    let xs = [0.0, x2 - x1, x3 - x1];

    let coefficients = matrix::multiply_col_vector(
        matrix::inverse_3x3([
            exp_sin(xs[0]),
            exp_cos(xs[0]),
            1.0,
            exp_sin(xs[1]),
            exp_cos(xs[1]),
            1.0,
            exp_sin(xs[2]),
            exp_cos(xs[2]),
            1.0,
        ]),
        [*y1, *y2, *y3],
    );

    let total_error: f64 = pts
        .iter()
        .skip(idx_start)
        .take(idx_end - idx_start)
        .map(|(x, y)| {
            let t = x - x1;
            let y_calc =
                coefficients[0] * exp_sin(t) + coefficients[1] * exp_cos(t) + coefficients[2];
            let dy = y - y_calc;
            dy * dy
        })
        .sum();

    let mse = total_error / (pts.len() as f64);
    mse < 0.001
}

// Generalized circle check function so that it can be used both
// for the normal circle case and the knot case (which is a circle
// where one of the coordinates is moving linearly over time).
fn inner_circle_goal_checker<I, P, T>(pts: I) -> bool
where
    I: Sequence<Item = P>,
    P: Point<Coord = T> + Copy,
    T: Copy
        + PartialOrd
        + std::ops::Add<Output = f64>
        + std::ops::Sub<Output = f64>
        + std::ops::Add<f64, Output = f64>
        + std::ops::Sub<f64, Output = f64>,
{
    let Some(p0) = pts.front() else {
        return false;
    };
    let x0 = p0.fst();
    let y0 = p0.snd();

    if pts.len() < 500 {
        return false;
    }

    let (min_x, max_x) = min_max(x0, pts.iter().map(Point::fst));
    let (min_y, max_y) = min_max(y0, pts.iter().map(Point::snd));

    let mx = (max_x + min_x) / 2.0;
    let rx = (max_x - min_x) / 2.0;

    let my = (max_y + min_y) / 2.0;
    let ry = (max_y - min_y) / 2.0;

    // After normalization, the radius of the circle should be 1.
    let total_error: f64 = pts
        .iter()
        .map(|p| {
            let x = p.fst();
            let y = p.snd();
            let norm_x = (x - mx) / rx;
            let norm_y = (y - my) / ry;

            let r = norm_x * norm_x + norm_y * norm_y;
            let dr = 1.0 - r;

            dr * dr
        })
        .sum();

    // Pass condition is based on the mean-square error being small.
    let mse = total_error / (pts.len() as f64);

    mse < 0.001
}

fn exp_sin(t: f64) -> f64 {
    f64::exp(0.5 * t) * f64::sin(ROOT_3_OVER_2 * t)
}

fn exp_cos(t: f64) -> f64 {
    f64::exp(0.5 * t) * f64::cos(ROOT_3_OVER_2 * t)
}

#[cfg(test)]
struct SinusoidalConfig {
    x_axis: u8,
    y_axis: u8,
    n_miners: usize,
    n_monsters: usize,
    update_duration: usize,
}

#[cfg(test)]
fn setup_sinusoidal(config: SinusoidalConfig) -> impl Iterator<Item = Msg> {
    let get_energy = build_furnace()
        .chain(std::iter::repeat_n(Msg::Chop, 2000))
        .chain(std::iter::repeat_n(Msg::Update, 200_000));
    let miners = std::iter::repeat_with(create_miner)
        .take(config.n_miners)
        .flatten();
    get_energy
        .chain(miners)
        .chain(std::iter::repeat_n(
            Msg::HireRecruiter,
            config.n_monsters + 1,
        ))
        .chain(std::iter::repeat_n(Msg::HireMonster, config.n_monsters))
        .chain(std::iter::repeat_n(Msg::HireLumberjack, 10))
        .chain(vec![
            Msg::SetXAxis(config.x_axis),
            Msg::SetYAxis(config.y_axis),
        ])
        .chain(std::iter::repeat_n(Msg::Update, config.update_duration))
}

#[test]
fn test_sinusoidal() {
    let config = SinusoidalConfig {
        x_axis: 0,
        y_axis: 5,
        n_miners: 100,
        n_monsters: 70,
        update_duration: 202_000,
    };
    let msgs = setup_sinusoidal(config);
    let pts = create_pts(msgs);
    assert!(wave_goal_checker(&pts));

    // Fails on horizontal line
    let pts = create_pts(std::iter::once(Msg::SetXAxis(0)));
    assert!(!wave_goal_checker(&pts));

    // Fails on vertical line
    let pts = create_pts(std::iter::once(Msg::SetYAxis(0)));
    assert!(!wave_goal_checker(&pts));

    // Fails on positive slope
    let msgs = create_miner().chain(vec![Msg::SetXAxis(0), Msg::SetYAxis(2)]);
    let pts = create_pts(msgs);
    assert!(!wave_goal_checker(&pts));

    // Fails on negative slope
    let msgs = add_gold(1100)
        .chain(std::iter::once(Msg::HireLumberjack))
        .chain(vec![Msg::SetXAxis(0), Msg::SetYAxis(2)]);
    let pts = create_pts(msgs);
    assert!(!wave_goal_checker(&pts));
}

#[test]
fn test_circle() {
    let config = SinusoidalConfig {
        x_axis: 2,
        y_axis: 5,
        n_miners: 100,
        n_monsters: 70,
        update_duration: 202_000,
    };
    let msgs = setup_sinusoidal(config);
    let pts = create_pts(msgs);
    assert!(circle_goal_checker(&pts));

    // Fails on horizontal line
    let pts = create_pts(std::iter::once(Msg::SetXAxis(0)));
    assert!(!circle_goal_checker(&pts));

    // Fails on vertical line
    let pts = create_pts(std::iter::once(Msg::SetYAxis(0)));
    assert!(!circle_goal_checker(&pts));

    // Fails on positive slope
    let msgs = create_miner().chain(vec![Msg::SetXAxis(0), Msg::SetYAxis(2)]);
    let pts = create_pts(msgs);
    assert!(!circle_goal_checker(&pts));

    // Fails on negative slope
    let msgs = add_gold(1100)
        .chain(std::iter::once(Msg::HireLumberjack))
        .chain(vec![Msg::SetXAxis(0), Msg::SetYAxis(2)]);
    let pts = create_pts(msgs);
    assert!(!circle_goal_checker(&pts));
}

#[test]
fn test_knot() {
    let config = SinusoidalConfig {
        x_axis: 5,
        y_axis: 2,
        n_miners: 30,
        n_monsters: 21,
        update_duration: 200_000,
    };
    let msgs = setup_sinusoidal(config)
        .chain(build_factory())
        .chain(std::iter::repeat_n(Msg::Update, 5125));
    let pts = create_pts(msgs);
    assert!(knot_goal_checker(&pts));
}

#[test]
fn test_bend() {
    let config = SinusoidalConfig {
        x_axis: 0,
        y_axis: 5,
        n_miners: 30,
        n_monsters: 21,
        update_duration: 200_000,
    };
    let msgs = setup_sinusoidal(config)
        .chain(build_bank())
        .chain(std::iter::repeat_n(Msg::Update, 5000));
    let pts = create_pts(msgs);
    assert!(bend_goal_checker(&pts));

    // Fails on horizontal line
    let pts = create_pts(std::iter::once(Msg::SetXAxis(0)));
    assert!(!bend_goal_checker(&pts));

    // Fails on vertical line
    let pts = create_pts(std::iter::once(Msg::SetYAxis(0)));
    assert!(!bend_goal_checker(&pts));

    // Fails on positive slope
    let msgs = create_miner().chain(vec![Msg::SetXAxis(0), Msg::SetYAxis(2)]);
    let pts = create_pts(msgs);
    assert!(!bend_goal_checker(&pts));

    // Fails on negative slope
    let msgs = add_gold(1100)
        .chain(std::iter::once(Msg::HireLumberjack))
        .chain(vec![Msg::SetXAxis(0), Msg::SetYAxis(2)]);
    let pts = create_pts(msgs);
    assert!(!bend_goal_checker(&pts));
}
