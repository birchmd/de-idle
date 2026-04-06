use {
    crate::{
        game_state::{
            BANK_GOLD_COST, BANK_WOOD_COST, FACTORY_GOLD_COST, FACTORY_WOOD_COST,
            FURNACE_GOLD_COST, MINER_GOLD_COST, MONSTER_ENERGY_COST, MONSTER_RECRUITER_COST,
            RECRUITER_ENERGY_COST, WOOD_PER_GOLD,
        },
        ui::{actor::Msg, tabs::TabsBuilder},
    },
    futures_channel::mpsc,
    wasm_bindgen::{JsCast, JsValue, closure::Closure},
    web_sys::{Document, Element, HtmlElement},
};

pub struct Dashboard {
    pub amounts: Vec<Element>,
    pub rows: Vec<HtmlElement>,
}

pub fn create_dashboard(
    document: &Document,
    tabs: &mut TabsBuilder,
    tx: mpsc::UnboundedSender<Msg>,
) -> Result<Dashboard, JsValue> {
    let tab_content = document.create_element("div")?;
    tab_content.set_class_name("tabcontent");

    let table = document.create_element("table")?;
    tab_content.append_child(&table)?;

    let resources = [
        wood_resource(tx.clone()),
        gold_resource(tx.clone()),
        energy_resource(tx.clone()),
        miner_resource(tx.clone()),
        lumberjack_resource(tx.clone()),
        recruiter_resource(tx.clone()),
        monster_resource(tx.clone()),
        factory_resource(tx.clone()),
        furnace_resource(tx.clone()),
        bank_resource(tx),
    ];
    let mut amounts = Vec::with_capacity(resources.len());
    let mut rows = Vec::with_capacity(resources.len());

    for resource in resources {
        let (amount, row) = create_resource_row(document, &table, resource)?;
        amounts.push(amount);
        rows.push(row);
    }

    tabs.with("Resources".into(), tab_content.unchecked_into())?;

    Ok(Dashboard { amounts, rows })
}

struct ResourceRow {
    remove_fn: Box<dyn FnMut()>,
    remove_label: &'static str,
    remove_description: String,
    name: &'static str,
    add_fn: Box<dyn FnMut()>,
    add_label: &'static str,
    add_description: String,
}

fn create_resource_row(
    document: &Document,
    table: &Element,
    resource: ResourceRow,
) -> Result<(Element, HtmlElement), JsValue> {
    let row = document.create_element("tr")?;
    let row: HtmlElement = row.unchecked_into();

    // All table rows start off hidden by default.
    row.style().set_property("display", "none")?;

    // Remove side
    let cell = document.create_element("td")?;
    let cell: HtmlElement = cell.unchecked_into();
    cell.set_class_name("resourceaction");
    cell.style().set_property("text-align", "center")?;
    let element_kind = if resource.remove_label.is_empty() {
        "h3"
    } else {
        "button"
    };
    let label = document.create_element(element_kind)?;
    label.set_text_content(Some(resource.remove_label));
    let description = document.create_element("p")?;
    description.set_text_content(Some(&resource.remove_description));
    cell.append_child(&label)?;
    cell.append_child(&description)?;
    row.append_child(&cell)?;
    set_onclick(&cell, resource.remove_fn)?;

    // Name and amount
    let cell = document.create_element("td")?;
    let cell: HtmlElement = cell.unchecked_into();
    cell.style().set_property("text-align", "center")?;
    cell.style().set_property("padding", "10px")?;
    let label = document.create_element("h2")?;
    label.set_text_content(Some(resource.name));
    let amount = document.create_element("p")?;
    amount.set_text_content(Some("0"));
    cell.append_child(&label)?;
    cell.append_child(&amount)?;
    row.append_child(&cell)?;

    // Add side
    let cell = document.create_element("td")?;
    let cell: HtmlElement = cell.unchecked_into();
    cell.set_class_name("resourceaction");
    cell.style().set_property("text-align", "center")?;
    let element_kind = if resource.add_label.is_empty() {
        "h3"
    } else {
        "button"
    };
    let label = document.create_element(element_kind)?;
    label.set_text_content(Some(resource.add_label));
    let description = document.create_element("p")?;
    description.set_text_content(Some(&resource.add_description));
    cell.append_child(&label)?;
    cell.append_child(&description)?;
    row.append_child(&cell)?;
    set_onclick(&cell, resource.add_fn)?;

    table.append_child(&row)?;

    Ok((amount, row))
}

fn wood_resource(tx: mpsc::UnboundedSender<Msg>) -> ResourceRow {
    ResourceRow {
        remove_fn: Box::new(|| {}),
        remove_label: "",
        remove_description: String::new(),
        name: "Wood",
        add_fn: Box::new(move || {
            tx.unbounded_send(Msg::Chop).ok();
        }),
        add_label: "Chop",
        add_description: "Go swing your axe to get 1 Wood".into(),
    }
}

fn gold_resource(tx: mpsc::UnboundedSender<Msg>) -> ResourceRow {
    ResourceRow {
        remove_fn: Box::new(|| {}),
        remove_label: "",
        remove_description: String::new(),
        name: "Gold",
        add_fn: Box::new(move || {
            tx.unbounded_send(Msg::Sell).ok();
        }),
        add_label: "Trade",
        add_description: format!("Sell {WOOD_PER_GOLD} Wood to obtain 1 Gold"),
    }
}

fn energy_resource(_tx: mpsc::UnboundedSender<Msg>) -> ResourceRow {
    ResourceRow {
        remove_fn: Box::new(|| {}),
        remove_label: "",
        remove_description: String::new(),
        name: "Energy",
        add_fn: Box::new(|| {}),
        add_label: "",
        add_description: String::new(),
    }
}

fn miner_resource(tx: mpsc::UnboundedSender<Msg>) -> ResourceRow {
    let local_tx = tx.clone();
    ResourceRow {
        remove_fn: Box::new(move || {
            local_tx.unbounded_send(Msg::FireMiner).ok();
        }),
        remove_label: "Dismantle",
        remove_description: "Turn a miner bot into scrap.".into(),
        name: "Miners",
        add_fn: Box::new(move || {
            tx.unbounded_send(Msg::HireMiner).ok();
        }),
        add_label: "Assemble",
        add_description: format!(
            "Spend {MINER_GOLD_COST} Gold to obtain 1 MinerBot. Each miner bot automatically produces 1 Gold per second."
        ),
    }
}

fn lumberjack_resource(tx: mpsc::UnboundedSender<Msg>) -> ResourceRow {
    let local_tx = tx.clone();
    ResourceRow {
        remove_fn: Box::new(move || {
            local_tx.unbounded_send(Msg::FireLumberjack).ok();
        }),
        remove_label: "Fire",
        remove_description: "Put a lumberjack out of a job.".into(),
        name: "Lumberjacks",
        add_fn: Box::new(move || {
            tx.unbounded_send(Msg::HireLumberjack).ok();
        }),
        add_label: "Hire",
        add_description: "No cost to hire a lumberjack. Each lumberjack will produce 1 wood per second as long as they are being paid 1 gold per second.".into(),
    }
}

fn recruiter_resource(tx: mpsc::UnboundedSender<Msg>) -> ResourceRow {
    let local_tx = tx.clone();
    ResourceRow {
        remove_fn: Box::new(move || {
            local_tx.unbounded_send(Msg::FireRecruiter).ok();
        }),
        remove_label: "Cancel",
        remove_description:
            "Stop running an ad; too many lumberjacks could put us out of business!".into(),
        name: "Advertisements",
        add_fn: Box::new(move || {
            tx.unbounded_send(Msg::HireRecruiter).ok();
        }),
        add_label: "Run ads",
        add_description: format!(
            "Spend {RECRUITER_ENERGY_COST} Energy to create an electronic ad. Each advertisement will automatically hire 1 Lumberjack per second per Gold we have. (The ads are more effective the better our company appears to be doing.)"
        ),
    }
}

fn monster_resource(tx: mpsc::UnboundedSender<Msg>) -> ResourceRow {
    let local_tx = tx.clone();
    ResourceRow {
        remove_fn: Box::new(move || {
            local_tx.unbounded_send(Msg::FireMonster).ok();
        }),
        remove_label: "Stake",
        remove_description: "Put a stake through the heart of a monster. Save the lumberjacks!"
            .into(),
        name: "Monsters",
        add_fn: Box::new(move || {
            tx.unbounded_send(Msg::HireMonster).ok();
        }),
        add_label: "Experiment",
        add_description: format!(
            "Spend {MONSTER_ENERGY_COST} Energy and {MONSTER_RECRUITER_COST} Advertisement to create a monster. Each monster will eat 1 lumberjack per second. (The Mad Science department has really done it this time...)"
        ),
    }
}

fn factory_resource(tx: mpsc::UnboundedSender<Msg>) -> ResourceRow {
    let local_tx = tx.clone();
    ResourceRow {
        remove_fn: Box::new(move || { local_tx.unbounded_send(Msg::DemolishFactory).ok();}),
        remove_label: "Renovate",
        remove_description: "Times are changing; let's turn one of those old factories into a swanky waterfront apartment.".into(),
        name: "Factories",
        add_fn: Box::new(move || {tx.unbounded_send(Msg::BuildFactory).ok();}),
        add_label: "Build",
        add_description: format!("Spend {FACTORY_WOOD_COST} Wood and {FACTORY_GOLD_COST} gold to build a miner bot factory. Each miner bot factory will spend 1 Energy to per second automatically produce 1 miner bot."),
    }
}

fn furnace_resource(tx: mpsc::UnboundedSender<Msg>) -> ResourceRow {
    let local_tx = tx.clone();
    ResourceRow {
        remove_fn: Box::new(move || {
            local_tx.unbounded_send(Msg::DemolishFurnace).ok();
        }),
        remove_label: "Close",
        remove_description: "Close down one furnace. Haven't you heard of climate change?".into(),
        name: "Furnaces",
        add_fn: Box::new(move || {
            tx.unbounded_send(Msg::BuildFurnace).ok();
        }),
        add_label: "Build",
        add_description: format!(
            "Spend {FURNACE_GOLD_COST} gold to build a furnace. Each furnace produces 1 energy per second."
        ),
    }
}

fn bank_resource(tx: mpsc::UnboundedSender<Msg>) -> ResourceRow {
    let local_tx = tx.clone();
    ResourceRow {
        remove_fn: Box::new(move || { local_tx.unbounded_send(Msg::DemolishBank).ok();}),
        remove_label: "Collapse",
        remove_description: "When the customers lose confidence in a bank and withdraw all their funds at once it is called a 'bank run.'".into(),
        name: "Banks",
        add_fn: Box::new(move || {tx.unbounded_send(Msg::BuildBank).ok();}),
        add_label: "Build",
        add_description: format!("Spend {BANK_WOOD_COST} Wood and {BANK_GOLD_COST} gold to build a bank. Each bank produces X gold per second, where X is 1% of the current gold amount."),
    }
}

fn set_onclick(element: &HtmlElement, onclick: Box<dyn FnMut()>) -> Result<(), JsValue> {
    let closure = Closure::new(onclick);
    element.set_onclick(Some(closure.as_ref().unchecked_ref()));
    closure.forget();
    Ok(())
}
