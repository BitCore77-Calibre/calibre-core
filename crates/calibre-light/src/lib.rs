//! Calibre light client v0.1 — trust tier 1, async.
//!
//! Trust model: trust the node for "what is the current UTXO root," but
//! verify every inclusion proof locally against that root with
//! `calibre-merkle`. An adversarial node cannot forge a UTXO: it would
//! need a Merkle path folding to a root we've seen, infeasible without
//! breaking blake2b.
//!
//! Tier 2 (verify root against the state trie) is not implemented.

mod error;
mod rpc;
mod root_tracker;

pub use error::LightError;
pub use root_tracker::RootTracker;

use calibre_merkle::{inner_hash, leaf_hash, Hash};
use rpc::RpcClient;

/// twox128("Qutxo") ++ twox128("UtxoSetRoot").
const UTXO_SET_ROOT_KEY: &str =
    "0x8feb94cbd57b65eb1436ba6db973e04df4b8d3c19af7d55511d730e48dc7dc87";

#[derive(Debug, Clone)]
pub struct UtxoInfo {
    pub utxo_id: Hash,
    pub value_hash: Hash,
    pub root: Hash,
    pub path_len: usize,
}

pub struct LightClient {
    rpc: RpcClient,
    roots: RootTracker,
    current_root: Option<Hash>,
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

pub fn hash_hex(h: &Hash) -> String {
    let mut s = String::with_capacity(66);
    s.push_str("0x");
    for b in h { s.push_str(&format!("{:02x}", b)); }
    s
}

impl LightClient {
    pub fn connect(rpc_url: impl Into<String>) -> Self {
        Self {
            rpc: RpcClient::new(rpc_url),
            roots: RootTracker::new(16),
            current_root: None,
        }
    }

    pub async fn sync(&mut self) -> Result<Hash, LightError> {
        let resp = self
            .rpc
            .call("state_getStorage", serde_json::json!([UTXO_SET_ROOT_KEY, null]))
            .await?;
        let root_hex = resp
            .as_str()
            .ok_or_else(|| LightError::Parse("storage root not a string".into()))?;
        let root = parse_hash(root_hex)?;
        self.roots.push(root);
        self.current_root = Some(root);
        Ok(root)
    }

    pub async fn utxo(&self, utxo_id: Hash) -> Result<Option<UtxoInfo>, LightError> {
        let resp = self
            .rpc
            .call(
                "qutxo_getInclusionProof",
                serde_json::json!([hash_hex(&utxo_id), null]),
            )
            .await?;
        if resp.is_null() { return Ok(None); }

        let root = parse_hash(resp["root"].as_str()
            .ok_or_else(|| LightError::Parse("root".into()))?)?;
        let id = parse_hash(resp["utxo_id"].as_str()
            .ok_or_else(|| LightError::Parse("utxo_id".into()))?)?;
        let vh = parse_hash(resp["value_hash"].as_str()
            .ok_or_else(|| LightError::Parse("value_hash".into()))?)?;

        if !self.roots.contains(&root) {
            return Err(LightError::UnknownRoot);
        }

        let mut cur = leaf_hash(&id, &vh);
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
            utxo_id: id,
            value_hash: vh,
            root,
            path_len: path.len(),
        }))
    }

    pub fn current_root(&self) -> Option<Hash> { self.current_root }
    pub fn known_roots(&self) -> usize { self.roots.len() }
}
