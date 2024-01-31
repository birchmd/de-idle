use std::cmp;

use self::{
    buildings::{Bank, BankMarker, Factory, FactoryMarker, Furnace, FurnaceMarker},
    resources::{Energy, Gold, Resource, ResourceDiff, Wood},
    workers::{
        Lumberjack, LumberjackMarker, Miner, MinerMarker, Monster, MonsterMarker, Recruiter,
        RecruiterMarker,
    },
};

pub mod buildings;
pub mod resources;
pub mod workers;

#[cfg(test)]
mod tests;

const WOOD_PER_GOLD: u128 = 25;
const MINER_COST: u128 = 5;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct GameState {
    // Basic resources
    wood: Wood,
    gold: Gold,
    energy: Energy,
    // Buildings
    banks: Bank,
    furnaces: Furnace,
    factories: Factory,
    // Workers
    miners: Miner,
    lumberjacks: Lumberjack,
    recruiters: Recruiter,
    monsters: Monster,
}

impl GameState {
    /// Manually chop down a tree to get wood.
    pub fn chop(&mut self) {
        self.wood.increment();
    }

    // Sell some wood at the marketplace for gold.
    pub fn sell_wood(&mut self) {
        if self.wood.raw_amount() >= WOOD_PER_GOLD {
            self.wood.remove(Wood::new(WOOD_PER_GOLD));
            self.gold.increment();
        }
    }

    pub fn hire_miner(&mut self) {
        if self.gold.raw_amount() >= MINER_COST {
            self.gold.remove(Gold::new(MINER_COST));
            self.miners.increment();
        }
    }

    pub fn hire_lumberjack(&mut self) {
        self.lumberjacks.increment();
    }

    /// Assume one update per 10ms (100 updates per second).
    pub fn update(&mut self) {
        let produced_gold = MinerMarker::produce(&self.miners)
            .saturating_add(BankMarker::produce(&self.gold, &self.banks));
        let produced_wood =
            LumberjackMarker::produce(&self.gold, &produced_gold, &self.lumberjacks);
        let produced_energy = FurnaceMarker::produce(&self.wood, &produced_wood, &self.furnaces);
        let produced_miners =
            FactoryMarker::produce(&self.energy, &produced_energy, &self.factories);
        let produced_lumberjacks = RecruiterMarker::produce(&self.gold, &self.recruiters);

        let added_wood = diff(produced_wood, FurnaceMarker::consume(&self.furnaces));
        let added_gold = diff(produced_gold, LumberjackMarker::consume(&self.lumberjacks));
        let added_energy = diff(produced_energy, FactoryMarker::consume(&self.factories));
        let added_lumberjacks = diff(produced_lumberjacks, MonsterMarker::consume(&self.monsters));

        self.wood.update(added_wood);
        self.gold.update(added_gold);
        self.energy.update(added_energy);
        self.lumberjacks.update(added_lumberjacks);
        self.miners
            .update(ResourceDiff::new(produced_miners.raw_milli_amount() as i128));
    }
}

trait Producer<R>: Sized {
    const MILLI_PRODUCE_PER_UPDATE: u128;

    fn produce(units: &Resource<Self>) -> Resource<R> {
        Resource::new_milli(
            units
                .raw_amount()
                .saturating_mul(Self::MILLI_PRODUCE_PER_UPDATE),
        )
    }
}

trait Consumer<R>: Sized {
    const MILLI_COST_PER_UPDATE: u128;

    fn consume(units: &Resource<Self>) -> Resource<R> {
        let amount = units
            .raw_amount()
            .saturating_mul(Self::MILLI_COST_PER_UPDATE);
        Resource::new_milli(amount)
    }
}

/// Convert resource R1 into resource R2
trait Converter<R1, R2>: Sized {
    const MILLI_COST_PER_UPDATE: u128;
    const MILLI_PRODUCE_PER_UPDATE: u128;

    fn consume(units: &Resource<Self>) -> Resource<R1> {
        let amount = units
            .raw_amount()
            .saturating_mul(Self::MILLI_COST_PER_UPDATE);
        Resource::new_milli(amount)
    }

    fn produce(
        base: &Resource<R1>,
        production: &Resource<R1>,
        units: &Resource<Self>,
    ) -> Resource<R2> {
        let active_units = cmp::min(
            units.raw_amount(),
            base.raw_milli_amount()
                .saturating_add(production.raw_milli_amount())
                .saturating_div(Self::MILLI_COST_PER_UPDATE),
        );
        let amount = active_units.saturating_mul(Self::MILLI_PRODUCE_PER_UPDATE);
        Resource::new_milli(amount)
    }
}

fn diff<R>(produced: Resource<R>, consumed: Resource<R>) -> ResourceDiff<R> {
    let produced = produced.raw_milli_amount();
    let consumed = consumed.raw_milli_amount();
    let diff = if produced > consumed {
        (produced - consumed) as i128
    } else {
        -((consumed - produced) as i128)
    };
    ResourceDiff::new(diff)
}
