//! Off-chain verification diagnostic: load the receipt+journal we saved,
//! verify with risc0-verifier using the same Vk the pallet uses.
//! If this fails, the bug is in data/format. If it succeeds, the pallet
//! is passing different bytes.

use std::fs;

fn main() {
    // Same 32-byte image_id as the runtime config
    let image_id_bytes: [u8; 32] = [
        0xf7, 0x74, 0xfe, 0x2d, 0x73, 0x2c, 0xa4, 0xeb,
        0x96, 0x09, 0x35, 0x40, 0x8c, 0xaa, 0x25, 0x30,
        0x79, 0x31, 0x73, 0x4e, 0x93, 0xbd, 0xe8, 0x92,
        0xed, 0x43, 0x37, 0xf6, 0xdd, 0x58, 0x71, 0x62,
    ];

    let receipt_bytes = fs::read("/tmp/calibre-receipt.cbor")
        .expect("read receipt");
    let journal_bytes = fs::read("/tmp/calibre-journal.bin")
        .expect("read journal");

    println!("receipt: {} bytes", receipt_bytes.len());
    println!("journal: {} bytes", journal_bytes.len());

    // Deserialize with the verifier's own type
    let proof: risc0_verifier::Proof = match ciborium::from_reader(receipt_bytes.as_slice()) {
        Ok(p) => p,
        Err(e) => {
            println!("DESERIALIZE FAILED: {:?}", e);
            std::process::exit(1);
        }
    };
    println!("deserialize: OK");

    // Try both Vk representations
    let vk = risc0_verifier::Vk::from(image_id_bytes);
    let j = risc0_verifier::Journal::new(journal_bytes.clone());

    match risc0_verifier::verify(&risc0_verifier::v3_0(), vk, proof.clone(), j) {
        Ok(()) => println!("verify with [u8;32] Vk: OK"),
        Err(e) => println!("verify with [u8;32] Vk: FAILED {:?}", e),
    }

    // Try the u32-word representation: same bytes interpreted as 8 LE u32 words.
    // If the risc0 digest From<[u8;32]> is big-endian-byte but risc0 internal
    // representation is LE words, this might be the correct form.
    let mut u32_words = [0u32; 8];
    for i in 0..8 {
        u32_words[i] = u32::from_le_bytes([
            image_id_bytes[i * 4],
            image_id_bytes[i * 4 + 1],
            image_id_bytes[i * 4 + 2],
            image_id_bytes[i * 4 + 3],
        ]);
    }
    let vk2 = risc0_verifier::Vk::from(u32_words);
    let j2 = risc0_verifier::Journal::new(journal_bytes);
    match risc0_verifier::verify(&risc0_verifier::v3_0(), vk2, proof, j2) {
        Ok(()) => println!("verify with [u32;8] LE Vk: OK"),
        Err(e) => println!("verify with [u32;8] LE Vk: FAILED {:?}", e),
    }
}
