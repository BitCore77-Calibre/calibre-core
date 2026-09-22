//! Calibre light client — trust tier 1, async, UniFFI-exported.
//!
//! Trust model: trust the node for "what is the current UTXO root," but
//! verify every inclusion proof locally against that root with
//! `calibre-merkle`. An adversarial node cannot forge a UTXO: it would
//! need a Merkle path folding to a root we've seen, infeasible without
//! breaking blake2b.

mod error;
mod rpc;
mod root_tracker;

pub use error::LightError;

use calibre_merkle::{inner_hash, leaf_hash};
use rpc::RpcClient;
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

#[derive(uniffi::Object)]
pub struct LightClient {
    rpc: RpcClient,
    roots: Mutex<root_tracker::RootTracker>,
    current_root: Mutex<Option<Hash>>,
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

#[uniffi::export(async_runtime = "tokio")]
impl LightClient {
    #[uniffi::constructor]
    pub fn connect(rpc_url: String) -> std::sync::Arc<Self> {
        std::sync::Arc::new(Self {
            rpc: RpcClient::new(rpc_url),
            roots: Mutex::new(root_tracker::RootTracker::new(16)),
            current_root: Mutex::new(None),
        })
    }

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
}

uniffi::setup_scaffolding!();
