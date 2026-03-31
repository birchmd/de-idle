use {crate::sliding3::Sliding3, std::collections::VecDeque};

#[cfg(test)]
use {
    super::tests::{add_gold, build_furnace, create_miner, create_pts},
    crate::ui::actor::Msg,
};

const TWO_PI: f64 = 2.0 * std::f64::consts::PI;

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
    let Some((x0, y0)) = pts.front() else {
        return false;
    };

    if pts.len() < 500 {
        return false;
    }

    let (min_x, max_x) = min_max(x0, pts.iter().map(|(x, _)| x));
    let (min_y, max_y) = min_max(y0, pts.iter().map(|(_, y)| y));

    let mx = (max_x + min_x) / 2.0;
    let rx = (max_x - min_x) / 2.0;

    let my = (max_y + min_y) / 2.0;
    let ry = (max_y - min_y) / 2.0;

    // After normalization, the radius of the circle should be 1.
    let total_error: f64 = pts
        .iter()
        .map(|(x, y)| {
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

fn min_max<'a, I>(x0: &'a f64, xs: I) -> (&'a f64, &'a f64)
where
    I: Iterator<Item = &'a f64>,
{
    xs.fold((x0, x0), |(min, max), x| {
        if x < min {
            (x, max)
        } else if max < x {
            (min, x)
        } else {
            (min, max)
        }
    })
}

#[cfg(test)]
fn setup_sinusoidal(x_axis: u8, y_axis: u8) -> impl Iterator<Item = Msg> {
    let get_energy = build_furnace()
        .chain(std::iter::repeat_n(Msg::Chop, 2000))
        .chain(std::iter::repeat_n(Msg::Update, 200_000));
    let miners = std::iter::repeat_with(create_miner).take(100).flatten();
    get_energy
        .chain(miners)
        .chain(std::iter::repeat_n(Msg::HireRecruiter, 71))
        .chain(std::iter::repeat_n(Msg::HireMonster, 70))
        .chain(std::iter::repeat_n(Msg::HireLumberjack, 10))
        .chain(vec![Msg::SetXAxis(x_axis), Msg::SetYAxis(y_axis)])
        .chain(std::iter::repeat_n(Msg::Update, 202_000))
}

#[test]
fn test_sinusoidal() {
    let msgs = setup_sinusoidal(0, 5);
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
    let msgs = setup_sinusoidal(2, 5);
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
