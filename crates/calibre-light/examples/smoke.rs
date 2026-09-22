use calibre_light::LightClient;

fn hex(v: &[u8]) -> String {
    let mut s = String::with_capacity(2 + v.len() * 2);
    s.push_str("0x");
    for b in v { s.push_str(&format!("{:02x}", b)); }
    s
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let rpc = args.get(1).cloned().unwrap_or_else(|| "http://127.0.0.1:9944".to_string());
    let utxo_hex = args.get(2).cloned();

    let client = LightClient::connect(rpc);

    println!("=== Tier 1: sync() ===");
    match client.sync().await {
        Ok(root) => println!("    root: {}", hex(&root)),
        Err(e) => { eprintln!("    sync failed: {}", e); std::process::exit(1); }
    }

    println!("\n=== Tier 2: sync_verified() ===");
    match client.sync_verified().await {
        Ok(sr) => {
            println!("    root:         {}", hex(&sr.utxo_set_root));
            println!("    block_number: {}", sr.block_number);
            println!("    block_hash:   {}", hex(&sr.block_hash));
            println!("    verified:     {}", sr.verified);
        }
        Err(e) => { eprintln!("    sync_verified failed: {}", e); std::process::exit(1); }
    }

    println!("\n    known_roots:           {}", client.known_roots());
    println!("    last_verified_block:   {:?}", client.last_verified_block_number());
    println!("    max_proof_age (blocks): {}", client.max_proof_age());

    let Some(hex_id) = utxo_hex else {
        println!("\n[no utxo_id given]");
        return;
    };

    let s = hex_id.strip_prefix("0x").unwrap_or(&hex_id);
    let mut arr = vec![0u8; 32];
    for i in 0..32 {
        arr[i] = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).unwrap();
    }

    println!("\n=== Inclusion proof ===");
    match client.utxo(arr).await {
        Ok(Some(info)) => {
            println!("    value_hash:      {}", hex(&info.value_hash));
            println!("    root:            {}", hex(&info.root));
            println!("    path_len:        {}", info.path_len);
            println!("    proof_age_blocks: {:?}", info.proof_age_blocks);
            println!("\n    ✅ VERIFIED against tier-2 root");
        }
        Ok(None) => println!("    UTXO not found"),
        Err(e) => println!("    error: {}", e),
    }
}
