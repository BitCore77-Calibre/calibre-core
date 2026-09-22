//! Weight definitions for pallet-qutxo.
//!
//! These are hand-derived constants, not benchmark output yet. Basis
//! (see docs/BENCHMARK.md):
//!   - ML-DSA-44 verify:        208 us = 208_000_000 ps  (per tx, first input only)
//!   - SCALE decode 4 KB:       ~15 us
//!   - Blake2_256 over payload: ~5 us
//!   - RocksDbWeight:           read ~25 us, write ~100 us
//!
//! Replace with benchmark-derived values once the pallet API is frozen:
//!   cargo run --release --features runtime-benchmarks -- \
//!     benchmark pallet --pallet pallet_qutxo --extrinsic '*' \
//!     --steps 50 --repeat 20

use frame_support::traits::Get;
use frame_support::weights::{constants::RocksDbWeight, Weight};

pub trait WeightInfo {
    fn execute_utxo_tx(inputs: u32, outputs: u32) -> Weight;
    fn sudo_mint() -> Weight;
}

pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    /// execute_utxo_tx cost components:
    ///   base       : one ML-DSA-44 verify + SCALE decode + payload hash
    ///   per-input  : one storage read + one storage remove
    ///   per-output : one storage insert
    fn execute_utxo_tx(inputs: u32, outputs: u32) -> Weight {
        Weight::from_parts(300_000_000, 0)
            .saturating_add(Weight::from_parts(50_000_000, 0).saturating_mul(inputs as u64))
            .saturating_add(Weight::from_parts(100_000_000, 0).saturating_mul(outputs as u64))
            .saturating_add(T::DbWeight::get().reads(2 + inputs as u64))
            .saturating_add(T::DbWeight::get().writes(1 + inputs as u64 + outputs as u64))
    }

    /// sudo_mint is root-only, so no signature verification.
    fn sudo_mint() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
}

/// Default impl for tests and benchmarks (no `T: Config` in scope).
impl WeightInfo for () {
    fn execute_utxo_tx(inputs: u32, outputs: u32) -> Weight {
        Weight::from_parts(300_000_000, 0)
            .saturating_add(Weight::from_parts(50_000_000, 0).saturating_mul(inputs as u64))
            .saturating_add(Weight::from_parts(100_000_000, 0).saturating_mul(outputs as u64))
            .saturating_add(RocksDbWeight::get().reads(2 + inputs as u64))
            .saturating_add(RocksDbWeight::get().writes(1 + inputs as u64 + outputs as u64))
    }
    fn sudo_mint() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(2))
    }
}
