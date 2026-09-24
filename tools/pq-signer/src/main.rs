//! calibre-keygen — validator identity generation and inspection.
//!
//! Shipped as two binary names from one source:
//!   - calibre-keygen : new subcommands (init / show / check)
//!   - pq-signer      : legacy subcommands (keygen / pubkey / sign / verify)
//!
//! Both binary names accept all commands. Docker-compose still calls
//! pq-signer keygen, which continues to work unchanged.

mod identity;
mod bond;

use dilithium::{DilithiumSignature, MlDsaKeyPair, ML_DSA_44};
use identity::Identity;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const VERSION: u32 = 1;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    let result: Result<(), String> = match cmd {
        // ── legacy pq-signer commands (preserved for docker-compose) ──
        "keygen" => legacy_keygen(),
        "pubkey" => legacy_pubkey(),
        "sign" => legacy_sign(&args),
        "verify" => legacy_verify(&args),
        "bond-payload" => bond::print_payload(&args),

        // ── new calibre-keygen commands ──
        "init" => cmd_init(&args),
        "show" => cmd_show(&args),
        "check" => cmd_check(&args),
        "export-chain-spec" => cmd_export_chain_spec(&args),

        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        _ => Err(format!("unknown command: {}", cmd)),
    };

    if let Err(e) = result {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

fn print_help() {
    eprintln!("calibre-keygen — validator identity tool");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  init --base-path <dir> --name <v1> [--suri <//Alice>]");
    eprintln!("        [--node-bin <path>] [--chain <staging>]");
    eprintln!("      Generate ML-DSA-44 identity + BABE/GRANDPA/node keys.");
    eprintln!("  show --base-path <dir> --name <v1>");
    eprintln!("      Print the public parts of an identity.");
    eprintln!("  check --base-path <dir> --name <v1>");
    eprintln!("      Verify all keystore files are present.");
    eprintln!("  export-chain-spec --base-path <dir> --validators <v1> <v2> ... \\");
    eprintln!("        [--template <staging>] [--out <file>] [--node-bin <path>]");
    eprintln!("      Patch a chain spec with the given validators' authority keys.");
    eprintln!();
    eprintln!("Legacy (docker-compose compatibility):");
    eprintln!("  keygen | pubkey | sign <hex> | verify <pk> <msg> <sig>");
    eprintln!("  bond-payload <genesis-hash-hex> <beneficiary-account32-hex> '<inputs-json>'");
    eprintln!("      Print canonical bond-v1 signing bytes; no keys are read and nothing is submitted.");
}

// ═══════════════════════════════════════════════════════════════
// Legacy commands — unchanged behavior
// ═══════════════════════════════════════════════════════════════

fn legacy_key_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(format!("{}/.calibre-pq-key", home))
}

fn legacy_keygen() -> Result<(), String> {
    let kp = MlDsaKeyPair::generate(ML_DSA_44).map_err(|e| format!("keygen: {:?}", e))?;
    let path = legacy_key_path();
    fs::write(&path, &*kp.to_bytes()).map_err(|e| format!("write {}: {}", path.display(), e))?;
    println!("{}", hex::encode(kp.public_key()));
    eprintln!("keypair saved to {}", path.display());
    Ok(())
}

fn legacy_pubkey() -> Result<(), String> {
    let path = legacy_key_path();
    let bytes = fs::read(&path).map_err(|e| format!("no key at {}: {}", path.display(), e))?;
    let kp = MlDsaKeyPair::from_bytes(&bytes).map_err(|e| format!("decode: {:?}", e))?;
    println!("{}", hex::encode(kp.public_key()));
    Ok(())
}

fn legacy_sign(args: &[String]) -> Result<(), String> {
    let msg_hex = args.get(2).ok_or("usage: sign <message-hex>")?;
    let msg = hex::decode(msg_hex.trim_start_matches("0x")).map_err(|e| format!("hex: {}", e))?;
    let path = legacy_key_path();
    let bytes = fs::read(&path).map_err(|e| format!("no key: {}", e))?;
    let kp = MlDsaKeyPair::from_bytes(&bytes).map_err(|e| format!("decode: {:?}", e))?;
    let sig = kp.sign(&msg, b"").map_err(|e| format!("sign: {:?}", e))?;
    println!("{}", hex::encode(sig.as_bytes()));
    Ok(())
}

fn legacy_verify(args: &[String]) -> Result<(), String> {
    let pk_hex = args.get(2).ok_or("usage: verify <pubkey> <msg> <sig>")?;
    let msg_hex = args.get(3).ok_or("usage: verify <pubkey> <msg> <sig>")?;
    let sig_hex = args.get(4).ok_or("usage: verify <pubkey> <msg> <sig>")?;
    let pk = hex::decode(pk_hex.trim_start_matches("0x")).map_err(|e| format!("pk hex: {}", e))?;
    let msg =
        hex::decode(msg_hex.trim_start_matches("0x")).map_err(|e| format!("msg hex: {}", e))?;
    let sig_bytes =
        hex::decode(sig_hex.trim_start_matches("0x")).map_err(|e| format!("sig hex: {}", e))?;
    let sig = DilithiumSignature::from_bytes(sig_bytes);
    if MlDsaKeyPair::verify(&pk, &sig, &msg, b"", ML_DSA_44) {
        println!("VALID");
        Ok(())
    } else {
        println!("INVALID");
        std::process::exit(2);
    }
}

// ═══════════════════════════════════════════════════════════════
// New commands
// ═══════════════════════════════════════════════════════════════

fn parse_flag(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1).cloned())
}

fn cmd_init(args: &[String]) -> Result<(), String> {
    let base_path = parse_flag(args, "--base-path").ok_or("init requires --base-path")?;
    let name = parse_flag(args, "--name").ok_or("init requires --name")?;
    let suri = parse_flag(args, "--suri");
    let chain_alias = parse_flag(args, "--chain").unwrap_or_else(|| "staging".into());
    let node_bin = parse_flag(args, "--node-bin")
        .or_else(find_node_binary)
        .ok_or("init requires --node-bin, or solochain-template-node in PATH or target/release/")?;
    // `chain_alias` is what we pass to the node CLI (e.g. "staging").
    // `chain_id`   is what we use for filesystem paths (e.g. "calibre_staging").
    let chain_id =
        resolve_chain_id(&node_bin, &chain_alias).unwrap_or_else(|_| chain_alias.clone());

    let base = Path::new(&base_path);
    let dir = Identity::dir(base, &name);
    fs::create_dir_all(&dir).map_err(|e| format!("mkdir {}: {}", dir.display(), e))?;

    eprintln!("[1/4] Generating ML-DSA-44 keypair");
    let kp = MlDsaKeyPair::generate(ML_DSA_44).map_err(|e| format!("keygen: {:?}", e))?;
    let pk_bytes = kp.public_key().to_vec();
    let pk_hex = hex::encode(&pk_bytes);
    let lock_hash = sp_core::hashing::blake2_256(&pk_bytes);
    let lock_hex = hex::encode(lock_hash);

    fs::write(Identity::pq_key_path(base, &name), &*kp.to_bytes())
        .map_err(|e| format!("write pq.key: {}", e))?;
    fs::write(Identity::pq_pub_path(base, &name), &pk_bytes)
        .map_err(|e| format!("write pq.pub: {}", e))?;

    let id = Identity {
        version: VERSION,
        name: name.clone(),
        pq_public_key_hex: pk_hex.clone(),
        pq_lock_hash_hex: lock_hex.clone(),
    };
    id.save(base)?;

    eprintln!("[2/4] Inserting BABE key (sr25519)");
    insert_key(
        &node_bin,
        &dir,
        &chain_alias,
        "sr25519",
        "babe",
        suri.as_deref(),
    )?;

    eprintln!("[3/4] Inserting GRANDPA key (ed25519)");
    insert_key(
        &node_bin,
        &dir,
        &chain_alias,
        "ed25519",
        "gran",
        suri.as_deref(),
    )?;

    eprintln!("[4/4] Generating node network key");
    let net_key = dir
        .join("chains")
        .join(&chain_id)
        .join("network")
        .join("secret_ed25519");
    if !net_key.exists() {
        let status = Command::new(&node_bin)
            .args(["key", "generate-node-key", "--base-path"])
            .arg(&dir)
            .args(["--chain", &chain_alias])
            .stdout(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("spawn node key generate-node-key: {}", e))?;
        if !status.success() {
            return Err("node key generate-node-key failed".into());
        }
    }

    println!("{}", pk_hex);
    eprintln!();
    eprintln!("Validator identity written:");
    eprintln!("  base-path       : {}", dir.display());
    eprintln!("  name            : {}", name);
    eprintln!("  pq_public_key   : {}", pk_hex);
    eprintln!("  pq_lock_hash    : {}", lock_hex);
    eprintln!(
        "  identity.json   : {}",
        Identity::identity_path(base, &name).display()
    );
    eprintln!();
    eprintln!(
        "Backup {} and {} — no recovery if lost.",
        Identity::pq_key_path(base, &name).display(),
        dir.join("chains")
            .join(&chain_id)
            .join("keystore")
            .display()
    );
    Ok(())
}

fn insert_key(
    node_bin: &str,
    base_dir: &Path,
    chain: &str,
    scheme: &str,
    key_type: &str,
    suri: Option<&str>,
) -> Result<(), String> {
    let keystore = base_dir.join("chains").join(chain).join("keystore");
    let prefix_hex = hex::encode(key_type.as_bytes());
    if keystore.exists() {
        if let Ok(entries) = fs::read_dir(&keystore) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with(&prefix_hex) {
                        eprintln!("       (already present, skipping)");
                        return Ok(());
                    }
                }
            }
        }
    }
    let mut cmd = Command::new(node_bin);
    cmd.args(["key", "insert", "--base-path"])
        .arg(base_dir)
        .args(["--chain", chain, "--scheme", scheme, "--key-type", key_type]);
    match suri {
        Some(s) => {
            cmd.args(["--suri", s]);
        }
        None => {
            return Err(format!(
                "no --suri provided for {}:{} — deterministic keys require a suri",
                scheme, key_type
            ));
        }
    }
    let status = cmd
        .status()
        .map_err(|e| format!("spawn key insert: {}", e))?;
    if !status.success() {
        return Err(format!("key insert {}:{} failed", scheme, key_type));
    }
    Ok(())
}

fn cmd_show(args: &[String]) -> Result<(), String> {
    let base_path = parse_flag(args, "--base-path").ok_or("show requires --base-path")?;
    let name = parse_flag(args, "--name").ok_or("show requires --name")?;
    let base = Path::new(&base_path);
    let id = Identity::load(base, &name)?;

    println!("name            : {}", id.name);
    println!("version         : {}", id.version);
    println!("pq_public_key   : {}", id.pq_public_key_hex);
    println!("pq_lock_hash    : {}", id.pq_lock_hash_hex);
    Ok(())
}

fn cmd_check(args: &[String]) -> Result<(), String> {
    let base_path = parse_flag(args, "--base-path").ok_or("check requires --base-path")?;
    let name = parse_flag(args, "--name").ok_or("check requires --name")?;
    let chain_alias = parse_flag(args, "--chain").unwrap_or_else(|| "staging".into());
    let node_bin = parse_flag(args, "--node-bin")
        .or_else(find_node_binary)
        .ok_or(
            "check requires --node-bin, or solochain-template-node in PATH or target/release/",
        )?;
    let chain_id =
        resolve_chain_id(&node_bin, &chain_alias).unwrap_or_else(|_| "calibre_staging".into()); // fallback for tests
    let base = Path::new(&base_path);
    let dir = Identity::dir(base, &name);

    let mut all_ok = true;
    {
        let mut report = |label: &str, ok: bool| {
            println!("  [{:>4}] {}", if ok { "ok" } else { "FAIL" }, label);
            if !ok {
                all_ok = false;
            }
        };
        report(
            "identity.json",
            Identity::identity_path(base, &name).exists(),
        );
        report("pq.key", Identity::pq_key_path(base, &name).exists());
        report("pq.pub", Identity::pq_pub_path(base, &name).exists());

        let keystore = dir.join("chains").join(&chain_id).join("keystore");
        let has_prefix = |prefix: &str| -> bool {
            if let Ok(entries) = fs::read_dir(&keystore) {
                for entry in entries.flatten() {
                    if let Some(n) = entry.file_name().to_str() {
                        if n.starts_with(prefix) {
                            return true;
                        }
                    }
                }
            }
            false
        };
        let babe_prefix = hex::encode(b"babe");
        let gran_prefix = hex::encode(b"gran");
        report("babe keystore", has_prefix(&babe_prefix));
        report("gran keystore", has_prefix(&gran_prefix));

        let net_key = dir
            .join("chains")
            .join(&chain_id)
            .join("network")
            .join("secret_ed25519");
        report("node network key", net_key.exists());
    }

    if all_ok {
        println!();
        println!("All checks passed.");
        Ok(())
    } else {
        Err("one or more checks failed".into())
    }
}

/// Resolve the on-disk chain directory name for a given `--chain` alias.
///
/// Node's `key insert` writes to `chains/<chain_id>/`, where `<chain_id>`
/// comes from the chain spec's `id` field (e.g. `calibre_staging`), not
/// from the CLI alias (e.g. `staging`). We ask the node binary to build
/// the spec and read the id back.
fn resolve_chain_id(node_bin: &str, chain_alias: &str) -> Result<String, String> {
    let out = Command::new(node_bin)
        .args([
            "build-spec",
            "--chain",
            chain_alias,
            "--disable-default-bootnode",
        ])
        .stderr(std::process::Stdio::null())
        .output()
        .map_err(|e| format!("spawn build-spec: {}", e))?;
    if !out.status.success() {
        return Err(format!("build-spec --chain {} failed", chain_alias));
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Find `"id": "..."` in the JSON. The first occurrence is the spec id.
    // Lightweight parse without serde: locate the substring.
    for line in stdout.lines() {
        if let Some(idx) = line.find("\"id\"") {
            let after = &line[idx + 4..];
            if let Some(colon) = after.find(':') {
                let rest = after[colon + 1..].trim();
                let rest = rest.trim_start_matches('"');
                if let Some(end) = rest.find('"') {
                    return Ok(rest[..end].to_string());
                }
            }
        }
    }
    Err(format!(
        "could not parse chain id from build-spec output for {}",
        chain_alias
    ))
}

/// Extract a pubkey hex string from a keystore filename.
/// Keystore entries are named `<key_type_ascii_hex><pubkey_hex>`.
/// Returns the pubkey hex (everything after the 8-char prefix).
fn read_keystore_pubkey(keystore: &Path, key_type: &str) -> Result<String, String> {
    let prefix = hex::encode(key_type.as_bytes());
    let entries =
        fs::read_dir(keystore).map_err(|e| format!("read {}: {}", keystore.display(), e))?;
    for entry in entries.flatten() {
        if let Some(name) = entry.file_name().to_str() {
            if name.starts_with(&prefix) {
                return Ok(name[prefix.len()..].to_string());
            }
        }
    }
    Err(format!(
        "no {} key found in {}",
        key_type,
        keystore.display()
    ))
}

/// hex (32 bytes) -> SS58 address for sr25519.
fn sr25519_hex_to_ss58(hex_str: &str) -> Result<String, String> {
    use sp_core::crypto::Ss58Codec;
    use sp_core::sr25519::Public;
    let bytes = hex::decode(hex_str).map_err(|e| format!("hex: {}", e))?;
    let arr: [u8; 32] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| format!("expected 32 bytes, got {}", bytes.len()))?;
    Ok(Public::from_raw(arr).to_ss58check())
}

/// hex (32 bytes) -> SS58 address for ed25519.
fn ed25519_hex_to_ss58(hex_str: &str) -> Result<String, String> {
    use sp_core::crypto::Ss58Codec;
    use sp_core::ed25519::Public;
    let bytes = hex::decode(hex_str).map_err(|e| format!("hex: {}", e))?;
    let arr: [u8; 32] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| format!("expected 32 bytes, got {}", bytes.len()))?;
    Ok(Public::from_raw(arr).to_ss58check())
}

fn cmd_export_chain_spec(args: &[String]) -> Result<(), String> {
    let base_path =
        parse_flag(args, "--base-path").ok_or("export-chain-spec requires --base-path")?;
    let template = parse_flag(args, "--template").unwrap_or_else(|| "staging".into());
    let out = parse_flag(args, "--out").ok_or("export-chain-spec requires --out")?;
    let node_bin = parse_flag(args, "--node-bin")
        .or_else(find_node_binary)
        .ok_or("export-chain-spec requires --node-bin, or solochain-template-node in PATH")?;

    // Parse validator names after --validators (all non-flag args until next --flag).
    let mut validators: Vec<String> = Vec::new();
    if let Some(i) = args.iter().position(|a| a == "--validators") {
        for a in &args[i + 1..] {
            if a.starts_with("--") {
                break;
            }
            validators.push(a.clone());
        }
    }
    if validators.is_empty() {
        return Err(
            "export-chain-spec requires at least one validator name after --validators".into(),
        );
    }

    // 1. Get the base spec.
    eprintln!("[1/3] Building base spec (template = {})", template);
    let out_spec = Command::new(&node_bin)
        .args([
            "build-spec",
            "--chain",
            &template,
            "--disable-default-bootnode",
        ])
        .stderr(std::process::Stdio::null())
        .output()
        .map_err(|e| format!("spawn build-spec: {}", e))?;
    if !out_spec.status.success() {
        return Err(format!("build-spec --chain {} failed", template));
    }
    let stdout = String::from_utf8_lossy(&out_spec.stdout);
    // Skip any log lines before the JSON starts.
    let json_start = stdout.find('{').ok_or("no JSON in build-spec output")?;
    let spec: serde_json::Value = serde_json::from_str(&stdout[json_start..])
        .map_err(|e| format!("parse base spec: {}", e))?;

    // 2. Collect authority keys from each validator.
    eprintln!("[2/3] Collecting authority keys");
    let base = Path::new(&base_path);
    let chain_id =
        resolve_chain_id(&node_bin, &template).unwrap_or_else(|_| "calibre_staging".into());

    let mut babe_ss58: Vec<String> = Vec::new();
    let mut grandpa_ss58: Vec<String> = Vec::new();
    let mut account_ss58: Vec<String> = Vec::new();

    for name in &validators {
        let dir = Identity::dir(base, name);
        let keystore = dir.join("chains").join(&chain_id).join("keystore");
        if !keystore.exists() {
            return Err(format!(
                "keystore missing for validator {}: {}",
                name,
                keystore.display()
            ));
        }
        let babe_hex = read_keystore_pubkey(&keystore, "babe")?;
        let gran_hex = read_keystore_pubkey(&keystore, "gran")?;
        let babe_ss = sr25519_hex_to_ss58(&babe_hex)?;
        let gran_ss = ed25519_hex_to_ss58(&gran_hex)?;
        // The validator's account ID is their sr25519 pubkey, same encoding.
        let account_ss = babe_ss.clone();

        eprintln!(
            "      {}: babe={} grandpa={}",
            name,
            &babe_ss[..12],
            &gran_ss[..12]
        );
        babe_ss58.push(babe_ss);
        grandpa_ss58.push(gran_ss);
        account_ss58.push(account_ss);
    }

    // 3. Patch the spec.
    eprintln!("[3/3] Patching genesis");
    let mut spec = spec;
    let patch = spec["genesis"]["runtimeGenesis"]["patch"]
        .as_object_mut()
        .ok_or("missing genesis.runtimeGenesis.patch")?;

    // pallet-session is the single source of truth for BABE and GRANDPA
    // authorities. Direct authority lists stay empty to avoid double init.
    let session_keys: Vec<serde_json::Value> = account_ss58
        .iter()
        .zip(babe_ss58.iter().zip(grandpa_ss58.iter()))
        .map(|(account, (babe, grandpa))| {
            serde_json::json!([
                account,
                account,
                { "babe": babe, "grandpa": grandpa }
            ])
        })
        .collect();
    patch["babe"]["authorities"] = serde_json::json!([]);
    patch["grandpa"]["authorities"] = serde_json::json!([]);
    patch["session"]["keys"] = serde_json::json!(session_keys);

    // Balances: leave as the template spec's default. The base staging
    // preset already endows the sp_keyring accounts (Alice..Ferdie+One),
    // which match our test validators when they're generated with
    // sp_keyring-derived suris. Endowing arbitrary accounts with
    // large u128 amounts requires a different JSON encoding strategy
    // (see Phase 9.1c), and isn't needed for the boot test.
    //
    // When random suris are supported, this is where balance endowments
    // land for the corresponding account IDs.

    // sudo: first validator.
    if let Some(first) = account_ss58.first() {
        patch["sudo"]["key"] = serde_json::json!(first);
    }

    // Update the spec's name + id for the new chain.
    spec["name"] = serde_json::json!(format!(
        "Calibre {} ({} validators)",
        template,
        validators.len()
    ));
    spec["id"] = serde_json::json!(format!("calibre_{}", template));

    // Write.
    let out_str = serde_json::to_string_pretty(&spec).map_err(|e| format!("serialize: {}", e))?;
    fs::write(&out, out_str).map_err(|e| format!("write {}: {}", out, e))?;

    eprintln!();
    eprintln!("Wrote chain spec:");
    eprintln!("  output      : {}", out);
    eprintln!("  validators  : {}", validators.len());
    eprintln!("  babe keys   : {}", babe_ss58.len());
    eprintln!("  grandpa keys: {}", grandpa_ss58.len());
    eprintln!("  sudo        : {}", account_ss58[0]);
    eprintln!();
    eprintln!("Boot with:");
    eprintln!(
        "  solochain-template-node --chain {} --validator --name <name> \\",
        out
    );
    eprintln!("      --base-path <path-to-that-validators-base>");
    Ok(())
}

fn find_node_binary() -> Option<String> {
    let rel = Path::new("target/release/solochain-template-node");
    if rel.exists() {
        return Some(rel.to_string_lossy().into_owned());
    }
    if Command::new("solochain-template-node")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        return Some("solochain-template-node".into());
    }
    None
}
