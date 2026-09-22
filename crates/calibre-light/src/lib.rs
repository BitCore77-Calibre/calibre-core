//! Calibre light client — trust tiers 1 and 2, async, UniFFI-exported.
//!
//! Trust tier 1 (`sync`): trusts the node for the UTXO set root.
//! Trust tier 2 (`sync_verified`): verifies the root is a real leaf in
//! the state trie anchored to a finalized block's `state_root`.

mod error;
mod rpc;
mod root_tracker;

pub use error::LightError;

use calibre_merkle::{inner_hash, leaf_hash};
use parity_scale_codec::Encode;
use rpc::RpcClient;
use smoldot::trie::{
    bytes_to_nibbles,
    proof_decode::{decode_and_verify_proof, Config as TrieConfig},
};
use std::sync::Mutex;

pub(crate) type Hash = [u8; 32];

const UTXO_SET_ROOT_KEY: &str =
    "0x8feb94cbd57b65eb1436ba6db973e04df4b8d3c19af7d55511d730e48dc7dc87";

#[derive(Debug, Clone, uniffi::Record)]
pub struct UtxoInfo {
    pub utxo_id: Vec<u8>,
    pub value_hash: Vec<u8>,
    pub root: Vec<u8>,
    pub path_len: u32,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SyncResult {
    pub utxo_set_root: Vec<u8>,
    pub block_hash: Vec<u8>,
    pub block_number: u32,
    pub verified: bool,
}

#[derive(uniffi::Object)]
pub struct LightClient {
    rpc: RpcClient,
    roots: Mutex<root_tracker::RootTracker>,
    current_root: Mutex<Option<Hash>>,
    last_block: Mutex<Option<(u32, Hash)>>,
}

fn parse_hash(s: &str) -> Result<Hash, LightError> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    if s.len() != 64 {
        return Err(LightError::Hex(format!("expected 64, got {}", s.len())));
    }
    let mut out = [0u8; 32];
    for i in 0..32 {
        out[i] = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16)
            .map_err(|e| LightError::Hex(e.to_string()))?;
    }
    Ok(out)
}

pub(crate) fn hash_hex(h: &Hash) -> String {
    let mut s = String::with_capacity(66);
    s.push_str("0x");
    for b in h { s.push_str(&format!("{:02x}", b)); }
    s
}

fn vec_to_hash(v: &[u8]) -> Result<Hash, LightError> {
    if v.len() != 32 {
        return Err(LightError::Hex(format!("expected 32 bytes, got {}", v.len())));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(v);
    Ok(out)
}

fn decode_hex(s: &str) -> Result<Vec<u8>, LightError> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    hex::decode(s).map_err(|e| LightError::Hex(e.to_string()))
}

/// Verify that `key` exists in the state trie rooted at `state_root`
/// and return its value. Uses a standard Substrate state proof.
fn verify_state_value(
    state_root: &Hash,
    key: &[u8],
    proof_nodes: &[Vec<u8>],
) -> Result<Vec<u8>, LightError> {
    let proof_scaled = proof_nodes.encode();
    let decoded = decode_and_verify_proof(TrieConfig { proof: proof_scaled })
        .map_err(|e| LightError::Parse(format!("proof decode: {:?}", e)))?;

    let key_nibbles: Vec<_> = bytes_to_nibbles(key.iter().copied()).collect();
    let entry = decoded
        .proof_entry(state_root, key_nibbles.into_iter())
        .ok_or_else(|| LightError::Parse("key not under expected state_root".into()))?;

    entry
        .unhashed_storage_value
        .map(|v| v.to_vec())
        .ok_or_else(|| LightError::Parse("storage value is hashed, not inline".into()))
}

#[uniffi::export(async_runtime = "tokio")]
impl LightClient {
    #[uniffi::constructor]
    pub fn connect(rpc_url: String) -> std::sync::Arc<Self> {
        std::sync::Arc::new(Self {
            rpc: RpcClient::new(rpc_url),
            roots: Mutex::new(root_tracker::RootTracker::new(16)),
            current_root: Mutex::new(None),
            last_block: Mutex::new(None),
        })
    }

    /// Trust tier 1: fetch root via `state_getStorage`, trust the node.
    pub async fn sync(&self) -> Result<Vec<u8>, LightError> {
        let resp = self
            .rpc
            .call("state_getStorage", serde_json::json!([UTXO_SET_ROOT_KEY, null]))
            .await?;
        let root_hex = resp
            .as_str()
            .ok_or_else(|| LightError::Parse("storage root not a string".into()))?;
        let root = parse_hash(root_hex)?;

        {
            let mut roots = self.roots.lock().unwrap();
            roots.push(root);
        }
        {
            let mut cur = self.current_root.lock().unwrap();
            *cur = Some(root);
        }
        Ok(root.to_vec())
    }

    /// Trust tier 2: fetch the finalized header, fetch a state proof for
    /// `Qutxo.UtxoSetRoot`, verify the proof against `header.state_root`.
    /// Returns the root plus the block hash and number it was proved against.
    pub async fn sync_verified(&self) -> Result<SyncResult, LightError> {
        // 1. Finalized head
        let r = self.rpc.call("chain_getFinalizedHead", serde_json::json!([])).await?;
        let block_hash_str = r
            .as_str()
            .ok_or_else(|| LightError::Parse("no finalized head".into()))?;
        let block_hash = parse_hash(block_hash_str)?;

        // 2. Header -> state_root + block number
        let r = self
            .rpc
            .call("chain_getHeader", serde_json::json!([block_hash_str]))
            .await?;
        let state_root_hex = r["stateRoot"]
            .as_str()
            .ok_or_else(|| LightError::Parse("no stateRoot".into()))?;
        let state_root = parse_hash(state_root_hex)?;
        let block_number_hex = r["number"]
            .as_str()
            .ok_or_else(|| LightError::Parse("no block number".into()))?;
        let block_number = u32::from_str_radix(
            block_number_hex.strip_prefix("0x").unwrap_or(block_number_hex),
            16,
        )
        .map_err(|e| LightError::Parse(format!("block number: {}", e)))?;

        // 3. Read proof for UtxoSetRoot
        let r = self
            .rpc
            .call(
                "state_getReadProof",
                serde_json::json!([[UTXO_SET_ROOT_KEY], block_hash_str]),
            )
            .await?;
        let proof_arr = r["proof"]
            .as_array()
            .ok_or_else(|| LightError::Parse("no proof array".into()))?;
        let proof_nodes: Result<Vec<Vec<u8>>, LightError> = proof_arr
            .iter()
            .map(|v| {
                let s = v.as_str().ok_or_else(|| LightError::Parse("proof node not string".into()))?;
                decode_hex(s)
            })
            .collect();
        let proof_nodes = proof_nodes?;

        // 4. Verify
        let key_bytes = decode_hex(UTXO_SET_ROOT_KEY)?;
        let value = verify_state_value(&state_root, &key_bytes, &proof_nodes)?;

        // 5. SCALE-decode H256 (32 raw bytes)
        let root = vec_to_hash(&value)?;

        {
            let mut roots = self.roots.lock().unwrap();
            roots.push(root);
        }
        {
            let mut cur = self.current_root.lock().unwrap();
            *cur = Some(root);
        }
        {
            let mut lb = self.last_block.lock().unwrap();
            *lb = Some((block_number, block_hash));
        }

        Ok(SyncResult {
            utxo_set_root: root.to_vec(),
            block_hash: block_hash.to_vec(),
            block_number,
            verified: true,
        })
    }

    pub async fn utxo(&self, utxo_id: Vec<u8>) -> Result<Option<UtxoInfo>, LightError> {
        let id = vec_to_hash(&utxo_id)?;

        let resp = self
            .rpc
            .call(
                "qutxo_getInclusionProof",
                serde_json::json!([hash_hex(&id), null]),
            )
            .await?;
        if resp.is_null() { return Ok(None); }

        let root = parse_hash(resp["root"].as_str()
            .ok_or_else(|| LightError::Parse("root".into()))?)?;
        let rid = parse_hash(resp["utxo_id"].as_str()
            .ok_or_else(|| LightError::Parse("utxo_id".into()))?)?;
        let vh = parse_hash(resp["value_hash"].as_str()
            .ok_or_else(|| LightError::Parse("value_hash".into()))?)?;

        {
            let roots = self.roots.lock().unwrap();
            if !roots.contains(&root) {
                return Err(LightError::UnknownRoot);
            }
        }

        let mut cur = leaf_hash(&rid, &vh);
        let path = resp["path"].as_array()
            .ok_or_else(|| LightError::Parse("path".into()))?;
        for step in path {
            let sib = parse_hash(step["sibling"].as_str()
                .ok_or_else(|| LightError::Parse("sibling".into()))?)?;
            let is_left = step["current_is_left"].as_bool()
                .ok_or_else(|| LightError::Parse("current_is_left".into()))?;
            cur = if is_left { inner_hash(&cur, &sib) }
                  else       { inner_hash(&sib, &cur) };
        }
        if cur != root {
            return Err(LightError::BadPath);
        }

        Ok(Some(UtxoInfo {
            utxo_id: rid.to_vec(),
            value_hash: vh.to_vec(),
            root: root.to_vec(),
            path_len: path.len() as u32,
        }))
    }

    pub fn current_root(&self) -> Option<Vec<u8>> {
        self.current_root.lock().unwrap().map(|h| h.to_vec())
    }

    pub fn known_roots(&self) -> u32 {
        self.roots.lock().unwrap().len() as u32
    }

    pub fn last_block(&self) -> Option<Vec<u8>> {
        self.last_block.lock().unwrap().as_ref().map(|(_n, h)| h.to_vec())
    }
}

uniffi::setup_scaffolding!();
