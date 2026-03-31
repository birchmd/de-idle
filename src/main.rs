use {
    self::ui::{
        actor::{Actor, Msg},
        plot::{self, PlotActor},
    },
    futures_channel::mpsc,
    gloo_timers::callback::Interval,
    wasm_bindgen::JsValue,
    web_sys::{Document, HtmlElement},
};

mod game_state;
mod sliding3;
mod ui;

fn main() -> Result<(), JsValue> {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    let mut tabs = ui::tabs::TabsBuilder::new(&document)?;

    let (goal_tx, goal_rx) = mpsc::unbounded();
    let (plot_actor, plot_tx) = PlotActor::create(&document, &body, goal_tx)?;
    let (tx, rx) = mpsc::unbounded();
    let dashboard = ui::dashboard::create_dashboard(&document, &mut tabs, tx.clone())?;
    let actor = Actor::create(rx, dashboard.amounts, plot_tx.clone());
    plot_actor.spawn();
    actor.spawn();

    let checkboxes = ui::goals::create_goals_tab(&document, &mut tabs)?;

    let messages = ui::messages::MessagesManager::new(
        &document,
        &mut tabs,
        dashboard.rows,
        checkboxes,
        goal_rx,
        plot_tx.clone(),
    )?;

    create_axis_selectors(&document, &body, plot_tx.clone(), tx.clone())?;
    create_pause_button(&document, &body, tx.clone())?;
    create_reset_button(&document, &body, tx.clone())?;

    tabs.build(&body)?;

    // Start with the `Messages` tab selected.
    messages.click_header();
    messages.spawn();

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

fn create_reset_button(
    document: &Document,
    body: &HtmlElement,
    tx: mpsc::UnboundedSender<Msg>,
) -> Result<(), JsValue> {
    let reset_button = ui::create_button(document, "Reset Resources", move |_| {
        tx.unbounded_send(Msg::Reset).ok();
    })?;
    reset_button.style().set_property("line-height", "1.6")?;
    reset_button.style().set_property("font-size", "1rem")?;
    body.append_child(&reset_button)?;

    Ok(())
}
