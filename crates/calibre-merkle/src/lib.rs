//! Calibre Merkle tree — blake2b-256, domain-separated.
//! Standalone `no_std` crate, usable by both the pallet and the
//! RISC Zero state-proof guest without modification.

#![cfg_attr(not(feature = "std"), no_std)]

use blake2::digest::{Update, VariableOutput};
use blake2::Blake2bVar;
use sp_std::prelude::*;

pub type Hash = [u8; 32];

const LEAF_PREFIX: u8 = 0x00;
const INNER_PREFIX: u8 = 0x01;

/// Blake2b with 32-byte output. Byte-identical to
/// `sp_core::hashing::blake2_256`, but no `sp_core` dependency.
pub fn blake2_256(data: &[u8]) -> Hash {
    let mut h = Blake2bVar::new(32).expect("32 <= 64");
    h.update(data);
    let mut out = [0u8; 32];
    h.finalize_variable(&mut out).expect("size matches");
    out
}

pub fn leaf_hash(utxo_id: &Hash, value_hash: &Hash) -> Hash {
    let mut buf = [0u8; 65];
    buf[0] = LEAF_PREFIX;
    buf[1..33].copy_from_slice(utxo_id);
    buf[33..65].copy_from_slice(value_hash);
    blake2_256(&buf)
}

pub fn inner_hash(left: &Hash, right: &Hash) -> Hash {
    let mut buf = [0u8; 65];
    buf[0] = INNER_PREFIX;
    buf[1..33].copy_from_slice(left);
    buf[33..65].copy_from_slice(right);
    blake2_256(&buf)
}

/// Merkle root over a set of (utxo_id, value_hash) pairs.
/// Order-independent. Empty set -> all-zero root.
pub fn utxo_set_root<I>(leaves: I) -> Hash
where
    I: IntoIterator<Item = (Hash, Hash)>,
{
    let mut items: Vec<(Hash, Hash)> = leaves.into_iter().collect();
    items.sort_by(|a, b| a.0.cmp(&b.0));
    if items.is_empty() {
        return [0u8; 32];
    }
    let mut level: Vec<Hash> =
        items.iter().map(|(id, v)| leaf_hash(id, v)).collect();
    level.resize(level.len().next_power_of_two(), [0u8; 32]);
    while level.len() > 1 {
        let mut next = Vec::with_capacity(level.len() / 2);
        for pair in level.chunks_exact(2) {
            next.push(inner_hash(&pair[0], &pair[1]));
        }
        level = next;
    }
    level[0]
}

#[cfg(test)]
mod tests {
    use super::*;
    fn h(b: u8) -> Hash { [b; 32] }

    #[test]
    fn empty_is_zero() {
        assert_eq!(utxo_set_root(Vec::<(Hash, Hash)>::new()), [0u8; 32]);
    }

    #[test]
    fn order_independent() {
        let a = (h(1), h(0xaa));
        let b = (h(2), h(0xbb));
        assert_eq!(utxo_set_root(vec![a, b]), utxo_set_root(vec![b, a]));
    }

    #[test]
    fn membership_affects_root() {
        let a = (h(1), h(0xaa));
        let b = (h(2), h(0xbb));
        assert_ne!(utxo_set_root(vec![a]), utxo_set_root(vec![a, b]));
    }

    #[test]
    fn value_change_affects_root() {
        let a = (h(1), h(0xaa));
        let b = (h(1), h(0xbb));
        assert_ne!(utxo_set_root(vec![a]), utxo_set_root(vec![b]));
    }
}


#[cfg(test)]
mod equivalence {
    use super::*;
    #[test]
    fn matches_sp_core() {
        for input in [
            b"".as_slice(),
            b"hello".as_slice(),
            &[0u8; 65][..],
            &[0xffu8; 200][..],
        ] {
            assert_eq!(blake2_256(input), sp_core::hashing::blake2_256(input));
        }
    }
}

#[cfg(test)]
mod equivalence {
    use super::*;
    #[test]
    fn matches_sp_core() {
        for input in [
            b"".as_slice(),
            b"hello".as_slice(),
            &[0u8; 65][..],
            &[0xffu8; 200][..],
        ] {
            assert_eq!(blake2_256(input), sp_core::hashing::blake2_256(input));
        }
    }
}
