use dilithium::{MlDsaKeyPair, ML_DSA_44, DilithiumSignature};
use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("help");
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let key_path = format!("{}/.calibre-pq-key", home);

    match cmd {
        "keygen" => {
            let kp = MlDsaKeyPair::generate(ML_DSA_44).expect("keygen");
            fs::write(&key_path, &*kp.to_bytes()).expect("write key");
            println!("{}", hex::encode(kp.public_key()));
            eprintln!("keypair saved to {}", key_path);
        }
        "pubkey" => {
            let bytes = fs::read(&key_path).expect("no key; run keygen first");
            let kp = MlDsaKeyPair::from_bytes(&bytes).expect("decode");
            println!("{}", hex::encode(kp.public_key()));
        }
        "sign" => {
            let msg_hex = args.get(2).expect("usage: pq-signer sign <message-hex>");
            let msg = hex::decode(msg_hex.trim_start_matches("0x")).expect("bad hex");
            let bytes = fs::read(&key_path).expect("no key; run keygen first");
            let kp = MlDsaKeyPair::from_bytes(&bytes).expect("decode");
            let sig = kp.sign(&msg, b"").expect("sign");
            println!("{}", hex::encode(sig.as_bytes()));
        }
        "verify" => {
            let pk_hex  = args.get(2).expect("verify <pubkey> <msg> <sig>");
            let msg_hex = args.get(3).expect("verify <pubkey> <msg> <sig>");
            let sig_hex = args.get(4).expect("verify <pubkey> <msg> <sig>");
            let pk  = hex::decode(pk_hex.trim_start_matches("0x")).expect("bad pk");
            let msg = hex::decode(msg_hex.trim_start_matches("0x")).expect("bad msg");
            let sig_bytes = hex::decode(sig_hex.trim_start_matches("0x")).expect("bad sig");
            let sig = DilithiumSignature::from_bytes(sig_bytes);
            if MlDsaKeyPair::verify(&pk, &sig, &msg, b"", ML_DSA_44) {
                println!("VALID");
            } else {
                println!("INVALID");
                std::process::exit(2);
            }
        }
        _ => {
            eprintln!("commands: keygen | pubkey | sign <hex> | verify <pk> <msg> <sig>");
            std::process::exit(1);
        }
    }
}
