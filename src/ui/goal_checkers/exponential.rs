use std::collections::VecDeque;

#[cfg(test)]
use {
    super::tests::{add_gold, build_bank, build_factory, build_furnace, create_miner, create_pts},
    crate::ui::actor::Msg,
};

pub fn exponential_goal_checker(pts: &VecDeque<(f64, f64)>) -> bool {
    let Some((x0, y0)) = pts.front() else {
        return false;
    };
    let Some((xn, yn)) = pts.back() else {
        return false;
    };
    if pts.len() < 500 {
        return false;
    }
    if x0 == xn || yn <= y0 || y0 <= &0.0 {
        return false;
    }

    let slope = (yn.ln() - y0.ln()) / (xn - x0);

    // Since the only way to produce exponential growth is with a bank,
    // which is effectively continuously compounding interest, the function
    // should be close to e^x (i.e. log slope close to 1). If there are multiple
    // banks then the slope could exceed 1 (e.g. 2 banks close to a slope of 2).
    if slope < 0.9 {
        return false;
    }

    let intercept = yn.ln() - x0 * slope;

    let (total_error, total_values) = pts.iter().fold((0.0, 0.0), |(te, tv), (x, y)| {
        let y_calc = slope * x + intercept;
        let ln_y = y.ln();
        let dy = y_calc - ln_y;
        (te + dy * dy, tv + ln_y)
    });
    let mv = total_values / (pts.len() as f64);
    let mean_error = total_error.sqrt() / mv;

    mean_error < 100.0
}

#[test]
fn test_exponential() {
    let msgs = add_gold(1)
        .chain(build_bank())
        .chain(vec![Msg::SetXAxis(0), Msg::SetYAxis(2)]);
    let pts = create_pts(msgs);
    assert!(exponential_goal_checker(&pts));

    // Fails on horizontal line
    let pts = create_pts(std::iter::once(Msg::SetXAxis(0)));
    assert!(!exponential_goal_checker(&pts));

    // Fails on vertical line
    let pts = create_pts(std::iter::once(Msg::SetYAxis(0)));
    assert!(!exponential_goal_checker(&pts));

    // Fails on positive slope
    let msgs = create_miner()
        .chain(add_gold(1))
        .chain(vec![Msg::SetXAxis(0), Msg::SetYAxis(2)]);
    let pts = create_pts(msgs);
    assert!(!exponential_goal_checker(&pts));

    // Fails on negative slope
    let msgs = add_gold(1100)
        .chain(std::iter::once(Msg::HireLumberjack))
        .chain(vec![Msg::SetXAxis(0), Msg::SetYAxis(2)]);
    let pts = create_pts(msgs);
    assert!(!exponential_goal_checker(&pts));

    // Fails on quadratic growth
    let msgs = add_gold(1)
        .chain(create_miner())
        .chain(create_miner())
        .chain(std::iter::once(Msg::HireLumberjack))
        .chain(build_furnace())
        .chain(build_factory())
        .chain(vec![Msg::SetXAxis(0), Msg::SetYAxis(2)]);
    let pts = create_pts(msgs);
    assert!(!exponential_goal_checker(&pts));
}
