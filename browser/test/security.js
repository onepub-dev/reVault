import { test } from 'node:test';
import assert from 'node:assert/strict';
import { wire, fromBase64, toBase64 } from '../src/codec.js';
let calls = [];
let missing = false;
globalThis.chrome = {
  runtime: { id: 'browser@revault.onepub.dev', onMessage: { addListener() {} },
    async sendNativeMessage(host, message) {
      if (missing) throw new Error('sensitive internal path');
      calls.push({ host, request: wire.BrowserRequest.decode(fromBase64(message.payload)) });
      return { payload: toBase64(wire.BrowserResponse.encode({ protocolVersion: 1, state: 'ok' }).finish()) };
    },
  },
  tabs: { async get() { return { url: 'https://test.example/current' }; } },
};
const { relay } = await import('../src/background.js');
const sender = { id: chrome.runtime.id, frameId: 0, tab: { id: 1 }, url: 'https://test.example/page', origin: 'https://test.example' };
function message(fields = {}) { return { payload: toBase64(wire.BrowserRequest.encode({ protocolVersion: 1, getCapabilities: true, ...fields }).finish()) }; }
function state(response) { return wire.BrowserResponse.decode(fromBase64(response.payload)).state; }

test('browser API context overwrites every webpage assertion', async () => {
  calls = [];
  assert.equal(state(await relay(message({ origin: 'https://evil.example', frameId: 999, extensionId: 'forged' }), sender)), 'ok');
  assert.equal(calls.length, 1);
  assert.equal(calls[0].request.origin, 'https://test.example');
  assert.equal(calls[0].request.frameId, 0);
  assert.equal(calls[0].request.extensionId, chrome.runtime.id);
  assert.equal(calls[0].host, 'dev.revault.browser');
});
test('wrong extension, iframe, opaque origin, navigation and HTTP are rejected', async () => {
  calls = [];
  for (const change of [
    { id: 'wrong' }, { frameId: 1 }, { tab: undefined }, { origin: 'null' },
    { url: 'http://test.example/page' }, { url: 'https://other.example/page', origin: 'https://other.example' },
    { url: 'chrome-extension://abc/options.html' },
  ]) assert.equal(state(await relay(message(), { ...sender, ...change })), 'invalid_request');
  assert.equal(calls.length, 0);
});
test('unknown operations, version and oversized input are rejected', async () => {
  calls = [];
  assert.equal(state(await relay(message({ protocolVersion: 2 }), sender)), 'upgrade_required');
  assert.equal(state(await relay({ payload: 'A'.repeat(23000) }, sender)), 'invalid_request');
  assert.equal(state(await relay({ payload: '?' }, sender)), 'invalid_request');
  const noOperation = { payload: toBase64(wire.BrowserRequest.encode({ protocolVersion: 1 }).finish()) };
  assert.equal(state(await relay(noOperation, sender)), 'invalid_request');
  assert.equal(calls.length, 0);
});
test('native errors reveal only a sanitised state', async () => {
  missing = true;
  const result = await relay(message(), sender);
  assert.equal(state(result), 'native_helper_missing');
  assert.ok(!JSON.stringify(result).includes('sensitive'));
  missing = false;
});
test('signed request bytes survive the extension relay without secret reads', async () => {
  calls = [];
  const signed = { request: { protocolVersion: 1, requestId: 'req-1', applicationId: 'test', recipientPublicKey: Uint8Array.of(1,2,3) }, signature: Uint8Array.of(4,5,6) };
  const packet = { payload: toBase64(wire.BrowserRequest.encode({ protocolVersion: 1, requestUnlock: signed }).finish()) };
  await relay(packet, sender);
  assert.deepEqual(wire.SignedUnlockRequest.encode(calls[0].request.requestUnlock).finish(), wire.SignedUnlockRequest.encode(signed).finish());
});

test('SDK rejects oversized input and removes listeners after response', async () => {
  const listeners = new Set();
  globalThis.location = { protocol: 'https:', origin: 'https://test.example' };
  globalThis.window = {
    addEventListener(_, fn) { listeners.add(fn); },
    removeEventListener(_, fn) { listeners.delete(fn); },
    postMessage(message, origin) {
      assert.equal(origin, location.origin);
      queueMicrotask(() => {
        const payload = toBase64(wire.BrowserResponse.encode({ protocolVersion: 1, state: 'approval_required' }).finish());
        for (const listener of listeners) listener({ source: window, origin, data: { channel: 'revault:response', id: message.id, payload } });
      });
    },
  };
  window.top = window;
  const sdk = await import('../src/sdk.js');
  assert.deepEqual(await sdk.requestUnlock(new Uint8Array(16385)), { state: 'invalid_request' });
  assert.equal(listeners.size, 0);
  assert.deepEqual(await sdk.getCapabilities(), { state: 'approval_required' });
  assert.equal(listeners.size, 0);
  assert.deepEqual(await sdk.cancelRequest('request-1'), { state: 'approval_required' });
  assert.equal(listeners.size, 0);
  window.top = {};
  assert.deepEqual(await sdk.getCapabilities(), { state: 'invalid_request' });
});
