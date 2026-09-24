# Phase 9.4 single-computer election run

**Status: prepared, not run.** This is a user-started, long-duration local
test. It does not require other computers.

## Start

On the computer with the `phase9.4-babe` checkout, run:

```bash
cd "$HOME/calibre-template"
bash tools/run_phase94_local.sh
```

The command builds the release node and embedded Wasm, creates a new
`.phase94-runs/<UTC timestamp>/` directory, prepares a raw staging chain
specification, starts seven validators plus one standby, and starts the
read-only Observatory. It never overwrites a previous run. It uses ports
20333–20340 for peers, 19944–19951 for local RPC, and 18765 for the
Observatory. Optional arguments to the launcher are `--p2p-start`,
`--rpc-start`, `--explorer-port`, and `--run-dir` (which must not exist).

Open **http://127.0.0.1:18765/** on the same computer. Select
**Slots & epochs** to see the current session, the seven active validator
accounts, the BABE and GRANDPA public keys, finalized heights on each node,
and the Phase 9.4 result. Keep the terminal open until the result appears.
The normal schedule is 600 six-second BABE slots per session, with the
replacement planned for session 6 after roughly six hours. Missed slots
can extend the run. Press Ctrl+C to stop; logs and the Observatory's local
observation database remain in the run directory.

## What PASS means

The Observatory reports **PASS** only after all eight nodes have the same
genesis, the expected runtime version (103) and 600-slot BABE epoch, and
have reached session 6. It then checks that exactly one of the initial
seven validators is replaced by `//Two`, that the BABE and GRANDPA key
sets also change by exactly one, and that all eight nodes agree on a
common finalized block after at least three more finalized blocks.
**AWAITING_NODES**, **AWAITING_ELECTION**, and **OBSERVING** are incomplete.
**FAIL** means a checked invariant failed. The node and explorer logs
remain available for diagnosis.

## Scope and limitations

This fixture starts from the tracked `staging` genesis in the current
release binary. It adds the well-known `//Two` test session keys and
synthetic stake for eight candidates to a disposable raw chain spec.
The top seven by stake are the first six incumbents plus `//Two`, so the
unchanged production session manager performs the election at session 6.
These synthetic stake entries are not backed by QUTXO bonds. This run
does not test signed `bond`, `join_candidates`, or `session.set_keys`
transactions; those have separate runtime test coverage. It also does
not verify multi-computer networking, an Aura-to-BABE in-place upgrade,
or production readiness. The identities are public development keys;
never use this chain specification for a real network.
