use {crate::sliding3::Sliding3, std::collections::VecDeque};

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

    let n = pts.len() as f64;
    let b = ((yn.ln() - y0.ln()) / n).exp();
    Sliding3::new(pts.iter()).all(|[(_, y0), (_, y1), (_, y2)]| {
        let e1 = (y1 - b * y0).abs() / y1;
        let e2 = (y2 - b * y1).abs() / y2;
        y0 < y1 && y1 < y2 && e1 < 0.001 && e2 < 0.001
    })
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
    let msgs = create_miner().chain(vec![Msg::SetXAxis(0), Msg::SetYAxis(2)]);
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
