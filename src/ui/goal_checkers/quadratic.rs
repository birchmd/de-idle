use {crate::utils::sliding3::Sliding3, std::collections::VecDeque};

#[cfg(test)]
use {
    super::tests::{
        add_gold, build_factory, build_furnace, create_miner, create_pts,
        create_pts_with_intervention,
    },
    crate::ui::actor::Msg,
};

pub fn parabola_goal_checker(pts: &VecDeque<(f64, f64)>) -> bool {
    let Some((x0, y0)) = pts.front() else {
        return false;
    };
    let Some((xn, yn)) = pts.back() else {
        return false;
    };

    if pts.len() < 100 {
        return false;
    }

    let ym = pts
        .iter()
        .fold(y0, |acc, (_, y)| if y < acc { y } else { acc });

    // Early return if the min is at the beginning or end
    if ym == y0 || ym == yn {
        return false;
    }

    let xm = {
        let (total, count) = pts.iter().fold((0.0, 0), |(sum, count), (x, y)| {
            if y == ym {
                (sum + x, count + 1)
            } else {
                (sum, count)
            }
        });
        total / (count as f64)
    };

    // Lowest point must be in the middle third of the plot
    if (xm - x0) > 2.0 * (xn - xm) {
        return false;
    }

    // The leading coefficient of the parabola is half of the second derivative.
    let a = {
        let (total, count) = Sliding3::new(pts.iter()).fold((0.0, 0), |(sum, count), window| {
            let (x0, y0) = window[0];
            let (x1, y1) = window[1];
            let (x2, y2) = window[2];
            let dx0 = x1 - x0;
            let dx1 = x2 - x1;
            let sd = (y2 * dx0 - y1 * (x2 - x0) + y0 * dx1) / (dx1 * dx0 * dx0);
            (sum + sd, count + 1)
        });
        total / ((2 * count) as f64)
    };

    let total_error: f64 = pts
        .iter()
        .map(|(x, y)| {
            let dx = x - xm;
            let y_calc = a * dx * dx + ym;
            let dy = y - y_calc;
            dy * dy
        })
        .sum();

    // Pass condition is based on the mean-square error being small.
    let mse = total_error / (pts.len() as f64);

    mse < 0.01
}

#[test]
fn test_parabola() {
    let msgs = add_gold(100)
        .chain(std::iter::repeat_n(Msg::HireLumberjack, 10))
        .chain(build_furnace())
        .chain(build_factory())
        .chain(vec![Msg::SetXAxis(0), Msg::SetYAxis(2)]);
    let pts = create_pts_with_intervention(msgs, std::iter::repeat_n(Msg::Update, 1550));
    assert!(parabola_goal_checker(&pts));

    // Fails on horizontal line
    let pts = create_pts(std::iter::once(Msg::SetXAxis(0)));
    assert!(!parabola_goal_checker(&pts));

    // Fails on vertical line
    let pts = create_pts(std::iter::once(Msg::SetYAxis(0)));
    assert!(!parabola_goal_checker(&pts));

    // Fails on positive slope
    let msgs = create_miner().chain(vec![Msg::SetXAxis(0), Msg::SetYAxis(2)]);
    let pts = create_pts(msgs);
    assert!(!parabola_goal_checker(&pts));

    // Fails on negative slope
    let msgs = add_gold(1100)
        .chain(std::iter::once(Msg::HireLumberjack))
        .chain(vec![Msg::SetXAxis(0), Msg::SetYAxis(2)]);
    let pts = create_pts(msgs);
    assert!(!parabola_goal_checker(&pts));

    // Also passes on a shape that kind of looks like this:
    // \_/
    // I think this is acceptable though; this is much harder to set up than
    // the "intended" solution above which only involves one factory.
    let msgs = add_gold(5)
        .chain(std::iter::repeat_n(Msg::Chop, 1000))
        .chain(build_furnace())
        .chain(build_furnace())
        .chain(build_factory())
        .chain(build_factory())
        .chain(std::iter::once(Msg::HireLumberjack))
        .chain(vec![Msg::SetXAxis(0), Msg::SetYAxis(2)]);
    let updates = |n: usize| std::iter::repeat_n(Msg::Update, n);
    let time_msgs = updates(100)
        .chain(std::iter::repeat_n(Msg::DemolishFactory, 2))
        .chain(updates(50));
    let pts = create_pts_with_intervention(msgs, time_msgs);
    assert!(parabola_goal_checker(&pts));
}
