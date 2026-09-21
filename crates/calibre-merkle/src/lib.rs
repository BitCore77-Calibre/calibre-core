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

/// Merkle inclusion path for `target_id` against the same tree
/// construction `utxo_set_root` uses.
///
/// Each element is `(sibling_hash, current_is_left)`:
///   * `current_is_left == true`  -> current is `inner_hash(current, sibling)`
///   * `current_is_left == false` -> current is `inner_hash(sibling, current)`
///
/// Returns `None` if `target_id` is not in `leaves`.
pub fn merkle_path(leaves: &[(Hash, Hash)], target_id: &Hash) -> Option<Vec<(Hash, bool)>> {
    let mut items: Vec<(Hash, Hash)> = leaves.to_vec();
    items.sort_by(|a, b| a.0.cmp(&b.0));

    let mut idx = items.iter().position(|(id, _)| id == target_id)?;

    let mut level: Vec<Hash> =
        items.iter().map(|(id, v)| leaf_hash(id, v)).collect();
    level.resize(level.len().next_power_of_two(), [0u8; 32]);

    let mut path: Vec<(Hash, bool)> = Vec::new();
    while level.len() > 1 {
        path.push((level[idx ^ 1], idx % 2 == 0));
        let mut next = Vec::with_capacity(level.len() / 2);
        for pair in level.chunks_exact(2) {
            next.push(inner_hash(&pair[0], &pair[1]));
        }
        level = next;
        idx /= 2;
    }
    Some(path)
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

    #[test]
    fn path_reconstructs_root() {
        let leaves = vec![(h(3), h(0xcc)), (h(1), h(0xaa)), (h(2), h(0xbb))];
        let root = utxo_set_root(leaves.clone());
        for (tid, tv) in leaves.iter() {
            let path = merkle_path(&leaves, tid).expect("leaf present");
            let mut cur = leaf_hash(tid, tv);
            for (sib, is_left) in path.iter() {
                cur = if *is_left { inner_hash(&cur, sib) }
                      else       { inner_hash(sib, &cur) };
            }
            assert_eq!(cur, root, "path for {:?} did not fold to root", tid);
        }
    }

    #[test]
    fn path_missing_leaf_is_none() {
        let leaves = vec![(h(1), h(0xaa))];
        assert!(merkle_path(&leaves, &h(99)).is_none());
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

