import { createRequire } from 'node:module';
import * as host from '@onepub-dev/revault-api';

// Keep transport implementation classes private. The hosted package's
// reviewed names are the only runtime concepts exported by this module.
/** Process-local encrypted archive handle supplied by the host package. */
export const Lockbox = host.Lockbox;
/** Token retaining one registered secret operation in the host agent. */
export const AgentActivity = host.AgentActivity;
/** A profile's private contact-encryption identity. */
export const ContactKeyPair = host.ContactKeyPair;
/** A contact's shareable encryption identity. */
export const ContactPublicKey = host.ContactPublicKey;
/** A content key encrypted for a specific contact. */
export const WrappedContactKey = host.WrappedContactKey;
/** A profile signing identity used when assigning Lockbox ownership. */
export const ProfileSigningKeyPair = host.ProfileSigningKeyPair;
/** The shareable verification half of a profile signing identity. */
export const ProfileSigningPublicKey = host.ProfileSigningPublicKey;
/** Operating-system credential-store facade supplied by the host. */
export const Platform = host.Platform;
/** Mutable binary secret that can be wiped deterministically. */
export const SecretBytes = host.SecretBytes;
/** Mutable UTF-8 secret that can be wiped deterministically. */
export const SecretString = host.SecretString;
/** Typed native failure carrying stable structured details. */
export const RevaultError = host.RevaultError;

const require = createRequire(import.meta.url);
const { Runtime } = require('./generated/revault_wasm_bindings.cjs');
const runtime = new Runtime();

const handles = new Set([
  'Revault',
  'Vault',
  'Lockbox', 'ContactKeyPair', 'ContactPublicKey', 'WrappedContactKey',
  'ProfileSigningKeyPair', 'ProfileSigningPublicKey', 'VaultDirectory',
  'ReadOnlyVaultDirectory', 'Agent', 'AgentSession', 'AgentActivity', 'Platform',
  'LocalVault',
]);
const snake = value => value.replace(/([a-z0-9])([A-Z])/g, '$1_$2').toLowerCase();
function operation(className, method) {
  const name = snake(method);
  if (className === 'Revault' || className === 'Vault') {
    // The hosted package uses the reviewed Vault name for both the root
    // dispatcher and the persistent store. Instance methods on the latter
    // retain the private transport prefix while the public name stays Vault.
    if (className === 'Vault' && new Set([
      'root', 'structureVersion', 'listPrivateKeys', 'listPrivateKeyNames',
      'listContactNames', 'listFormAliases', 'privateKeyExists',
      'deletePrivateKey', 'storePrivateKey', 'loadPrivateKey',
      'loadPrivateKeyGeneration', 'storeContact', 'loadContact',
      'contactExists', 'deleteContact', 'listContacts', 'storeProfileEmail',
      'profileEmail', 'storeBackup', 'loadBackup', 'backupCount',
      'restorePrivateKey', 'storeContactSigningKey',
      'loadContactSigningKey', 'listProfileGenerations', 'rotatePrivateKey',
      'rememberLockbox', 'forgetLockbox',
      'rememberKnownLockbox', 'listKnownLockboxes', 'forgetKnownLockbox',
      'rememberAccessSlotLabel', 'listAccessSlotLabels',
      'findAccessSlotLabels', 'forgetAccessSlotLabel', 'defineForm',
      'defineFormWithDescription', 'defineFormWithTypeId',
      'defineFormWithTypeIdAndDescription', 'importFormDefinition',
      'resolveForm', 'listForms', 'listFormRevisions', 'seedForms',
      'rememberPassword', 'rememberedPassword', 'free', 'close',
    ]).has(method)) return `vault_directory_${name}`;
    if (className === 'Vault' && method === 'loadProfileSigningKey') {
      return 'vault_directory_load_owner_signing_key';
    }
    if (className === 'Vault' && method === 'loadProfileSigningKeyGeneration') {
      return 'vault_directory_load_owner_signing_key_generation';
    }
    if (method === 'generateProfileSigningKeyPair') return 'key_signing_generate';
    if (method === 'profileSigningKeyPairFromPrivate') return 'key_signing_from_private';
    if (method === 'profileSigningPublicKeyFromBytes') return 'key_signing_public_from_bytes';
    if (method === 'lastError') return 'buffer_last_error';
    if (method === 'lastErrorDetails') return 'buffer_last_error_details';
    return name;
  }
  if (className === 'Lockbox') {
    // Descriptions are stored as the reserved encrypted variable rather than
    // having a separate native ABI operation.
    if (method === 'setDescription') return 'lockbox_set_variable';
    if (method === 'clearDescription') return 'lockbox_delete_variable';
    if (method === 'withSecretVariable') return 'lockbox_get_secret_variable';
    if (method === 'withSecretFormField') return 'lockbox_get_secret_form_field';
    return `lockbox_${name}`;
  }
  if (className === 'ContactKeyPair') return `key_contact_${name}`;
  if (className === 'ContactPublicKey') return method === 'encrypt' ? 'key_contact_encrypt' : `key_contact_${name}`;
  if (className === 'WrappedContactKey') return `key_contact_wrapped_${name}`;
  if (className === 'AgentSession') {
    if (new Set([
      'createLockboxPassword', 'openLockboxPassword', 'createLockboxContentKey',
      'openLockboxContentKey', 'createLockboxContact', 'cacheLockboxPassword',
      'free',
    ]).has(method)) return `vault_${name}`;
    if (method === 'closeLockbox') return 'vault_close_lockbox';
    if (method === 'closeAll') return 'vault_close_all';
    return operation('Agent', method);
  }
  if (className === 'ProfileSigningKeyPair') {
    if (method === 'publicBytes') return 'key_signing_public';
    if (method === 'privateRecord') return 'key_signing_private';
    if (method === 'publicKey') return 'key_signing_public_from_bytes';
    if (method === 'dispose') return 'key_signing_free';
  }
  if (className === 'ProfileSigningPublicKey' && method === 'dispose') {
    return 'key_signing_public_free';
  }
  if (className === 'VaultDirectory') return `vault_directory_${name}`;
  if (className === 'ReadOnlyVaultDirectory') return `vault_read_only_${name}`;
  if (className === 'Agent') {
    if (method === 'isRunning') return 'vault_is_running';
    if (method === 'forgetAll') return 'vault_forget_all';
    if (method === 'profileSigningKey') return 'vault_agent_get_owner_signing_key';
    if (method === 'cacheProfileSigningKey') return 'vault_agent_put_owner_signing_key';
    if (method === 'forgetProfileSigningKey') return 'vault_agent_forget_owner_signing_key';
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

/** Hosted runtime entry point. WebAssembly performs portable lockbox work and
 * delegates filesystem, Vault, credential-store, and AgentSession operations
 * to the installed host package. */
export class Revault {
  /** Creates a facade whose binding calls are dispatched through the WASM runtime. */
  constructor() { return wrap(new host.Revault()); }
  /** Loads the linked host runtime; WASM itself does not open a Vault. */
  static async load() { return new Revault(); }
}

/** Explicit controller for the optional host session agent. */
export class AgentSession {
  /** Connects to the installed host's single agent controller. */
  constructor() { return wrap(new host.AgentSession()); }
  /** Returns the process-wide host agent controller. */
  static get instance() { return wrap(host.AgentSession.instance); }
}

function persistentVault(value) {
  return new Proxy({ native: wrap(value) }, {
    get(target, property, receiver) {
      if (property in target) return Reflect.get(target, property, receiver);
      const member = target.native[property];
      return typeof member === 'function'
        ? (...arguments_) => member.apply(target.native, arguments_)
        : member;
    },
  });
}

/** Persistent encrypted Vault supplied by the host package. */
export class Vault {
  /** Opens an existing Vault without creating or replacing storage. */
  static open(root, vaultPassphrase) {
    return persistentVault(host.Vault.open(root, vaultPassphrase));
  }
  /** Opens a Vault or creates it when absent. */
  static openOrCreate(root, vaultPassphrase) {
    return persistentVault(host.Vault.openOrCreate(root, vaultPassphrase));
  }
  /** Replaces a Vault explicitly; this discards the previous store. */
  static replace(root, vaultPassphrase) {
    return persistentVault(host.Vault.replace(root, vaultPassphrase));
  }
}
/** Returns the number of binding calls dispatched through the WASM runtime. */
export function wasmDispatchCount() { return runtime.calls; }
