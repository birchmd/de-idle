use std::collections::VecDeque;

#[cfg(test)]
use {
    super::tests::{add_gold, create_miner, create_pts, create_pts_with_intervention},
    crate::ui::actor::Msg,
};

pub fn horizontal_goal_checker(pts: &VecDeque<(f64, f64)>) -> bool {
    let Some((x0, y0)) = pts.front() else {
        return false;
    };
    if pts.len() < 100 {
        return false;
    }
    pts.iter().any(|(x, _)| x != x0) && pts.iter().all(|(_, y)| y == y0)
}

pub fn vertical_goal_checker(pts: &VecDeque<(f64, f64)>) -> bool {
    let Some((x0, y0)) = pts.front() else {
        return false;
    };
    if pts.len() < 100 {
        return false;
    }
    pts.iter().any(|(_, y)| y != y0) && pts.iter().all(|(x, _)| x == x0)
}

pub fn step_goal_checker(pts: &VecDeque<(f64, f64)>) -> bool {
    let Some((x0, y0)) = pts.front() else {
        return false;
    };
    let Some((xn, yn)) = pts.back() else {
        return false;
    };
    if pts.len() < 100 {
        return false;
    }
    let Some(i) = pts.iter().position(|(_, y)| y != y0) else {
        return false;
    };
    x0 != xn && pts.iter().skip(i + 1).all(|(_, y)| y == yn)
}

pub fn positive_slope_goal_checker(pts: &VecDeque<(f64, f64)>) -> bool {
    let Some((x0, y0)) = pts.front() else {
        return false;
    };
    let Some((xn, yn)) = pts.back() else {
        return false;
    };
    if pts.len() < 100 {
        return false;
    }
    if x0 == xn || yn <= y0 {
        return false;
    }

    let m = (yn - y0) / (xn - x0);
    let b = yn - m * xn;

    pts.iter().all(|(x, y)| {
        let y_calc = m * x + b;
        (y_calc - y).abs() < 0.001
    })
}

pub fn negative_slope_goal_checker(pts: &VecDeque<(f64, f64)>) -> bool {
    let Some((x0, y0)) = pts.front() else {
        return false;
    };
    let Some((xn, yn)) = pts.back() else {
        return false;
    };
    if pts.len() < 100 {
        return false;
    }
    if x0 == xn || yn >= y0 {
        return false;
    }

    let m = (yn - y0) / (xn - x0);
    let b = yn - m * xn;

    pts.iter().all(|(x, y)| {
        let y_calc = m * x + b;
        (y_calc - y).abs() < 0.001
    })
}

pub fn peak_goal_checker(pts: &VecDeque<(f64, f64)>) -> bool {
    let Some((x0, y0)) = pts.front() else {
        return false;
    };
    let Some((xn, yn)) = pts.back() else {
        return false;
    };
    if pts.len() < 100 {
        return false;
    }
    let (xm, ym) = pts.iter().fold(
        (x0, y0),
        |(xm, ym), (x, y)| {
            if ym < y { (x, y) } else { (xm, ym) }
        },
    );

    // Peak cannot be at the beginning or end.
    if xm == x0 || xm == xn {
        return false;
    }

    // The line must go up and back down (not only plateau)
    if y0 == ym || yn == ym {
        return false;
    }

    // Require the peak to be within the middle third of the plot
    if (xm - x0) > 2.0 * (xn - xm) {
        return false;
    }

    let m = (ym - y0) / (xm - x0);
    let b1 = ym - m * xm;
    let b2 = yn + m * xn;

    pts.iter().all(|(x, y)| {
        let y_calc = if x <= xm { m * x + b1 } else { -m * x + b2 };
        // Must be close to the calculated line or part of the peak
        // (which is allowed to be a short plateau).
        (y_calc - y).abs() < 0.001 || y == ym
    })
}

#[test]
fn test_horizontal() {
    let pts = create_pts(std::iter::once(Msg::SetXAxis(0)));
    assert!(horizontal_goal_checker(&pts));
}

#[test]
fn test_vertical() {
    let pts = create_pts(std::iter::once(Msg::SetYAxis(0)));
    assert!(vertical_goal_checker(&pts));
}

#[test]
fn test_step() {
    let updates = || std::iter::repeat_n(Msg::Update, 500);
    let time_msgs = updates().chain(std::iter::once(Msg::Chop)).chain(updates());
    let pts = create_pts_with_intervention(std::iter::once(Msg::SetXAxis(0)), time_msgs);
    assert!(step_goal_checker(&pts));
}

#[test]
fn test_slope() {
    // Positive slope passes positive check, fails negative one
    let msgs = create_miner().chain(vec![Msg::SetXAxis(0), Msg::SetYAxis(2)]);
    let pts = create_pts(msgs);
    assert!(positive_slope_goal_checker(&pts));
    assert!(!negative_slope_goal_checker(&pts));

    // Vice versa on negative slope
    let msgs = add_gold(1100)
        .chain(std::iter::once(Msg::HireLumberjack))
        .chain(vec![Msg::SetXAxis(0), Msg::SetYAxis(2)]);
    let pts = create_pts(msgs);
    assert!(!positive_slope_goal_checker(&pts));
    assert!(negative_slope_goal_checker(&pts));

    // Both fail on horizontal line
    let pts = create_pts(std::iter::once(Msg::SetXAxis(0)));
    assert!(!positive_slope_goal_checker(&pts));
    assert!(!negative_slope_goal_checker(&pts));

    // Both fail on vertical line
    let pts = create_pts(std::iter::once(Msg::SetYAxis(0)));
    assert!(!positive_slope_goal_checker(&pts));
    assert!(!negative_slope_goal_checker(&pts));
}

#[test]
fn test_peak() {
    let updates = || std::iter::repeat_n(Msg::Update, 500);
    let init_msgs = create_miner().chain(vec![Msg::SetXAxis(0), Msg::SetYAxis(2)]);
    let time_msgs = updates()
        .chain(std::iter::repeat_n(Msg::HireLumberjack, 2))
        .chain(updates());
    let pts = create_pts_with_intervention(init_msgs, time_msgs);
    assert!(peak_goal_checker(&pts));

    // Fails on horizontal line
    let pts = create_pts(std::iter::once(Msg::SetXAxis(0)));
    assert!(!peak_goal_checker(&pts));

    // Fails on vertical line
    let pts = create_pts(std::iter::once(Msg::SetYAxis(0)));
    assert!(!peak_goal_checker(&pts));
}
