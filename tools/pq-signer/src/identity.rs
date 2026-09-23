//! Validator identity file format + helpers.
//!
//! An identity is written by `init` and consumed by `show`, `check`, and
//! (later) `export-chain-spec`. It is a plain JSON file with the public
//! parts of a validator: the ML-DSA-44 pubkey and its blake2_256 lock hash.
//! Private key material lives separately in `pq.key`.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const IDENTITY_FILE: &str = "identity.json";
pub const PQ_KEY_FILE: &str = "pq.key";
pub const PQ_PUB_FILE: &str = "pq.pub";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Identity {
    /// Schema version. Increment if the format changes incompatibly.
    pub version: u32,
    /// Human-readable validator name (e.g. "v1").
    pub name: String,
    /// ML-DSA-44 public key, hex-encoded (no 0x prefix).
    pub pq_public_key_hex: String,
    /// blake2_256(pq_public_key), hex-encoded. This is what goes into
    /// `QuantumLock::AegisThreshold(...)` for on-chain UTXO locks.
    pub pq_lock_hash_hex: String,
}

impl Identity {
    pub fn dir(base_path: &Path, name: &str) -> PathBuf {
        base_path.join(name)
    }

    pub fn identity_path(base_path: &Path, name: &str) -> PathBuf {
        Self::dir(base_path, name).join(IDENTITY_FILE)
    }

    pub fn pq_key_path(base_path: &Path, name: &str) -> PathBuf {
        Self::dir(base_path, name).join(PQ_KEY_FILE)
    }

    pub fn pq_pub_path(base_path: &Path, name: &str) -> PathBuf {
        Self::dir(base_path, name).join(PQ_PUB_FILE)
    }

    pub fn load(base_path: &Path, name: &str) -> Result<Self, String> {
        let path = Self::identity_path(base_path, name);
        let raw = fs::read_to_string(&path)
            .map_err(|e| format!("read {}: {}", path.display(), e))?;
        let id: Self = serde_json::from_str(&raw)
            .map_err(|e| format!("parse {}: {}", path.display(), e))?;
        Ok(id)
    }

    pub fn save(&self, base_path: &Path) -> Result<(), String> {
        let dir = Self::dir(base_path, &self.name);
        fs::create_dir_all(&dir)
            .map_err(|e| format!("mkdir {}: {}", dir.display(), e))?;
        let path = Self::identity_path(base_path, &self.name);
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("serialize: {}", e))?;
        fs::write(&path, json)
            .map_err(|e| format!("write {}: {}", path.display(), e))?;
        Ok(())
    }
}
