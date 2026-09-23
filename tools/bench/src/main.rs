//! calibre-bench — throughput benchmark for Calibre Q-UTXO chain.
//!
//! Modes:
//!   keygen <N> <out.json>
//!   burst  <keys.json> <mints.json> <rpc> <out.json>
//!   report <results.json>

use parity_scale_codec::Encode;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;
use tokio::sync::Semaphore;
use std::sync::Arc;

// --- wire types (mirror the pallet's SCALE encoding) ---------------------

#[derive(Encode, Clone)]
struct TxInput { tx_hash: [u8; 32], output_index: u32 }

#[derive(Encode, Clone)]
enum QuantumLock { SingleSig([u8; 32]), AegisThreshold([u8; 32]) }

#[derive(Encode, Clone)]
struct TxOutput { value: u128, lock: QuantumLock }

#[derive(Encode)]
struct DeviceAttestation {
    hardware_signature: Vec<u8>,
    app_binary_hash: [u8; 32],
    backend_challenge: [u8; 32],
}

#[derive(Encode)]
struct AegisWitness {
    pub_keys: Vec<Vec<u8>>,
    aggregated_signature: Vec<u8>,
    attestation: DeviceAttestation,
    time_drift: u64,
}

#[derive(Encode)]
struct Transaction {
    inputs: Vec<TxInput>,
    outputs: Vec<TxOutput>,
    pq_signature: Vec<u8>,
    witness: Vec<u8>,
}

// --- json shapes ---------------------------------------------------------

#[derive(Serialize, Deserialize, Clone)]
struct Key { sk_hex: String, pk_hex: String, lock_hex: String }

#[derive(Serialize, Deserialize, Clone)]
struct MintRecord {
    lock_hex: String,
    block_number: u32,
    value: u128,
    #[serde(default)]
    tx_hash_hex: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct ResultEntry {
    utxo_id_hex: String,
    submitted_at_ms: u128,
    included_at_ms: u128,
    latency_ms: u128,
    ok: bool,
    error: Option<String>,
}

// --- RPC -----------------------------------------------------------------

struct Rpc { url: String, http: reqwest::Client }

impl Rpc {
    fn new(url: impl Into<String>) -> Self {
        Self { url: url.into(), http: reqwest::Client::new() }
    }
    async fn call(&self, method: &str, params: serde_json::Value)
        -> Result<serde_json::Value, String>
    {
        let body = serde_json::json!({
            "jsonrpc": "2.0", "id": 1,
            "method": method, "params": params
        });
        let resp: serde_json::Value = self.http.post(&self.url)
            .json(&body).send().await.map_err(|e| e.to_string())?
            .json().await.map_err(|e| e.to_string())?;
        if let Some(err) = resp.get("error") {
            return Err(format!("{}", err));
        }
        Ok(resp.get("result").cloned().unwrap_or(serde_json::Value::Null))
    }
}

// --- helpers -------------------------------------------------------------


fn blake2_256(data: &[u8]) -> [u8; 32] {
    use blake2::digest::{Update, VariableOutput};
    use blake2::Blake2bVar;
    let mut h = Blake2bVar::new(32).expect("32 <= 64");
    h.update(data);
    let mut out = [0u8; 32];
    h.finalize_variable(&mut out).expect("size matches");
    out
}


/// SCALE compact encoding for a u32 length prefix.
fn compact_u32(n: u32) -> Vec<u8> {
    if n < 64 {
        vec![(n as u8) << 2]
    } else if n < 16384 {
        (((n as u16) << 2) | 0b01).to_le_bytes().to_vec()
    } else {
        ((n << 2) | 0b10).to_le_bytes().to_vec()
    }
}

fn now_ms() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
}

fn hex_to_32(s: &str) -> [u8; 32] {
    let s = s.trim_start_matches("0x");
    let mut out = [0u8; 32];
    for i in 0..32 {
        out[i] = u8::from_str_radix(&s[i*2..i*2+2], 16).unwrap();
    }
    out
}


// --- keygen --------------------------------------------------------------

async fn cmd_keygen(args: &[String]) {
    let n: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(10);
    let out = args.get(3).map(|s| s.as_str()).unwrap_or("/tmp/bench-keys.json");

    let mut keys = Vec::with_capacity(n);
    for i in 0..n {
        let kp = dilithium::MlDsaKeyPair::generate(dilithium::ML_DSA_44).expect("keygen");
        let pk = kp.public_key().to_vec();
        let sk = kp.to_bytes().to_vec();
        let lock = blake2_256(&pk);
        keys.push(Key {
            sk_hex: hex::encode(&sk),
            pk_hex: hex::encode(&pk),
            lock_hex: hex::encode(&lock),
        });
        if (i + 1) % 10 == 0 { eprintln!("keygen {}/{}", i + 1, n); }
    }
    std::fs::write(out, serde_json::to_string(&keys).unwrap()).unwrap();
    eprintln!("wrote {} keys to {}", n, out);
}

// --- burst ---------------------------------------------------------------

async fn cmd_burst(args: &[String]) {
    let keys_path = args.get(2).map(|s| s.as_str()).unwrap_or("/tmp/bench-keys.json");
    let mints_path = args.get(3).map(|s| s.as_str()).unwrap_or("/tmp/bench-mints.json");
    // Per-tx fee in CAL base units. Must be >= minimum_fee(inputs, outputs)
    // as computed by pallet-calibre-fees, or pool admission rejects the tx
    // with InvalidTransaction::Payment (Phase 8.3). Runtime minimum for a
    // 1-in/1-out tx is BaseTxFee + PerInOutFee * 2. Default 10_000 is
    // comfortably above that; override as the 5th positional arg.
    let fee_per_tx: u128 = args.get(4)
        .and_then(|s| s.parse().ok())
        .unwrap_or(10_000);
    eprintln!("burst fee per tx: {} base units", fee_per_tx);
    let rpc_url = args.get(4).map(|s| s.as_str()).unwrap_or("http://127.0.0.1:9944");
    let out = args.get(5).map(|s| s.as_str()).unwrap_or("/tmp/bench-results.json");

    let keys_txt = std::fs::read_to_string(keys_path)
        .unwrap_or_else(|e| { eprintln!("cannot read {}: {}", keys_path, e); std::process::exit(1); });
    let mints_txt = std::fs::read_to_string(mints_path)
        .unwrap_or_else(|e| { eprintln!("cannot read {}: {}", mints_path, e); std::process::exit(1); });
    let keys: Vec<Key> = serde_json::from_str(&keys_txt).expect("parse keys");
    let mints: Vec<MintRecord> = serde_json::from_str(&mints_txt).expect("parse mints");

    let key_by_lock: std::collections::HashMap<String, Key> =
        keys.into_iter().map(|k| (k.lock_hex.clone(), k)).collect();

    let rpc = std::sync::Arc::new(Rpc::new(rpc_url));

    eprintln!("preparing {} spends", mints.len());

    // Pre-build (sign) all extrinsics sequentially — ML-DSA-44 signing is
    // CPU-heavy, so this is done before the burst to isolate submission
    // latency from signing latency.
    let mut prepared: Vec<(String, String, String)> = Vec::new(); // (utxo_id_hex, extrinsic_hex, future_key)
    for (i, m) in mints.iter().enumerate() {
        let key = match key_by_lock.get(&m.lock_hex) {
            Some(k) => k.clone(),
            None => { eprintln!("skip: no key for lock {}", m.lock_hex); continue; }
        };
        let lock_inner = hex_to_32(&m.lock_hex);

        // Prefer tx_hash_hex from the mint event (authoritative).
        // Fall back to reconstruction only if absent.
        let genesis_hash: [u8; 32] = if let Some(ref th) = m.tx_hash_hex {
            hex_to_32(th)
        } else {
            let mut seed = Vec::new();
            seed.extend_from_slice(b"calibre-mint");
            seed.extend_from_slice(&m.block_number.to_le_bytes());
            seed.extend_from_slice(&lock_inner);
            blake2_256(&seed)
        };

        let mut uh = Vec::new();
        uh.extend_from_slice(&genesis_hash);
        uh.extend_from_slice(&0u32.to_le_bytes());
        let utxo_id = blake2_256(&uh);

        // Build tx. Leave `fee_per_tx` behind as the implicit fee so pool
        // admission (validate_unsigned) accepts the tx. Without this, output
        // equals input, fee == 0, and 8.3 rejects the tx.
        if m.value < fee_per_tx {
            eprintln!(
                "skip: UTXO {} value {} < fee {}",
                hex::encode(&lock_inner), m.value, fee_per_tx
            );
            continue;
        }
        let output_value = m.value.saturating_sub(fee_per_tx);
        let inputs = vec![TxInput { tx_hash: genesis_hash, output_index: 0 }];
        let outputs = vec![TxOutput {
            value: output_value,
            lock: QuantumLock::AegisThreshold(lock_inner),
        }];

        // Sign
        let sk_bytes = hex::decode(&key.sk_hex).unwrap();
        let kp = dilithium::MlDsaKeyPair::from_bytes(&sk_bytes).expect("decode sk");
        let msg = (inputs.clone(), outputs.clone()).encode();
        let sig = kp.sign(&msg, b"").expect("sign");
        let pk_bytes = hex::decode(&key.pk_hex).unwrap();

        let witness = AegisWitness {
            pub_keys: vec![pk_bytes],
            aggregated_signature: sig.as_bytes().to_vec(),
            attestation: DeviceAttestation {
                hardware_signature: Vec::new(),
                app_binary_hash: [0u8; 32],
                backend_challenge: [0u8; 32],
            },
            time_drift: 0,
        };

        // Capture sub-lengths BEFORE moving into Transaction
        let inputs_len  = inputs.encode().len();
        let outputs_len = outputs.encode().len();
        let witness_enc = witness.encode();
        let witness_len = witness_enc.len();
        let pqsig_len   = Vec::<u8>::new().encode().len();

        let tx = Transaction {
            inputs,
            outputs,
            pq_signature: Vec::new(),
            witness: witness_enc,
        };

        // Wire-format extrinsic:
        //   [compact_len(version + call)] [version=0x04] [pallet=0x08] [call=0x00] [SCALE(tx)]
        let tx_encoded = tx.encode();
        let inner_len = (3 + tx_encoded.len()) as u32; // 1 (version) + 2 (pallet+call) + args
        let len_prefix = compact_u32(inner_len);
        let mut bytes = Vec::with_capacity(len_prefix.len() + inner_len as usize);
        bytes.extend_from_slice(&len_prefix);
        bytes.push(0x04);
        bytes.push(0x08);
        bytes.push(0x00);
        bytes.extend_from_slice(&tx_encoded);
        let byte_len = bytes.len();
        let extrinsic_hex = format!("0x{}", hex::encode(&bytes));

        if prepared.is_empty() {
            std::fs::write("/tmp/bench-first-extrinsic.hex", &extrinsic_hex).ok();
            eprintln!("first extrinsic: {} bytes ({} hex chars)", byte_len, extrinsic_hex.len());
            eprintln!("first 120 hex chars: {}", &extrinsic_hex[..120.min(extrinsic_hex.len())]);
            eprintln!("tx sub-lengths:");
            eprintln!("  tx total encoded:    {}", tx_encoded.len());
            eprintln!("  inputs.encode:       {}", inputs_len);
            eprintln!("  outputs.encode:      {}", outputs_len);
            eprintln!("  witness.encode:      {}", witness_len);
            eprintln!("  pq_signature.encode: {}", pqsig_len);
        }
        prepared.push((format!("0x{}", hex::encode(&utxo_id)), extrinsic_hex, key.lock_hex));

        if (i + 1) % 10 == 0 { eprintln!("signed {}/{}", i + 1, mints.len()); }
    }

    eprintln!("submitting {} extrinsics in parallel", prepared.len());

    // Bounded concurrency + retry: jsonrpsee drops connections above ~200
    // in-flight HTTP requests, which shows up as "error decoding response body".
    let concurrency: usize = std::env::var("BENCH_CONCURRENCY")
        .ok().and_then(|s| s.parse().ok()).unwrap_or(40);
    let sem = Arc::new(Semaphore::new(concurrency));
    eprintln!("concurrency limit = {}", concurrency);

    let mut tasks = Vec::new();
    for (utxo_id_hex, extrinsic_hex, _) in prepared {
        let rpc = rpc.clone();
        let sem = sem.clone();
        tasks.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.expect("semaphore");
            let submitted_at_ms = now_ms();
            let mut last_err = None;
            for attempt in 0..4 {
                let r = rpc.call("author_submitExtrinsic",
                    serde_json::json!([extrinsic_hex.clone()])).await;
                match r {
                    Ok(_) => {
                        return ResultEntry {
                            utxo_id_hex,
                            submitted_at_ms,
                            included_at_ms: 0,
                            latency_ms: 0,
                            ok: true,
                            error: None,
                        };
                    }
                    Err(e) => {
                        let transient = e.contains("decoding response body")
                            || e.contains("Connection")
                            || e.contains("timed out");
                        last_err = Some(e);
                        if !transient { break; }
                        sleep(Duration::from_millis(50 * (1 << attempt))).await;
                    }
                }
            }
            ResultEntry {
                utxo_id_hex,
                submitted_at_ms,
                included_at_ms: 0,
                latency_ms: 0,
                ok: false,
                error: last_err,
            }
        }));
    }

    let mut results: Vec<ResultEntry> = Vec::new();
    for t in tasks {
        if let Ok(r) = t.await { results.push(r); }
    }

    let submitted = results.iter().filter(|r| r.ok).count();
    eprintln!("{} accepted by pool, waiting for inclusion...", submitted);

    // Poll: watch each utxo_id until it disappears (spent).
    // Use qutxo_getInclusionProof, which returns null for spent UTXOs.
    // (Raw state_getStorage is fiddly: StorageMap keys are
    //  twox128(pallet) ++ twox128(item) ++ blake2_128(key) ++ key,
    //  and getting that wrong returns Null for every query.)
    let deadline = Instant::now() + Duration::from_secs(120);
    let mut pending: std::collections::HashSet<String> = results.iter()
        .filter(|r| r.ok)
        .map(|r| r.utxo_id_hex.clone())
        .collect();

    eprintln!("waiting for {} UTXOs to be spent...", pending.len());

    while !pending.is_empty() && Instant::now() < deadline {
        let mut gone: Vec<String> = Vec::new();
        for u in &pending {
            if let Ok(serde_json::Value::Null) = rpc.call(
                "qutxo_getInclusionProof",
                serde_json::json!([u, null]),
            ).await
            {
                gone.push(u.clone());
            }
        }
        if !gone.is_empty() {
            let t = now_ms();
            for g in &gone {
                for r in results.iter_mut() {
                    if &r.utxo_id_hex == g && r.ok && r.included_at_ms == 0 {
                        r.included_at_ms = t;
                        r.latency_ms = t.saturating_sub(r.submitted_at_ms);
                    }
                }
                pending.remove(g);
            }
            eprintln!("  {} spent, {} pending", gone.len(), pending.len());
        }
        if !pending.is_empty() {
            sleep(Duration::from_millis(500)).await;
        }
    }

    std::fs::write(out, serde_json::to_string_pretty(&results).unwrap()).unwrap();
    eprintln!("wrote {} results -> {}", results.len(), out);
}

// --- report --------------------------------------------------------------

async fn cmd_report(args: &[String]) {
    let path = args.get(2).map(|s| s.as_str()).unwrap_or("/tmp/bench-results.json");
    let txt = std::fs::read_to_string(path)
        .unwrap_or_else(|e| { eprintln!("cannot read {}: {}", path, e); std::process::exit(1); });
    let results: Vec<ResultEntry> = serde_json::from_str(&txt).expect("parse results");

    let ok: Vec<&ResultEntry> = results.iter().filter(|r| r.ok).collect();
    let included: Vec<&ResultEntry> = ok.iter().filter(|r| r.latency_ms > 0).copied().collect();
    let failed = results.iter().filter(|r| !r.ok).count();

    println!("┌─────────────────────────────────────────┐");
    println!("│          CALIBRE BENCHMARK              │");
    println!("├─────────────────────────────────────────┤");
    println!("│ submitted          : {:>6}            │", results.len());
    println!("│ pool-accepted      : {:>6}            │", ok.len());
    println!("│ pool-rejected      : {:>6}            │", failed);
    println!("│ included & final   : {:>6}            │", included.len());

    if !included.is_empty() {
        let first = included.iter().map(|r| r.submitted_at_ms).min().unwrap();
        let last  = included.iter().map(|r| r.included_at_ms).max().unwrap();
        let dur_ms = last.saturating_sub(first);
        if dur_ms > 0 {
            let tps = included.len() as f64 * 1000.0 / dur_ms as f64;
            println!("│ wall window        : {:>6} ms       │", dur_ms);
            println!("│ throughput         : {:>6.1} tx/s    │", tps);
        }

        let mut lat: Vec<u128> = included.iter().map(|r| r.latency_ms).collect();
        lat.sort();
        let p = |q: f64| -> u128 {
            let i = ((lat.len() as f64 - 1.0) * q) as usize;
            lat[i]
        };
        println!("│ latency p50        : {:>6} ms       │", p(0.50));
        println!("│ latency p95        : {:>6} ms       │", p(0.95));
        println!("│ latency p99        : {:>6} ms       │", p(0.99));
    }
    println!("└─────────────────────────────────────────┘");

    if failed > 0 {
        println!();
        println!("first rejections:");
        for r in results.iter().filter(|r| !r.ok).take(5) {
            println!("  {} -> {}", r.utxo_id_hex, r.error.as_deref().unwrap_or("?"));
        }
    }
}

// --- main ----------------------------------------------------------------

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()).unwrap_or("help") {
        "keygen" => cmd_keygen(&args).await,
        "burst"  => cmd_burst(&args).await,
        "report" => cmd_report(&args).await,
        "blake"  => {
            let input = args.get(2).map(|s| s.as_str()).unwrap_or("hello");
            println!("{}", hex::encode(blake2_256(input.as_bytes())));
        },
        _ => eprintln!(
            "usage:\n  calibre-bench keygen <N> <out.json>\n  calibre-bench burst <keys.json> <mints.json> <rpc> <out.json> [fee_per_tx]\n  calibre-bench report <results.json>"
        ),
    }
}
