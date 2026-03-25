use {
    self::ui::{
        actor::{Actor, Msg},
        plot::{self, PlotActor},
    },
    gloo_timers::callback::Interval,
    wasm_bindgen::{JsCast, JsValue},
};

mod game_state;
mod ui;

fn main() -> Result<(), JsValue> {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    let mut tabs = ui::tabs::TabsBuilder::new(&document)?;

    let text = document.create_element("div")?;
    text.set_class_name("tabcontent");
    let label = document.create_element("div")?;
    label.set_text_content(Some("Hello, world!"));
    text.append_child(&label)?;
    tabs.with("Messages".into(), text.unchecked_into())?;

    let (plot_actor, plot_tx) = PlotActor::create(&document, &body)?;
    let (actor, tx) = Actor::create(&document, &mut tabs, plot_tx.clone())?;
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

    let buttons = document.create_element("div")?;
    buttons.set_class_name("tabcontent");
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
        buttons.append_child(&button)?;
    }
    tabs.with("Actions".into(), buttons.unchecked_into())?;

    tabs.build(&body)?;

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
