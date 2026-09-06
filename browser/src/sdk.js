import { wire, fromBase64, toBase64, MAX_BYTES } from './codec.js';
function exchange(operation, timeout) {
  let payload;
  try { payload = toBase64(wire.BrowserRequest.encode({ protocolVersion: 1, ...operation }).finish()); }
  catch { return Promise.resolve({ state: "invalid_request" }); }
  if (window !== window.top || location.protocol !== 'https:') return Promise.resolve({ state: 'invalid_request' });
  return new Promise(resolve => {
    const id = crypto.randomUUID();
    const listener = event => {
      if (event.source !== window || event.origin !== location.origin || event.data?.channel !== 'revault:response' || event.data.id !== id) return;
      try {
        const response = wire.BrowserResponse.decode(fromBase64(event.data.payload));
        finish(response.protocolVersion !== 1 ? { state: 'upgrade_required' } : {
          state: response.state,
          ...(response.envelope ? { envelope: wire.UnlockEnvelope.encode(response.envelope).finish() } : {}),
        });
      } catch { finish({ state: 'invalid_request' }); }
    };
    const timer = setTimeout(() => finish({ state: 'extension_missing' }), timeout);
    const finish = result => { clearTimeout(timer); window.removeEventListener('message', listener); resolve(result); };
    window.addEventListener('message', listener);
    window.postMessage({ channel: 'revault:request', id, payload }, location.origin);
  });
}
/** Detect the extension and native agent without releasing secrets. */
export function getCapabilities() { return exchange({ getCapabilities: true }, 2000); }
/** Supply server-signed protobuf bytes; receive only an encrypted protobuf envelope. */
export function requestUnlock(serverRequest) {
  try {
    if (!(serverRequest instanceof Uint8Array) || serverRequest.length > MAX_BYTES) throw new Error();
    const requestUnlock = wire.SignedUnlockRequest.decode(serverRequest);
    return exchange({ requestUnlock }, 130000);
  } catch { return Promise.resolve({ state: 'invalid_request' }); }
}
/** Cancel a pending request in this origin and browser, including native approval. */
export function cancelRequest(requestId) {
  if (typeof requestId !== 'string' || !/^[A-Za-z0-9_.:@-]{1,128}$/.test(requestId)) return Promise.resolve({ state: 'invalid_request' });
  return exchange({ cancelRequest: requestId }, 5000);
}
