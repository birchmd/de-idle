use {
    self::ui::{
        actor::{Actor, Msg},
        plot::{self, PlotActor},
    },
    gloo_timers::callback::Interval,
    wasm_bindgen::JsValue,
};

mod game_state;
mod ui;

fn main() -> Result<(), JsValue> {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    let (plot_actor, plot_tx) = PlotActor::create(&document, &body)?;
    let (actor, tx) = Actor::create(&document, &body, plot_tx.clone())?;
    plot_actor.spawn();
    actor.spawn();

    let labels = &[
        "Time",
        "Wood",
        "Gold",
        "Energy",
        "Miner",
        "Lumberjack",
        "Recruiter",
        "Monster",
        "Factory",
        "Furnace",
        "Bank",
    ];
    let local_plot_tx = plot_tx.clone();
    let local_tx = tx.clone();
    let y_axis_selector = ui::create_drop_down(&document, labels, move |value| {
        local_plot_tx.unbounded_send(plot::Msg::Clear).ok();
        let index = labels
            .iter()
            .position(|name| name == &value)
            .unwrap_or_default();
        local_tx.unbounded_send(Msg::SetYAxis(index as u8)).ok();
    })?;

    let local_plot_tx = plot_tx.clone();
    let local_tx = tx.clone();
    let x_axis_selector = ui::create_drop_down(&document, labels, move |value| {
        local_plot_tx.unbounded_send(plot::Msg::Clear).ok();
        let index = labels
            .iter()
            .position(|name| name == &value)
            .unwrap_or_default();
        local_tx.unbounded_send(Msg::SetXAxis(index as u8)).ok();
    })?;
    body.append_child(&x_axis_selector)?;
    body.append_child(&y_axis_selector)?;

    for (name, msg) in [
        ("Chop", Msg::Chop),
        ("Sell wood", Msg::Sell),
        ("Hire miner", Msg::HireMiner),
        ("Hire lumberjack", Msg::HireLumberjack),
        ("Hire Recruiter", Msg::HireRecruiter),
        ("Hire Monster", Msg::HireMonster),
        ("Build factory", Msg::BuildFactory),
        ("Build furnace", Msg::BuildFurnace),
        ("Build bank", Msg::BuildBank),
    ] {
        let local_tx = tx.clone();
        let button = ui::create_button(&document, name, move || {
            local_tx.unbounded_send(msg).ok();
        })?;
        body.append_child(&button)?;
    }

    // State update loop
    Interval::new(10, move || {
        tx.unbounded_send(Msg::Update).ok();
    })
    .forget();

    // Plot update loop
    Interval::new(33, move || {
        plot_tx.unbounded_send(plot::Msg::Draw).ok();
    })
    .forget();

    Ok(())
}
