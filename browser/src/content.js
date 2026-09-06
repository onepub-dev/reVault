const api = globalThis.browser ?? globalThis.chrome;
// Isolated world. No secret, filesystem, arbitrary native command or RPC API.
if (window === window.top && location.protocol === 'https:') {
  let active = 0;
  window.addEventListener('message', async event => {
    const message = event.data;
    if (event.source !== window || event.origin !== location.origin || message?.channel !== 'revault:request'
        || typeof message.id !== 'string' || !/^[0-9a-f-]{36}$/.test(message.id)
        || typeof message.payload !== 'string' || message.payload.length > 22000 || active >= 10) return;
    active++;
    try {
      const result = await api.runtime.sendMessage({ payload: message.payload });
      window.postMessage({ channel: 'revault:response', id: message.id, payload: result.payload }, location.origin);
    } catch { /* SDK timeout reports a missing extension/runtime. */ }
    finally { active--; }
  });
}
