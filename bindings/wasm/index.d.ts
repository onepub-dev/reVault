/**
 * Hosted WebAssembly adapter for the complete reVault JavaScript API.
 * OS-backed operations are explicitly dispatched to the native host package.
 *
 * @see {@link https://github.com/onepub-dev/reVault#readme | Repository README}
 * @packageDocumentation
 */
export type * from '@onepub-dev/revault-api';
/** Public host types reused unchanged by the WebAssembly dispatcher. */
export {
  Lockbox, AgentSession, AgentActivity, ContactKeyPair, ContactPublicKey,
  WrappedContactKey, ProfileSigningKeyPair, ProfileSigningPublicKey, Platform,
  SecretBytes, SecretString, RevaultError, LockboxCacheMode, LockboxWorkload,
  LockboxWorker, AgentActivityKind, KeyExportFormat,
} from '@onepub-dev/revault-api';
/** Native-backed owned handles that can cross the hosted dispatch boundary. */
export type NativeHandle = ContactKeyPair | ContactPublicKey | WrappedContactKey |
  ProfileSigningKeyPair | ProfileSigningPublicKey | AgentActivity;
import { Revault as HostedRuntime } from '@onepub-dev/revault-api';
import type {
  ContactKeyPair, ContactPublicKey, WrappedContactKey,
  ProfileSigningKeyPair, ProfileSigningPublicKey, AgentActivity,
} from '@onepub-dev/revault-api';
/** Hosted runtime entry point whose calls are counted by the WASM dispatcher. */
export class Revault extends HostedRuntime {
  /** Creates a hosted runtime facade; it does not open a Vault. */
  constructor();
  /** Loads the installed host runtime without opening persistent state. */
  static load(): Promise<Revault>;
}
/** Persistent encrypted Vault supplied by the host package. */
export class Vault {
  /** Constructs only through a static Vault lifecycle factory. */
  private constructor();
  /** Opens an existing Vault without creating or replacing storage. */
  static open(root: string, vaultPassphrase: import('@onepub-dev/revault-api').BinaryInput): Vault;
  /** Opens a Vault or creates it when absent. */
  static openOrCreate(root: string, vaultPassphrase: import('@onepub-dev/revault-api').BinaryInput): Vault;
  /** Replaces a Vault explicitly; destructive. */
  static replace(root: string, vaultPassphrase: import('@onepub-dev/revault-api').BinaryInput): Vault;
  /** Returns the filesystem root containing this persistent Vault. */
  root(): string;
  /** Returns the authenticated persistent Vault structure version. */
  structureVersion(): number;
  /** Releases the persistent Vault handle and its decrypted state. */
  close(): void;
  /** Alias for close used by generated hosted-handle cleanup. */
  free(): void;
  /** Routes additional reviewed Vault operations to the installed host facade. */
  [method: string]: unknown;
}
/** Returns the number of binding calls dispatched through the WASM runtime. */
export function wasmDispatchCount(): number;
