//! Tier-2 prototype v2: verify Qutxo.UtxoSetRoot via a state-trie proof.
//!
//! Flow:
//!   chain_getFinalizedHead                     -> block hash
//!   chain_getHeader                            -> state_root (trusted)
//!   state_getStorage(KEY, finalized)           -> claimed value (unverified)
//!   state_getReadProof([KEY], finalized)       -> proof nodes
//!   decode_and_verify_proof(scaled_proof)      -> decode + check internal consistency
//!   decoded.proof_entry(&state_root, key)      -> verifies proof contains this root,
//!                                                 returns the entry for our key
//!   extract unhashed_storage_value             -> UtxoSetRoot
//!
//! If proof_entry returns Some and the value matches state_getStorage,
//! the claimed value is cryptographically tied to the finalized header.

use parity_scale_codec::Encode;
use smoldot::trie::{
    bytes_to_nibbles,
    proof_decode::{decode_and_verify_proof, Config},
};

const UTXO_SET_ROOT_KEY: &str =
    "0x8feb94cbd57b65eb1436ba6db973e04df4b8d3c19af7d55511d730e48dc7dc87";

fn decode_hex(s: &str) -> Vec<u8> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    hex::decode(s).expect("hex decode")
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let rpc_url = args.get(1).cloned()
        .unwrap_or_else(|| "http://127.0.0.1:9944".to_string());

    let http = reqwest::Client::new();

    // --- 1. Finalized head ---
    let r: serde_json::Value = http.post(&rpc_url)
        .json(&serde_json::json!({"jsonrpc":"2.0","id":1,"method":"chain_getFinalizedHead","params":[]}))
        .send().await.unwrap().json().await.unwrap();
    let block_hash = r["result"].as_str().expect("no finalized head");
    println!("finalized head: {}", block_hash);

    // --- 2. Header -> state_root ---
    let r: serde_json::Value = http.post(&rpc_url)
        .json(&serde_json::json!({"jsonrpc":"2.0","id":1,"method":"chain_getHeader","params":[block_hash]}))
        .send().await.unwrap().json().await.unwrap();
    let state_root_hex = r["result"]["stateRoot"].as_str().expect("no stateRoot");
    let state_root_vec = decode_hex(state_root_hex);
    let mut state_root = [0u8; 32];
    state_root.copy_from_slice(&state_root_vec);
    println!("state_root:     {}", state_root_hex);

    // --- 3. Claimed value (unverified) ---
    let r: serde_json::Value = http.post(&rpc_url)
        .json(&serde_json::json!({"jsonrpc":"2.0","id":1,"method":"state_getStorage","params":[UTXO_SET_ROOT_KEY, block_hash]}))
        .send().await.unwrap().json().await.unwrap();
    let claimed_hex = r["result"].as_str().expect("no storage value");
    let claimed = decode_hex(claimed_hex);
    println!("claimed value:  0x{}", hex::encode(&claimed));

    // --- 4. Read proof ---
    let r: serde_json::Value = http.post(&rpc_url)
        .json(&serde_json::json!({"jsonrpc":"2.0","id":1,"method":"state_getReadProof","params":[[UTXO_SET_ROOT_KEY], block_hash]}))
        .send().await.unwrap().json().await.unwrap();
    let proof_arr = r["result"]["proof"].as_array().expect("no proof array");
    let proof_nodes: Vec<Vec<u8>> = proof_arr.iter()
        .map(|v| decode_hex(v.as_str().unwrap()))
        .collect();
    println!("proof nodes:    {}", proof_nodes.len());

    // --- 5. SCALE-encode Vec<Vec<u8>> ---
    let proof_scaled = proof_nodes.encode();
    println!("encoded proof:  {} bytes", proof_scaled.len());

    // --- 6. Decode + internal consistency check ---
    let decoded = decode_and_verify_proof(Config { proof: proof_scaled })
        .expect("proof decode failed");
    println!("proof decoded:  OK");

    // --- 7. Look up our key under the expected state_root ---
    let key_bytes = decode_hex(UTXO_SET_ROOT_KEY);
    let key_nibbles: Vec<_> = bytes_to_nibbles(key_bytes.iter().copied()).collect();

    let entry = decoded.proof_entry(&state_root, key_nibbles.into_iter())
        .expect("key not found under expected state_root");

    // --- 8. Extract the value ---
    let extracted = entry.unhashed_storage_value
        .expect("no unhashed storage value in proof entry");
    println!("extracted:      0x{}", hex::encode(extracted));

    // --- 9. Compare ---
    if extracted == claimed.as_slice() {
        println!("\n✅ TIER 2 PROOF VERIFIED");
        println!("   The claimed UtxoSetRoot is cryptographically tied");
        println!("   to the finalized block's state_root.");
    } else {
        println!("\n❌ MISMATCH — proof value differs from storage query");
        std::process::exit(1);
    }
}
