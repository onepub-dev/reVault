import { createRequire } from 'node:module';
import * as host from '@onepub-dev/revault-api';

const require = createRequire(import.meta.url);
const { Runtime } = require('./generated/revault_wasm_bindings.cjs');
const runtime = new Runtime();

const handles = new Set([
  'Lockbox', 'ContactKeyPair', 'ContactPublicKey', 'WrappedContactKey',
  'SigningKeyPair', 'SigningPublicKey', 'VaultDirectory', 'ReadOnlyVaultDirectory', 'Agent',
  'AgentActivity', 'Platform', 'LocalVault',
]);
const snake = value => value.replace(/([a-z0-9])([A-Z])/g, '$1_$2').toLowerCase();
function operation(className, method) {
  const name = snake(method);
  if (className === 'Vault') {
    if (method === 'lastError') return 'buffer_last_error';
    if (method === 'lastErrorDetails') return 'buffer_last_error_details';
    return name;
  }
  if (className === 'Lockbox') return `lockbox_${name}`;
  if (className === 'ContactKeyPair') return `key_contact_${name}`;
  if (className === 'ContactPublicKey') return method === 'encrypt' ? 'key_contact_encrypt' : `key_contact_${name}`;
  if (className === 'WrappedContactKey') return `key_contact_wrapped_${name}`;
  if (className === 'SigningKeyPair' || className === 'SigningPublicKey') return `key_signing_${name}`;
  if (className === 'VaultDirectory') return `vault_directory_${name}`;
  if (className === 'ReadOnlyVaultDirectory') return `vault_read_only_${name}`;
  if (className === 'Agent') {
    if (method === 'isRunning') return 'vault_is_running';
    if (method === 'forgetAll') return 'vault_forget_all';
    return `vault_agent_${name}`;
  }
  if (className === 'Platform') return `vault_platform_${name}`;
  if (className === 'LocalVault') return `vault_${name}`;
  throw new TypeError(`unsupported hosted WebAssembly class: ${className}`);
}
function wrap(value) {
  if (value == null || !handles.has(value.constructor?.name)) return value;
  return new Proxy(value, {
    get(target, property, receiver) {
      const member = Reflect.get(target, property, receiver);
      if (typeof member !== 'function') return wrap(member);
      return (...arguments_) => {
        runtime.before_call(operation(target.constructor.name, String(property)));
        const result = member.apply(target, arguments_);
        if (result instanceof Uint8Array || typeof result === 'string' ||
            (result != null && result.constructor?.name?.startsWith('Revault'))) {
          runtime.before_call('buffer_free');
        }
        return wrap(result);
      };
    },
  });
}

export class Vault {
  constructor() { return wrap(new host.Vault()); }
}
export const createMessage = host.createMessage;
export const encodeMessage = host.encodeMessage;
export const revault = host.revault;
export function wasmDispatchCount() { return runtime.calls; }
