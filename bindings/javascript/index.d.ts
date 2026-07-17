// Generated complete TypeScript API. Do not edit.
import type * as $protobuf from 'protobufjs';
export { revault } from './generated/messages.js';
export type Binary = Uint8Array;
export type BinaryInput = Uint8Array | string;
export type NativeHandle = ContactKeyPair | ContactPublicKey | WrappedContactKey | SigningKeyPair | SigningPublicKey | VaultDirectory | ReadOnlyVaultDirectory | AgentActivity | LocalVault;
export function createMessage(name: string, fields?: object): $protobuf.Message;
export function encodeMessage(message: $protobuf.Message): Binary;

export class Vault {
  constructor();
  readonly agent: Agent;
  readonly platform: Platform;
  lastError(): string;
  lastErrorDetails(): import('./generated/messages.js').revault.bindings.ErrorDetails;
  lockboxFormatVersion(): number;
  lockboxProbeFormatVersion(bytes: BinaryInput): number;
  lockboxCreate(key: BinaryInput): Lockbox;
  lockboxCreateWithOptions(key: BinaryInput, cacheMode: string, cacheBytes: number, workload: string, worker: string, jobs: number): Lockbox;
  lockboxCreatePassword(password: BinaryInput): Lockbox;
  lockboxCreateContact(contact: NativeHandle): Lockbox;
  lockboxCreateWithSigningKey(contentKey: BinaryInput, signingKey: NativeHandle): Lockbox;
  lockboxOpen(archive: BinaryInput, key: BinaryInput): Lockbox;
  lockboxOpenWithOptions(archive: BinaryInput, key: BinaryInput, cacheMode: string, cacheBytes: number, workload: string, worker: string, jobs: number): Lockbox;
  lockboxOpenPassword(archive: BinaryInput, password: BinaryInput): Lockbox;
  lockboxOpenContact(archive: BinaryInput, contact: NativeHandle): Lockbox;
  lockboxInspectFile(path: string): import('./generated/messages.js').revault.bindings.FileInspection;
  lockboxRecoveryScanPath(path: string, key: BinaryInput): import('./generated/messages.js').revault.bindings.RecoveryReport;
  lockboxRecoveryScan(bytes: BinaryInput, key: BinaryInput): import('./generated/messages.js').revault.bindings.RecoveryReport;
  lockboxRecoverySalvage(bytes: BinaryInput, key: BinaryInput, signingKey: NativeHandle): Lockbox;
  keyContactGenerate(): ContactKeyPair;
  keyContactFromPrivate(bytes: BinaryInput): ContactKeyPair;
  keyContactPublicFromBytes(bytes: BinaryInput): ContactPublicKey;
  keySigningGenerate(): SigningKeyPair;
  keySigningFromPrivate(bytes: BinaryInput): SigningKeyPair;
  keySigningPublicFromBytes(bytes: BinaryInput): SigningPublicKey;
  vaultKeyExportPrivate(key: NativeHandle, format: string): Binary;
  vaultKeyExportPublic(key: NativeHandle, format: string): Binary;
  vaultKeyImportPrivate(bytes: BinaryInput): ContactKeyPair;
  vaultKeyImportPublic(bytes: BinaryInput): ContactPublicKey;
  vaultKeyFingerprint(key: NativeHandle): Binary;
  vaultKeyFormatHex(bytes: BinaryInput): string;
  vaultKeyDecodeHex(text: string): Binary;
  vaultKeyFormatCrockford(bytes: BinaryInput): string;
  vaultKeyFormatCrockfordReading(code: string): string;
  vaultKeyDecodeCrockford(code: string): Binary;
  vaultKeyHexEncode(bytes: BinaryInput): string;
  vaultKeyHexDecode(text: string): Binary;
  vaultDirectoryOpen(root: string, password: BinaryInput): VaultDirectory;
  vaultStructureVersionCurrent(): number;
  vaultDirectoryProbeStructureVersion(root: string, password: BinaryInput): number;
  vaultDirectoryOpenOrCreateDefault(password: BinaryInput): VaultDirectory;
  vaultDirectoryReplaceDefault(password: BinaryInput): VaultDirectory;
  vaultDirectoryChangePassword(root: string, oldPassword: BinaryInput, newPassword: BinaryInput): boolean;
  vaultDirectoryChangeDefaultPassword(oldPassword: BinaryInput, newPassword: BinaryInput): boolean;
  vaultDirectoryReplace(root: string, password: BinaryInput): VaultDirectory;
  vaultDirectoryOpenOrCreate(root: string, password: BinaryInput): VaultDirectory;
  vaultBackupDefault(path: string, overwrite: boolean): import('./generated/messages.js').revault.bindings.VaultBackupManifest;
  vaultRestoreDefault(path: string, overwrite: boolean): import('./generated/messages.js').revault.bindings.VaultBackupManifest;
  vaultReadOnlyOpen(root: string, password: BinaryInput): ReadOnlyVaultDirectory;
  vaultReadOnlyOpenDefault(password: BinaryInput): ReadOnlyVaultDirectory;
  vaultDefaultDirectory(): string;
  vaultDefaultPath(): string;
  vaultAgentLogPath(): string;
  vaultAgentLogDestination(): string;
  vaultLocal(): LocalVault;
}

export class Lockbox {
  addFile(path: string, data: BinaryInput, replace: boolean): boolean;
  addFileWithPermissions(path: string, data: BinaryInput, permissions: number, replace: boolean): boolean;
  getFile(path: string): Binary;
  extractFile(source: string, destination: string, replace: boolean): boolean;
  extractDirectory(destination: string, maxFileBytes: number, maxTotalBytes: number, maxFiles: number, restoreSymlinks: boolean, restorePermissions: boolean, overwrite: boolean): boolean;
  streamContent(physical: boolean): import('./generated/messages.js').revault.bindings.StreamChunkList;
  cacheStats(): import('./generated/messages.js').revault.bindings.CacheStats;
  importStats(): import('./generated/messages.js').revault.bindings.ImportStats;
  resetImportStats(): boolean;
  pageInspection(): import('./generated/messages.js').revault.bindings.PageInspectionList;
  recoveryReport(): import('./generated/messages.js').revault.bindings.RecoveryReport;
  recoveryReportRender(verbose: boolean, maxEntries: number): string;
  storageLen(): number;
  setWorkloadProfile(profile: string): boolean;
  setWorkerPolicy(mode: string, jobs: number): boolean;
  runtimeOptions(): import('./generated/messages.js').revault.bindings.RuntimeOptions;
  commit(): boolean;
  createDir(path: string, createParents: boolean): boolean;
  delete(path: string): boolean;
  removeDir(path: string, recursive: boolean): boolean;
  createParentDirs(path: string): boolean;
  rename(from: string, to: string): boolean;
  list(path: string, recursive: boolean): import('./generated/messages.js').revault.bindings.LockboxEntryList;
  listWithOptions(path: string, glob: string, recursive: boolean, includeFiles: boolean, includeSymlinks: boolean, includeDirectories: boolean, limit: number): import('./generated/messages.js').revault.bindings.LockboxEntryList;
  stat(path: string): import('./generated/messages.js').revault.bindings.OptionalLockboxEntry;
  setVariable(name: string, value: string): boolean;
  setSecretVariable(name: string, value: BinaryInput): boolean;
  getVariable(name: string): string | undefined;
  withSecretVariable<T>(name: string, callback: (value: Uint8Array) => T): T | undefined;
  deleteVariable(name: string): boolean;
  moveVariables(movesProto: BinaryInput): boolean;
  listVariables(): import('./generated/messages.js').revault.bindings.VariableList;
  variableSensitivity(name: string): import('./generated/messages.js').revault.bindings.OptionalString;
  addSymlink(path: string, target: string, replace: boolean): boolean;
  getSymlinkTarget(path: string): string;
  id(): Binary;
  exists(path: string): boolean;
  isDir(path: string): boolean;
  permissions(path: string): number;
  setPermissions(path: string, permissions: number): boolean;
  readRange(path: string, offset: number, len: number): Binary;
  addPassword(password: BinaryInput): number;
  addContact(contact: NativeHandle, name: string): number;
  deleteKey(id: number): boolean;
  listKeySlots(): import('./generated/messages.js').revault.bindings.KeySlotList;
  setOwnerSigningKey(key: NativeHandle): boolean;
  ownerInspection(): import('./generated/messages.js').revault.bindings.OwnerInspection;
  defineForm(alias: string, name: string, description: string, fieldsProto: BinaryInput): import('./generated/messages.js').revault.bindings.FormDefinition;
  listFormDefinitions(): import('./generated/messages.js').revault.bindings.FormDefinitionList;
  resolveForm(reference: string): import('./generated/messages.js').revault.bindings.FormDefinition;
  listFormRevisions(typeId: string): import('./generated/messages.js').revault.bindings.FormDefinitionList;
  createFormRecord(path: string, typeReference: string, name: string): import('./generated/messages.js').revault.bindings.FormRecord;
  setFormField(path: string, field: string, value: string): boolean;
  setSecretFormField(path: string, field: string, value: BinaryInput): boolean;
  listFormRecords(): import('./generated/messages.js').revault.bindings.FormRecordList;
  getFormRecord(path: string): import('./generated/messages.js').revault.bindings.OptionalFormRecord;
  deleteFormRecord(path: string): boolean;
  moveFormRecords(movesProto: BinaryInput): boolean;
  getFormField(path: string, field: string): import('./generated/messages.js').revault.bindings.OptionalFormValue;
  withSecretFormField<T>(path: string, field: string, callback: (value: Uint8Array) => T): T | undefined;
  toBytes(): Binary;
  free(): void;
}

export class ContactKeyPair {
  public(): Binary;
  private(): Binary;
  free(): void;
  decrypt(wrapped: NativeHandle): Binary;
}

export class ContactPublicKey {
  publicFree(): void;
  encrypt(contentKey: BinaryInput): WrappedContactKey;
}

export class WrappedContactKey {
  public(): Binary;
  ciphertext(): Binary;
  encrypted(): Binary;
  free(): void;
}

export class SigningKeyPair {
  public(): Binary;
  private(): Binary;
  free(): void;
}

export class SigningPublicKey {
  publicFree(): void;
}

export class VaultDirectory {
  root(): string;
  structureVersion(): number;
  listPrivateKeys(): import('./generated/messages.js').revault.bindings.StringList;
  listPrivateKeyNames(): import('./generated/messages.js').revault.bindings.StringList;
  listContactNames(): import('./generated/messages.js').revault.bindings.StringList;
  listFormAliases(): import('./generated/messages.js').revault.bindings.StringList;
  privateKeyExists(name: string): boolean;
  deletePrivateKey(name: string): boolean;
  storePrivateKey(name: string, key: NativeHandle): boolean;
  loadPrivateKey(name: string): ContactKeyPair;
  loadPrivateKeyGeneration(name: string, index: number): ContactKeyPair;
  storeContact(name: string, key: NativeHandle): boolean;
  loadContact(name: string): ContactPublicKey;
  contactExists(name: string): boolean;
  deleteContact(name: string): boolean;
  listContacts(): import('./generated/messages.js').revault.bindings.ContactList;
  storeProfileEmail(name: string, email: string): boolean;
  profileEmail(name: string): import('./generated/messages.js').revault.bindings.OptionalString;
  storeBackup(id: BinaryInput, bytes: BinaryInput): boolean;
  loadBackup(id: BinaryInput): Binary;
  backupCount(): number;
  restorePrivateKey(name: string, key: NativeHandle, signingKey: NativeHandle, overwrite: boolean): boolean;
  loadOwnerSigningKey(name: string): SigningKeyPair;
  loadOwnerSigningKeyGeneration(name: string, index: number): SigningKeyPair;
  storeContactSigningKey(name: string, key: NativeHandle): boolean;
  loadContactSigningKey(name: string): SigningPublicKey;
  listProfileGenerations(name: string): import('./generated/messages.js').revault.bindings.ProfileHistory;
  rotatePrivateKey(name: string): import('./generated/messages.js').revault.bindings.ProfileHistory;
  rememberLockbox(id: BinaryInput, path: string): boolean;
  listKnownLockboxes(): import('./generated/messages.js').revault.bindings.KnownLockboxList;
  forgetLockbox(path: string): boolean;
  rememberAccessSlotLabel(id: BinaryInput, slotId: number, name: string): boolean;
  listAccessSlotLabels(id: BinaryInput): import('./generated/messages.js').revault.bindings.AccessSlotLabelList;
  findAccessSlotLabels(id: BinaryInput, name: string): import('./generated/messages.js').revault.bindings.AccessSlotLabelList;
  forgetAccessSlotLabel(id: BinaryInput, slotId: number): boolean;
  defineForm(alias: string, name: string, description: string, fieldsProto: BinaryInput): import('./generated/messages.js').revault.bindings.FormDefinition;
  resolveForm(reference: string): import('./generated/messages.js').revault.bindings.FormDefinition;
  listForms(): import('./generated/messages.js').revault.bindings.FormDefinitionList;
  listFormRevisions(typeId: string): import('./generated/messages.js').revault.bindings.FormDefinitionList;
  seedForms(): number;
  rememberPassword(id: BinaryInput, password: BinaryInput): boolean;
  rememberedPassword(id: BinaryInput): Binary;
  free(): void;
}

export class ReadOnlyVaultDirectory {
  listProfileNames(): import('./generated/messages.js').revault.bindings.StringList;
  listContactNames(): import('./generated/messages.js').revault.bindings.StringList;
  listFormAliases(): import('./generated/messages.js').revault.bindings.StringList;
  listKnownLockboxes(): import('./generated/messages.js').revault.bindings.KnownLockboxList;
  free(): void;
}

export class Agent {
  isRunning(): boolean;
  forgetAll(): boolean;
  serve(): boolean;
  verifyTransport(): boolean;
  get(id: BinaryInput): Binary;
  put(id: BinaryInput, key: BinaryInput): boolean;
  forget(id: BinaryInput): boolean;
  stop(): boolean;
  start(): boolean;
  list(): import('./generated/messages.js').revault.bindings.AgentEntryList;
  sleepSupport(): import('./generated/messages.js').revault.bindings.SleepSupport;
  getVaultUnlockKey(vaultId: string): Binary;
  putVaultUnlockKey(vaultId: string, key: BinaryInput, ttlSeconds: number): boolean;
  forgetVaultUnlockKey(vaultId: string): boolean;
  getOwnerSigningKey(vaultId: string, profile: string): SigningKeyPair;
  putOwnerSigningKey(vaultId: string, profile: string, key: NativeHandle, ttlSeconds: number): boolean;
  forgetOwnerSigningKey(vaultId: string, profile: string): boolean;
  beginActivity(kind: string): AgentActivity;
  endActivity(handle: NativeHandle): void;
}

export class AgentActivity {
}

export class Platform {
  status(): import('./generated/messages.js').revault.bindings.PlatformStatus;
  setScope(scope: string): boolean;
  forgetPassword(): boolean;
  putPassword(password: BinaryInput): boolean;
  enable(): boolean;
  disable(): boolean;
  disabled(): boolean;
  getPassword(): Binary;
}

export class LocalVault {
  createLockboxPassword(path: string, password: BinaryInput): Lockbox;
  openLockboxPassword(path: string, password: BinaryInput): Lockbox;
  createLockboxContentKey(path: string, contentKey: BinaryInput, signingKey: NativeHandle): Lockbox;
  createLockboxContact(path: string, contact: NativeHandle, name: string, signingKey: NativeHandle): Lockbox;
  openLockboxContentKey(path: string, contentKey: BinaryInput, signingKey: NativeHandle): Lockbox;
  cacheLockboxPassword(path: string, password: BinaryInput, ttlSeconds: number): boolean;
  closeLockbox(path: string): boolean;
  closeAll(): boolean;
  free(): void;
}
