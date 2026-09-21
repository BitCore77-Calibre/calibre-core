use calibre_merkle::{inner_hash, leaf_hash, Hash};
use risc0_zkvm::guest::env;

fn main() {
    // ── Public inputs (via env::read) ──
    let root: Hash = env::read();
    let utxo_id: Hash = env::read();
    let value_hash: Hash = env::read();

    // ── Private witness ──
    // Each step: (sibling_hash, current_node_is_left_child)
    let path: Vec<(Hash, bool)> = env::read();

    // Recompute leaf and fold up the path.
    let mut current = leaf_hash(&utxo_id, &value_hash);
    for (sibling, current_is_left) in path.iter() {
        current = if *current_is_left {
            inner_hash(&current, sibling)
        } else {
            inner_hash(sibling, &current)
        };
    }

    assert_eq!(current, root, "merkle path does not fold to root");

    // ── Public outputs (journal, single tuple) ──
    env::commit(&(root, utxo_id, value_hash));
}
