//! Public-data-only bond authorization encoder. Existing offline `sign`
//! accepts the returned hex; this command never reads a private key.
use calibre_primitives::{staking_bond_payload, TransactionInput};
use serde::Deserialize;
use sp_core::H256;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    tx_hash: String,
    output_index: u32,
}

fn hex32(value: &str, label: &str) -> Result<[u8; 32], String> {
    let value = value.strip_prefix("0x").unwrap_or(value);
    if value.len() != 64 {
        return Err(format!(
            "{label} must be exactly 32 bytes of hex (not an SS58 address)"
        ));
    }
    let mut bytes = [0; 32];
    hex::decode_to_slice(value, &mut bytes).map_err(|_| format!("invalid {label} hex"))?;
    Ok(bytes)
}

pub fn payload(genesis: &str, beneficiary: &str, json: &str) -> Result<Vec<u8>, String> {
    let genesis = H256::from(hex32(genesis, "genesis hash")?);
    let beneficiary = hex32(beneficiary, "beneficiary account")?;
    // The current runtime limits bonds to 16 inputs. Bound JSON before decoding.
    if json.len() > 4096 {
        return Err("inputs JSON exceeds 4096 bytes".into());
    }
    let parsed: Vec<Input> = serde_json::from_str(json)
        .map_err(|_| "inputs must be JSON objects with tx_hash and u32 output_index".to_string())?;
    if parsed.is_empty() || parsed.len() > 16 {
        return Err("a bond requires 1 to 16 inputs".into());
    }
    let mut inputs = Vec::with_capacity(parsed.len());
    for input in parsed {
        let input = TransactionInput {
            tx_hash: H256::from(hex32(&input.tx_hash, "input transaction hash")?),
            output_index: input.output_index,
        };
        if inputs.contains(&input) {
            return Err("duplicate bond input".into());
        }
        inputs.push(input);
    }
    Ok(staking_bond_payload(&genesis, &beneficiary, &inputs))
}

pub fn print_payload(args: &[String]) -> Result<(), String> {
    if args.len() != 5 {
        return Err(
            "usage: bond-payload <genesis-hash-hex> <beneficiary-account32-hex> '<inputs-json>'"
                .into(),
        );
    }
    println!("{}", hex::encode(payload(&args[2], &args[3], &args[4])?));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input_json(index: u32) -> String {
        format!(
            r#"[{{"tx_hash":"{}","output_index":{index}}}]"#,
            "33".repeat(32)
        )
    }

    #[test]
    fn canonical_account32_golden_vector() {
        // Independent fixed layout: compact domain length, ASCII domain,
        // genesis[32], AccountId32[32], compact input count, hash[32], LE index.
        let mut expected = vec![96]; // 24-byte domain, SCALE compact length
        expected.extend_from_slice(b"CALIBRE::stake::bond::v1");
        expected.extend_from_slice(&[0x11; 32]);
        expected.extend_from_slice(&[0x22; 32]);
        expected.push(4);
        expected.extend_from_slice(&[0x33; 32]);
        expected.extend_from_slice(&7u32.to_le_bytes());
        assert_eq!(
            payload(&"11".repeat(32), &"22".repeat(32), &input_json(7)).unwrap(),
            expected
        );
    }

    #[test]
    fn rejects_invalid_public_fields() {
        let hash = "11".repeat(32);
        assert!(payload("bad", &hash, &input_json(0)).is_err());
        assert!(payload(&hash, "SS58-address", &input_json(0)).is_err());
        assert!(payload(&"zz".repeat(32), &hash, &input_json(0)).is_err());
        assert!(payload(&hash, &hash, "[]").is_err());
        assert!(payload(&hash, &hash, r#"[{"tx_hash":"x","output_index":-1}]"#).is_err());
        assert!(payload(&hash, &hash, &" ".repeat(4097)).is_err());
        assert!(payload(
            &hash,
            &hash,
            r#"[{"tx_hash":"x","output_index":4294967296}]"#
        )
        .is_err());
    }

    #[test]
    fn rejects_duplicate_oversized_and_unknown_fields() {
        let hash = "11".repeat(32);
        let item = format!(r#"{{"tx_hash":"{hash}","output_index":0}}"#);
        assert!(payload(&hash, &hash, &format!("[{item},{item}]")).is_err());
        let many = (0..17)
            .map(|i| format!(r#"{{"tx_hash":"{hash}","output_index":{i}}}"#))
            .collect::<Vec<_>>()
            .join(",");
        assert!(payload(&hash, &hash, &format!("[{many}]")).is_err());
        let extra = format!(r#"[{{"tx_hash":"{hash}","output_index":0,"extra":1}}]"#);
        assert!(payload(&hash, &hash, &extra).is_err());
    }

    #[test]
    fn command_requires_exact_arguments() {
        assert!(print_payload(&["pq-signer".into(), "bond-payload".into()]).is_err());
        let args = vec![
            "pq-signer".into(),
            "bond-payload".into(),
            "11".repeat(32),
            "22".repeat(32),
            input_json(0),
        ];
        assert!(print_payload(&args).is_ok());
        let mut extra = args;
        extra.push("unexpected".into());
        assert!(print_payload(&extra).is_err());
    }

    #[test]
    fn signer_payload_signature_verifies_for_runtime_account_id32_only() {
        use dilithium::{MlDsaKeyPair, ML_DSA_44};
        let genesis = H256::repeat_byte(0x11);
        let account = sp_core::crypto::AccountId32::new([0x22; 32]);
        let inputs = vec![TransactionInput {
            tx_hash: H256::repeat_byte(0x33),
            output_index: 7,
        }];
        let cli_bytes = payload(&"11".repeat(32), &"22".repeat(32), &input_json(7)).unwrap();
        let kp = MlDsaKeyPair::generate(ML_DSA_44).unwrap();
        let sig = kp.sign(&cli_bytes, b"").unwrap();
        assert!(MlDsaKeyPair::verify(
            kp.public_key(),
            &sig,
            &staking_bond_payload(&genesis, &account, &inputs),
            b"",
            ML_DSA_44
        ));
        let other = sp_core::crypto::AccountId32::new([0x23; 32]);
        assert!(!MlDsaKeyPair::verify(
            kp.public_key(),
            &sig,
            &staking_bond_payload(&genesis, &other, &inputs),
            b"",
            ML_DSA_44
        ));
    }
}
