use {
    crate::{
        game_state::GameState,
        ui::{plot, tabs::TabsBuilder},
    },
    futures_channel::mpsc,
    wasm_bindgen::{JsCast, JsValue},
    web_sys::{Document, HtmlElement, Node},
};

#[derive(Debug, Clone, Copy)]
pub enum Msg {
    Update,
    Chop,
    Sell,
    HireLumberjack,
    HireMiner,
    HireRecruiter,
    HireMonster,
    BuildFactory,
    BuildFurnace,
    BuildBank,
    SetXAxis(u8),
    SetYAxis(u8),
}

pub struct Actor {
    state: GameState,
    plot_tx: mpsc::UnboundedSender<plot::Msg>,
    rx: mpsc::UnboundedReceiver<Msg>,
    quantities: Vec<Node>,
    x_axis_quantity: u8,
    y_axis_quantity: u8,
}

impl Actor {
    pub fn create(
        document: &Document,
        tabs: &mut TabsBuilder,
        plot_tx: mpsc::UnboundedSender<plot::Msg>,
    ) -> Result<(Self, mpsc::UnboundedSender<Msg>), JsValue> {
        let resources = document.create_element("div")?;
        resources.set_class_name("tabcontent");

        let labels = [
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
        let mut nodes = Vec::with_capacity(labels.len());
        for name in labels {
            let cell = document.create_element("div")?;
            let cell: HtmlElement = cell.unchecked_into();
            cell.style().set_property("display", "block")?;

            let label = document.create_element("p")?;
            label.set_text_content(Some(name));
            cell.append_child(&label)?;

            let amount = document.create_element("p")?;
            amount.set_text_content(Some("0"));
            cell.append_child(&amount)?;

            resources.append_child(&cell)?;
            nodes.push(amount.into());
        }

        tabs.with("Resources".into(), resources.unchecked_into())?;

        let (tx, rx) = mpsc::unbounded();
        let this = Self {
            state: GameState::default(),
            plot_tx,
            rx,
            quantities: nodes,
            x_axis_quantity: 0,
            y_axis_quantity: 0,
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
            Msg::Update => {
                self.state.update();

                // Update text in UI
                for (node, amount) in self.quantities.iter().zip(self.state.view_resources()) {
                    node.set_text_content(Some(&format!("{amount}")));
                }

                let x = select_quantity_by_index(&self.state, self.x_axis_quantity);
                let y = select_quantity_by_index(&self.state, self.y_axis_quantity);
                self.plot_tx.unbounded_send(plot::Msg::Push((x, y))).ok();
            }
            Msg::Chop => self.state.chop(),
            Msg::Sell => self.state.sell_wood(),
            Msg::HireLumberjack => self.state.hire_lumberjack(),
            Msg::HireMiner => self.state.hire_miner(),
            Msg::HireRecruiter => self.state.hire_recruiter(),
            Msg::HireMonster => self.state.hire_monster(),
            Msg::BuildFactory => self.state.build_factory(),
            Msg::BuildFurnace => self.state.build_furnace(),
            Msg::BuildBank => self.state.build_bank(),
            Msg::SetXAxis(value) => {
                self.x_axis_quantity = value;
            }
            Msg::SetYAxis(value) => {
                self.y_axis_quantity = value;
            }
        }
    }
}

fn select_quantity_by_index(state: &GameState, index: u8) -> f64 {
    match index {
        0 => (state.view_time() as f64) / 1000.0,
        1 => state.wood_f64(),
        2 => state.gold_f64(),
        3 => state.energy_f64(),
        4 => state.miners_f64(),
        5 => state.lumberjacks_f64(),
        6 => state.recruiters_f64(),
        7 => state.monsters_f64(),
        8 => state.factories_f64(),
        9 => state.furnaces_f64(),
        10 => state.banks_f64(),
        _ => 0.0,
    }
}
