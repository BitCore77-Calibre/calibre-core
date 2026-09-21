
use calibre_merkle::Hash;
use methods::{CALIBRE_ZK_GUEST_ELF, CALIBRE_ZK_GUEST_ID};
use risc0_zkvm::{default_prover, ExecutorEnv};

fn hex_to_hash(s: &str) -> Hash {
    let s = s.strip_prefix("0x").unwrap_or(s);
    assert_eq!(s.len(), 64, "hash must be 32 bytes");
    let mut out = [0u8; 32];
    for i in 0..32 {
        out[i] = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).expect("hex decode");
    }
    out
}

fn hex32(x: &Hash) -> String {
    x.iter().map(|b| format!("{:02x}", b)).collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: host <rpc_url> <utxo_id_hex>");
        std::process::exit(1);
    }
    let rpc_url = &args[1];
    let utxo_id_arg = &args[2];

    let body = serde_json::json!({
        "jsonrpc": "2.0", "id": 1,
        "method": "qutxo_getInclusionProof",
        "params": [utxo_id_arg, null]
    });
    let client = reqwest::blocking::Client::new();
    let resp: serde_json::Value = client
        .post(rpc_url)
        .json(&body)
        .send()
        .expect("rpc request failed")
        .json()
        .expect("invalid json");

    let result = resp.get("result").expect("no result field");
    if result.is_null() {
        eprintln!("RPC returned null — UTXO not found");
        std::process::exit(2);
    }
    let root = hex_to_hash(result["root"].as_str().unwrap());
    let utxo_id = hex_to_hash(result["utxo_id"].as_str().unwrap());
    let value_hash = hex_to_hash(result["value_hash"].as_str().unwrap());
    let mut path: Vec<(Hash, bool)> = Vec::new();
    for step in result["path"].as_array().unwrap() {
        let sib = hex_to_hash(step["sibling"].as_str().unwrap());
        let is_left = step["current_is_left"].as_bool().unwrap();
        path.push((sib, is_left));
    }
    println!("root:       0x{}", hex32(&root));
    println!("utxo_id:    0x{}", hex32(&utxo_id));
    println!("value_hash: 0x{}", hex32(&value_hash));
    println!("path len:   {}", path.len());

    let env = ExecutorEnv::builder()
        .write(&root).unwrap()
        .write(&utxo_id).unwrap()
        .write(&value_hash).unwrap()
        .write(&path).unwrap()
        .build().unwrap();

    println!("\nProving...");
    let receipt = default_prover()
        .prove(env, CALIBRE_ZK_GUEST_ELF)
        .unwrap()
        .receipt;
    receipt.verify(CALIBRE_ZK_GUEST_ID).unwrap();

    let (j_root, j_id, j_val): (Hash, Hash, Hash) = receipt.journal.decode().unwrap();
    assert_eq!(j_root, root);
    assert_eq!(j_id, utxo_id);
    assert_eq!(j_val, value_hash);
    println!("\n✅ INCLUSION VERIFIED against on-chain root");
}
