/**
 * Hosted WebAssembly adapter for the complete reVault JavaScript API.
 * OS-backed operations are explicitly dispatched to the native host package.
 *
 * @see {@link https://github.com/onepub-dev/reVault#readme | Repository README}
 * @packageDocumentation
 */
export * from '@onepub-dev/revault-api';
/** Returns the number of binding calls dispatched through the WASM runtime. */
export function wasmDispatchCount(): number;
