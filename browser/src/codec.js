import { revault } from '../generated/protocol.js';
export const wire = revault.browser.v1;
export const MAX_BYTES = 16384;
export function fromBase64(value) {
  if (typeof value !== 'string' || value.length > 22000) throw new Error('invalid_request');
  const bytes = Uint8Array.from(atob(value), c => c.charCodeAt(0));
  if (!bytes.length || bytes.length > MAX_BYTES) throw new Error('invalid_request');
  return bytes;
}
export function toBase64(bytes) {
  if (!bytes.length || bytes.length > MAX_BYTES) throw new Error('invalid_request');
  return btoa(String.fromCharCode(...bytes));
}
export function failure(state) {
  return { payload: toBase64(wire.BrowserResponse.encode({ protocolVersion: 1, state }).finish()) };
}
