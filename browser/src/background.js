import { wire, fromBase64, toBase64, failure } from './codec.js';
const api = globalThis.browser ?? globalThis.chrome;
const active = new Map();

// Exported for tests using browser MessageSender fixtures.
export async function relay(message, sender) {
  if (sender.id !== api.runtime.id || sender.frameId !== 0 || !sender.tab || typeof sender.url !== 'string') {
    return failure('invalid_request');
  }
  let origin;
  try {
    const url = new URL(sender.url);
    if (url.protocol !== 'https:' || (sender.origin && sender.origin !== url.origin)) return failure('invalid_request');
    origin = url.origin;
    const tab = await api.tabs.get(sender.tab.id);
    if (new URL(tab.url).origin !== origin) return failure('invalid_request');
  } catch { return failure('invalid_request'); }
  let request;
  try {
    request = wire.BrowserRequest.decode(fromBase64(message?.payload));
    if (request.protocolVersion !== 1) return failure('upgrade_required');
    if (!['getCapabilities', 'requestUnlock', 'cancelRequest'].includes(request.operation)) return failure('invalid_request');
    // These fields always come from the browser, replacing all page values.
    request.origin = origin;
    request.extensionId = api.runtime.id;
    request.frameId = 0;
  } catch { return failure('invalid_request'); }
  const tabId = sender.tab.id;
  const count = active.get(tabId) ?? 0;
  if (count >= 8 && request.operation !== 'cancelRequest') return failure('busy');
  if (count >= 10) return failure('busy');
  active.set(tabId, count + 1);
  try {
    const result = await api.runtime.sendNativeMessage('dev.revault.browser', {
      payload: toBase64(wire.BrowserRequest.encode(request).finish()),
    });
    const response = wire.BrowserResponse.decode(fromBase64(result?.payload));
    if (response.protocolVersion !== 1) return failure('upgrade_required');
    // Envelopes contain ciphertext only. Never permit arbitrary host fields.
    return { payload: toBase64(wire.BrowserResponse.encode(response).finish()) };
  } catch { return failure('native_helper_missing'); }
  finally {
    const remaining = (active.get(tabId) ?? 1) - 1;
    if (remaining) active.set(tabId, remaining); else active.delete(tabId);
  }
}
api.runtime.onMessage.addListener((message, sender, respond) => {
  relay(message, sender).then(respond, () => respond(failure('invalid_request')));
  return true;
});
