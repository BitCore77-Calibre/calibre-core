use calibre_merkle::{merkle_path, utxo_set_root, Hash};
use methods::{CALIBRE_ZK_GUEST_ELF, CALIBRE_ZK_GUEST_ID};
use risc0_zkvm::{default_prover, ExecutorEnv};

fn h(b: u8) -> Hash { [b; 32] }

fn hex32(x: &Hash) -> String {
    x.iter().map(|b| format!("{:02x}", b)).collect()
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    // Deterministic test UTXO set: 4 leaves, arbitrary value hashes.
    // The circuit only cares about structure, not the actual UTXO contents.
    let leaves: Vec<(Hash, Hash)> = vec![
        (h(1), h(0xa1)),
        (h(2), h(0xa2)),
        (h(3), h(0xa3)),
        (h(4), h(0xa4)),
    ];
    let root = utxo_set_root(leaves.clone());

    // Prove inclusion of the third UTXO.
    let (utxo_id, value_hash) = leaves[2];
    let path = merkle_path(&leaves, &utxo_id).expect("target in set");

    println!("UTXO set size: {}", leaves.len());
    println!("root:          0x{}", hex32(&root));
    println!("target utxo:   0x{}", hex32(&utxo_id));
    println!("value hash:    0x{}", hex32(&value_hash));
    println!("path length:   {}", path.len());
    println!();

    // Feed the guest in the exact order it reads.
    let env = ExecutorEnv::builder()
        .write(&root).unwrap()
        .write(&utxo_id).unwrap()
        .write(&value_hash).unwrap()
        .write(&path).unwrap()
        .build().unwrap();

    println!("Proving (this takes ~10-60s on first run)...");
    let prove_info = default_prover()
        .prove(env, CALIBRE_ZK_GUEST_ELF)
        .unwrap();
    let receipt = prove_info.receipt;

    println!("Verifying receipt...");
    receipt.verify(CALIBRE_ZK_GUEST_ID).unwrap();

    let (j_root, j_utxo_id, j_value_hash): (Hash, Hash, Hash) =
        receipt.journal.decode().unwrap();

    assert_eq!(j_root, root, "journal root mismatch");
    assert_eq!(j_utxo_id, utxo_id, "journal utxo_id mismatch");
    assert_eq!(j_value_hash, value_hash, "journal value_hash mismatch");

    println!();
    println!("Journal root:       0x{}", hex32(&j_root));
    println!("Journal utxo_id:    0x{}", hex32(&j_utxo_id));
    println!("Journal value_hash: 0x{}", hex32(&j_value_hash));
    println!();
    println!("✅ INCLUSION VERIFIED");
}
