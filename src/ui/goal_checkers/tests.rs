use {
    crate::{
        game_state::{
            BANK_GOLD_COST, BANK_WOOD_COST, FACTORY_GOLD_COST, FACTORY_WOOD_COST,
            FURNACE_GOLD_COST, MINER_GOLD_COST, WOOD_PER_GOLD,
        },
        ui::{
            actor::{Actor, Msg},
            plot,
        },
    },
    futures_channel::mpsc,
    std::{collections::VecDeque, iter},
};

const MAX_PLOT_HISTORY: usize = 1000;

// Takes messages for initializing the state and returns the
// points that would be plotted in the UI.
pub fn create_pts_with_intervention<I, J>(init_messages: I, time_msgs: J) -> VecDeque<(f64, f64)>
where
    I: IntoIterator<Item = Msg>,
    J: IntoIterator<Item = Msg>,
{
    let (_, rx) = mpsc::unbounded();
    let (plot_tx, plot_rx) = mpsc::unbounded();
    let plot_actor = StubPlotActor::new(plot_rx);
    let mut actor = Actor::create(rx, Vec::new(), plot_tx);

    // Initialize state
    for msg in init_messages {
        actor.process(msg);
    }

    // Time passes
    for msg in time_msgs {
        actor.process(msg);
    }

    // Return plot points
    plot_actor.drain()
}

pub fn create_pts<I>(init_messages: I) -> VecDeque<(f64, f64)>
where
    I: IntoIterator<Item = Msg>,
{
    create_pts_with_intervention(init_messages, iter::repeat_n(Msg::Update, 1000))
}

pub fn add_gold(amount: usize) -> impl Iterator<Item = Msg> {
    iter::repeat_n(Msg::Chop, (WOOD_PER_GOLD as usize) * amount)
        .chain(iter::repeat_n(Msg::Sell, amount))
}

pub fn create_miner() -> impl Iterator<Item = Msg> {
    add_gold(MINER_GOLD_COST as usize).chain(iter::once(Msg::HireMiner))
}

pub fn build_furnace() -> impl Iterator<Item = Msg> {
    add_gold(FURNACE_GOLD_COST as usize).chain(iter::once(Msg::BuildFurnace))
}

pub fn build_factory() -> impl Iterator<Item = Msg> {
    add_gold(FACTORY_GOLD_COST as usize)
        .chain(iter::repeat_n(Msg::Chop, FACTORY_WOOD_COST as usize))
        .chain(iter::once(Msg::BuildFactory))
}

pub fn build_bank() -> impl Iterator<Item = Msg> {
    add_gold(BANK_GOLD_COST as usize)
        .chain(iter::repeat_n(Msg::Chop, BANK_WOOD_COST as usize))
        .chain(iter::once(Msg::BuildBank))
}

struct StubPlotActor {
    pts: VecDeque<(f64, f64)>,
    rx: mpsc::UnboundedReceiver<plot::Msg>,
}

impl StubPlotActor {
    fn new(plot_rx: mpsc::UnboundedReceiver<plot::Msg>) -> Self {
        Self {
            pts: VecDeque::new(),
            rx: plot_rx,
        }
    }

    fn drain(mut self) -> VecDeque<(f64, f64)> {
        while let Ok(msg) = self.rx.try_recv() {
            if let plot::Msg::Push(p) = msg {
                self.pts.push_back(p);
                if MAX_PLOT_HISTORY < self.pts.len() {
                    self.pts.pop_front();
                }
            }
        }
        self.pts
    }
}
