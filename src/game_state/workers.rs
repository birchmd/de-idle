use super::{
    Consumer, Converter, Producer,
    resources::{Gold, GoldMarker, Resource, WoodMarker},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct MinerMarker;
/// Worker.
/// Net produces 1 gold per second
/// (imagine paying miners, but they still mine more than you pay them).
pub type Miner = Resource<MinerMarker>;
impl Producer<GoldMarker> for MinerMarker {
    const MILLI_PRODUCE_PER_UPDATE: u128 = 10;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct LumberjackMarker;
/// Worker.
/// Consumes 1 gold per second.
/// Produces 1 wood per second.
pub type Lumberjack = Resource<LumberjackMarker>;
impl Converter<GoldMarker, WoodMarker> for LumberjackMarker {
    const MILLI_COST_PER_UPDATE: u128 = 10;
    const MILLI_PRODUCE_PER_UPDATE: u128 = 10;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RecruiterMarker;
/// Worker.
/// Produces (auto-hires) 1 Lumberjack per second per unit of gold in our coffers.
pub type Recruiter = Resource<RecruiterMarker>;
impl RecruiterMarker {
    pub const fn produce(base: &Gold, units: &Resource<Self>) -> Lumberjack {
        let amount_per_recruiter = base.raw_milli_amount().saturating_div(100);
        let total_amount = units.raw_amount().saturating_mul(amount_per_recruiter);
        Resource::new_milli(total_amount)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct MonsterMarker;
/// Worker.
/// Consumes (eats) 1 Lumberjack per second.
pub type Monster = Resource<MonsterMarker>;
impl Consumer<LumberjackMarker> for MonsterMarker {
    const MILLI_COST_PER_UPDATE: u128 = 10;
}
