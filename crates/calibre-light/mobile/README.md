# calibre-light — Mobile Integration

Native library + UniFFI bindings for iOS (Swift) and Android (Kotlin).

## What's here

    crates/calibre-light/
      bindings/swift/          Generated Swift (already committed)
      bindings/kotlin/         Generated Kotlin (already committed)
      mobile/                  This directory

## Build the native library

From repo root:

    bash scripts/build-mobile.sh

Outputs:

    target/mobile/android/jniLibs/{arm64-v8a,armeabi-v7a,x86_64}/libcalibre_light.so
    target/mobile/ios/libcalibre_light.a          (macOS only)
    target/mobile/ios/libcalibre_light.dylib      (macOS only)

## Android integration

1. Run `scripts/build-mobile.sh` on a machine with an Android NDK.
   Set `ANDROID_NDK_HOME` first:

       export ANDROID_NDK_HOME=$HOME/Android/Sdk/ndk/26.1.10909125

2. Copy `target/mobile/android/jniLibs/` into your module:

       app/src/main/jniLibs/

3. Copy the Kotlin bindings:

       crates/calibre-light/bindings/kotlin/uniffi/
    into

       app/src/main/kotlin/uniffi/

4. In your Kotlin code:

       import uniffi.calibre_light.LightClient
       import uniffi.calibre_light.SyncResult
       import uniffi.calibre_light.UtxoInfo

       // on a background thread (kotlinx.coroutines):
       withContext(Dispatchers.IO) {
           val client = LightClient.connect("https://rpc.calibre.example")
           val syncResult: SyncResult = client.syncVerified()
           println("verified root at block ${syncResult.blockNumber}")

           val utxo: UtxoInfo? = client.utxo(utxoIdBytes)
           if (utxo != null) {
               println("path_len=${utxo.pathLen}, age=${utxo.proofAgeBlocks}")
           }
       }

5. Load the native library once, in `Application.onCreate()` or a static init:

       companion object { init { System.loadLibrary("calibre_light") } }

## iOS integration

1. Run `scripts/build-mobile.sh` on a Mac with Xcode. Produces `target/mobile/ios/`.

2. In Xcode, drag the whole `crates/calibre-light/bindings/swift/` directory
   into your project:
   - `calibre_light.swift`
   - `calibre_lightFFI.h`
   - `calibre_lightFFI.modulemap`

3. Add `libcalibre_light.a` under "Link Binary With Libraries".

4. In your bridging header (or via a modulemap), expose the FFI header:

       #import "calibre_lightFFI.h"

5. Usage:

       import Foundation

       Task {
           let client = CalibreLightLightClient(rpcUrl: "https://rpc.calibre.example")
           let sync: SyncResult = try await client.syncVerified()
           print("verified root at block \(sync.blockNumber)")

           if let utxo = try await client.utxo(utxoId: utxoIdBytes) {
               print("path_len=\(utxo.pathLen) age=\(utxo.proofAgeBlocks)")
           }
       }

## API

| Method | Returns | Notes |
|---|---|---|
| `connect(rpcUrl)` | `LightClient` | Constructor. No I/O. |
| `sync()` | `[u8]` (32) | Tier 1. Fast. Trusts the RPC's storage answer. |
| `syncVerified()` | `SyncResult` | Tier 2. Verifies state proof against a finalized header. |
| `utxo(utxoId)` | `UtxoInfo?` | Fetches + verifies inclusion. Returns null if not in set. |
| `currentRoot()` | `[u8]?` | Last seen UTXO set root. |
| `lastVerifiedBlockNumber()` | `u32?` | Block number of last successful `syncVerified()`. |
| `knownRoots()` | `u32` | Ring buffer size. |
| `setMaxProofAge(blocks)` | — | Reject proofs older than this. Default 100. |
| `maxProofAge()` | `u32` | Current setting. |

## Errors

All methods throw `LightError` (or `throws` in Swift). Variants:

| Variant | Meaning |
|---|---|
| `Transport(String)` | HTTP/network failure. |
| `Rpc { code, message }` | Node returned a JSON-RPC error. |
| `Parse(String)` | Malformed response. |
| `UnknownRoot` | Proof root not in `knownRoots`. Likely a lying or forked node. |
| `BadPath` | Merkle path did not fold to claimed root. Node is provably lying. |
| `Hex(String)` | Hex decode failed. |
| `StaleProof { ageBlocks, maxBlocks }` | Proof too old. Call `syncVerified()` again. |
| `FinalizedHeadRegressed { prev, now }` | Node's finalized head went backwards. |

## Recommended usage pattern

1. On app launch: `sync()` — instant, tier 1, show UI immediately.
2. In the background: `syncVerified()` — replaces root with a proven one.
3. When showing a balance: `utxo(id)` — proof verified against the tier-2 root.
4. Every ~100 blocks: call `syncVerified()` again to advance the freshness window.

## Trust model (recap)

Tier 1 trusts the node's storage RPC. Tier 2 verifies the root is a real
state-trie leaf under a finalized header. Tier 3 (GRANDPA finality) is
not implemented; the mobile client trusts that the node's
`chain_getFinalizedHead` is honest. This is a smaller trust surface than
tier 1 and is standard for production light clients today.
