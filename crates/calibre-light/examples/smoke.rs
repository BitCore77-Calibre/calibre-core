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

    println!("[1] sync() — fetching current UTXO root...");
    match client.sync().await {
        Ok(root) => println!("    root:        {}", hex(&root)),
        Err(e) => { eprintln!("    sync failed: {}", e); std::process::exit(1); }
    }
    println!("    known_roots: {}", client.known_roots());

    let Some(hex_id) = utxo_hex else {
        println!("\n[2] no utxo_id given — pass one as 2nd arg to test inclusion");
        return;
    };

    let s = hex_id.strip_prefix("0x").unwrap_or(&hex_id);
    let mut arr = vec![0u8; 32];
    for i in 0..32 {
        arr[i] = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).unwrap();
    }

    println!("\n[2] utxo() — fetching and verifying inclusion...");
    match client.utxo(arr).await {
        Ok(Some(info)) => {
            println!("    utxo_id:    {}", hex(&info.utxo_id));
            println!("    value_hash: {}", hex(&info.value_hash));
            println!("    root:       {}", hex(&info.root));
            println!("    path_len:   {}", info.path_len);
            println!("\n    ✅ VERIFIED (locally folded to known root)");
        }
        Ok(None) => println!("    UTXO not found (null)"),
        Err(e) => println!("    error: {}", e),
    }
}
