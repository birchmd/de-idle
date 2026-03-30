use {
    crate::ui::{plot::Msg, tabs::TabsBuilder},
    futures_channel::mpsc,
    std::collections::VecDeque,
    wasm_bindgen::{JsCast, JsValue},
    web_sys::{Document, Element, HtmlElement},
};

type GoalCheckerFn = fn(&VecDeque<(f64, f64)>) -> bool;

const GOAL_CHECKERS: [GoalCheckerFn; 7] = [
    horizontal_goal_checker,
    vertical_goal_checker,
    step_goal_checker,
    positive_slope_goal_checker,
    negative_slope_goal_checker,
    peak_goal_checker,
    parabola_goal_checker,
];
const MAX_GOALS: usize = GOAL_CHECKERS.len() - 1;

pub struct MessagesManager {
    resources: Vec<HtmlElement>,
    goal_checkboxes: Vec<Element>,
    messages_header: Element,
    message_rows: Vec<HtmlElement>,
    goal_rx: mpsc::UnboundedReceiver<()>,
    plot_tx: mpsc::UnboundedSender<Msg>,
    completed_goals: usize,
}

impl MessagesManager {
    pub fn new(
        document: &Document,
        tabs: &mut TabsBuilder,
        resources: Vec<HtmlElement>,
        goal_checkboxes: Vec<Element>,
        goal_rx: mpsc::UnboundedReceiver<()>,
        plot_tx: mpsc::UnboundedSender<Msg>,
    ) -> Result<Self, JsValue> {
        let text = document.create_element("div")?;
        text.set_class_name("tabcontent");

        let message_rows = vec![
            add_message(document, &text, game_completed())?,
            add_message(document, &text, factories_intro())?,
            add_message(document, &text, peak_goal_intro())?,
            add_message(document, &text, lumberjacks_intro())?,
            add_message(document, &text, miners_intro())?,
            add_message(document, &text, wood_intro())?,
            add_message(document, &text, vertical_line_preamble())?,
        ];

        let label = document.create_element("p")?;
        label.set_text_content(Some(welcome_message()));
        text.append_child(&label)?;

        let messages_header = tabs.with("Messages".into(), text.unchecked_into())?;

        Ok(Self {
            resources,
            goal_checkboxes,
            messages_header,
            message_rows,
            goal_rx,
            plot_tx,
            completed_goals: 0,
        })
    }

    pub fn click_header(&self) {
        self.messages_header.unchecked_ref::<HtmlElement>().click();
    }

    pub fn spawn(mut self) {
        // Initial goal
        self.plot_tx
            .unbounded_send(Msg::SetGoal(horizontal_goal_checker))
            .ok();

        wasm_bindgen_futures::spawn_local(async move {
            while self.goal_rx.recv().await.is_ok() {
                self.handle_goal_completed();
            }
        })
    }

    fn open_resources(&self) {
        match self.completed_goals {
            0 => (),
            1 => {
                self.reveal_resource(0);
            }
            2 => {
                self.reveal_resource(1);
                self.reveal_resource(3);
            }
            3 => {
                self.reveal_resource(4);
            }
            5 => {
                self.reveal_resource(2);
                self.reveal_resource(7);
                self.reveal_resource(8);
            }
            _ => (),
        }
    }

    fn reveal_resource(&self, index: usize) {
        self.resources[index]
            .style()
            .set_property("display", "block")
            .ok();
    }

    fn handle_goal_completed(&mut self) {
        self.messages_header.set_text_content(Some("Messages (*)"));
        self.goal_checkboxes[self.completed_goals].set_text_content(Some("✅"));

        let n = self.message_rows.len();
        self.message_rows[n - 1 - self.completed_goals]
            .style()
            .set_property("display", "block")
            .ok();

        self.open_resources();

        if self.completed_goals < MAX_GOALS {
            self.completed_goals += 1;
            self.plot_tx
                .unbounded_send(Msg::SetGoal(GOAL_CHECKERS[self.completed_goals]))
                .ok();
        }
    }
}

fn add_message(
    document: &Document,
    tab_content: &Element,
    text: &str,
) -> Result<HtmlElement, JsValue> {
    let label: HtmlElement = document.create_element("p")?.unchecked_into();
    label.set_text_content(Some(text));
    tab_content.append_child(&label)?;
    label.style().set_property("display", "none")?;
    Ok(label)
}

const fn welcome_message() -> &'static str {
    r#"Welcome to DE-Idle!
In this game you will manipulate the resources at your disposal to produce particular patterns in the above plot.
In the "Goals" tab you will see the list of goals you have to complete (they must be completed in order).
We start off with an easy one. To produce a horizontal line in the plot simply change the x-axis to plot Time.
Plotting a constant quantity (wood in this case -- we don't have a way to change the amount of wood we have yet) over time creates a flat line.
Set the x-axis to plot Time to complete the first goal!"#
}

const fn vertical_line_preamble() -> &'static str {
    r#"Nice job!
We can change a horizontal line to a vertical line we simply swap the axes in the plot.
Change the y-axis to plot Time and the x-axis to plot Wood to complete the second goal!"#
}

const fn wood_intro() -> &'static str {
    r#"Great! Looks like you have the hang of things. Let's step up the difficulty a bit.
On the Resources tab you now have access to Wood. You can manually chop down trees to obtain Wood.
Can you use this new skill to complete the next goal?"#
}

const fn miners_intro() -> &'static str {
    r#"Way to step up to the challenge!
Clicking the Chop button was a bit of manual effort, let's see if you can build something that works for you instead of the other way around.
On the Resources tab you have discovered Gold as a new kind of resource.
You can obtain Gold manually by selling Wood, but that's a tiresome way to go about things.
The more automatic approach is to buy a Miner Bot that will dig up gold for you!
Can you use these new resources to accomplish the next goal?"#
}

const fn lumberjacks_intro() -> &'static str {
    r#"You're really on a roll!
Automatically getting Gold is cool, but it would be even better if we could have zero-effort wood too.
On the Resources tab you can now hire Lumberjacks! They can chop wood for you, but only if you have the gold to pay them.
Go ahead and make use of them to complete the next goal."#
}

const fn peak_goal_intro() -> &'static str {
    r#"You're really getting the hang of this! You already have everything you need to complete the next goal as well."#
}

const fn factories_intro() -> &'static str {
    r#"Amazing! With all this wood we can really start industrializing.
You now have access to energy as a resources as well as two buildings: furnaces and factories.
Take a look at them in the Resources tab and figure out how to accomplish the next goal."#
}

const fn game_completed() -> &'static str {
    r#"Great work! You have completed the game. Thanks for playing!"#
}

fn horizontal_goal_checker(pts: &VecDeque<(f64, f64)>) -> bool {
    let Some((x0, y0)) = pts.front() else {
        return false;
    };
    if pts.len() < 100 {
        return false;
    }
    pts.iter().any(|(x, _)| x != x0) && pts.iter().all(|(_, y)| y == y0)
}

fn vertical_goal_checker(pts: &VecDeque<(f64, f64)>) -> bool {
    let Some((x0, y0)) = pts.front() else {
        return false;
    };
    if pts.len() < 100 {
        return false;
    }
    pts.iter().any(|(_, y)| y != y0) && pts.iter().all(|(x, _)| x == x0)
}

fn step_goal_checker(pts: &VecDeque<(f64, f64)>) -> bool {
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

fn positive_slope_goal_checker(pts: &VecDeque<(f64, f64)>) -> bool {
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

fn negative_slope_goal_checker(pts: &VecDeque<(f64, f64)>) -> bool {
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

fn peak_goal_checker(pts: &VecDeque<(f64, f64)>) -> bool {
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

fn parabola_goal_checker(pts: &VecDeque<(f64, f64)>) -> bool {
    let Some((x0, y0)) = pts.front() else {
        return false;
    };
    let Some((xn, yn)) = pts.back() else {
        return false;
    };
    if pts.len() < 100 {
        return false;
    }
    // For an upward-facing parabola, the vertex is the lowest point.
    let (xv, yv) = pts.iter().fold(
        (x0, y0),
        |(xv, yv), (x, y)| {
            if y < yv { (x, y) } else { (xv, yv) }
        },
    );

    // Do not allow the vertex to be at the start or the end.
    if x0 == xv || xn == xv {
        return false;
    }

    let a = (yn - yv) / ((xn - xv) * (xn - xv));

    pts.iter().all(|(x, y)| {
        let y_calc = a * (x - xv) * (x - xv) + yv;
        let relative_deviation = (y_calc - y).abs() / y_calc;
        // Require points to be within 1% of calculated
        relative_deviation < 0.01
    })
}
