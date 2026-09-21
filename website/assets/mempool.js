
/* ════════════════════════════════════════════════════════════════
   CALIBRE MEMPOOL — client-side simulation engine
   ════════════════════════════════════════════════════════════════ */

(() => {
  'use strict';

  // ═══════════════════════════════════════════════════════════
  // STATE
  // ═══════════════════════════════════════════════════════════

  const state = {
    running: true,
    chaos: false,
    speed: 2,
    height: 0,
    utxos: [],              // [{id, value, lock, bornAtBlock}]
    mempool: [],            // pending txs
    recentBlocks: [],
    stats: {
      passed: 0,
      rejected: 0,
      gates: {
        1: { pass: 0, fail: 0 },
        2: { pass: 0, fail: 0 },
        3: { pass: 0, fail: 0 },
        4: { pass: 0, fail: 0 },
      }
    },
    blockSlotMs: 6000,
    lastBlockTime: Date.now(),
    nextBlockAt: Date.now() + 6000,
    startTime: Date.now(),
    totalTxCreated: 0,
    txLog: [],              // recent transactions for stream
    txIdCounter: 0,
    utxoIdCounter: 0,
    tpsSamples: [],
  };

  // ═══════════════════════════════════════════════════════════
  // HELPERS
  // ═══════════════════════════════════════════════════════════

  const randHex = (n) => {
    let s = '';
    for (let i = 0; i < n; i++) s += '0123456789abcdef'[(Math.random() * 16) | 0];
    return s;
  };

  const shortHash = (h) => h.slice(0, 6) + '…' + h.slice(-4);

  const fmt = (n) => n.toLocaleString('en-US');

  const now = () => Date.now();

  // ═══════════════════════════════════════════════════════════
  // UTXO POOL MANAGEMENT
  // ═══════════════════════════════════════════════════════════

  function createUtxo(value, lock, bornAtBlock) {
    const id = randHex(64);
    const u = { id, value, lock: lock || randHex(64), bornAtBlock };
    state.utxos.push(u);
    state.utxoIdCounter++;
    return u;
  }

  function spendUtxo(id) {
    const idx = state.utxos.findIndex(u => u.id === id);
    if (idx === -1) return null;
    const [u] = state.utxos.splice(idx, 1);
    return u;
  }

  function getUtxo(id) {
    return state.utxos.find(u => u.id === id) || null;
  }

  function seedUtxos() {
    state.utxos = [];
    // Start with 8 UTXOs, varied values
    const values = [1_000_000_000, 750_000_000, 500_000_000, 400_000_000,
                    300_000_000, 250_000_000, 100_000_000, 50_000_000];
    for (const v of values) createUtxo(v, randHex(64), 0);
  }

  // ═══════════════════════════════════════════════════════════
  // TRANSACTION GENERATION
  // ═══════════════════════════════════════════════════════════

  function generateNormalTx() {
    // Pick a random UTXO to spend
    if (state.utxos.length === 0) {
      // Refill if empty
      createUtxo(1_000_000_000, randHex(64), state.height);
      createUtxo(500_000_000, randHex(64), state.height);
    }

    const spendIdx = (Math.random() * state.utxos.length) | 0;
    const inputUtxo = state.utxos[spendIdx];

    // Split into 2 outputs, both slightly less than input (fee = 0.1%)
    const fee = Math.floor(inputUtxo.value * 0.001);
    const netValue = inputUtxo.value - fee;
    const split1 = Math.floor(netValue * (0.4 + Math.random() * 0.2));
    const split2 = netValue - split1;

    return {
      type: 'normal',
      inputs: [inputUtxo.id],
      outputs: [
        { value: split1, lock: randHex(64) },
        { value: split2, lock: randHex(64) },
      ],
      witness: { pubKey: randHex(64), signature: randHex(240) },
    };
  }

  function generateDoubleSpendTx() {
    // Try to spend a UTXO already claimed by a pending tx
    if (state.mempool.length === 0) return generateNormalTx();
    const victim = state.mempool[(Math.random() * state.mempool.length) | 0];
    const inputId = victim.tx.inputs[0];

    return {
      type: 'double-spend',
      inputs: [inputId],
      outputs: [
        { value: 100_000_000, lock: randHex(64) },
      ],
      witness: { pubKey: randHex(64), signature: randHex(240) },
    };
  }

  function generateForgedTx() {
    // Normal-looking tx but with a garbage signature
    const base = generateNormalTx();
    base.type = 'forged';
    // Random garbage — not a valid ML-DSA-44 signature
    base.witness.signature = randHex(240);
    base.witness.forged = true;
    return base;
  }

  function generateReplayTx() {
    // Re-spend a UTXO from an already-included block
    if (state.recentBlocks.length === 0) return generateNormalTx();
    const oldBlock = state.recentBlocks[(Math.random() * state.recentBlocks.length) | 0];
    if (!oldBlock || !oldBlock.spentUtxoIds || oldBlock.spentUtxoIds.length === 0) {
      return generateNormalTx();
    }
    const deadId = oldBlock.spentUtxoIds[0];
    return {
      type: 'replay',
      inputs: [deadId],
      outputs: [{ value: 100_000_000, lock: randHex(64) }],
      witness: { pubKey: randHex(64), signature: randHex(240) },
    };
  }

  function createTransaction() {
    let tx;
    if (state.chaos && Math.random() < 0.35) {
      const kind = Math.random();
      if (kind < 0.4) tx = generateDoubleSpendTx();
      else if (kind < 0.75) tx = generateForgedTx();
      else tx = generateReplayTx();
    } else {
      tx = generateNormalTx();
    }

    state.txIdCounter++;
    const fullTx = {
      id: state.txIdCounter,
      hash: randHex(64),
      tx,
      createdAt: now(),
      verdict: 'pending',
      verdictReason: null,
      gateFailed: null,
    };

    state.mempool.push(fullTx);
    state.txLog.unshift(fullTx);
    if (state.txLog.length > 30) state.txLog.pop();

    return fullTx;
  }

  // ═══════════════════════════════════════════════════════════
  // THE FOUR GATES
  // ═══════════════════════════════════════════════════════════

  function gate1Conflict(tx) {
    // Input tags must be unique across mempool
    const inputId = tx.inputs[0];
    const claimed = state.mempool.some(other =>
      other !== tx && other.tx.inputs.includes(inputId) && other.verdict === 'pending'
    );
    return !claimed;
  }

  function gate2Exists(tx) {
    // Input must exist in UTXO set
    const inputId = tx.inputs[0];
    return getUtxo(inputId) !== null;
  }

  function gate3LockBinding(tx) {
    // blake2(witness.pubkey) must equal UTXO.lock (mock: always true for normal)
    const u = getUtxo(tx.inputs[0]);
    if (!u) return false;
    // In our mock, we say the witness pubkey hashes to the lock
    // For forged attacks, we simulate mismatch
    if (tx.tx.type === 'forged' && Math.random() < 0.5) return false;
    return true;
  }

  function gate4Crypto(tx) {
    // ML-DSA-44 signature verification
    // In our mock: normal txs always pass; forged txs always fail
    if (tx.tx.type === 'forged') return false;
    return true;
  }

  function runGates(tx) {
    if (!gate1Conflict(tx.tx)) {
      tx.verdict = 'rejected';
      tx.verdictReason = 'Priority is too low';
      tx.gateFailed = 1;
      state.stats.gates[1].fail++;
      return false;
    }
    state.stats.gates[1].pass++;

    if (!gate2Exists(tx.tx)) {
      tx.verdict = 'rejected';
      tx.verdictReason = 'Transaction is outdated';
      tx.gateFailed = 2;
      state.stats.gates[2].fail++;
      return false;
    }
    state.stats.gates[2].pass++;

    if (!gate3LockBinding(tx.tx)) {
      tx.verdict = 'rejected';
      tx.verdictReason = 'LockMismatch';
      tx.gateFailed = 3;
      state.stats.gates[3].fail++;
      return false;
    }
    state.stats.gates[3].pass++;

    if (!gate4Crypto(tx.tx)) {
      tx.verdict = 'rejected';
      tx.verdictReason = 'SignatureVerificationFailed';
      tx.gateFailed = 4;
      state.stats.gates[4].fail++;
      return false;
    }
    state.stats.gates[4].pass++;

    tx.verdict = 'passed';
    return true;
  }

  // ═══════════════════════════════════════════════════════════
  // BLOCK PRODUCTION
  // ═══════════════════════════════════════════════════════════

  function produceBlock() {
    state.height++;

    const ready = state.mempool.filter(t => t.verdict === 'passed');
    const spentUtxoIds = [];

    // Consume inputs, create outputs
    for (const tx of ready) {
      const inputId = tx.tx.inputs[0];
      const consumed = spendUtxo(inputId);
      if (consumed) spentUtxoIds.push(inputId);

      for (const out of tx.tx.outputs) {
        createUtxo(out.value, out.lock, state.height);
      }

      tx.verdict = 'included';
    }

    // Remove included txs from mempool
    state.mempool = state.mempool.filter(t => t.verdict === 'pending');

    const block = {
      height: state.height,
      hash: randHex(64),
      txCount: ready.length,
      spentUtxoIds,
      producedAt: now(),
    };
    state.recentBlocks.unshift(block);
    if (state.recentBlocks.length > 8) state.recentBlocks.pop();

    state.lastBlockTime = now();
    state.nextBlockAt = now() + state.blockSlotMs;

    // Track TPS
    const elapsedSec = (now() - state.startTime) / 1000;
    state.tpsSamples.push({ t: elapsedSec, count: state.stats.passed });

    renderAll();
  }

  // ═══════════════════════════════════════════════════════════
  // RENDER
  // ═══════════════════════════════════════════════════════════

  const $ = (id) => document.getElementById(id);

  function flash(el) {
    el.classList.remove('flash');
    void el.offsetWidth;
    el.classList.add('flash');
  }

  function renderMetrics() {
    $('m-height').textContent = '#' + state.height;

    const pending = state.mempool.filter(t => t.verdict === 'pending').length;
    $('m-mempool').textContent = pending;
    $('m-utxo').textContent = state.utxos.length;

    // TPS since start
    const elapsedSec = (now() - state.startTime) / 1000;
    const tps = elapsedSec > 0 ? (state.stats.passed / elapsedSec).toFixed(1) : '0.0';
    $('m-tps').textContent = tps;

    $('m-passed').textContent = state.stats.passed;
    $('m-rejected').textContent = state.stats.rejected;

    for (let g = 1; g <= 4; g++) {
      $('g' + g + '-pass').textContent = state.stats.gates[g].pass;
      $('g' + g + '-fail').textContent = state.stats.gates[g].fail;
    }
  }

  function renderBlockCountdown() {
    const remaining = Math.max(0, state.nextBlockAt - now());
    const sec = (remaining / 1000).toFixed(1);
    $('block-seconds').textContent = sec;

    const ratio = remaining / state.blockSlotMs;
    const dashoffset = 283 * (1 - ratio);
    $('block-ring').style.strokeDashoffset = dashoffset;

    $('block-next').textContent = '#' + (state.height + 1);

    const pending = state.mempool.filter(t => t.verdict === 'pending').length;
    $('block-tx-count').textContent = pending + ' transaction' + (pending === 1 ? '' : 's');
  }

  function renderRecentBlocks() {
    const el = $('recent-blocks');
    if (state.recentBlocks.length === 0) {
      el.innerHTML = '<div style="color: var(--text-faint); font-size: 11px;">awaiting first block…</div>';
      return;
    }
    el.innerHTML = state.recentBlocks.map(b => `
      <div class="mp-recent-block">
        <span class="rb-height">#${b.height}</span>
        <span class="rb-hash">0x${shortHash(b.hash)}</span>
        <span class="rb-count">${b.txCount} tx</span>
      </div>
    `).join('');
  }

  function renderUtxoPool() {
    const el = $('utxo-list');
    // Show latest 12 UTXOs
    const display = state.utxos.slice(-12).reverse();
    el.innerHTML = display.map(u => `
      <div class="mp-utxo">
        <span class="mp-utxo-id">0x${shortHash(u.id)}</span>
        <span class="mp-utxo-val">${fmt(u.value)}</span>
      </div>
    `).join('');
  }

  function renderTxStream() {
    const el = $('tx-stream');
    el.innerHTML = state.txLog.slice(0, 25).map(tx => {
      const t = new Date(tx.createdAt).toLocaleTimeString('en-US', { hour12: false });
      const txType = tx.tx.type;
      const verdict = tx.verdict;
      const verdictLabel = {
        pending:  'PENDING',
        passed:   'PASSED',
        included: 'INCLUDED',
        rejected: 'REJECTED · ' + (tx.verdictReason || ''),
      }[verdict] || verdict;

      const body = txType === 'normal' ? 'split → 2 outputs' :
                   txType === 'double-spend' ? 'DOUBLE-SPEND attempt' :
                   txType === 'forged' ? 'FORGED signature' :
                   txType === 'replay' ? 'REPLAY of spent UTXO' :
                   'tx';

      return `
        <div class="mp-tx ${verdict}">
          <span class="mp-tx-time">${t}</span>
          <span class="mp-tx-id">#${tx.id}</span>
          <span class="mp-tx-body">${body}</span>
          <span class="mp-tx-verdict">${verdictLabel}</span>
        </div>
      `;
    }).join('');
  }

  function renderAll() {
    renderMetrics();
    renderBlockCountdown();
    renderRecentBlocks();
    renderUtxoPool();
    renderTxStream();
  }

  // ═══════════════════════════════════════════════════════════
  // MAIN LOOPS
  // ═══════════════════════════════════════════════════════════

  function txGeneratorTick() {
    if (!state.running) return;
    const count = state.speed >= 5 ? 2 : 1;
    for (let i = 0; i < count; i++) {
      const tx = createTransaction();
      const ok = runGates(tx);
      if (ok) state.stats.passed++;
      else state.stats.rejected++;

      // Flash the gate that failed
      if (!ok && tx.gateFailed) {
        const row = document.querySelector(`.mp-gate-row[data-gate="${tx.gateFailed}"]`);
        if (row) {
          row.classList.add('flash-fail');
          setTimeout(() => row.classList.remove('flash-fail'), 400);
        }
      }
    }
    renderAll();
  }

  function blockTick() {
    if (!state.running) return;
    produceBlock();
  }

  function uiTick() {
    renderBlockCountdown();
  }

  // ═══════════════════════════════════════════════════════════
  // CONTROLS
  // ═══════════════════════════════════════════════════════════

  function setupControls() {
    $('btn-pause').addEventListener('click', () => {
      state.running = !state.running;
      $('btn-pause').textContent = state.running ? '⏸ Pause' : '▶ Resume';
      $('btn-pause').classList.toggle('active', !state.running);
    });

    $('btn-chaos').addEventListener('click', () => {
      state.chaos = !state.chaos;
      $('btn-chaos').textContent = state.chaos ? '☠ Chaos: ON' : '☠ Chaos: OFF';
      $('btn-chaos').classList.toggle('active', state.chaos);
    });

    $('speed-select').addEventListener('change', (e) => {
      state.speed = parseInt(e.target.value, 10);
    });

    $('btn-reset').addEventListener('click', () => {
      reset();
    });
  }

  function reset() {
    state.height = 0;
    state.mempool = [];
    state.recentBlocks = [];
    state.txLog = [];
    state.stats = {
      passed: 0, rejected: 0,
      gates: {
        1: { pass: 0, fail: 0 }, 2: { pass: 0, fail: 0 },
        3: { pass: 0, fail: 0 }, 4: { pass: 0, fail: 0 },
      }
    };
    state.startTime = now();
    state.lastBlockTime = now();
    state.nextBlockAt = now() + state.blockSlotMs;
    state.txIdCounter = 0;
    state.utxoIdCounter = 0;
    seedUtxos();
    renderAll();
  }

  // ═══════════════════════════════════════════════════════════
  // BOOT
  // ═══════════════════════════════════════════════════════════

  function boot() {
    seedUtxos();
    setupControls();
    renderAll();

    // Transaction generator: fires every 500ms (scaled by speed)
    setInterval(() => {
      const interval = 500 / state.speed;
      if (state.running) txGeneratorTick();
    }, 500);

    // Block producer: every 6 seconds (scaled by speed)
    setInterval(() => {
      if (state.running) blockTick();
    }, 6000 / state.speed);

    // UI tick: every 100ms for countdown smoothness
    setInterval(uiTick, 100);
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', boot);
  } else {
    boot();
  }
})();
