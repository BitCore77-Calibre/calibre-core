# calibre-light

Async light client for Calibre. Verifies UTXO inclusion proofs locally
against the on-chain UTXO set root. Trust tier 1: trusts the node for
"what is the current root," verifies all inclusion math locally.

## Rust usage

    use calibre_light::LightClient;

    let client = LightClient::connect("http://127.0.0.1:9944".into());
    let root = client.sync().await?;
    if let Some(info) = client.utxo(utxo_id_bytes).await? {
        println!("verified, path_len={}", info.path_len);
    }

## Mobile bindings

Swift and Kotlin bindings are generated with UniFFI and checked into
`bindings/`. To regenerate after an API change:

    cargo build -p calibre-light --release
    cargo run --features uniffi/cli --bin uniffi-bindgen -- generate \
        --library target/release/libcalibre_light.so \
        --language swift \
        --out-dir crates/calibre-light/bindings/swift
    cargo run --features uniffi/cli --bin uniffi-bindgen -- generate \
        --library target/release/libcalibre_light.so \
        --language kotlin \
        --out-dir crates/calibre-light/bindings/kotlin

Optional formatters: swiftformat, ktlint (warnings are cosmetic).

## Trust model

- Trusts the node for the value of Qutxo.UtxoSetRoot (state_getStorage).
- Verifies every inclusion proof locally by folding the Merkle path
  with calibre-merkle. A lying node cannot forge a UTXO.
- Rejects proofs whose root is not in the local ring buffer of seen roots.

Tier 2 (verify the root against the state trie + block header) is not
implemented yet.
