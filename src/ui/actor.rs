use {
    crate::{game_state::GameState, ui::plot},
    futures_channel::mpsc,
    wasm_bindgen::JsValue,
    web_sys::Element,
};

#[derive(Debug, Clone, Copy)]
pub enum Msg {
    Update,
    Chop,
    Sell,
    FireLumberjack,
    HireLumberjack,
    FireMiner,
    HireMiner,
    FireRecruiter,
    HireRecruiter,
    FireMonster,
    HireMonster,
    DemolishFactory,
    BuildFactory,
    DemolishFurnace,
    BuildFurnace,
    DemolishBank,
    BuildBank,
    SetXAxis(u8),
    SetYAxis(u8),
    TogglePause,
    Reset,
}

pub struct Actor {
    paused: bool,
    state: GameState,
    plot_tx: mpsc::UnboundedSender<plot::Msg>,
    rx: mpsc::UnboundedReceiver<Msg>,
    quantities: Vec<Element>,
    x_axis_quantity: u8,
    y_axis_quantity: u8,
}

impl Actor {
    pub fn create(
        rx: mpsc::UnboundedReceiver<Msg>,
        quantities: Vec<Element>,
        plot_tx: mpsc::UnboundedSender<plot::Msg>,
    ) -> Result<Self, JsValue> {
        Ok(Self {
            paused: false,
            state: GameState::default(),
            plot_tx,
            rx,
            quantities,
            x_axis_quantity: 1,
            y_axis_quantity: 1,
        })
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
                if self.paused {
                    return;
                }
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
            Msg::FireLumberjack => self.state.remove_lumberjack(),
            Msg::HireLumberjack => self.state.hire_lumberjack(),
            Msg::FireMiner => self.state.remove_miner(),
            Msg::HireMiner => self.state.hire_miner(),
            Msg::FireRecruiter => self.state.remove_recruiter(),
            Msg::HireRecruiter => self.state.hire_recruiter(),
            Msg::FireMonster => self.state.remove_monster(),
            Msg::HireMonster => self.state.hire_monster(),
            Msg::DemolishFactory => self.state.remove_factory(),
            Msg::BuildFactory => self.state.build_factory(),
            Msg::DemolishFurnace => self.state.remove_furnace(),
            Msg::BuildFurnace => self.state.build_furnace(),
            Msg::DemolishBank => self.state.remove_bank(),
            Msg::BuildBank => self.state.build_bank(),
            Msg::SetXAxis(value) => {
                self.x_axis_quantity = value;
            }
            Msg::SetYAxis(value) => {
                self.y_axis_quantity = value;
            }
            Msg::TogglePause => {
                self.paused = !self.paused;
            }
            Msg::Reset => self.state.reset(),
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
