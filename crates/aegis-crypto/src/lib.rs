#![cfg_attr(not(feature = "std"), no_std)]

use parity_scale_codec::{Decode, Encode};
use sp_core::hashing::blake2_256;
use sp_std::prelude::*;
use sha3::{Shake256, digest::{Update, ExtendableOutput}};
use calibre_primitives::QuantumLock;
use dilithium::{MlDsaKeyPair, ML_DSA_44, DilithiumSignature};

#[derive(Clone, Encode, Decode, PartialEq, Eq, sp_runtime::RuntimeDebug, scale_info::TypeInfo)]
pub struct DeviceAttestation {
    pub hardware_signature: Vec<u8>,
    pub app_binary_hash: [u8; 32],
    pub backend_challenge: [u8; 32],
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, sp_runtime::RuntimeDebug, scale_info::TypeInfo)]
pub struct AegisWitness {
    pub pub_keys: Vec<Vec<u8>>,
    pub aggregated_signature: Vec<u8>,
    pub attestation: DeviceAttestation,
    pub time_drift: u64,
}

pub struct AegisCryptoCore;

impl AegisCryptoCore {
    pub fn derive_context(
        biometric_entropy: &[u8],
        attestation: &DeviceAttestation,
        time_drift: u64,
    ) -> [u8; 32] {
        let mut hasher = Shake256::default();
        hasher.update(biometric_entropy);
        hasher.update(&blake2_256(&attestation.encode()));
        hasher.update(&time_drift.to_le_bytes());
        let mut context = [0u8; 32];
        hasher.finalize_xof_into(&mut context);
        context
    }

    pub fn verify_aegis_transaction(
        lock: &QuantumLock,
        tx_payload: &[u8],
        witness: &AegisWitness,
    ) -> Result<(), AegisCryptoError> {
        let expected_lock = match lock {
            QuantumLock::AegisThreshold(h) => h,
            QuantumLock::SingleSig(_) => return Err(AegisCryptoError::LockTypeMismatch),
        };
        if witness.pub_keys.is_empty() {
            return Err(AegisCryptoError::InsufficientShares);
        }
        let pk_hash = blake2_256(&witness.pub_keys[0]);
        if &pk_hash != expected_lock {
            return Err(AegisCryptoError::LockMismatch);
        }
        Self::verify_dilithium_signature(
            &witness.pub_keys[0],
            tx_payload,
            &witness.aggregated_signature,
        )?;
        Ok(())
    }

    pub fn verify_dilithium_signature(
        pub_key: &[u8],
        message: &[u8],
        signature: &[u8],
    ) -> Result<(), AegisCryptoError> {
        let sig = DilithiumSignature::from_bytes(signature.to_vec());
        if MlDsaKeyPair::verify(pub_key, &sig, message, b"", ML_DSA_44) {
            Ok(())
        } else {
            Err(AegisCryptoError::SignatureVerificationFailed)
        }
    }
}

#[derive(Clone, PartialEq, Eq, sp_runtime::RuntimeDebug, scale_info::TypeInfo)]
pub enum AegisCryptoError {
    LockTypeMismatch,
    LockMismatch,
    InvalidSignatureSize,
    InvalidPublicKeySize,
    InsufficientShares,
    SignatureVerificationFailed,
}
