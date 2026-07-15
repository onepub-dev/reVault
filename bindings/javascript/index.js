// Generated complete class-oriented JavaScript API. Do not edit.
import { BindingOperations } from './native.js';
export { createMessage, encodeMessage } from './native.js';
export { revault } from './generated/messages.js';

class OwnedHandle {
  constructor(operations, nativeHandle) { this.operations = operations; this.nativeHandle = nativeHandle; }
}

export class Vault {
  constructor() { this.operations = new BindingOperations(); this.agent = new Agent(this.operations); this.platform = new Platform(this.operations); }
  lastError() { return this.operations.lastErrorMessage(); }
  lastErrorDetails() { return this.operations.bufferLastErrorDetails(); }

  lockboxFormatVersion() {
    return this.operations.lockboxFormatVersion();
  }

  lockboxProbeFormatVersion(bytes) {
    return this.operations.lockboxProbeFormatVersion(bytes);
  }

  lockboxCreate(key) {
    return new Lockbox(this.operations, this.operations.lockboxCreate(key));
  }

  lockboxCreateWithOptions(key, cacheMode, cacheBytes, workload, worker, jobs) {
    return new Lockbox(this.operations, this.operations.lockboxCreateWithOptions(key, cacheMode, cacheBytes, workload, worker, jobs));
  }

  lockboxCreatePassword(password) {
    return new Lockbox(this.operations, this.operations.lockboxCreatePassword(password));
  }

  lockboxCreateContact(contact) {
    return new Lockbox(this.operations, this.operations.lockboxCreateContact(contact?.nativeHandle ?? null));
  }

  lockboxCreateWithSigningKey(contentKey, signingKey) {
    return new Lockbox(this.operations, this.operations.lockboxCreateWithSigningKey(contentKey, signingKey?.nativeHandle ?? null));
  }

  lockboxOpen(archive, key) {
    return new Lockbox(this.operations, this.operations.lockboxOpen(archive, key));
  }

  lockboxOpenWithOptions(archive, key, cacheMode, cacheBytes, workload, worker, jobs) {
    return new Lockbox(this.operations, this.operations.lockboxOpenWithOptions(archive, key, cacheMode, cacheBytes, workload, worker, jobs));
  }

  lockboxOpenPassword(archive, password) {
    return new Lockbox(this.operations, this.operations.lockboxOpenPassword(archive, password));
  }

  lockboxOpenContact(archive, contact) {
    return new Lockbox(this.operations, this.operations.lockboxOpenContact(archive, contact?.nativeHandle ?? null));
  }

  lockboxInspectFile(path) {
    return this.operations.lockboxInspectFile(path);
  }

  lockboxRecoveryScanPath(path, key) {
    return this.operations.lockboxRecoveryScanPath(path, key);
  }

  lockboxRecoveryScan(bytes, key) {
    return this.operations.lockboxRecoveryScan(bytes, key);
  }

  lockboxRecoverySalvage(bytes, key, signingKey) {
    return new Lockbox(this.operations, this.operations.lockboxRecoverySalvage(bytes, key, signingKey?.nativeHandle ?? null));
  }

  keyContactGenerate() {
    return new ContactKeyPair(this.operations, this.operations.keyContactGenerate());
  }

  keyContactFromPrivate(bytes) {
    return new ContactKeyPair(this.operations, this.operations.keyContactFromPrivate(bytes));
  }

  keyContactPublicFromBytes(bytes) {
    return new ContactPublicKey(this.operations, this.operations.keyContactPublicFromBytes(bytes));
  }

  keySigningGenerate() {
    return new SigningKeyPair(this.operations, this.operations.keySigningGenerate());
  }

  keySigningFromPrivate(bytes) {
    return new SigningKeyPair(this.operations, this.operations.keySigningFromPrivate(bytes));
  }

  keySigningPublicFromBytes(bytes) {
    return new SigningPublicKey(this.operations, this.operations.keySigningPublicFromBytes(bytes));
  }

  vaultKeyExportPrivate(key, format) {
    return this.operations.vaultKeyExportPrivate(key?.nativeHandle ?? null, format);
  }

  vaultKeyExportPublic(key, format) {
    return this.operations.vaultKeyExportPublic(key?.nativeHandle ?? null, format);
  }

  vaultKeyImportPrivate(bytes) {
    return new ContactKeyPair(this.operations, this.operations.vaultKeyImportPrivate(bytes));
  }

  vaultKeyImportPublic(bytes) {
    return new ContactPublicKey(this.operations, this.operations.vaultKeyImportPublic(bytes));
  }

  vaultKeyFingerprint(key) {
    return this.operations.vaultKeyFingerprint(key?.nativeHandle ?? null);
  }

  vaultKeyFormatHex(bytes) {
    return this.operations.vaultKeyFormatHex(bytes);
  }

  vaultKeyDecodeHex(text) {
    return this.operations.vaultKeyDecodeHex(text);
  }

  vaultKeyFormatCrockford(bytes) {
    return this.operations.vaultKeyFormatCrockford(bytes);
  }

  vaultKeyFormatCrockfordReading(code) {
    return this.operations.vaultKeyFormatCrockfordReading(code);
  }

  vaultKeyDecodeCrockford(code) {
    return this.operations.vaultKeyDecodeCrockford(code);
  }

  vaultKeyHexEncode(bytes) {
    return this.operations.vaultKeyHexEncode(bytes);
  }

  vaultKeyHexDecode(text) {
    return this.operations.vaultKeyHexDecode(text);
  }

  vaultDirectoryOpen(root, password) {
    return new VaultDirectory(this.operations, this.operations.vaultDirectoryOpen(root, password));
  }

  vaultStructureVersionCurrent() {
    return this.operations.vaultStructureVersionCurrent();
  }

  vaultDirectoryProbeStructureVersion(root, password) {
    return this.operations.vaultDirectoryProbeStructureVersion(root, password);
  }

  vaultDirectoryOpenOrCreateDefault(password) {
    return new VaultDirectory(this.operations, this.operations.vaultDirectoryOpenOrCreateDefault(password));
  }

  vaultDirectoryReplaceDefault(password) {
    return new VaultDirectory(this.operations, this.operations.vaultDirectoryReplaceDefault(password));
  }

  vaultDirectoryChangePassword(root, oldPassword, newPassword) {
    return this.operations.vaultDirectoryChangePassword(root, oldPassword, newPassword);
  }

  vaultDirectoryChangeDefaultPassword(oldPassword, newPassword) {
    return this.operations.vaultDirectoryChangeDefaultPassword(oldPassword, newPassword);
  }

  vaultDirectoryReplace(root, password) {
    return new VaultDirectory(this.operations, this.operations.vaultDirectoryReplace(root, password));
  }

  vaultDirectoryOpenOrCreate(root, password) {
    return new VaultDirectory(this.operations, this.operations.vaultDirectoryOpenOrCreate(root, password));
  }

  vaultBackupDefault(path, overwrite) {
    return this.operations.vaultBackupDefault(path, overwrite);
  }

  vaultRestoreDefault(path, overwrite) {
    return this.operations.vaultRestoreDefault(path, overwrite);
  }

  vaultReadOnlyOpen(root, password) {
    return new ReadOnlyVaultDirectory(this.operations, this.operations.vaultReadOnlyOpen(root, password));
  }

  vaultReadOnlyOpenDefault(password) {
    return new ReadOnlyVaultDirectory(this.operations, this.operations.vaultReadOnlyOpenDefault(password));
  }

  vaultDefaultDirectory() {
    return this.operations.vaultDefaultDirectory();
  }

  vaultDefaultPath() {
    return this.operations.vaultDefaultPath();
  }

  vaultAgentLogPath() {
    return this.operations.vaultAgentLogPath();
  }

  vaultAgentLogDestination() {
    return this.operations.vaultAgentLogDestination();
  }

  vaultLocal() {
    return new LocalVault(this.operations, this.operations.vaultLocal());
  }

}

export class Lockbox extends OwnedHandle {
  addFile(path, data, replace) {
    return this.operations.lockboxAddFile(this.nativeHandle, path, data, replace);
  }

  addFileWithPermissions(path, data, permissions, replace) {
    return this.operations.lockboxAddFileWithPermissions(this.nativeHandle, path, data, permissions, replace);
  }

  getFile(path) {
    return this.operations.lockboxGetFile(this.nativeHandle, path);
  }

  extractFile(source, destination, replace) {
    return this.operations.lockboxExtractFile(this.nativeHandle, source, destination, replace);
  }

  extractDirectory(destination, maxFileBytes, maxTotalBytes, maxFiles, restoreSymlinks, restorePermissions, overwrite) {
    return this.operations.lockboxExtractDirectory(this.nativeHandle, destination, maxFileBytes, maxTotalBytes, maxFiles, restoreSymlinks, restorePermissions, overwrite);
  }

  streamContent(physical) {
    return this.operations.lockboxStreamContent(this.nativeHandle, physical);
  }

  cacheStats() {
    return this.operations.lockboxCacheStats(this.nativeHandle);
  }

  importStats() {
    return this.operations.lockboxImportStats(this.nativeHandle);
  }

  resetImportStats() {
    return this.operations.lockboxResetImportStats(this.nativeHandle);
  }

  pageInspection() {
    return this.operations.lockboxPageInspection(this.nativeHandle);
  }

  recoveryReport() {
    return this.operations.lockboxRecoveryReport(this.nativeHandle);
  }

  recoveryReportRender(verbose, maxEntries) {
    return this.operations.lockboxRecoveryReportRender(this.nativeHandle, verbose, maxEntries);
  }

  storageLen() {
    return this.operations.lockboxStorageLen(this.nativeHandle);
  }

  setWorkloadProfile(profile) {
    return this.operations.lockboxSetWorkloadProfile(this.nativeHandle, profile);
  }

  setWorkerPolicy(mode, jobs) {
    return this.operations.lockboxSetWorkerPolicy(this.nativeHandle, mode, jobs);
  }

  runtimeOptions() {
    return this.operations.lockboxRuntimeOptions(this.nativeHandle);
  }

  commit() {
    return this.operations.lockboxCommit(this.nativeHandle);
  }

  createDir(path, createParents) {
    return this.operations.lockboxCreateDir(this.nativeHandle, path, createParents);
  }

  delete(path) {
    return this.operations.lockboxDelete(this.nativeHandle, path);
  }

  removeDir(path, recursive) {
    return this.operations.lockboxRemoveDir(this.nativeHandle, path, recursive);
  }

  createParentDirs(path) {
    return this.operations.lockboxCreateParentDirs(this.nativeHandle, path);
  }

  rename(from, to) {
    return this.operations.lockboxRename(this.nativeHandle, from, to);
  }

  list(path, recursive) {
    return this.operations.lockboxList(this.nativeHandle, path, recursive);
  }

  listWithOptions(path, glob, recursive, includeFiles, includeSymlinks, includeDirectories, limit) {
    return this.operations.lockboxListWithOptions(this.nativeHandle, path, glob, recursive, includeFiles, includeSymlinks, includeDirectories, limit);
  }

  stat(path) {
    return this.operations.lockboxStat(this.nativeHandle, path);
  }

  setVariable(name, value, secret) {
    return this.operations.lockboxSetVariable(this.nativeHandle, name, value, secret);
  }

  getVariable(name) {
    return this.operations.lockboxGetVariable(this.nativeHandle, name);
  }

  deleteVariable(name) {
    return this.operations.lockboxDeleteVariable(this.nativeHandle, name);
  }

  moveVariables(movesProto) {
    return this.operations.lockboxMoveVariables(this.nativeHandle, movesProto);
  }

  listVariables() {
    return this.operations.lockboxListVariables(this.nativeHandle);
  }

  variableSensitivity(name) {
    return this.operations.lockboxVariableSensitivity(this.nativeHandle, name);
  }

  addSymlink(path, target, replace) {
    return this.operations.lockboxAddSymlink(this.nativeHandle, path, target, replace);
  }

  getSymlinkTarget(path) {
    return this.operations.lockboxGetSymlinkTarget(this.nativeHandle, path);
  }

  id() {
    return this.operations.lockboxId(this.nativeHandle);
  }

  exists(path) {
    return this.operations.lockboxExists(this.nativeHandle, path);
  }

  isDir(path) {
    return this.operations.lockboxIsDir(this.nativeHandle, path);
  }

  permissions(path) {
    return this.operations.lockboxPermissions(this.nativeHandle, path);
  }

  setPermissions(path, permissions) {
    return this.operations.lockboxSetPermissions(this.nativeHandle, path, permissions);
  }

  readRange(path, offset, len) {
    return this.operations.lockboxReadRange(this.nativeHandle, path, offset, len);
  }

  addPassword(password) {
    return this.operations.lockboxAddPassword(this.nativeHandle, password);
  }

  addContact(contact, name) {
    return this.operations.lockboxAddContact(this.nativeHandle, contact?.nativeHandle ?? null, name);
  }

  deleteKey(id) {
    return this.operations.lockboxDeleteKey(this.nativeHandle, id);
  }

  listKeySlots() {
    return this.operations.lockboxListKeySlots(this.nativeHandle);
  }

  setOwnerSigningKey(key) {
    return this.operations.lockboxSetOwnerSigningKey(this.nativeHandle, key?.nativeHandle ?? null);
  }

  ownerInspection() {
    return this.operations.lockboxOwnerInspection(this.nativeHandle);
  }

  defineForm(alias, name, description, fieldsProto) {
    return this.operations.lockboxDefineForm(this.nativeHandle, alias, name, description, fieldsProto);
  }

  listFormDefinitions() {
    return this.operations.lockboxListFormDefinitions(this.nativeHandle);
  }

  resolveForm(reference) {
    return this.operations.lockboxResolveForm(this.nativeHandle, reference);
  }

  listFormRevisions(typeId) {
    return this.operations.lockboxListFormRevisions(this.nativeHandle, typeId);
  }

  createFormRecord(path, typeReference, name) {
    return this.operations.lockboxCreateFormRecord(this.nativeHandle, path, typeReference, name);
  }

  setFormField(path, field, value, secret) {
    return this.operations.lockboxSetFormField(this.nativeHandle, path, field, value, secret);
  }

  listFormRecords() {
    return this.operations.lockboxListFormRecords(this.nativeHandle);
  }

  getFormRecord(path) {
    return this.operations.lockboxGetFormRecord(this.nativeHandle, path);
  }

  deleteFormRecord(path) {
    return this.operations.lockboxDeleteFormRecord(this.nativeHandle, path);
  }

  moveFormRecords(movesProto) {
    return this.operations.lockboxMoveFormRecords(this.nativeHandle, movesProto);
  }

  getFormField(path, field) {
    return this.operations.lockboxGetFormField(this.nativeHandle, path, field);
  }

  toBytes() {
    return this.operations.lockboxToBytes(this.nativeHandle);
  }

  free() {
    this.operations.lockboxFree(this.nativeHandle);
    this.nativeHandle = null;
  }

}

export class ContactKeyPair extends OwnedHandle {
  public() {
    return this.operations.keyContactPublic(this.nativeHandle);
  }

  private() {
    return this.operations.keyContactPrivate(this.nativeHandle);
  }

  free() {
    this.operations.keyContactFree(this.nativeHandle);
    this.nativeHandle = null;
  }

  decrypt(wrapped) {
    return this.operations.keyContactDecrypt(this.nativeHandle, wrapped?.nativeHandle ?? null);
  }

}

export class ContactPublicKey extends OwnedHandle {
  publicFree() {
    this.operations.keyContactPublicFree(this.nativeHandle);
    this.nativeHandle = null;
  }

  encrypt(contentKey) {
    return new WrappedContactKey(this.operations, this.operations.keyContactEncrypt(this.nativeHandle, contentKey));
  }

}

export class WrappedContactKey extends OwnedHandle {
  public() {
    return this.operations.keyContactWrappedPublic(this.nativeHandle);
  }

  ciphertext() {
    return this.operations.keyContactWrappedCiphertext(this.nativeHandle);
  }

  encrypted() {
    return this.operations.keyContactWrappedEncrypted(this.nativeHandle);
  }

  free() {
    this.operations.keyContactWrappedFree(this.nativeHandle);
    this.nativeHandle = null;
  }

}

export class SigningKeyPair extends OwnedHandle {
  public() {
    return this.operations.keySigningPublic(this.nativeHandle);
  }

  private() {
    return this.operations.keySigningPrivate(this.nativeHandle);
  }

  free() {
    this.operations.keySigningFree(this.nativeHandle);
    this.nativeHandle = null;
  }

}

export class SigningPublicKey extends OwnedHandle {
  publicFree() {
    this.operations.keySigningPublicFree(this.nativeHandle);
    this.nativeHandle = null;
  }

}

export class VaultDirectory extends OwnedHandle {
  root() {
    return this.operations.vaultDirectoryRoot(this.nativeHandle);
  }

  structureVersion() {
    return this.operations.vaultDirectoryStructureVersion(this.nativeHandle);
  }

  listPrivateKeys() {
    return this.operations.vaultDirectoryListPrivateKeys(this.nativeHandle);
  }

  listPrivateKeyNames() {
    return this.operations.vaultDirectoryListPrivateKeyNames(this.nativeHandle);
  }

  listContactNames() {
    return this.operations.vaultDirectoryListContactNames(this.nativeHandle);
  }

  listFormAliases() {
    return this.operations.vaultDirectoryListFormAliases(this.nativeHandle);
  }

  privateKeyExists(name) {
    return this.operations.vaultDirectoryPrivateKeyExists(this.nativeHandle, name);
  }

  deletePrivateKey(name) {
    return this.operations.vaultDirectoryDeletePrivateKey(this.nativeHandle, name);
  }

  storePrivateKey(name, key) {
    return this.operations.vaultDirectoryStorePrivateKey(this.nativeHandle, name, key?.nativeHandle ?? null);
  }

  loadPrivateKey(name) {
    return new ContactKeyPair(this.operations, this.operations.vaultDirectoryLoadPrivateKey(this.nativeHandle, name));
  }

  loadPrivateKeyGeneration(name, index) {
    return new ContactKeyPair(this.operations, this.operations.vaultDirectoryLoadPrivateKeyGeneration(this.nativeHandle, name, index));
  }

  storeContact(name, key) {
    return this.operations.vaultDirectoryStoreContact(this.nativeHandle, name, key?.nativeHandle ?? null);
  }

  loadContact(name) {
    return new ContactPublicKey(this.operations, this.operations.vaultDirectoryLoadContact(this.nativeHandle, name));
  }

  contactExists(name) {
    return this.operations.vaultDirectoryContactExists(this.nativeHandle, name);
  }

  deleteContact(name) {
    return this.operations.vaultDirectoryDeleteContact(this.nativeHandle, name);
  }

  listContacts() {
    return this.operations.vaultDirectoryListContacts(this.nativeHandle);
  }

  storeProfileEmail(name, email) {
    return this.operations.vaultDirectoryStoreProfileEmail(this.nativeHandle, name, email);
  }

  profileEmail(name) {
    return this.operations.vaultDirectoryProfileEmail(this.nativeHandle, name);
  }

  storeBackup(id, bytes) {
    return this.operations.vaultDirectoryStoreBackup(this.nativeHandle, id, bytes);
  }

  loadBackup(id) {
    return this.operations.vaultDirectoryLoadBackup(this.nativeHandle, id);
  }

  backupCount() {
    return this.operations.vaultDirectoryBackupCount(this.nativeHandle);
  }

  restorePrivateKey(name, key, signingKey, overwrite) {
    return this.operations.vaultDirectoryRestorePrivateKey(this.nativeHandle, name, key?.nativeHandle ?? null, signingKey?.nativeHandle ?? null, overwrite);
  }

  loadOwnerSigningKey(name) {
    return new SigningKeyPair(this.operations, this.operations.vaultDirectoryLoadOwnerSigningKey(this.nativeHandle, name));
  }

  loadOwnerSigningKeyGeneration(name, index) {
    return new SigningKeyPair(this.operations, this.operations.vaultDirectoryLoadOwnerSigningKeyGeneration(this.nativeHandle, name, index));
  }

  storeContactSigningKey(name, key) {
    return this.operations.vaultDirectoryStoreContactSigningKey(this.nativeHandle, name, key?.nativeHandle ?? null);
  }

  loadContactSigningKey(name) {
    return new SigningPublicKey(this.operations, this.operations.vaultDirectoryLoadContactSigningKey(this.nativeHandle, name));
  }

  listProfileGenerations(name) {
    return this.operations.vaultDirectoryListProfileGenerations(this.nativeHandle, name);
  }

  rotatePrivateKey(name) {
    return this.operations.vaultDirectoryRotatePrivateKey(this.nativeHandle, name);
  }

  rememberLockbox(id, path) {
    return this.operations.vaultDirectoryRememberLockbox(this.nativeHandle, id, path);
  }

  listKnownLockboxes() {
    return this.operations.vaultDirectoryListKnownLockboxes(this.nativeHandle);
  }

  forgetLockbox(path) {
    return this.operations.vaultDirectoryForgetLockbox(this.nativeHandle, path);
  }

  rememberAccessSlotLabel(id, slotId, name) {
    return this.operations.vaultDirectoryRememberAccessSlotLabel(this.nativeHandle, id, slotId, name);
  }

  listAccessSlotLabels(id) {
    return this.operations.vaultDirectoryListAccessSlotLabels(this.nativeHandle, id);
  }

  findAccessSlotLabels(id, name) {
    return this.operations.vaultDirectoryFindAccessSlotLabels(this.nativeHandle, id, name);
  }

  forgetAccessSlotLabel(id, slotId) {
    return this.operations.vaultDirectoryForgetAccessSlotLabel(this.nativeHandle, id, slotId);
  }

  defineForm(alias, name, description, fieldsProto) {
    return this.operations.vaultDirectoryDefineForm(this.nativeHandle, alias, name, description, fieldsProto);
  }

  resolveForm(reference) {
    return this.operations.vaultDirectoryResolveForm(this.nativeHandle, reference);
  }

  listForms() {
    return this.operations.vaultDirectoryListForms(this.nativeHandle);
  }

  listFormRevisions(typeId) {
    return this.operations.vaultDirectoryListFormRevisions(this.nativeHandle, typeId);
  }

  seedForms() {
    return this.operations.vaultDirectorySeedForms(this.nativeHandle);
  }

  rememberPassword(id, password) {
    return this.operations.vaultDirectoryRememberPassword(this.nativeHandle, id, password);
  }

  rememberedPassword(id) {
    return this.operations.vaultDirectoryRememberedPassword(this.nativeHandle, id);
  }

  free() {
    this.operations.vaultDirectoryFree(this.nativeHandle);
    this.nativeHandle = null;
  }

}

export class ReadOnlyVaultDirectory extends OwnedHandle {
  listProfileNames() {
    return this.operations.vaultReadOnlyListProfileNames(this.nativeHandle);
  }

  listContactNames() {
    return this.operations.vaultReadOnlyListContactNames(this.nativeHandle);
  }

  listFormAliases() {
    return this.operations.vaultReadOnlyListFormAliases(this.nativeHandle);
  }

  listKnownLockboxes() {
    return this.operations.vaultReadOnlyListKnownLockboxes(this.nativeHandle);
  }

  free() {
    return this.operations.vaultReadOnlyFree(this.nativeHandle);
  }

}

export class Agent {
  constructor(operations) { this.operations = operations; }

  isRunning() {
    return this.operations.vaultIsRunning();
  }

  forgetAll() {
    return this.operations.vaultForgetAll();
  }

  serve() {
    return this.operations.vaultAgentServe();
  }

  verifyTransport() {
    return this.operations.vaultAgentVerifyTransport();
  }

  get(id) {
    return this.operations.vaultAgentGet(id);
  }

  put(id, key) {
    return this.operations.vaultAgentPut(id, key);
  }

  forget(id) {
    return this.operations.vaultAgentForget(id);
  }

  stop() {
    return this.operations.vaultAgentStop();
  }

  start() {
    return this.operations.vaultAgentStart();
  }

  list() {
    return this.operations.vaultAgentList();
  }

  sleepSupport() {
    return this.operations.vaultAgentSleepSupport();
  }

  getVaultUnlockKey(vaultId) {
    return this.operations.vaultAgentGetVaultUnlockKey(vaultId);
  }

  putVaultUnlockKey(vaultId, key, ttlSeconds) {
    return this.operations.vaultAgentPutVaultUnlockKey(vaultId, key, ttlSeconds);
  }

  forgetVaultUnlockKey(vaultId) {
    return this.operations.vaultAgentForgetVaultUnlockKey(vaultId);
  }

  getOwnerSigningKey(vaultId, profile) {
    return new SigningKeyPair(this.operations, this.operations.vaultAgentGetOwnerSigningKey(vaultId, profile));
  }

  putOwnerSigningKey(vaultId, profile, key, ttlSeconds) {
    return this.operations.vaultAgentPutOwnerSigningKey(vaultId, profile, key?.nativeHandle ?? null, ttlSeconds);
  }

  forgetOwnerSigningKey(vaultId, profile) {
    return this.operations.vaultAgentForgetOwnerSigningKey(vaultId, profile);
  }

  beginActivity(kind) {
    return new AgentActivity(this.operations, this.operations.vaultAgentBeginActivity(kind));
  }

  endActivity(handle) {
    return this.operations.vaultAgentEndActivity(handle?.nativeHandle ?? null);
  }

}

export class AgentActivity extends OwnedHandle {
}

export class Platform {
  constructor(operations) { this.operations = operations; }

  status() {
    return this.operations.vaultPlatformStatus();
  }

  setScope(scope) {
    return this.operations.vaultPlatformSetScope(scope);
  }

  forgetPassword() {
    return this.operations.vaultPlatformForgetPassword();
  }

  putPassword(password) {
    return this.operations.vaultPlatformPutPassword(password);
  }

  enable() {
    return this.operations.vaultPlatformEnable();
  }

  disable() {
    return this.operations.vaultPlatformDisable();
  }

  disabled() {
    return this.operations.vaultPlatformDisabled();
  }

  getPassword() {
    return this.operations.vaultPlatformGetPassword();
  }

}

export class LocalVault extends OwnedHandle {
  createLockboxPassword(path, password) {
    return new Lockbox(this.operations, this.operations.vaultCreateLockboxPassword(this.nativeHandle, path, password));
  }

  openLockboxPassword(path, password) {
    return new Lockbox(this.operations, this.operations.vaultOpenLockboxPassword(this.nativeHandle, path, password));
  }

  createLockboxContentKey(path, contentKey, signingKey) {
    return new Lockbox(this.operations, this.operations.vaultCreateLockboxContentKey(this.nativeHandle, path, contentKey, signingKey?.nativeHandle ?? null));
  }

  createLockboxContact(path, contact, name, signingKey) {
    return new Lockbox(this.operations, this.operations.vaultCreateLockboxContact(this.nativeHandle, path, contact?.nativeHandle ?? null, name, signingKey?.nativeHandle ?? null));
  }

  openLockboxContentKey(path, contentKey, signingKey) {
    return new Lockbox(this.operations, this.operations.vaultOpenLockboxContentKey(this.nativeHandle, path, contentKey, signingKey?.nativeHandle ?? null));
  }

  cacheLockboxPassword(path, password, ttlSeconds) {
    return this.operations.vaultCacheLockboxPassword(this.nativeHandle, path, password, ttlSeconds);
  }

  closeLockbox(path) {
    return this.operations.vaultCloseLockbox(this.nativeHandle, path);
  }

  closeAll() {
    return this.operations.vaultCloseAll(this.nativeHandle);
  }

  free() {
    this.operations.vaultFree(this.nativeHandle);
    this.nativeHandle = null;
  }

}
