use std::marker::PhantomData;

/// 1000 milli units per unit
const MILLI_MULTIPLIER: u128 = 1000;

/// Generic basic resource.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Resource<Marker> {
    milli_amount: u128,
    pd: PhantomData<Marker>,
}

impl<Marker> Resource<Marker> {
    pub const fn new(amount: u128) -> Self {
        Self {
            milli_amount: amount.saturating_mul(MILLI_MULTIPLIER),
            pd: PhantomData,
        }
    }

    pub const fn new_milli(milli_amount: u128) -> Self {
        Self {
            milli_amount,
            pd: PhantomData,
        }
    }

    pub fn increment(&mut self) {
        self.milli_amount = self.milli_amount.saturating_add(MILLI_MULTIPLIER);
    }

    pub fn decrement(&mut self) {
        self.milli_amount = self.milli_amount.saturating_sub(MILLI_MULTIPLIER);
    }

    pub const fn saturating_add(self, amount: Self) -> Self {
        Self {
            milli_amount: self.milli_amount.saturating_add(amount.milli_amount),
            pd: PhantomData,
        }
    }

    pub fn remove(&mut self, amount: Self) {
        self.milli_amount = self.milli_amount.saturating_sub(amount.milli_amount);
    }

    pub const fn raw_amount(&self) -> u128 {
        self.milli_amount.saturating_div(MILLI_MULTIPLIER)
    }

    pub const fn raw_milli_amount(&self) -> u128 {
        self.milli_amount
    }

    pub const fn to_f64(&self) -> f64 {
        (self.milli_amount as f64) / 1000.0
    }

    pub fn update(&mut self, diff: ResourceDiff<Marker>) {
        self.milli_amount = if diff.milli_amount > 0 {
            self.milli_amount.saturating_add(diff.milli_amount as u128)
        } else {
            self.milli_amount
                .saturating_sub(diff.milli_amount.unsigned_abs())
        };
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ResourceDiff<Marker> {
    milli_amount: i128,
    pd: PhantomData<Marker>,
}

impl<Marker> ResourceDiff<Marker> {
    pub const fn new(milli_amount: i128) -> Self {
        Self {
            milli_amount,
            pd: PhantomData,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct WoodMarker;
/// Basic resource.
/// Produced manually (chopping).
/// Produced by Lumberjack worker.
/// Consumed by Furnace building.
pub type Wood = Resource<WoodMarker>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct GoldMarker;
/// Basic resource.
/// Produced manually (selling wood).
/// Produced by Miner worker.
/// Consumed by Lumberjack worker.
pub type Gold = Resource<GoldMarker>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct EnergyMarker;
/// Basic resource.
/// Produced by Furnace.
pub type Energy = Resource<EnergyMarker>;
