/**
 * Encrypt files, variables, and typed form records in portable reVault
 * lockboxes, and manage keys and local vault metadata.
 *
 * Start with {@link Revault}. Use {@link Vault} for persistent metadata. Call `free()` on owned handles and use the
 * callback-scoped secret accessors to avoid retaining plaintext.
 *
 * @see {@link https://github.com/onepub-dev/reVault#readme | Repository README}
 * for installation, security guidance, and complete examples.
 * @packageDocumentation
 */
export type * from './domain.js';
/** Returns the binary. */
export type Binary = Uint8Array;
/** Returns the binary input. */
export type BinaryInput = Uint8Array | string;
/** Owning binary secret; call close() when the native operation ends. */
export class SecretBytes extends Uint8Array { close(): void; }
/** Owning UTF-8 passphrase; call close() to wipe its mutable bytes. */
export class SecretString extends SecretBytes { constructor(value: string); toString(): string; }
/** Error raised when the native ABI rejects an operation; `details` identifies the failing category. */
export class RevaultError extends Error { readonly details?: import('./domain.js').ErrorDetails; }
/** Cache policies for an open Lockbox. */
export const LockboxCacheMode: { readonly BYTES: 'bytes'; readonly DISABLED: 'disabled'; readonly AUTOMATIC: 'automatic'; };
/** Workload profiles used to tune archive import and reads. */
export const LockboxWorkload: { readonly INTERACTIVE: 'interactive'; readonly BULK_IMPORT: 'bulk-import'; readonly READ_MOSTLY: 'read-mostly'; };
/** Worker policies used by archive operations. */
export const LockboxWorker: { readonly AUTO: 'auto'; readonly SINGLE: 'single'; readonly THREADS: 'threads'; };
/** Categories recorded by the explicit Session Agent. */
export const AgentActivityKind: { readonly OPEN: 'open'; readonly CLOSE: 'close'; readonly VARIABLES: 'variables'; readonly FORM: 'form'; readonly RECOVERY: 'recovery'; readonly VAULT: 'vault'; };
/** Stable key export encodings accepted by Revault key methods. */
export const KeyExportFormat: { readonly LOCKBOX_PEM: 'lockbox-pem'; readonly JWK: 'jwk'; readonly JWKS: 'jwks'; readonly RAW_HEX: 'raw-hex'; };
/** Returns the native handle. */
export type NativeHandle = ContactKeyPair | ContactPublicKey | WrappedContactKey | ProfileSigningKeyPair | ProfileSigningPublicKey | AgentActivity;
/** Primary API used to open lockboxes, manage keys and metadata, use the
 * Session Agent, and access the platform credential store. Create one
 * when the application starts. */
export class Revault {
  /** Creates a new facade over the bundled native library. */
  constructor(nativeLibraryPath?: string);
  /** Load the installed native carrier and create the runtime facade. */
  static load(nativeLibraryPath?: string): Promise<Revault>;
  /** Return a new synchronous runtime facade for factory operations. */
  static readonly runtime: Revault;
  /** Returns the agent. */
  readonly agent: Agent;
  /** Returns the platform. */
  readonly platform: Platform;
  /** Returns the last error. */
  lastError(): string;
  /** Returns the last error details. */
  lastErrorDetails(): import('./domain.js').ErrorDetails;
  /** Returns the newest Lockbox archive format version supported by this engine. */
  lockboxFormatVersion(): number;
  /** Reads the format version from serialized Lockbox bytes without opening them. */
  lockboxProbeFormatVersion(bytes: BinaryInput): number;
  /** Creates an in memory Lockbox protected by a 32 byte content key. */
  lockboxCreate(key: BinaryInput): Lockbox;
  /**
   * Creates a lockbox with explicit runtime tuning.
   * `cacheMode` selects the cache strategy, `cacheBytes` its capacity,
   * `workload` the workload profile, `worker` the worker policy, and zero
   * `jobs` lets the library select the worker count.
   */
  lockboxCreateWithOptions(key: BinaryInput, cacheMode: string, cacheBytes: number, workload: string, worker: string, jobs: number): Lockbox;
  /** Creates an in memory Lockbox protected by the supplied password. */
  lockboxCreatePassword(password: BinaryInput): Lockbox;
  /** Creates an in memory Lockbox that the supplied contact can open. */
  lockboxCreateContact(contact: NativeHandle): Lockbox;
  /** Creates an in memory Lockbox and assigns its profile signing key. */
  lockboxCreateWithSigningKey(contentKey: BinaryInput, signingKey: NativeHandle): Lockbox;
  /** Opens serialized Lockbox bytes with a 32 byte content key. */
  lockboxOpen(archive: BinaryInput, key: BinaryInput): Lockbox;
  /** Opens a lockbox using the cache and worker tuning described by `lockboxCreateWithOptions`. */
  lockboxOpenWithOptions(archive: BinaryInput, key: BinaryInput, cacheMode: string, cacheBytes: number, workload: string, worker: string, jobs: number): Lockbox;
  /** Opens serialized Lockbox bytes with the supplied password. */
  lockboxOpenPassword(archive: BinaryInput, password: BinaryInput): Lockbox;
  /** Opens serialized Lockbox bytes with the supplied contact private key. */
  lockboxOpenContact(archive: BinaryInput, contact: NativeHandle): Lockbox;
  /** Reads public header, signature, and access slot metadata from a Lockbox file. */
  lockboxInspectFile(path: string): import('./domain.js').FileInspection;
  /** Scans a damaged Lockbox file with its 32 byte content key. */
  lockboxRecoveryScanPath(path: string, key: BinaryInput): import('./domain.js').RecoveryReport;
  /** Scans damaged serialized Lockbox bytes with their 32 byte content key. */
  lockboxRecoveryScan(bytes: BinaryInput, key: BinaryInput): import('./domain.js').RecoveryReport;
  /** Builds a new Lockbox from recoverable records without changing the source. */
  lockboxRecoverySalvage(bytes: BinaryInput, key: BinaryInput, signingKey: NativeHandle): Lockbox;
  /** Generates a contact encryption key pair using secure random data. */
  keyContactGenerate(): ContactKeyPair;
  /** Imports a contact key pair from its private binary record. */
  keyContactFromPrivate(bytes: BinaryInput): ContactKeyPair;
  /** Imports a contact public key from its binary representation. */
  keyContactPublicFromBytes(bytes: BinaryInput): ContactPublicKey;
  /** Generates a signing identity owned by a Vault Profile. */
  generateProfileSigningKeyPair(): ProfileSigningKeyPair;
  /** Imports a Vault Profile signing identity from its private record. */
  profileSigningKeyPairFromPrivate(bytes: BinaryInput): ProfileSigningKeyPair;
  /** Imports the public half of a Vault Profile signing identity. */
  profileSigningPublicKeyFromBytes(bytes: BinaryInput): ProfileSigningPublicKey;
  /** Exports a private key in the requested key format. */
  vaultKeyExportPrivate(key: NativeHandle, format: string): Binary;
  /** Exports a public key in the requested key format. */
  vaultKeyExportPublic(key: NativeHandle, format: string): Binary;
  /** Imports a private contact key from a detected supported encoding. */
  vaultKeyImportPrivate(bytes: BinaryInput): ContactKeyPair;
  /** Imports a public contact key from a detected supported encoding. */
  vaultKeyImportPublic(bytes: BinaryInput): ContactPublicKey;
  /** Returns the stable fingerprint used to verify a public key. */
  vaultKeyFingerprint(key: NativeHandle): Binary;
  /** Encodes key bytes as hexadecimal text. */
  vaultKeyFormatHex(bytes: BinaryInput): string;
  /** Decodes hexadecimal key text and rejects malformed input. */
  vaultKeyDecodeHex(text: string): Binary;
  /** Encodes key bytes using Crockford Base32. */
  vaultKeyFormatCrockford(bytes: BinaryInput): string;
  /** Groups a Crockford code for easier reading and transcription. */
  vaultKeyFormatCrockfordReading(code: string): string;
  /** Decodes Crockford Base32 key text and rejects malformed input. */
  vaultKeyDecodeCrockford(code: string): Binary;
  /** Encodes arbitrary bytes as hexadecimal text. */
  vaultKeyHexEncode(bytes: BinaryInput): string;
  /** Decodes arbitrary hexadecimal text and rejects malformed input. */
  vaultKeyHexDecode(text: string): Binary;
  /** Opens an existing Vault directory with its passphrase. */
  vaultDirectoryOpen(root: string, password: BinaryInput): Vault;
  /** Returns the newest Vault structure version supported by this engine. */
  vaultStructureVersionCurrent(): number;
  /** Reads an existing Vault structure version without changing it. */
  vaultDirectoryProbeStructureVersion(root: string, password: BinaryInput): number;
  /** Opens or creates the default Vault without replacing existing state. */
  vaultDirectoryOpenOrCreateDefault(password: BinaryInput): Vault;
  /** Replaces the default Vault and all persistent data it contains. */
  vaultDirectoryReplaceDefault(password: BinaryInput): Vault;
  /** Changes the passphrase for an existing Vault. */
  vaultDirectoryChangePassword(root: string, oldPassword: BinaryInput, newPassword: BinaryInput): boolean;
  /** Changes the passphrase for the default Vault. */
  vaultDirectoryChangeDefaultPassword(oldPassword: BinaryInput, newPassword: BinaryInput): boolean;
  /** Replaces the selected Vault and all persistent data it contains. */
  vaultDirectoryReplace(root: string, password: BinaryInput): Vault;
  /** Opens the selected Vault, creating it only when absent. */
  vaultDirectoryOpenOrCreate(root: string, password: BinaryInput): Vault;
  /** Writes a backup of the default Vault to the selected path. */
  vaultBackupDefault(path: string, overwrite: boolean): import('./domain.js').VaultBackupManifest;
  /** Restores the default Vault from the selected backup. */
  vaultRestoreDefault(path: string, overwrite: boolean): import('./domain.js').VaultBackupManifest;
  /** Opens an existing Vault metadata view that cannot load private keys. */
  vaultReadOnlyOpen(root: string, password: BinaryInput): ReadOnlyVault;
  /** Opens the default Vault metadata view without loading private keys. */
  vaultReadOnlyOpenDefault(password: BinaryInput): ReadOnlyVault;
  /** Returns the platform default Vault directory. */
  vaultDefaultDirectory(): string;
  /** Returns the path of the default Vault file. */
  vaultDefaultPath(): string;
  /** Returns the session agent log path. */
  vaultAgentLogPath(): string;
  /** Returns the configured session agent log destination. */
  vaultAgentLogDestination(): string;
  /** Returns the vault local. */
}

/** An open encrypted archive containing files, variables, secrets, and forms.
 * Obtain one from {@link Revault} or {@link AgentSession}, commit pending changes,
 * and call {@link free} when finished with its decrypted contents. */
export class Lockbox {
  /** Create an in-memory archive protected by one password, key, or contact. */
  static createInMemory(options?: { password?: BinaryInput; contentKey?: BinaryInput; contact?: NativeHandle; signingKey?: NativeHandle; options?: object }): Lockbox;
  /** Open serialized archive bytes with one password, key, or contact. */
  static openBytes(archive: BinaryInput, options?: { password?: BinaryInput; contentKey?: BinaryInput; contact?: NativeHandle; options?: object }): Lockbox;
  /** Create a host archive file and return its owned handle. */
  static create(path: string, options?: { password?: BinaryInput; contentKey?: BinaryInput; contact?: NativeHandle; signingKey?: NativeHandle; options?: object; overwrite?: boolean }): Lockbox;
  /** Open a host archive file without consulting the Session Agent. */
  static open(path: string, options?: { password?: BinaryInput; contentKey?: BinaryInput; contact?: NativeHandle; options?: object }): Lockbox;
  /** Release the process-local content key; repeated calls are safe. */
  /** Release this owned handle and wipe any native secret state. */
  close(): void;
  /** Stages a file at the Lockbox path; replace controls an existing entry. */
  addFile(path: string, data: BinaryInput, replace: boolean): boolean;
  /** Stages a file and its portable Unix permission bits. */
  addFileWithPermissions(path: string, data: BinaryInput, permissions: number, replace: boolean): boolean;
  /** Reads the complete file stored at the Lockbox path. */
  getFile(path: string): Binary;
  /** Writes one Lockbox file to the host filesystem. */
  extractFile(source: string, destination: string, replace: boolean): boolean;
  /** Extracts the Lockbox with explicit size, count, link, and permission limits. */
  extractDirectory(destination: string, maxFileBytes: number, maxTotalBytes: number, maxFiles: number, restoreSymlinks: boolean, restorePermissions: boolean, overwrite: boolean): boolean;
  /** Lists logical or physical content chunks for streaming diagnostics. */
  streamContent(physical: boolean): ReadonlyArray<import('./domain.js').StreamChunk>;
  /** Returns cache statistics for this lockbox. */
  cacheStats(): import('./domain.js').CacheStats;
  /** Returns import statistics for this lockbox. */
  importStats(): import('./domain.js').ImportStats;
  /** Updates import stats. */
  resetImportStats(): boolean;
  /** Returns page metadata for diagnostics without exposing plaintext secrets. */
  pageInspection(): ReadonlyArray<import('./domain.js').PageInspection>;
  /** Scans the open archive and returns its structured recovery report. */
  recoveryReport(): import('./domain.js').RecoveryReport;
  /** Renders the recovery report for a person, capped at maxEntries. */
  recoveryReportRender(verbose: boolean, maxEntries: number): string;
  /** Returns the current serialized archive size in bytes. */
  storageLen(): number;
  /** Sets workload profile. */
  setWorkloadProfile(profile: string): boolean;
  /** Sets worker policy. */
  setWorkerPolicy(mode: string, jobs: number): boolean;
  /** Returns the cache, workload, and worker settings used by this Lockbox. */
  runtimeOptions(): import('./domain.js').RuntimeOptions;
  /** Authenticates and publishes the staged changes. */
  commit(): boolean;
  /** Stages a directory entry and optionally creates missing parents. */
  createDir(path: string, createParents: boolean): boolean;
  /** Stages removal of a file, link, or empty directory at path. */
  delete(path: string): boolean;
  /** Stages removal of a directory, optionally including its descendants. */
  removeDir(path: string, recursive: boolean): boolean;
  /** Stages every missing parent directory for path. */
  createParentDirs(path: string): boolean;
  /** Stages an atomic move from one Lockbox path to another. */
  rename(from: string, to: string): boolean;
  /** Lists entries below path, optionally including descendants. */
  list(path: string, recursive: boolean): ReadonlyArray<import('./domain.js').LockboxEntry>;
  /** Lists entries using glob, type, recursion, and result limit filters. */
  listWithOptions(path: string, glob: string, recursive: boolean, includeFiles: boolean, includeSymlinks: boolean, includeDirectories: boolean, limit: number): ReadonlyArray<import('./domain.js').LockboxEntry>;
  /** Returns metadata for the selected lockbox entry. */
  stat(path: string): import('./domain.js').LockboxEntry | undefined;
  /** Stages a plain text variable; commit to publish the change. */
  setVariable(name: string, value: string): boolean;
  /** Stores a secret value without first converting it to a JavaScript string. */
  setSecretVariable(name: string, value: BinaryInput): boolean;
  /** Returns a plain variable when it is present. */
  getVariable(name: string): string | undefined;
  /** Encrypted Lockbox description, or undefined when unset. Example lifecycle: set it, commit, then print this property. */
  readonly description: string | undefined;
  /** Stages encrypted description text; call commit to publish it. Example lifecycle: set "Production credentials", then commit the box. */
  setDescription(description: string): boolean;
  /** Stages removal of the encrypted description; call commit. Example lifecycle: clear it, then commit the box. */
  clearDescription(): boolean;
  /**
   * Invokes `callback` with temporary secret bytes and overwrites the native
   * transfer buffer immediately afterwards. Do not retain plaintext unless the
   * resulting security tradeoff is intentional.
   */
  withSecretVariable<T>(name: string, callback: (value: Uint8Array) => T): T | undefined;
  /** Stages removal of a variable. */
  deleteVariable(name: string): boolean;
  /** Updates variables. */
  moveVariables(moves: ReadonlyArray<import('./domain.js').PathMoveInput>): boolean;
  /** Lists variable names and metadata without exposing secret values. */
  listVariables(): ReadonlyArray<import('./domain.js').Variable>;
  /** Returns whether a variable is plain or secret. */
  variableSensitivity(name: string): string | undefined;
  /** Stages a symbolic link with its stored target text. */
  addSymlink(path: string, target: string, replace: boolean): boolean;
  /** Returns the target text stored for a symbolic link. */
  getSymlinkTarget(path: string): string;
  /** Returns the stable public identifier stored in the Lockbox header. */
  id(): Binary;
  /** Reports whether an entry exists at path. */
  exists(path: string): boolean;
  /** Reports whether path names a directory entry. */
  isDir(path: string): boolean;
  /** Returns the portable Unix permission bits stored for path. */
  permissions(path: string): number;
  /** Stages portable Unix permission bits for path. */
  setPermissions(path: string, permissions: number): boolean;
  /** Reads the requested byte range from a stored file. */
  readRange(path: string, offset: number, len: number): Binary;
  /** Adds a password access slot and returns its slot identifier. */
  addPassword(password: BinaryInput): number;
  /** Grants a named contact access and returns the new slot identifier. */
  addContact(contact: NativeHandle, name: string): number;
  /** Removes an access slot; at least one usable slot must remain. */
  deleteKey(id: number): boolean;
  /** Lists public access slot metadata without returning credentials. */
  listKeySlots(): ReadonlyArray<import('./domain.js').KeySlot>;
  /** Assigns a profile signing key to the Lockbox owner role. */
  setOwnerSigningKey(key: NativeHandle): boolean;
  /** Returns public signing and ownership metadata for the current revision. */
  ownerInspection(): import('./domain.js').OwnerInspection;
  /** Defines and stores a reusable versioned form. */
  defineForm(alias: string, name: string, description: string, fields: ReadonlyArray<import('./domain.js').FormFieldInput>): import('./domain.js').FormDefinition;
  /** Lists the form definitions stored in this Lockbox. */
  listFormDefinitions(): ReadonlyArray<import('./domain.js').FormDefinition>;
  /** Resolves a form alias, type identifier, or revision. */
  resolveForm(reference: string): import('./domain.js').FormDefinition;
  /** Lists every stored revision for a form type identifier. */
  listFormRevisions(typeId: string): ReadonlyArray<import('./domain.js').FormDefinition>;
  /** Stages a form record at path using the referenced definition. */
  createFormRecord(path: string, typeReference: string, name: string): import('./domain.js').FormRecord;
  /** Stages a plain field value in a form record. */
  setFormField(path: string, field: string, value: string): boolean;
  /** Stores a secret form field from bytes without creating an immutable string. */
  setSecretFormField(path: string, field: string, value: BinaryInput): boolean;
  /** Lists form records without exposing secret field values. */
  listFormRecords(): ReadonlyArray<import('./domain.js').FormRecord>;
  /** Returns the form record at path when present. */
  getFormRecord(path: string): import('./domain.js').FormRecord | undefined;
  /** Stages removal of a form record. */
  deleteFormRecord(path: string): boolean;
  /** Updates form records. */
  moveFormRecords(moves: ReadonlyArray<import('./domain.js').PathMoveInput>): boolean;
  /** Returns a plain form field when it exists. */
  getFormField(path: string, field: string): import('./domain.js').FormValue | undefined;
  /** Calls `callback` with temporary secret field bytes, then overwrites the transfer buffer. */
  withSecretFormField<T>(path: string, field: string, callback: (value: Uint8Array) => T): T | undefined;
  /** Serializes the current Lockbox, including committed changes. */
  toBytes(): Binary;
  /** Releases the native resources held by this object. */
  free(): void;
}

/** A profile's contact-encryption identity. Distribute its public half and
 * retain the private half to decrypt content keys addressed to the profile. */
export class ContactKeyPair {
  /** Returns the public. */
  public(): Binary;
  /** Returns the private. */
  private(): Binary;
  /** Releases the native resources held by this object. */
  free(): void;
  /** Release this owned handle and wipe any native secret state. */
  close(): void;
  /** Decrypts a wrapped content key for this contact. */
  decrypt(wrapped: NativeHandle): Binary;
}

/** A recipient's shareable encryption identity. Use it when granting that
 * recipient lockbox access; it contains no private key material. */
export class ContactPublicKey {
  /** Releases this public contact key. */
  publicFree(): void;
  /** Release this owned handle and wipe any native secret state. */
  close(): void;
  /** Encrypts a content key for the selected contact. */
  encrypt(contentKey: BinaryInput): WrappedContactKey;
}

/** A content key encrypted for one contact. Store or transfer it with an access
 * record; only the matching {@link ContactKeyPair} can recover the key. */
export class WrappedContactKey {
  /** Returns the public. */
  public(): Binary;
  /** Returns the encrypted content key bytes. */
  ciphertext(): Binary;
  /** Returns the complete wrapped key record for storage or transport. */
  encrypted(): Binary;
  /** Releases the native resources held by this object. */
  free(): void;
  /** Release this owned handle and wipe any native secret state. */
  close(): void;
}

/** A Vault Profile signing identity used to authorize mutable Lockbox revisions. */
export class ProfileSigningKeyPair {
  /** Returns the canonical public bytes paired with this identity. */
  publicBytes(): Binary;
  /** Returns the private signing-key record for secure binary backup. */
  privateRecord(): Binary;
  /** Creates an independently owned public verification-key handle. */
  publicKey(): ProfileSigningPublicKey;
  /** Wipes and releases the native signing-key handle. */
  dispose(): void;
}

/** The shareable half of a Vault Profile signing identity. */
export class ProfileSigningPublicKey {
  /** Releases the native verification-key handle. */
  dispose(): void;
}

/** A writable, password-protected store for Profile keys, contacts, forms,
 * backups, and remembered lockbox paths. Lockbox contents remain separate. */
export class Vault {
  /** Open an existing Vault; does not create or replace it. */
  static open(root: string, vaultPassphrase: BinaryInput): Vault;
  /** Open or create a Vault explicitly. */
  static openOrCreate(root: string, vaultPassphrase: BinaryInput): Vault;
  /** Create a Vault at a new path. */
  static create(root: string, vaultPassphrase: BinaryInput): Vault;
  /** Replace a Vault explicitly; destructive. */
  static replace(root: string, vaultPassphrase: BinaryInput): Vault;
  /** Returns the canonical root directory of this Vault. */
  root(): string;
  /** Returns the persistent structure version of this Vault. */
  structureVersion(): number;
  /** Lists private keys. */
  listPrivateKeys(): ReadonlyArray<string>;
  /** Lists private key names. */
  listPrivateKeyNames(): ReadonlyArray<string>;
  /** Lists contact names. */
  listContactNames(): ReadonlyArray<string>;
  /** Lists form aliases. */
  listFormAliases(): ReadonlyArray<string>;
  /** Reports whether the named profile private key exists. */
  privateKeyExists(name: string): boolean;
  /** Removes private key. */
  deletePrivateKey(name: string): boolean;
  /** Stores private key. */
  storePrivateKey(name: string, key: NativeHandle): boolean;
  /** Loads private key. */
  loadPrivateKey(name: string): ContactKeyPair;
  /** Loads private key generation. */
  loadPrivateKeyGeneration(name: string, index: number): ContactKeyPair;
  /** Stores contact. */
  storeContact(name: string, key: NativeHandle): boolean;
  /** Loads contact. */
  loadContact(name: string): ContactPublicKey;
  /** Reports whether the named contact exists. */
  contactExists(name: string): boolean;
  /** Removes contact. */
  deleteContact(name: string): boolean;
  /** Lists contacts. */
  listContacts(): ReadonlyArray<import('./domain.js').Contact>;
  /** Stores profile email. */
  storeProfileEmail(name: string, email: string): boolean;
  /** Returns the email recorded for a profile, when present. */
  profileEmail(name: string): string | undefined;
  /** Stores backup. */
  storeBackup(id: BinaryInput, bytes: BinaryInput): boolean;
  /** Loads backup. */
  loadBackup(id: BinaryInput): Binary;
  /** Returns the number of stored key recovery backups. */
  backupCount(): number;
  /** Restores a profile private key and signing key from recovery material. */
  restorePrivateKey(name: string, key: NativeHandle, signingKey: NativeHandle, overwrite: boolean): boolean;
  /** Loads owner signing key. */
  loadProfileSigningKey(name: string): ProfileSigningKeyPair;
  /** Loads owner signing key generation. */
  loadProfileSigningKeyGeneration(name: string, index: number): ProfileSigningKeyPair;
  /** Stores contact signing key. */
  storeContactSigningKey(name: string, key: NativeHandle): boolean;
  /** Loads contact signing key. */
  loadContactSigningKey(name: string): ProfileSigningPublicKey;
  /** Lists profile generations. */
  listProfileGenerations(name: string): import('./domain.js').ProfileHistory;
  /** Updates private key. */
  rotatePrivateKey(name: string): import('./domain.js').ProfileHistory;
  /** Stores lockbox. */
  rememberLockbox(id: BinaryInput, path: string): boolean;
  /** Lists known lockboxes. */
  listKnownLockboxes(): ReadonlyArray<import('./domain.js').KnownLockbox>;
  /** Removes lockbox. */
  forgetLockbox(path: string): boolean;
  /** Stores access slot label. */
  rememberAccessSlotLabel(id: BinaryInput, slotId: number, name: string): boolean;
  /** Lists access slot labels. */
  listAccessSlotLabels(id: BinaryInput): ReadonlyArray<import('./domain.js').AccessSlotLabel>;
  /** Finds access slot labels with the supplied name for one Lockbox. */
  findAccessSlotLabels(id: BinaryInput, name: string): ReadonlyArray<import('./domain.js').AccessSlotLabel>;
  /** Removes access slot label. */
  forgetAccessSlotLabel(id: BinaryInput, slotId: number): boolean;
  /** Defines and stores a reusable versioned form. */
  defineForm(alias: string, name: string, description: string, fields: ReadonlyArray<import('./domain.js').FormFieldInput>): import('./domain.js').FormDefinition;
  /** Resolves a form alias, type identifier, or revision. */
  resolveForm(reference: string): import('./domain.js').FormDefinition;
  /** Lists forms. */
  listForms(): ReadonlyArray<import('./domain.js').FormDefinition>;
  /** Lists every stored revision for a form type identifier. */
  listFormRevisions(typeId: string): ReadonlyArray<import('./domain.js').FormDefinition>;
  /** Adds missing standard form definitions and returns the number added. */
  seedForms(): number;
  /** Stores password. */
  rememberPassword(id: BinaryInput, password: BinaryInput): boolean;
  /** Returns the Lockbox password encrypted inside this Vault. */
  rememberedPassword(id: BinaryInput): Binary;
  /** Releases the native resources held by this object. */
  free(): void;
  /** Release this owned handle and wipe any native secret state. */
  close(): void;
}

/** A restricted metadata view for discovery or diagnostics that lists local
 * profiles, contacts, forms, and lockboxes without loading owner signing keys. */
export class ReadOnlyVault {
  /** Lists profile names. */
  listProfileNames(): ReadonlyArray<string>;
  /** Lists contact names. */
  listContactNames(): ReadonlyArray<string>;
  /** Lists form aliases. */
  listFormAliases(): ReadonlyArray<string>;
  /** Lists known lockboxes. */
  listKnownLockboxes(): ReadonlyArray<import('./domain.js').KnownLockbox>;
  /** Releases the native resources held by this object. */
  free(): void;
  /** Release this owned handle and wipe any native secret state. */
  close(): void;
}

/** Client for the local session service that temporarily caches vault unlock
 * and owner signing keys across application operations. */
declare class Agent {
  /** Reports whether running. */
  isRunning(): boolean;
  /** Removes all. */
  forgetAll(): boolean;
  /** Runs the session agent server until it is stopped. */
  serve(): boolean;
  /** Verifies transport. */
  verifyTransport(): boolean;
  /** Returns get. */
  get(id: BinaryInput): Binary;
  /** Stores put. */
  put(id: BinaryInput, key: BinaryInput): boolean;
  /** Removes forget. */
  forget(id: BinaryInput): boolean;
  /** Stops stop. */
  stop(): boolean;
  /** Starts start. */
  start(): boolean;
  /** Lists entries below path, optionally including descendants. */
  list(): ReadonlyArray<import('./domain.js').AgentEntry>;
  /** Reports how the platform handles agent expiry during system sleep. */
  sleepSupport(): import('./domain.js').SleepSupport;
  /** Returns vault unlock key. */
  getVaultUnlockKey(vaultId: string): Binary;
  /** Stores vault unlock key. */
  putVaultUnlockKey(vaultId: string, key: BinaryInput, ttlSeconds: number): boolean;
  /** Removes vault unlock key. */
  forgetVaultUnlockKey(vaultId: string): boolean;
  /** Returns the cached signing identity for a Vault Profile. */
  profileSigningKey(vaultId: string, profile: string): ProfileSigningKeyPair;
  /** Caches a signing identity for a Vault Profile. */
  cacheProfileSigningKey(vaultId: string, profile: string, key: NativeHandle, ttlSeconds: number): boolean;
  /** Removes a cached Vault Profile signing identity. */
  forgetProfileSigningKey(vaultId: string, profile: string): boolean;
  /** Starts activity. */
  beginActivity(kind: string): AgentActivity;
  /** Stops activity. */
  endActivity(handle: NativeHandle): void;
}
/** Explicit controller for the optional Session Agent. */
export class AgentSession extends Agent {
  static readonly instance: AgentSession;
  /** Remove one cached lockbox key from this explicit session. */
  closeLockbox(lockboxPath: string): boolean;
  /** Remove all lockbox keys cached by this explicit session. */
  closeAll(): boolean;
  /** Create a password-protected lockbox file through this session. */
  createLockboxPassword(path: string, password: BinaryInput): Lockbox;
  /** Open a password-protected lockbox file through this session. */
  openLockboxPassword(path: string, password: BinaryInput): Lockbox;
  /** Create a lockbox file protected by an explicit content key. */
  createLockboxContentKey(path: string, contentKey: BinaryInput, signingKey: NativeHandle): Lockbox;
  /** Create a lockbox file addressed to a contact. */
  createLockboxContact(path: string, contact: NativeHandle, name: string, signingKey: NativeHandle): Lockbox;
  /** Open a content-key lockbox file through this session. */
  openLockboxContentKey(path: string, contentKey: BinaryInput, signingKey: NativeHandle): Lockbox;
  /** Cache a password-derived key for the requested number of seconds. */
  cacheLockboxPassword(path: string, password: BinaryInput, ttlSeconds: number): boolean;
  /** Release the local session handle. */
  free(): void;
}
/** Profile-oriented names for the signing identities used by persistent Vault records. */

/** A lifetime token kept alive while an operation needs cached secrets. Release
 * it afterward so the Session Agent can expire unused secrets. */
export class AgentActivity {
  /** Release this owned handle and wipe any native secret state. */
  close(): void;
}

/** Access to the platform credential store for a scoped Vault passphrase. */
declare class Platform {
  /** Returns availability and user presence guarantees for platform storage. */
  status(): import('./domain.js').PlatformStatus;
  /** Sets scope. */
  setScope(scope: string): boolean;
  /** Removes password. */
  forgetPassword(): boolean;
  /** Stores password. */
  putPassword(password: BinaryInput): boolean;
  /** Enables storage of the Vault passphrase in platform credentials. */
  enable(): boolean;
  /** Disables platform credential use without deleting the stored value. */
  disable(): boolean;
  /** Reports whether platform credential use is disabled. */
  disabled(): boolean;
  /** Returns password. */
  getPassword(): Binary;
}

/** A session for creating or opening lockboxes by host path, caching short-lived
 * passwords, and committing and closing the files used by a local application. */
declare class LocalVault {
  /** Creates Lockbox password. */
  createLockboxPassword(path: string, password: BinaryInput): Lockbox;
  /** Opens Lockbox password. */
  openLockboxPassword(path: string, password: BinaryInput): Lockbox;
  /** Creates lockbox content key. */
  createLockboxContentKey(path: string, contentKey: BinaryInput, signingKey: NativeHandle): Lockbox;
  /** Creates lockbox contact. */
  createLockboxContact(path: string, contact: NativeHandle, name: string, signingKey: NativeHandle): Lockbox;
  /** Opens lockbox content key. */
  openLockboxContentKey(path: string, contentKey: BinaryInput, signingKey: NativeHandle): Lockbox;
  /** Stores Lockbox password. */
  cacheLockboxPassword(path: string, password: BinaryInput, ttlSeconds: number): boolean;
  /** Releases the native resources held by lockbox. */
  closeLockbox(path: string): boolean;
  /** Releases the native resources held by all. */
  closeAll(): boolean;
  /** Releases the native resources held by this object. */
  free(): void;
}
