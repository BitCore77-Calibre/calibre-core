// Hand-derived weights for pallet-stake. Regenerate via frame-benchmarking
// once the pallet API is frozen. See pallets/qutxo/src/weights.rs for the
// same convention.

use frame_support::weights::{constants::RocksDbWeight, Weight};

pub trait WeightInfo {
    fn bond() -> Weight;
    fn unbond() -> Weight;
    fn join_candidates() -> Weight;
    fn leave_candidates() -> Weight;
}

impl WeightInfo for () {
    // Placeholder: one read + one write, plus a nominal UTXO consumption cost.
    fn bond() -> Weight {
        Weight::from_parts(100_000_000, 100_000)
    }
    fn unbond() -> Weight {
        Weight::from_parts(50_000_000, 50_000)
    }

    fn join_candidates() -> Weight {
        Weight::from_parts(15_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(1))
    }

    fn leave_candidates() -> Weight {
        Weight::from_parts(10_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
}

pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn bond() -> Weight {
        Weight::from_parts(100_000_000, 100_000)
    }
    fn unbond() -> Weight {
        Weight::from_parts(50_000_000, 50_000)
    }

    fn join_candidates() -> Weight {
        Weight::from_parts(15_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(1))
    }

    fn leave_candidates() -> Weight {
        Weight::from_parts(10_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
}
