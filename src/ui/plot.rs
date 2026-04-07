use {
    crate::ui::goal_checkers::GoalCheckerFn,
    futures_channel::mpsc,
    plotters::{
        coord::Shift,
        prelude::{
            BLACK, ChartBuilder, DrawingArea, DrawingAreaErrorKind, IntoDrawingArea, LineSeries,
            WHITE,
        },
        style::ShapeStyle,
    },
    plotters_backend::DrawingBackend,
    plotters_canvas::CanvasBackend,
    std::collections::VecDeque,
    wasm_bindgen::{JsCast, JsValue},
    web_sys::{Document, HtmlCanvasElement, HtmlElement},
};

const MAX_PLOT_HISTORY: usize = 1000;

type CanvasError = <CanvasBackend as DrawingBackend>::ErrorType;
type DrawingError = DrawingAreaErrorKind<CanvasError>;

struct MyCanvasError {
    inner: DrawingError,
}

impl From<DrawingError> for MyCanvasError {
    fn from(value: DrawingError) -> Self {
        Self { inner: value }
    }
}

impl From<MyCanvasError> for JsValue {
    fn from(value: MyCanvasError) -> Self {
        let msg = value.inner;
        JsValue::from_str(&format!("{msg}"))
    }
}

pub enum Msg {
    Draw,
    Clear,
    Push((f64, f64)),
    SetGoal(GoalCheckerFn),
}

pub struct PlotActor {
    root: DrawingArea<CanvasBackend, Shift>,
    pts: VecDeque<(f64, f64)>,
    rx: mpsc::UnboundedReceiver<Msg>,
    goal_checker: fn(&VecDeque<(f64, f64)>) -> bool,
    goal_notification: mpsc::UnboundedSender<()>,
}

impl PlotActor {
    pub fn create(
        document: &Document,
        body: &HtmlElement,
        goal_notification: mpsc::UnboundedSender<()>,
    ) -> Result<(Self, mpsc::UnboundedSender<Msg>), JsValue> {
        let canvas = document.create_element("canvas")?;
        let canvas: HtmlCanvasElement = canvas.unchecked_into();
        canvas.set_width(800);
        canvas.set_height(800); // Set height = width for square chart area
        body.append_child(&canvas)?;

        let backend = CanvasBackend::with_canvas_object(canvas)
            .ok_or_else(|| JsValue::from_str("Failed to create backend"))?;
        let root = backend.into_drawing_area();
        let (tx, rx) = mpsc::unbounded();
        let this = Self {
            root,
            pts: VecDeque::new(),
            rx,
            goal_checker: no_goal,
            goal_notification,
        };
        Ok((this, tx))
    }

    pub fn spawn(mut self) {
        wasm_bindgen_futures::spawn_local(async move {
            while let Ok(msg) = self.rx.recv().await {
                self.process(msg);
            }
        })
    }

    fn process(&mut self, msg: Msg) {
        match msg {
            Msg::Draw => {
                self.draw().ok();
            }
            Msg::Clear => self.clear(),
            Msg::Push((x, y)) => self.push_pt(x, y),
            Msg::SetGoal(goal_checker) => {
                self.goal_checker = goal_checker;
            }
        }
    }

    fn draw(&self) -> Result<(), JsValue> {
        create_chart(&self.root, &self.pts)?;
        Ok(())
    }

    fn push_pt(&mut self, x: f64, y: f64) {
        self.pts.push_back((x, y));
        if MAX_PLOT_HISTORY < self.pts.len() {
            self.pts.pop_front();
        }
        if (self.goal_checker)(&self.pts) {
            self.goal_notification.unbounded_send(()).ok();
            self.goal_checker = no_goal;
        }
    }

    fn clear(&mut self) {
        self.pts.clear();
    }
}

fn create_chart(
    root: &DrawingArea<CanvasBackend, Shift>,
    pts: &VecDeque<(f64, f64)>,
) -> Result<(), MyCanvasError> {
    root.fill(&WHITE)?;
    let (min_x, min_y) = pts.front().copied().unwrap_or((0.0, 0.0));
    let (min_x, max_x, min_y, max_y) = pts.iter().copied().fold(
        (min_x, min_x, min_y, min_y),
        |(min_x, max_x, min_y, max_y), (x, y)| {
            let min_x = if x < min_x { x } else { min_x };
            let max_x = if max_x < x { x } else { max_x };
            let min_y = if y < min_y { y } else { min_y };
            let max_y = if max_y < y { y } else { max_y };
            (min_x, max_x, min_y, max_y)
        },
    );
    let dx = (max_x.abs() / 100.0).max(0.01);
    let dy = (max_y.abs() / 100.0).max(0.01);
    let mut chart = ChartBuilder::on(root)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .margin(20)
        .build_cartesian_2d((min_x - dx)..(max_x + dx), (min_y - dy)..(max_y + dy))?;
    chart.configure_mesh().draw()?;
    let style: ShapeStyle = BLACK.into();
    chart.draw_series(LineSeries::new(pts.iter().copied(), style.stroke_width(3)))?;
    root.present()?;

    Ok(())
}

const fn no_goal(_: &VecDeque<(f64, f64)>) -> bool {
    false
}
