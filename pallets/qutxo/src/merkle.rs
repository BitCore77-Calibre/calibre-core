
use sp_core::H256;
use sp_core::hashing::blake2_256;
use sp_std::prelude::*;

const LEAF_PREFIX: u8 = 0x00;
const INNER_PREFIX: u8 = 0x01;

pub fn leaf_hash(utxo_id: &H256, value_hash: &H256) -> H256 {
    let mut buf = [0u8; 65];
    buf[0] = LEAF_PREFIX;
    buf[1..33].copy_from_slice(utxo_id.as_bytes());
    buf[33..65].copy_from_slice(value_hash.as_bytes());
    H256::from(blake2_256(&buf))
}

pub fn inner_hash(left: &H256, right: &H256) -> H256 {
    let mut buf = [0u8; 65];
    buf[0] = INNER_PREFIX;
    buf[1..33].copy_from_slice(left.as_bytes());
    buf[33..65].copy_from_slice(right.as_bytes());
    H256::from(blake2_256(&buf))
}

pub fn utxo_set_root<I>(leaves: I) -> H256
where I: IntoIterator<Item = (H256, H256)> {
    let mut items: Vec<(H256, H256)> = leaves.into_iter().collect();
    items.sort_by(|a, b| a.0.cmp(&b.0));
    if items.is_empty() { return H256::zero(); }
    let mut level: Vec<H256> =
        items.iter().map(|(id, v)| leaf_hash(id, v)).collect();
    level.resize(level.len().next_power_of_two(), H256::zero());
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
    fn h(b: u8) -> H256 { H256::repeat_byte(b) }

    #[test]
    fn empty_set_is_zero() {
        assert_eq!(utxo_set_root(Vec::<(H256, H256)>::new()), H256::zero());
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
