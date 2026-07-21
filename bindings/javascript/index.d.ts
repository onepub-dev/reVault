/**
 * Encrypt files, variables, and typed form records in portable reVault
 * lockboxes, and manage keys and local vault metadata.
 *
 * Start with {@link Vault}. Call `free()` on owned handles and use the
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
/** Returns the native handle. */
export type NativeHandle = ContactKeyPair | ContactPublicKey | WrappedContactKey | SigningKeyPair | SigningPublicKey | VaultDirectory | ReadOnlyVaultDirectory | AgentActivity | LocalVault;
/** Primary API used to open lockboxes, manage keys and metadata, use the
 * session agent, and access operating-system credential storage. Create one
 * when the application starts. */
export class Vault {
  /** Creates a new facade over the bundled native library. */
  constructor();
  /** Returns the agent. */
  readonly agent: Agent;
  /** Returns the platform. */
  readonly platform: Platform;
  /** Returns the last error. */
  lastError(): string;
  /** Returns the last error details. */
  lastErrorDetails(): import('./domain.js').ErrorDetails;
  /** Returns the lockbox format version. */
  lockboxFormatVersion(): number;
  /** Returns the lockbox probe format version. */
  lockboxProbeFormatVersion(bytes: BinaryInput): number;
  /** Returns the lockbox create. */
  lockboxCreate(key: BinaryInput): Lockbox;
  /**
   * Creates a lockbox with explicit runtime tuning.
   * `cacheMode` selects the cache strategy, `cacheBytes` its capacity,
   * `workload` the workload profile, `worker` the worker policy, and zero
   * `jobs` lets the library select the worker count.
   */
  lockboxCreateWithOptions(key: BinaryInput, cacheMode: string, cacheBytes: number, workload: string, worker: string, jobs: number): Lockbox;
  /** Returns the lockbox create password. */
  lockboxCreatePassword(password: BinaryInput): Lockbox;
  /** Returns the lockbox create contact. */
  lockboxCreateContact(contact: NativeHandle): Lockbox;
  /** Returns the lockbox create with signing key. */
  lockboxCreateWithSigningKey(contentKey: BinaryInput, signingKey: NativeHandle): Lockbox;
  /** Returns the lockbox open. */
  lockboxOpen(archive: BinaryInput, key: BinaryInput): Lockbox;
  /** Opens a lockbox using the cache and worker tuning described by `lockboxCreateWithOptions`. */
  lockboxOpenWithOptions(archive: BinaryInput, key: BinaryInput, cacheMode: string, cacheBytes: number, workload: string, worker: string, jobs: number): Lockbox;
  /** Returns the lockbox open password. */
  lockboxOpenPassword(archive: BinaryInput, password: BinaryInput): Lockbox;
  /** Returns the lockbox open contact. */
  lockboxOpenContact(archive: BinaryInput, contact: NativeHandle): Lockbox;
  /** Returns the lockbox inspect file. */
  lockboxInspectFile(path: string): import('./domain.js').FileInspection;
  /** Returns the lockbox recovery scan path. */
  lockboxRecoveryScanPath(path: string, key: BinaryInput): import('./domain.js').RecoveryReport;
  /** Returns the lockbox recovery scan. */
  lockboxRecoveryScan(bytes: BinaryInput, key: BinaryInput): import('./domain.js').RecoveryReport;
  /** Returns the lockbox recovery salvage. */
  lockboxRecoverySalvage(bytes: BinaryInput, key: BinaryInput, signingKey: NativeHandle): Lockbox;
  /** Returns the key contact generate. */
  keyContactGenerate(): ContactKeyPair;
  /** Returns the key contact from private. */
  keyContactFromPrivate(bytes: BinaryInput): ContactKeyPair;
  /** Returns the key contact public from bytes. */
  keyContactPublicFromBytes(bytes: BinaryInput): ContactPublicKey;
  /** Returns the key signing generate. */
  keySigningGenerate(): SigningKeyPair;
  /** Returns the key signing from private. */
  keySigningFromPrivate(bytes: BinaryInput): SigningKeyPair;
  /** Returns the key signing public from bytes. */
  keySigningPublicFromBytes(bytes: BinaryInput): SigningPublicKey;
  /** Returns the vault key export private. */
  vaultKeyExportPrivate(key: NativeHandle, format: string): Binary;
  /** Returns the vault key export public. */
  vaultKeyExportPublic(key: NativeHandle, format: string): Binary;
  /** Returns the vault key import private. */
  vaultKeyImportPrivate(bytes: BinaryInput): ContactKeyPair;
  /** Returns the vault key import public. */
  vaultKeyImportPublic(bytes: BinaryInput): ContactPublicKey;
  /** Returns the vault key fingerprint. */
  vaultKeyFingerprint(key: NativeHandle): Binary;
  /** Returns the vault key format hex. */
  vaultKeyFormatHex(bytes: BinaryInput): string;
  /** Returns the vault key decode hex. */
  vaultKeyDecodeHex(text: string): Binary;
  /** Returns the vault key format crockford. */
  vaultKeyFormatCrockford(bytes: BinaryInput): string;
  /** Returns the vault key format crockford reading. */
  vaultKeyFormatCrockfordReading(code: string): string;
  /** Returns the vault key decode crockford. */
  vaultKeyDecodeCrockford(code: string): Binary;
  /** Returns the vault key hex encode. */
  vaultKeyHexEncode(bytes: BinaryInput): string;
  /** Returns the vault key hex decode. */
  vaultKeyHexDecode(text: string): Binary;
  /** Returns the vault directory open. */
  vaultDirectoryOpen(root: string, password: BinaryInput): VaultDirectory;
  /** Returns the vault structure version current. */
  vaultStructureVersionCurrent(): number;
  /** Returns the vault directory probe structure version. */
  vaultDirectoryProbeStructureVersion(root: string, password: BinaryInput): number;
  /** Returns the vault directory open or create default. */
  vaultDirectoryOpenOrCreateDefault(password: BinaryInput): VaultDirectory;
  /** Returns the vault directory replace default. */
  vaultDirectoryReplaceDefault(password: BinaryInput): VaultDirectory;
  /** Returns the vault directory change password. */
  vaultDirectoryChangePassword(root: string, oldPassword: BinaryInput, newPassword: BinaryInput): boolean;
  /** Returns the vault directory change default password. */
  vaultDirectoryChangeDefaultPassword(oldPassword: BinaryInput, newPassword: BinaryInput): boolean;
  /** Returns the vault directory replace. */
  vaultDirectoryReplace(root: string, password: BinaryInput): VaultDirectory;
  /** Returns the vault directory open or create. */
  vaultDirectoryOpenOrCreate(root: string, password: BinaryInput): VaultDirectory;
  /** Returns the vault backup default. */
  vaultBackupDefault(path: string, overwrite: boolean): import('./domain.js').VaultBackupManifest;
  /** Returns the vault restore default. */
  vaultRestoreDefault(path: string, overwrite: boolean): import('./domain.js').VaultBackupManifest;
  /** Returns the vault read only open. */
  vaultReadOnlyOpen(root: string, password: BinaryInput): ReadOnlyVaultDirectory;
  /** Returns the vault read only open default. */
  vaultReadOnlyOpenDefault(password: BinaryInput): ReadOnlyVaultDirectory;
  /** Returns the vault default directory. */
  vaultDefaultDirectory(): string;
  /** Returns the vault default path. */
  vaultDefaultPath(): string;
  /** Returns the vault agent log path. */
  vaultAgentLogPath(): string;
  /** Returns the vault agent log destination. */
  vaultAgentLogDestination(): string;
  /** Returns the vault local. */
  vaultLocal(): LocalVault;
}

/** An open encrypted archive containing files, variables, secrets, and forms.
 * Obtain one from {@link Vault} or {@link LocalVault}, commit pending changes,
 * and call {@link free} when finished with its decrypted contents. */
export class Lockbox {
  /** Adds file. */
  addFile(path: string, data: BinaryInput, replace: boolean): boolean;
  /** Adds file with permissions. */
  addFileWithPermissions(path: string, data: BinaryInput, permissions: number, replace: boolean): boolean;
  /** Returns file. */
  getFile(path: string): Binary;
  /** Extracts file. */
  extractFile(source: string, destination: string, replace: boolean): boolean;
  /** Extracts directory. */
  extractDirectory(destination: string, maxFileBytes: number, maxTotalBytes: number, maxFiles: number, restoreSymlinks: boolean, restorePermissions: boolean, overwrite: boolean): boolean;
  /** Returns the stream content. */
  streamContent(physical: boolean): ReadonlyArray<import('./domain.js').StreamChunk>;
  /** Returns cache statistics for this lockbox. */
  cacheStats(): import('./domain.js').CacheStats;
  /** Returns import statistics for this lockbox. */
  importStats(): import('./domain.js').ImportStats;
  /** Updates import stats. */
  resetImportStats(): boolean;
  /** Returns the page inspection. */
  pageInspection(): ReadonlyArray<import('./domain.js').PageInspection>;
  /** Returns the recovery report. */
  recoveryReport(): import('./domain.js').RecoveryReport;
  /** Returns the recovery report render. */
  recoveryReportRender(verbose: boolean, maxEntries: number): string;
  /** Returns the storage len. */
  storageLen(): number;
  /** Sets workload profile. */
  setWorkloadProfile(profile: string): boolean;
  /** Sets worker policy. */
  setWorkerPolicy(mode: string, jobs: number): boolean;
  /** Returns the runtime options. */
  runtimeOptions(): import('./domain.js').RuntimeOptions;
  /** Authenticates and publishes the staged changes. */
  commit(): boolean;
  /** Creates dir. */
  createDir(path: string, createParents: boolean): boolean;
  /** Removes delete. */
  delete(path: string): boolean;
  /** Removes dir. */
  removeDir(path: string, recursive: boolean): boolean;
  /** Creates parent dirs. */
  createParentDirs(path: string): boolean;
  /** Updates rename. */
  rename(from: string, to: string): boolean;
  /** Lists list. */
  list(path: string, recursive: boolean): ReadonlyArray<import('./domain.js').LockboxEntry>;
  /** Lists with options. */
  listWithOptions(path: string, glob: string, recursive: boolean, includeFiles: boolean, includeSymlinks: boolean, includeDirectories: boolean, limit: number): ReadonlyArray<import('./domain.js').LockboxEntry>;
  /** Returns metadata for the selected lockbox entry. */
  stat(path: string): import('./domain.js').LockboxEntry | undefined;
  /** Sets variable. */
  setVariable(name: string, value: string): boolean;
  /** Stores a secret value without first converting it to a JavaScript string. */
  setSecretVariable(name: string, value: BinaryInput): boolean;
  /** Returns variable. */
  getVariable(name: string): string | undefined;
  /**
   * Invokes `callback` with temporary secret bytes and overwrites the native
   * transfer buffer immediately afterwards. Do not retain plaintext unless the
   * resulting security tradeoff is intentional.
   */
  withSecretVariable<T>(name: string, callback: (value: Uint8Array) => T): T | undefined;
  /** Removes variable. */
  deleteVariable(name: string): boolean;
  /** Updates variables. */
  moveVariables(moves: ReadonlyArray<import('./domain.js').PathMoveInput>): boolean;
  /** Lists variables. */
  listVariables(): ReadonlyArray<import('./domain.js').Variable>;
  /** Returns the variable sensitivity. */
  variableSensitivity(name: string): string | undefined;
  /** Adds symlink. */
  addSymlink(path: string, target: string, replace: boolean): boolean;
  /** Returns symlink target. */
  getSymlinkTarget(path: string): string;
  /** Returns the id. */
  id(): Binary;
  /** Reports whether exists. */
  exists(path: string): boolean;
  /** Reports whether dir. */
  isDir(path: string): boolean;
  /** Returns the permissions. */
  permissions(path: string): number;
  /** Sets permissions. */
  setPermissions(path: string, permissions: number): boolean;
  /** Returns range. */
  readRange(path: string, offset: number, len: number): Binary;
  /** Adds password. */
  addPassword(password: BinaryInput): number;
  /** Adds contact. */
  addContact(contact: NativeHandle, name: string): number;
  /** Removes key. */
  deleteKey(id: number): boolean;
  /** Lists key slots. */
  listKeySlots(): ReadonlyArray<import('./domain.js').KeySlot>;
  /** Sets owner signing key. */
  setOwnerSigningKey(key: NativeHandle): boolean;
  /** Returns the owner inspection. */
  ownerInspection(): import('./domain.js').OwnerInspection;
  /** Returns the define form. */
  defineForm(alias: string, name: string, description: string, fields: ReadonlyArray<import('./domain.js').FormFieldInput>): import('./domain.js').FormDefinition;
  /** Lists form definitions. */
  listFormDefinitions(): ReadonlyArray<import('./domain.js').FormDefinition>;
  /** Returns the resolve form. */
  resolveForm(reference: string): import('./domain.js').FormDefinition;
  /** Lists form revisions. */
  listFormRevisions(typeId: string): ReadonlyArray<import('./domain.js').FormDefinition>;
  /** Creates form record. */
  createFormRecord(path: string, typeReference: string, name: string): import('./domain.js').FormRecord;
  /** Sets form field. */
  setFormField(path: string, field: string, value: string): boolean;
  /** Stores a secret form field from bytes without creating an immutable string. */
  setSecretFormField(path: string, field: string, value: BinaryInput): boolean;
  /** Lists form records. */
  listFormRecords(): ReadonlyArray<import('./domain.js').FormRecord>;
  /** Returns form record. */
  getFormRecord(path: string): import('./domain.js').FormRecord | undefined;
  /** Removes form record. */
  deleteFormRecord(path: string): boolean;
  /** Updates form records. */
  moveFormRecords(moves: ReadonlyArray<import('./domain.js').PathMoveInput>): boolean;
  /** Returns form field. */
  getFormField(path: string, field: string): import('./domain.js').FormValue | undefined;
  /** Calls `callback` with temporary secret field bytes, then overwrites the transfer buffer. */
  withSecretFormField<T>(path: string, field: string, callback: (value: Uint8Array) => T): T | undefined;
  /** Returns the to bytes. */
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
  /** Decrypts a wrapped content key for this contact. */
  decrypt(wrapped: NativeHandle): Binary;
}

/** A recipient's shareable encryption identity. Use it when granting that
 * recipient lockbox access; it contains no private key material. */
export class ContactPublicKey {
  /** Returns the public free. */
  publicFree(): void;
  /** Encrypts a content key for the selected contact. */
  encrypt(contentKey: BinaryInput): WrappedContactKey;
}

/** A content key encrypted for one contact. Store or transfer it with an access
 * record; only the matching {@link ContactKeyPair} can recover the key. */
export class WrappedContactKey {
  /** Returns the public. */
  public(): Binary;
  /** Returns the ciphertext. */
  ciphertext(): Binary;
  /** Returns the encrypted. */
  encrypted(): Binary;
  /** Releases the native resources held by this object. */
  free(): void;
}

/** A lockbox owner's signing identity. Supply it when creating or committing a
 * mutable lockbox so readers can authenticate revisions. */
export class SigningKeyPair {
  /** Returns the public. */
  public(): Binary;
  /** Returns the private. */
  private(): Binary;
  /** Releases the native resources held by this object. */
  free(): void;
}

/** The shareable half of a lockbox owner's signing identity, used by readers to
 * verify owner-authorized revisions. */
export class SigningPublicKey {
  /** Returns the public free. */
  publicFree(): void;
}

/** A writable, password-protected store for profile keys, contacts, forms,
 * backups, and remembered lockbox paths. Lockbox contents remain separate. */
export class VaultDirectory {
  /** Returns the root. */
  root(): string;
  /** Returns the structure version. */
  structureVersion(): number;
  /** Lists private keys. */
  listPrivateKeys(): ReadonlyArray<string>;
  /** Lists private key names. */
  listPrivateKeyNames(): ReadonlyArray<string>;
  /** Lists contact names. */
  listContactNames(): ReadonlyArray<string>;
  /** Lists form aliases. */
  listFormAliases(): ReadonlyArray<string>;
  /** Returns the private key exists. */
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
  /** Returns the contact exists. */
  contactExists(name: string): boolean;
  /** Removes contact. */
  deleteContact(name: string): boolean;
  /** Lists contacts. */
  listContacts(): ReadonlyArray<import('./domain.js').Contact>;
  /** Stores profile email. */
  storeProfileEmail(name: string, email: string): boolean;
  /** Returns the profile email. */
  profileEmail(name: string): string | undefined;
  /** Stores backup. */
  storeBackup(id: BinaryInput, bytes: BinaryInput): boolean;
  /** Loads backup. */
  loadBackup(id: BinaryInput): Binary;
  /** Returns the backup count. */
  backupCount(): number;
  /** Returns the restore private key. */
  restorePrivateKey(name: string, key: NativeHandle, signingKey: NativeHandle, overwrite: boolean): boolean;
  /** Loads owner signing key. */
  loadOwnerSigningKey(name: string): SigningKeyPair;
  /** Loads owner signing key generation. */
  loadOwnerSigningKeyGeneration(name: string, index: number): SigningKeyPair;
  /** Stores contact signing key. */
  storeContactSigningKey(name: string, key: NativeHandle): boolean;
  /** Loads contact signing key. */
  loadContactSigningKey(name: string): SigningPublicKey;
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
  /** Returns the find access slot labels. */
  findAccessSlotLabels(id: BinaryInput, name: string): ReadonlyArray<import('./domain.js').AccessSlotLabel>;
  /** Removes access slot label. */
  forgetAccessSlotLabel(id: BinaryInput, slotId: number): boolean;
  /** Returns the define form. */
  defineForm(alias: string, name: string, description: string, fields: ReadonlyArray<import('./domain.js').FormFieldInput>): import('./domain.js').FormDefinition;
  /** Returns the resolve form. */
  resolveForm(reference: string): import('./domain.js').FormDefinition;
  /** Lists forms. */
  listForms(): ReadonlyArray<import('./domain.js').FormDefinition>;
  /** Lists form revisions. */
  listFormRevisions(typeId: string): ReadonlyArray<import('./domain.js').FormDefinition>;
  /** Returns the seed forms. */
  seedForms(): number;
  /** Stores password. */
  rememberPassword(id: BinaryInput, password: BinaryInput): boolean;
  /** Returns the remembered password. */
  rememberedPassword(id: BinaryInput): Binary;
  /** Releases the native resources held by this object. */
  free(): void;
}

/** A restricted metadata view for discovery or diagnostics that lists local
 * profiles, contacts, forms, and lockboxes without loading owner signing keys. */
export class ReadOnlyVaultDirectory {
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
}

/** Client for the local session service that temporarily caches vault unlock
 * and owner signing keys across application operations. */
export class Agent {
  /** Reports whether running. */
  isRunning(): boolean;
  /** Removes all. */
  forgetAll(): boolean;
  /** Returns the serve. */
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
  /** Lists list. */
  list(): ReadonlyArray<import('./domain.js').AgentEntry>;
  /** Returns the sleep support. */
  sleepSupport(): import('./domain.js').SleepSupport;
  /** Returns vault unlock key. */
  getVaultUnlockKey(vaultId: string): Binary;
  /** Stores vault unlock key. */
  putVaultUnlockKey(vaultId: string, key: BinaryInput, ttlSeconds: number): boolean;
  /** Removes vault unlock key. */
  forgetVaultUnlockKey(vaultId: string): boolean;
  /** Returns owner signing key. */
  getOwnerSigningKey(vaultId: string, profile: string): SigningKeyPair;
  /** Stores owner signing key. */
  putOwnerSigningKey(vaultId: string, profile: string, key: NativeHandle, ttlSeconds: number): boolean;
  /** Removes owner signing key. */
  forgetOwnerSigningKey(vaultId: string, profile: string): boolean;
  /** Starts activity. */
  beginActivity(kind: string): AgentActivity;
  /** Stops activity. */
  endActivity(handle: NativeHandle): void;
}

/** A lifetime token kept alive while an operation needs cached secrets. Release
 * it afterward so the session agent can expire unused secrets. */
export class AgentActivity {
}

/** Access to operating-system credential storage for a scoped vault password. */
export class Platform {
  /** Returns the status. */
  status(): import('./domain.js').PlatformStatus;
  /** Sets scope. */
  setScope(scope: string): boolean;
  /** Removes password. */
  forgetPassword(): boolean;
  /** Stores password. */
  putPassword(password: BinaryInput): boolean;
  /** Returns the enable. */
  enable(): boolean;
  /** Returns the disable. */
  disable(): boolean;
  /** Returns the disabled. */
  disabled(): boolean;
  /** Returns password. */
  getPassword(): Binary;
}

/** A session for creating or opening lockboxes by host path, caching short-lived
 * passwords, and committing and closing the files used by a local application. */
export class LocalVault {
  /** Creates lockbox password. */
  createLockboxPassword(path: string, password: BinaryInput): Lockbox;
  /** Opens lockbox password. */
  openLockboxPassword(path: string, password: BinaryInput): Lockbox;
  /** Creates lockbox content key. */
  createLockboxContentKey(path: string, contentKey: BinaryInput, signingKey: NativeHandle): Lockbox;
  /** Creates lockbox contact. */
  createLockboxContact(path: string, contact: NativeHandle, name: string, signingKey: NativeHandle): Lockbox;
  /** Opens lockbox content key. */
  openLockboxContentKey(path: string, contentKey: BinaryInput, signingKey: NativeHandle): Lockbox;
  /** Stores lockbox password. */
  cacheLockboxPassword(path: string, password: BinaryInput, ttlSeconds: number): boolean;
  /** Releases the native resources held by lockbox. */
  closeLockbox(path: string): boolean;
  /** Releases the native resources held by all. */
  closeAll(): boolean;
  /** Releases the native resources held by this object. */
  free(): void;
}
