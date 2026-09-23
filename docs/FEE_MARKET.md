# Calibre Fee Market — Design (Phase 8)

## The core constraint

`execute_utxo_tx` is **unsigned**. There is no signer account, no
`pallet_transaction_payment` charge, no `Balances::withdraw`. Fees
must be captured from the UTXO flow itself.

## The natural fee

Every UTXO transaction has an implicit fee:

    fee = sum(inputs) - sum(outputs)

The current implementation computes this difference and *discards it*.
Phase 8 captures it and routes it.

## The seven decisions (locked 2026-09-23)

1. **Fee source:** implicit (`inputs - outputs`) for base, plus an
   optional explicit priority output.
2. **Fee split:** 50% producer / 30% treasury / 20% burned.
3. **Base fee:** fixed per-tx floor for Phase 8.1; dynamic
   (EIP-1559-style) for Phase 8.3.
4. **Block rewards:** small inflationary issuance per block,
   tunable by governance, treasury-funded bootstrap.
5. **Stake in Phase 8:** bond/unbond only. Election, slashing, and
   rewards distribution are Phase 9.
6. **Custom pallet:** `pallet-calibre-fees`, purpose-built for UTXO.
7. **Treasury control:** root in Phase 8, governance in Phase 9+.

## Architecture

    ┌────────────────────────────────────────────────────────┐
    │  pallet-qutxo                                          │
    │  execute_utxo_tx(tx)                                   │
    │    1. verify ML-DSA                                    │
    │    2. read inputs, remove UTXOs                        │
    │    3. write outputs                                    │
    │    4. fee = inputs - outputs                           │
    │    5. T::FeeHandler::charge_fee(fee, producer)   ────┐ │
    └──────────────────────────────────────────────────────┼─┘
                                                          │
    ┌──────────────────────────────────────────────────────▼─┐
    │  pallet-calibre-fees                                    │
    │  charge_fee(fee, producer):                             │
    │    - split 50/30/20                                     │
    │    - credit producer (UTXO lock, or account)            │
    │    - accumulate treasury                                │
    │    - increment burn counter                             │
    │    - emit FeeCharged event                              │
    └────────────────────────────────────────────────────────┘

The pallet also exposes `minimum_fee(inputs, outputs)` which
`validate_unsigned` calls to reject under-priced txs at pool admission.

## Storage

| Storage | Type | Purpose |
|---------|------|---------|
| `TreasuryLock` | `[u8; 32]` | ML-DSA pubkey hash of the treasury UTXO lock |
| `ProducerLocks` | `StorageMap<AccountId, [u8; 32]>` | Validator → their UTXO lock |
| `TreasuryAccumulated` | `Balance` | Fees awaiting sweep into a UTXO |
| `BurnCounter` | `Balance` | Cumulative burned total (stats) |

## Extrinsics

| Call | Who | Purpose |
|------|-----|---------|
| `set_treasury_lock` | root | Set the treasury ML-DSA lock |
| `register_producer_lock` | root | Register a validator's UTXO lock |

Phase 9 makes `register_producer_lock` self-service and tied to the
validator's session keys.

## Configuration

| Constant | Value | Meaning |
|----------|-------|---------|
| `BaseTxFee` | 1000 | Minimum fee per tx |
| `ProducerFeeShare` | 50 | % to producer |
| `TreasuryFeeShare` | 30 | % to treasury |
| `BurnFeeShare` | 20 | % burned |
| `PerInOutFee` | 100 | Additional fee per input + output |

Sum of shares must equal 100. Enforced in genesis or on-chain check.

## Phasing

- **8.1 (this block):** pallet exists, stores treasury lock, exposes
  `charge_fee` and `minimum_fee`. qutxo does NOT yet call it.
- **8.2:** qutxo calls `charge_fee` in `execute_utxo_tx`. Split applies.
- **8.3:** `minimum_fee` used in `validate_unsigned`. Under-priced txs
  rejected at pool admission.
- **8.4:** dynamic base fee (EIP-1559 style). Per-block adjustment.
- **8.5:** block rewards. Treasury-funded issuance.
- **8.6:** priority fee (optional producer-routed output).
- **8.7:** stake pallet (bond/unbond only).

## Not yet designed

- Producer UTXO creation at block finalize (needs `pallet-authorship`)
- Treasury UTXO sweep (needs a treasury spending mechanism, root or gov)
- Slashing mechanics (Phase 9)
- Governance over fee parameters (Phase 9)

## Open questions

- Should the priority fee be a separate output or a field in the tx?
- Should the producer be paid per-tx or per-block?
- How is `ProducerLocks` populated before any validator registers?
- Does burn reduce `qutxo::TotalIssuance`, or is it tracked separately?
