// signs sudo.sudo(qutxo.sudoMint(...)) as Alice and submits it.
// usage: node sign_mint.js <inner-call-hex>
//        node sign_mint.js warmup

const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');

const WS_URL = process.env.CALIBRE_WS || 'ws://127.0.0.1:9944';

(async () => {
  const innerHex = process.argv[2];
  if (!innerHex) {
    console.error('usage: sign_mint.js <inner-call-hex> | warmup');
    process.exit(1);
  }

  console.error('[signer] connecting to ' + WS_URL + '...');
  const api = await ApiPromise.create({
    provider: new WsProvider(WS_URL),
    noInitWarn: true,
  });
  console.error('[signer] connected');

  if (innerHex === 'warmup') {
    const chain = (await api.rpc.system.chain()).toString();
    console.error('[signer] warmup OK, chain=' + chain);
    process.exit(0);
  }

  const innerCall = api.registry.createType('Call', '0x' + innerHex);
  const kr = new Keyring({ type: 'sr25519' });
  const alice = kr.addFromUri('//Alice');

  const unsub = await api.tx.sudo.sudo(innerCall).signAndSend(alice, ({ status, dispatchError }) => {
    if (dispatchError) {
      let msg = dispatchError.toString();
      if (dispatchError.isModule) {
        try {
          const d = api.registry.findMetaError(dispatchError.asModule);
          msg = d.section + '.' + d.name;
        } catch {}
      }
      console.error('[signer] dispatch error: ' + msg);
      process.exit(2);
    }
    if (status.isInBlock) {
      console.log(status.asInBlock.toHex());
      unsub();
      process.exit(0);
    }
  });

  setTimeout(() => {
    console.error('[signer] timeout waiting for inclusion');
    process.exit(3);
  }, 60000);
})().catch(e => {
  console.error('[signer] fatal: ' + e.message);
  process.exit(1);
});
