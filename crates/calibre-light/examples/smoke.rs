use calibre_light::{hash_hex, LightClient, LightError};

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let rpc = args.get(1).map(|s| s.as_str()).unwrap_or("http://127.0.0.1:9944");
    let utxo_hex = args.get(2).map(|s| s.as_str());

    let mut client = LightClient::connect(rpc);

    println!("[1] sync() — fetching current UTXO root...");
    match client.sync().await {
        Ok(root) => println!("    root:        {}", hash_hex(&root)),
        Err(e) => { eprintln!("    sync failed: {}", e); std::process::exit(1); }
    }
    println!("    known_roots: {}", client.known_roots());

    let Some(hex_id) = utxo_hex else {
        println!("\n[2] no utxo_id given — pass one as 2nd arg to test inclusion");
        return;
    };

    let s = hex_id.strip_prefix("0x").unwrap_or(hex_id);
    let mut arr = [0u8; 32];
    for i in 0..32 {
        arr[i] = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).unwrap();
    }

    println!("\n[2] utxo() — fetching and verifying inclusion...");
    match client.utxo(arr).await {
        Ok(Some(info)) => {
            println!("    utxo_id:    {}", hex_id);
            println!("    value_hash: {}", hash_hex(&info.value_hash));
            println!("    root:       {}", hash_hex(&info.root));
            println!("    path_len:   {}", info.path_len);
            println!("\n    ✅ VERIFIED (locally folded to known root)");
        }
        Ok(None) => println!("    UTXO not found (null)"),
        Err(LightError::UnknownRoot) =>
            println!("    ❌ proof root not in known_roots — stale or forked node"),
        Err(LightError::BadPath) =>
            println!("    ❌ path did not fold to claimed root — MALICIOUS NODE"),
        Err(e) => println!("    error: {}", e),
    }
}
