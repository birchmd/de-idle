use {
    self::ui::{
        actor::{Actor, Msg},
        plot::{self, PlotActor},
    },
    futures_channel::mpsc,
    gloo_timers::callback::Interval,
    wasm_bindgen::{JsCast, JsValue},
    web_sys::{Document, HtmlElement},
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

    create_axis_selectors(&document, &body, plot_tx.clone(), tx.clone())?;
    create_pause_button(&document, &body, tx.clone())?;

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
        let button = ui::create_button(&document, name, move |_| {
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

fn create_axis_selectors(
    document: &Document,
    body: &HtmlElement,
    plot_tx: mpsc::UnboundedSender<plot::Msg>,
    tx: mpsc::UnboundedSender<Msg>,
) -> Result<(), JsValue> {
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
    let y_axis_selector = ui::create_drop_down(document, labels[1], labels, move |value| {
        local_plot_tx.unbounded_send(plot::Msg::Clear).ok();
        let index = labels
            .iter()
            .position(|name| name == &value)
            .unwrap_or_default();
        local_tx.unbounded_send(Msg::SetYAxis(index as u8)).ok();
    })?;

    let x_axis_selector = ui::create_drop_down(document, labels[1], labels, move |value| {
        plot_tx.unbounded_send(plot::Msg::Clear).ok();
        let index = labels
            .iter()
            .position(|name| name == &value)
            .unwrap_or_default();
        tx.unbounded_send(Msg::SetXAxis(index as u8)).ok();
    })?;

    let axes_select_table = document.create_element("table")?;
    let row = document.create_element("tr")?;
    axes_select_table.append_child(&row)?;

    let cell = document.create_element("td")?;
    cell.set_text_content(Some("x-axis: "));
    row.append_child(&cell)?;
    let cell = document.create_element("td")?;
    cell.append_child(&x_axis_selector)?;
    row.append_child(&cell)?;

    let row = document.create_element("tr")?;
    axes_select_table.append_child(&row)?;
    let cell = document.create_element("td")?;
    cell.set_text_content(Some("y-axis: "));
    row.append_child(&cell)?;
    let cell = document.create_element("td")?;
    cell.append_child(&y_axis_selector)?;
    row.append_child(&cell)?;

    body.append_child(&axes_select_table)?;

    Ok(())
}

fn create_pause_button(
    document: &Document,
    body: &HtmlElement,
    tx: mpsc::UnboundedSender<Msg>,
) -> Result<(), JsValue> {
    let pause_button = ui::create_button(document, "Pause", move |button| {
        tx.unbounded_send(Msg::TogglePause).ok();
        if button.text_content().as_deref() == Some("Pause") {
            button.set_text_content(Some("Play"));
        } else {
            button.set_text_content(Some("Pause"));
        }
    })?;
    pause_button.style().set_property("line-height", "1.6")?;
    pause_button.style().set_property("font-size", "1rem")?;
    body.append_child(&pause_button)?;

    Ok(())
}
