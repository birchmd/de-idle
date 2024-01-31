use super::{
    resources::{EnergyMarker, Gold, Resource, WoodMarker},
    workers::MinerMarker,
    Converter,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FurnaceMarker;
/// Building.
/// Consumes 1 Wood per second.
/// Produces 1 Energy per second.
pub type Furnace = Resource<FurnaceMarker>;
impl Converter<WoodMarker, EnergyMarker> for FurnaceMarker {
    const MILLI_COST_PER_UPDATE: u128 = 10;
    const MILLI_PRODUCE_PER_UPDATE: u128 = 10;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FactoryMarker;
/// Building.
/// Consumes 1 Energy per second.
/// Produces 1 (robotic) Miner worker per second.
pub type Factory = Resource<FactoryMarker>;
impl Converter<EnergyMarker, MinerMarker> for FactoryMarker {
    const MILLI_COST_PER_UPDATE: u128 = 10;
    const MILLI_PRODUCE_PER_UPDATE: u128 = 10;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct BankMarker;
/// Building.
/// Increases gold supply by 1% per second per bank
pub type Bank = Resource<BankMarker>;
impl BankMarker {
    pub const fn produce(base: &Gold, units: &Resource<Self>) -> Gold {
        let amount_per_bank = base.raw_milli_amount().saturating_div(100);
        let total_amount = units.raw_amount().saturating_mul(amount_per_bank);
        Resource::new_milli(total_amount)
    }
}
