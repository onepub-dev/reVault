/**
 * Owned JavaScript API for encrypted reVault lockboxes and local vaults.
 * See https://github.com/onepub-dev/reVault#readme for installation, security
 * guidance, and complete examples.
 * @module @onepub-dev/revault-api
 */
import { BindingOperations, createMessage, encodeMessage } from './native.js';

function encodePathMoves(moves) {
  return encodeMessage(createMessage('PathMoveList', {
    values: moves.map((move) => createMessage('PathMove', move)),
  }));
}

function encodeFormFields(fields) {
  return encodeMessage(createMessage('FormFieldList', {
    values: fields.map((field) => createMessage('FormField', field)),
  }));
}

class OwnedHandle {
  /** Creates a new facade over the bundled native library. */
  constructor(operations, nativeHandle) { this.operations = operations; this.nativeHandle = nativeHandle; }
}

/** Primary API used to open lockboxes, manage keys and metadata, use the
 * session agent, and access operating-system credential storage. */
export class Vault {
  /** Creates a new facade over the bundled native library. */
  constructor() { this.operations = new BindingOperations(); this.agent = new Agent(this.operations); this.platform = new Platform(this.operations); }
  /** Returns the last error. */
  lastError() { return this.operations.lastErrorMessage(); }
  /** Returns the last error details. */
  lastErrorDetails() { return this.operations.bufferLastErrorDetails(); }

  /** Returns the lockbox format version. */
  lockboxFormatVersion() {
    return this.operations.lockboxFormatVersion();
  }

  /** Returns the lockbox probe format version. */
  lockboxProbeFormatVersion(bytes) {
    return this.operations.lockboxProbeFormatVersion(bytes);
  }

  /** Returns the lockbox create. */
  lockboxCreate(key) {
    return new Lockbox(this.operations, this.operations.lockboxCreate(key));
  }

  /** Creates a lockbox with explicit cache capacity, workload, worker policy, and job count. */
  lockboxCreateWithOptions(key, cacheMode, cacheBytes, workload, worker, jobs) {
    return new Lockbox(this.operations, this.operations.lockboxCreateWithOptions(key, cacheMode, cacheBytes, workload, worker, jobs));
  }

  /** Returns the lockbox create password. */
  lockboxCreatePassword(password) {
    return new Lockbox(this.operations, this.operations.lockboxCreatePassword(password));
  }

  /** Returns the lockbox create contact. */
  lockboxCreateContact(contact) {
    return new Lockbox(this.operations, this.operations.lockboxCreateContact(contact?.nativeHandle ?? null));
  }

  /** Returns the lockbox create with signing key. */
  lockboxCreateWithSigningKey(contentKey, signingKey) {
    return new Lockbox(this.operations, this.operations.lockboxCreateWithSigningKey(contentKey, signingKey?.nativeHandle ?? null));
  }

  /** Returns the lockbox open. */
  lockboxOpen(archive, key) {
    return new Lockbox(this.operations, this.operations.lockboxOpen(archive, key));
  }

  /** Opens a lockbox with explicit cache capacity, workload, worker policy, and job count. */
  lockboxOpenWithOptions(archive, key, cacheMode, cacheBytes, workload, worker, jobs) {
    return new Lockbox(this.operations, this.operations.lockboxOpenWithOptions(archive, key, cacheMode, cacheBytes, workload, worker, jobs));
  }

  /** Returns the lockbox open password. */
  lockboxOpenPassword(archive, password) {
    return new Lockbox(this.operations, this.operations.lockboxOpenPassword(archive, password));
  }

  /** Returns the lockbox open contact. */
  lockboxOpenContact(archive, contact) {
    return new Lockbox(this.operations, this.operations.lockboxOpenContact(archive, contact?.nativeHandle ?? null));
  }

  /** Returns the lockbox inspect file. */
  lockboxInspectFile(path) {
    return this.operations.lockboxInspectFile(path);
  }

  /** Returns the lockbox recovery scan path. */
  lockboxRecoveryScanPath(path, key) {
    return this.operations.lockboxRecoveryScanPath(path, key);
  }

  /** Returns the lockbox recovery scan. */
  lockboxRecoveryScan(bytes, key) {
    return this.operations.lockboxRecoveryScan(bytes, key);
  }

  /** Returns the lockbox recovery salvage. */
  lockboxRecoverySalvage(bytes, key, signingKey) {
    return new Lockbox(this.operations, this.operations.lockboxRecoverySalvage(bytes, key, signingKey?.nativeHandle ?? null));
  }

  /** Returns the key contact generate. */
  keyContactGenerate() {
    return new ContactKeyPair(this.operations, this.operations.keyContactGenerate());
  }

  /** Returns the key contact from private. */
  keyContactFromPrivate(bytes) {
    return new ContactKeyPair(this.operations, this.operations.keyContactFromPrivate(bytes));
  }

  /** Returns the key contact public from bytes. */
  keyContactPublicFromBytes(bytes) {
    return new ContactPublicKey(this.operations, this.operations.keyContactPublicFromBytes(bytes));
  }

  /** Returns the key signing generate. */
  keySigningGenerate() {
    return new SigningKeyPair(this.operations, this.operations.keySigningGenerate());
  }

  /** Returns the key signing from private. */
  keySigningFromPrivate(bytes) {
    return new SigningKeyPair(this.operations, this.operations.keySigningFromPrivate(bytes));
  }

  /** Returns the key signing public from bytes. */
  keySigningPublicFromBytes(bytes) {
    return new SigningPublicKey(this.operations, this.operations.keySigningPublicFromBytes(bytes));
  }

  /** Returns the vault key export private. */
  vaultKeyExportPrivate(key, format) {
    return this.operations.vaultKeyExportPrivate(key?.nativeHandle ?? null, format);
  }

  /** Returns the vault key export public. */
  vaultKeyExportPublic(key, format) {
    return this.operations.vaultKeyExportPublic(key?.nativeHandle ?? null, format);
  }

  /** Returns the vault key import private. */
  vaultKeyImportPrivate(bytes) {
    return new ContactKeyPair(this.operations, this.operations.vaultKeyImportPrivate(bytes));
  }

  /** Returns the vault key import public. */
  vaultKeyImportPublic(bytes) {
    return new ContactPublicKey(this.operations, this.operations.vaultKeyImportPublic(bytes));
  }

  /** Returns the vault key fingerprint. */
  vaultKeyFingerprint(key) {
    return this.operations.vaultKeyFingerprint(key?.nativeHandle ?? null);
  }

  /** Returns the vault key format hex. */
  vaultKeyFormatHex(bytes) {
    return this.operations.vaultKeyFormatHex(bytes);
  }

  /** Returns the vault key decode hex. */
  vaultKeyDecodeHex(text) {
    return this.operations.vaultKeyDecodeHex(text);
  }

  /** Returns the vault key format crockford. */
  vaultKeyFormatCrockford(bytes) {
    return this.operations.vaultKeyFormatCrockford(bytes);
  }

  /** Returns the vault key format crockford reading. */
  vaultKeyFormatCrockfordReading(code) {
    return this.operations.vaultKeyFormatCrockfordReading(code);
  }

  /** Returns the vault key decode crockford. */
  vaultKeyDecodeCrockford(code) {
    return this.operations.vaultKeyDecodeCrockford(code);
  }

  /** Returns the vault key hex encode. */
  vaultKeyHexEncode(bytes) {
    return this.operations.vaultKeyHexEncode(bytes);
  }

  /** Returns the vault key hex decode. */
  vaultKeyHexDecode(text) {
    return this.operations.vaultKeyHexDecode(text);
  }

  /** Returns the vault directory open. */
  vaultDirectoryOpen(root, password) {
    return new VaultDirectory(this.operations, this.operations.vaultDirectoryOpen(root, password));
  }

  /** Returns the vault structure version current. */
  vaultStructureVersionCurrent() {
    return this.operations.vaultStructureVersionCurrent();
  }

  /** Returns the vault directory probe structure version. */
  vaultDirectoryProbeStructureVersion(root, password) {
    return this.operations.vaultDirectoryProbeStructureVersion(root, password);
  }

  /** Returns the vault directory open or create default. */
  vaultDirectoryOpenOrCreateDefault(password) {
    return new VaultDirectory(this.operations, this.operations.vaultDirectoryOpenOrCreateDefault(password));
  }

  /** Returns the vault directory replace default. */
  vaultDirectoryReplaceDefault(password) {
    return new VaultDirectory(this.operations, this.operations.vaultDirectoryReplaceDefault(password));
  }

  /** Returns the vault directory change password. */
  vaultDirectoryChangePassword(root, oldPassword, newPassword) {
    return this.operations.vaultDirectoryChangePassword(root, oldPassword, newPassword);
  }

  /** Returns the vault directory change default password. */
  vaultDirectoryChangeDefaultPassword(oldPassword, newPassword) {
    return this.operations.vaultDirectoryChangeDefaultPassword(oldPassword, newPassword);
  }

  /** Returns the vault directory replace. */
  vaultDirectoryReplace(root, password) {
    return new VaultDirectory(this.operations, this.operations.vaultDirectoryReplace(root, password));
  }

  /** Returns the vault directory open or create. */
  vaultDirectoryOpenOrCreate(root, password) {
    return new VaultDirectory(this.operations, this.operations.vaultDirectoryOpenOrCreate(root, password));
  }

  /** Returns the vault backup default. */
  vaultBackupDefault(path, overwrite) {
    return this.operations.vaultBackupDefault(path, overwrite);
  }

  /** Returns the vault restore default. */
  vaultRestoreDefault(path, overwrite) {
    return this.operations.vaultRestoreDefault(path, overwrite);
  }

  /** Returns the vault read only open. */
  vaultReadOnlyOpen(root, password) {
    return new ReadOnlyVaultDirectory(this.operations, this.operations.vaultReadOnlyOpen(root, password));
  }

  /** Returns the vault read only open default. */
  vaultReadOnlyOpenDefault(password) {
    return new ReadOnlyVaultDirectory(this.operations, this.operations.vaultReadOnlyOpenDefault(password));
  }

  /** Returns the vault default directory. */
  vaultDefaultDirectory() {
    return this.operations.vaultDefaultDirectory();
  }

  /** Returns the vault default path. */
  vaultDefaultPath() {
    return this.operations.vaultDefaultPath();
  }

  /** Returns the vault agent log path. */
  vaultAgentLogPath() {
    return this.operations.vaultAgentLogPath();
  }

  /** Returns the vault agent log destination. */
  vaultAgentLogDestination() {
    return this.operations.vaultAgentLogDestination();
  }

  /** Returns the vault local. */
  vaultLocal() {
    return new LocalVault(this.operations, this.operations.vaultLocal());
  }

}

/** An open encrypted archive containing files, variables, secrets, and forms.
 * Commit pending changes and release it when finished with decrypted content. */
export class Lockbox extends OwnedHandle {
  /** Adds file. */
  addFile(path, data, replace) {
    return this.operations.lockboxAddFile(this.nativeHandle, path, data, replace);
  }

  /** Adds file with permissions. */
  addFileWithPermissions(path, data, permissions, replace) {
    return this.operations.lockboxAddFileWithPermissions(this.nativeHandle, path, data, permissions, replace);
  }

  /** Returns file. */
  getFile(path) {
    return this.operations.lockboxGetFile(this.nativeHandle, path);
  }

  /** Extracts file. */
  extractFile(source, destination, replace) {
    return this.operations.lockboxExtractFile(this.nativeHandle, source, destination, replace);
  }

  /** Extracts directory. */
  extractDirectory(destination, maxFileBytes, maxTotalBytes, maxFiles, restoreSymlinks, restorePermissions, overwrite) {
    return this.operations.lockboxExtractDirectory(this.nativeHandle, destination, maxFileBytes, maxTotalBytes, maxFiles, restoreSymlinks, restorePermissions, overwrite);
  }

  /** Returns the stream content. */
  streamContent(physical) {
    return this.operations.lockboxStreamContent(this.nativeHandle, physical);
  }

  /** Returns cache statistics for this lockbox. */
  cacheStats() {
    return this.operations.lockboxCacheStats(this.nativeHandle);
  }

  /** Returns import statistics for this lockbox. */
  importStats() {
    return this.operations.lockboxImportStats(this.nativeHandle);
  }

  /** Updates import stats. */
  resetImportStats() {
    return this.operations.lockboxResetImportStats(this.nativeHandle);
  }

  /** Returns the page inspection. */
  pageInspection() {
    return this.operations.lockboxPageInspection(this.nativeHandle);
  }

  /** Returns the recovery report. */
  recoveryReport() {
    return this.operations.lockboxRecoveryReport(this.nativeHandle);
  }

  /** Returns the recovery report render. */
  recoveryReportRender(verbose, maxEntries) {
    return this.operations.lockboxRecoveryReportRender(this.nativeHandle, verbose, maxEntries);
  }

  /** Returns the storage len. */
  storageLen() {
    return this.operations.lockboxStorageLen(this.nativeHandle);
  }

  /** Sets workload profile. */
  setWorkloadProfile(profile) {
    return this.operations.lockboxSetWorkloadProfile(this.nativeHandle, profile);
  }

  /** Sets worker policy. */
  setWorkerPolicy(mode, jobs) {
    return this.operations.lockboxSetWorkerPolicy(this.nativeHandle, mode, jobs);
  }

  /** Returns the runtime options. */
  runtimeOptions() {
    return this.operations.lockboxRuntimeOptions(this.nativeHandle);
  }

  /** Authenticates and publishes the staged changes. */
  commit() {
    return this.operations.lockboxCommit(this.nativeHandle);
  }

  /** Creates dir. */
  createDir(path, createParents) {
    return this.operations.lockboxCreateDir(this.nativeHandle, path, createParents);
  }

  /** Removes delete. */
  delete(path) {
    return this.operations.lockboxDelete(this.nativeHandle, path);
  }

  /** Removes dir. */
  removeDir(path, recursive) {
    return this.operations.lockboxRemoveDir(this.nativeHandle, path, recursive);
  }

  /** Creates parent dirs. */
  createParentDirs(path) {
    return this.operations.lockboxCreateParentDirs(this.nativeHandle, path);
  }

  /** Updates rename. */
  rename(from, to) {
    return this.operations.lockboxRename(this.nativeHandle, from, to);
  }

  /** Lists list. */
  list(path, recursive) {
    return this.operations.lockboxList(this.nativeHandle, path, recursive);
  }

  /** Lists with options. */
  listWithOptions(path, glob, recursive, includeFiles, includeSymlinks, includeDirectories, limit) {
    return this.operations.lockboxListWithOptions(this.nativeHandle, path, glob, recursive, includeFiles, includeSymlinks, includeDirectories, limit);
  }

  /** Returns metadata for the selected lockbox entry. */
  stat(path) {
    return this.operations.lockboxStat(this.nativeHandle, path);
  }

  /** Sets variable. */
  setVariable(name, value) {
    return this.operations.lockboxSetVariable(this.nativeHandle, name, value);
  }

  /** Sets secret variable. */
  setSecretVariable(name, value) {
    return this.operations.lockboxSetSecretVariable(this.nativeHandle, name, value);
  }

  /** Returns variable. */
  getVariable(name) {
    return this.operations.lockboxGetVariable(this.nativeHandle, name);
  }

  /** Returns the with secret variable. */
  withSecretVariable(name, callback) {
    return this.operations.lockboxWithSecretVariable(this.nativeHandle, name, callback);
  }

  /** Removes variable. */
  deleteVariable(name) {
    return this.operations.lockboxDeleteVariable(this.nativeHandle, name);
  }

  /** Updates variables. */
  moveVariables(moves) {
    return this.operations.lockboxMoveVariables(this.nativeHandle, encodePathMoves(moves));
  }

  /** Lists variables. */
  listVariables() {
    return this.operations.lockboxListVariables(this.nativeHandle);
  }

  /** Returns the variable sensitivity. */
  variableSensitivity(name) {
    return this.operations.lockboxVariableSensitivity(this.nativeHandle, name);
  }

  /** Adds symlink. */
  addSymlink(path, target, replace) {
    return this.operations.lockboxAddSymlink(this.nativeHandle, path, target, replace);
  }

  /** Returns symlink target. */
  getSymlinkTarget(path) {
    return this.operations.lockboxGetSymlinkTarget(this.nativeHandle, path);
  }

  /** Returns the id. */
  id() {
    return this.operations.lockboxId(this.nativeHandle);
  }

  /** Reports whether exists. */
  exists(path) {
    return this.operations.lockboxExists(this.nativeHandle, path);
  }

  /** Reports whether dir. */
  isDir(path) {
    return this.operations.lockboxIsDir(this.nativeHandle, path);
  }

  /** Returns the permissions. */
  permissions(path) {
    return this.operations.lockboxPermissions(this.nativeHandle, path);
  }

  /** Sets permissions. */
  setPermissions(path, permissions) {
    return this.operations.lockboxSetPermissions(this.nativeHandle, path, permissions);
  }

  /** Returns range. */
  readRange(path, offset, len) {
    return this.operations.lockboxReadRange(this.nativeHandle, path, offset, len);
  }

  /** Adds password. */
  addPassword(password) {
    return this.operations.lockboxAddPassword(this.nativeHandle, password);
  }

  /** Adds contact. */
  addContact(contact, name) {
    return this.operations.lockboxAddContact(this.nativeHandle, contact?.nativeHandle ?? null, name);
  }

  /** Removes key. */
  deleteKey(id) {
    return this.operations.lockboxDeleteKey(this.nativeHandle, id);
  }

  /** Lists key slots. */
  listKeySlots() {
    return this.operations.lockboxListKeySlots(this.nativeHandle);
  }

  /** Sets owner signing key. */
  setOwnerSigningKey(key) {
    return this.operations.lockboxSetOwnerSigningKey(this.nativeHandle, key?.nativeHandle ?? null);
  }

  /** Returns the owner inspection. */
  ownerInspection() {
    return this.operations.lockboxOwnerInspection(this.nativeHandle);
  }

  /** Returns the define form. */
  defineForm(alias, name, description, fields) {
    return this.operations.lockboxDefineForm(this.nativeHandle, alias, name, description, encodeFormFields(fields));
  }

  /** Lists form definitions. */
  listFormDefinitions() {
    return this.operations.lockboxListFormDefinitions(this.nativeHandle);
  }

  /** Returns the resolve form. */
  resolveForm(reference) {
    return this.operations.lockboxResolveForm(this.nativeHandle, reference);
  }

  /** Lists form revisions. */
  listFormRevisions(typeId) {
    return this.operations.lockboxListFormRevisions(this.nativeHandle, typeId);
  }

  /** Creates form record. */
  createFormRecord(path, typeReference, name) {
    return this.operations.lockboxCreateFormRecord(this.nativeHandle, path, typeReference, name);
  }

  /** Sets form field. */
  setFormField(path, field, value) {
    return this.operations.lockboxSetFormField(this.nativeHandle, path, field, value);
  }

  /** Sets secret form field. */
  setSecretFormField(path, field, value) {
    return this.operations.lockboxSetSecretFormField(this.nativeHandle, path, field, value);
  }

  /** Lists form records. */
  listFormRecords() {
    return this.operations.lockboxListFormRecords(this.nativeHandle);
  }

  /** Returns form record. */
  getFormRecord(path) {
    return this.operations.lockboxGetFormRecord(this.nativeHandle, path);
  }

  /** Removes form record. */
  deleteFormRecord(path) {
    return this.operations.lockboxDeleteFormRecord(this.nativeHandle, path);
  }

  /** Updates form records. */
  moveFormRecords(moves) {
    return this.operations.lockboxMoveFormRecords(this.nativeHandle, encodePathMoves(moves));
  }

  /** Returns form field. */
  getFormField(path, field) {
    return this.operations.lockboxGetFormField(this.nativeHandle, path, field);
  }

  /** Returns the with secret form field. */
  withSecretFormField(path, field, callback) {
    return this.operations.lockboxWithSecretFormField(this.nativeHandle, path, field, callback);
  }

  /** Returns the to bytes. */
  toBytes() {
    return this.operations.lockboxToBytes(this.nativeHandle);
  }

  /** Releases the native resources held by this object. */
  free() {
    this.operations.lockboxFree(this.nativeHandle);
    this.nativeHandle = null;
  }

}

/** A profile's contact-encryption identity, retained to decrypt content keys
 * addressed to the profile. */
export class ContactKeyPair extends OwnedHandle {
  /** Returns the public. */
  public() {
    return this.operations.keyContactPublic(this.nativeHandle);
  }

  /** Returns the private. */
  private() {
    return this.operations.keyContactPrivate(this.nativeHandle);
  }

  /** Releases the native resources held by this object. */
  free() {
    this.operations.keyContactFree(this.nativeHandle);
    this.nativeHandle = null;
  }

  /** Decrypts a wrapped content key for this contact. */
  decrypt(wrapped) {
    return this.operations.keyContactDecrypt(this.nativeHandle, wrapped?.nativeHandle ?? null);
  }

}

/** A recipient's shareable encryption identity, used when granting access. */
export class ContactPublicKey extends OwnedHandle {
  /** Returns the public free. */
  publicFree() {
    this.operations.keyContactPublicFree(this.nativeHandle);
    this.nativeHandle = null;
  }

  /** Encrypts a content key for the selected contact. */
  encrypt(contentKey) {
    return new WrappedContactKey(this.operations, this.operations.keyContactEncrypt(this.nativeHandle, contentKey));
  }

}

/** A content key encrypted for one contact and recoverable by its private key. */
export class WrappedContactKey extends OwnedHandle {
  /** Returns the public. */
  public() {
    return this.operations.keyContactWrappedPublic(this.nativeHandle);
  }

  /** Returns the ciphertext. */
  ciphertext() {
    return this.operations.keyContactWrappedCiphertext(this.nativeHandle);
  }

  /** Returns the encrypted. */
  encrypted() {
    return this.operations.keyContactWrappedEncrypted(this.nativeHandle);
  }

  /** Releases the native resources held by this object. */
  free() {
    this.operations.keyContactWrappedFree(this.nativeHandle);
    this.nativeHandle = null;
  }

}

/** A lockbox owner's signing identity, used to authorize mutable revisions. */
export class SigningKeyPair extends OwnedHandle {
  /** Returns the public. */
  public() {
    return this.operations.keySigningPublic(this.nativeHandle);
  }

  /** Returns the private. */
  private() {
    return this.operations.keySigningPrivate(this.nativeHandle);
  }

  /** Releases the native resources held by this object. */
  free() {
    this.operations.keySigningFree(this.nativeHandle);
    this.nativeHandle = null;
  }

}

/** The public identity readers use to verify owner-authorized revisions. */
export class SigningPublicKey extends OwnedHandle {
  /** Returns the public free. */
  publicFree() {
    this.operations.keySigningPublicFree(this.nativeHandle);
    this.nativeHandle = null;
  }

}

/** A password-protected local store for profile keys, contacts, forms, backups,
 * and remembered lockbox paths; it does not contain lockbox file contents. */
export class VaultDirectory extends OwnedHandle {
  /** Returns the root. */
  root() {
    return this.operations.vaultDirectoryRoot(this.nativeHandle);
  }

  /** Returns the structure version. */
  structureVersion() {
    return this.operations.vaultDirectoryStructureVersion(this.nativeHandle);
  }

  /** Lists private keys. */
  listPrivateKeys() {
    return this.operations.vaultDirectoryListPrivateKeys(this.nativeHandle);
  }

  /** Lists private key names. */
  listPrivateKeyNames() {
    return this.operations.vaultDirectoryListPrivateKeyNames(this.nativeHandle);
  }

  /** Lists contact names. */
  listContactNames() {
    return this.operations.vaultDirectoryListContactNames(this.nativeHandle);
  }

  /** Lists form aliases. */
  listFormAliases() {
    return this.operations.vaultDirectoryListFormAliases(this.nativeHandle);
  }

  /** Returns the private key exists. */
  privateKeyExists(name) {
    return this.operations.vaultDirectoryPrivateKeyExists(this.nativeHandle, name);
  }

  /** Removes private key. */
  deletePrivateKey(name) {
    return this.operations.vaultDirectoryDeletePrivateKey(this.nativeHandle, name);
  }

  /** Stores private key. */
  storePrivateKey(name, key) {
    return this.operations.vaultDirectoryStorePrivateKey(this.nativeHandle, name, key?.nativeHandle ?? null);
  }

  /** Loads private key. */
  loadPrivateKey(name) {
    return new ContactKeyPair(this.operations, this.operations.vaultDirectoryLoadPrivateKey(this.nativeHandle, name));
  }

  /** Loads private key generation. */
  loadPrivateKeyGeneration(name, index) {
    return new ContactKeyPair(this.operations, this.operations.vaultDirectoryLoadPrivateKeyGeneration(this.nativeHandle, name, index));
  }

  /** Stores contact. */
  storeContact(name, key) {
    return this.operations.vaultDirectoryStoreContact(this.nativeHandle, name, key?.nativeHandle ?? null);
  }

  /** Loads contact. */
  loadContact(name) {
    return new ContactPublicKey(this.operations, this.operations.vaultDirectoryLoadContact(this.nativeHandle, name));
  }

  /** Returns the contact exists. */
  contactExists(name) {
    return this.operations.vaultDirectoryContactExists(this.nativeHandle, name);
  }

  /** Removes contact. */
  deleteContact(name) {
    return this.operations.vaultDirectoryDeleteContact(this.nativeHandle, name);
  }

  /** Lists contacts. */
  listContacts() {
    return this.operations.vaultDirectoryListContacts(this.nativeHandle);
  }

  /** Stores profile email. */
  storeProfileEmail(name, email) {
    return this.operations.vaultDirectoryStoreProfileEmail(this.nativeHandle, name, email);
  }

  /** Returns the profile email. */
  profileEmail(name) {
    return this.operations.vaultDirectoryProfileEmail(this.nativeHandle, name);
  }

  /** Stores backup. */
  storeBackup(id, bytes) {
    return this.operations.vaultDirectoryStoreBackup(this.nativeHandle, id, bytes);
  }

  /** Loads backup. */
  loadBackup(id) {
    return this.operations.vaultDirectoryLoadBackup(this.nativeHandle, id);
  }

  /** Returns the backup count. */
  backupCount() {
    return this.operations.vaultDirectoryBackupCount(this.nativeHandle);
  }

  /** Returns the restore private key. */
  restorePrivateKey(name, key, signingKey, overwrite) {
    return this.operations.vaultDirectoryRestorePrivateKey(this.nativeHandle, name, key?.nativeHandle ?? null, signingKey?.nativeHandle ?? null, overwrite);
  }

  /** Loads owner signing key. */
  loadOwnerSigningKey(name) {
    return new SigningKeyPair(this.operations, this.operations.vaultDirectoryLoadOwnerSigningKey(this.nativeHandle, name));
  }

  /** Loads owner signing key generation. */
  loadOwnerSigningKeyGeneration(name, index) {
    return new SigningKeyPair(this.operations, this.operations.vaultDirectoryLoadOwnerSigningKeyGeneration(this.nativeHandle, name, index));
  }

  /** Stores contact signing key. */
  storeContactSigningKey(name, key) {
    return this.operations.vaultDirectoryStoreContactSigningKey(this.nativeHandle, name, key?.nativeHandle ?? null);
  }

  /** Loads contact signing key. */
  loadContactSigningKey(name) {
    return new SigningPublicKey(this.operations, this.operations.vaultDirectoryLoadContactSigningKey(this.nativeHandle, name));
  }

  /** Lists profile generations. */
  listProfileGenerations(name) {
    return this.operations.vaultDirectoryListProfileGenerations(this.nativeHandle, name);
  }

  /** Updates private key. */
  rotatePrivateKey(name) {
    return this.operations.vaultDirectoryRotatePrivateKey(this.nativeHandle, name);
  }

  /** Stores lockbox. */
  rememberLockbox(id, path) {
    return this.operations.vaultDirectoryRememberLockbox(this.nativeHandle, id, path);
  }

  /** Lists known lockboxes. */
  listKnownLockboxes() {
    return this.operations.vaultDirectoryListKnownLockboxes(this.nativeHandle);
  }

  /** Removes lockbox. */
  forgetLockbox(path) {
    return this.operations.vaultDirectoryForgetLockbox(this.nativeHandle, path);
  }

  /** Stores access slot label. */
  rememberAccessSlotLabel(id, slotId, name) {
    return this.operations.vaultDirectoryRememberAccessSlotLabel(this.nativeHandle, id, slotId, name);
  }

  /** Lists access slot labels. */
  listAccessSlotLabels(id) {
    return this.operations.vaultDirectoryListAccessSlotLabels(this.nativeHandle, id);
  }

  /** Returns the find access slot labels. */
  findAccessSlotLabels(id, name) {
    return this.operations.vaultDirectoryFindAccessSlotLabels(this.nativeHandle, id, name);
  }

  /** Removes access slot label. */
  forgetAccessSlotLabel(id, slotId) {
    return this.operations.vaultDirectoryForgetAccessSlotLabel(this.nativeHandle, id, slotId);
  }

  /** Returns the define form. */
  defineForm(alias, name, description, fields) {
    return this.operations.vaultDirectoryDefineForm(this.nativeHandle, alias, name, description, encodeFormFields(fields));
  }

  /** Returns the resolve form. */
  resolveForm(reference) {
    return this.operations.vaultDirectoryResolveForm(this.nativeHandle, reference);
  }

  /** Lists forms. */
  listForms() {
    return this.operations.vaultDirectoryListForms(this.nativeHandle);
  }

  /** Lists form revisions. */
  listFormRevisions(typeId) {
    return this.operations.vaultDirectoryListFormRevisions(this.nativeHandle, typeId);
  }

  /** Returns the seed forms. */
  seedForms() {
    return this.operations.vaultDirectorySeedForms(this.nativeHandle);
  }

  /** Stores password. */
  rememberPassword(id, password) {
    return this.operations.vaultDirectoryRememberPassword(this.nativeHandle, id, password);
  }

  /** Returns the remembered password. */
  rememberedPassword(id) {
    return this.operations.vaultDirectoryRememberedPassword(this.nativeHandle, id);
  }

  /** Releases the native resources held by this object. */
  free() {
    this.operations.vaultDirectoryFree(this.nativeHandle);
    this.nativeHandle = null;
  }

}

/** A restricted local metadata view for discovery without signing-key access. */
export class ReadOnlyVaultDirectory extends OwnedHandle {
  /** Lists profile names. */
  listProfileNames() {
    return this.operations.vaultReadOnlyListProfileNames(this.nativeHandle);
  }

  /** Lists contact names. */
  listContactNames() {
    return this.operations.vaultReadOnlyListContactNames(this.nativeHandle);
  }

  /** Lists form aliases. */
  listFormAliases() {
    return this.operations.vaultReadOnlyListFormAliases(this.nativeHandle);
  }

  /** Lists known lockboxes. */
  listKnownLockboxes() {
    return this.operations.vaultReadOnlyListKnownLockboxes(this.nativeHandle);
  }

  /** Releases the native resources held by this object. */
  free() {
    return this.operations.vaultReadOnlyFree(this.nativeHandle);
  }

}

/** Client for the session service that temporarily caches vault unlock and
 * owner signing keys across application operations. */
export class Agent {
  /** Creates a new facade over the bundled native library. */
  constructor(operations) { this.operations = operations; }

  /** Reports whether running. */
  isRunning() {
    return this.operations.vaultIsRunning();
  }

  /** Removes all. */
  forgetAll() {
    return this.operations.vaultForgetAll();
  }

  /** Returns the serve. */
  serve() {
    return this.operations.vaultAgentServe();
  }

  /** Verifies transport. */
  verifyTransport() {
    return this.operations.vaultAgentVerifyTransport();
  }

  /** Returns get. */
  get(id) {
    return this.operations.vaultAgentGet(id);
  }

  /** Stores put. */
  put(id, key) {
    return this.operations.vaultAgentPut(id, key);
  }

  /** Removes forget. */
  forget(id) {
    return this.operations.vaultAgentForget(id);
  }

  /** Stops stop. */
  stop() {
    return this.operations.vaultAgentStop();
  }

  /** Starts start. */
  start() {
    return this.operations.vaultAgentStart();
  }

  /** Lists list. */
  list() {
    return this.operations.vaultAgentList();
  }

  /** Returns the sleep support. */
  sleepSupport() {
    return this.operations.vaultAgentSleepSupport();
  }

  /** Returns vault unlock key. */
  getVaultUnlockKey(vaultId) {
    return this.operations.vaultAgentGetVaultUnlockKey(vaultId);
  }

  /** Stores vault unlock key. */
  putVaultUnlockKey(vaultId, key, ttlSeconds) {
    return this.operations.vaultAgentPutVaultUnlockKey(vaultId, key, ttlSeconds);
  }

  /** Removes vault unlock key. */
  forgetVaultUnlockKey(vaultId) {
    return this.operations.vaultAgentForgetVaultUnlockKey(vaultId);
  }

  /** Returns owner signing key. */
  getOwnerSigningKey(vaultId, profile) {
    return new SigningKeyPair(this.operations, this.operations.vaultAgentGetOwnerSigningKey(vaultId, profile));
  }

  /** Stores owner signing key. */
  putOwnerSigningKey(vaultId, profile, key, ttlSeconds) {
    return this.operations.vaultAgentPutOwnerSigningKey(vaultId, profile, key?.nativeHandle ?? null, ttlSeconds);
  }

  /** Removes owner signing key. */
  forgetOwnerSigningKey(vaultId, profile) {
    return this.operations.vaultAgentForgetOwnerSigningKey(vaultId, profile);
  }

  /** Starts activity. */
  beginActivity(kind) {
    return new AgentActivity(this.operations, this.operations.vaultAgentBeginActivity(kind));
  }

  /** Stops activity. */
  endActivity(handle) {
    return this.operations.vaultAgentEndActivity(handle?.nativeHandle ?? null);
  }

}

/** A token kept alive while an operation needs secrets cached by the agent. */
export class AgentActivity extends OwnedHandle {
}

/** Access to operating-system credential storage for a scoped vault password. */
export class Platform {
  /** Creates a new facade over the bundled native library. */
  constructor(operations) { this.operations = operations; }

  /** Returns the status. */
  status() {
    return this.operations.vaultPlatformStatus();
  }

  /** Sets scope. */
  setScope(scope) {
    return this.operations.vaultPlatformSetScope(scope);
  }

  /** Removes password. */
  forgetPassword() {
    return this.operations.vaultPlatformForgetPassword();
  }

  /** Stores password. */
  putPassword(password) {
    return this.operations.vaultPlatformPutPassword(password);
  }

  /** Returns the enable. */
  enable() {
    return this.operations.vaultPlatformEnable();
  }

  /** Returns the disable. */
  disable() {
    return this.operations.vaultPlatformDisable();
  }

  /** Returns the disabled. */
  disabled() {
    return this.operations.vaultPlatformDisabled();
  }

  /** Returns password. */
  getPassword() {
    return this.operations.vaultPlatformGetPassword();
  }

}

/** A session for opening lockboxes by host path, caching short-lived passwords,
 * and committing and closing locally used lockbox files. */
export class LocalVault extends OwnedHandle {
  /** Creates lockbox password. */
  createLockboxPassword(path, password) {
    return new Lockbox(this.operations, this.operations.vaultCreateLockboxPassword(this.nativeHandle, path, password));
  }

  /** Opens lockbox password. */
  openLockboxPassword(path, password) {
    return new Lockbox(this.operations, this.operations.vaultOpenLockboxPassword(this.nativeHandle, path, password));
  }

  /** Creates lockbox content key. */
  createLockboxContentKey(path, contentKey, signingKey) {
    return new Lockbox(this.operations, this.operations.vaultCreateLockboxContentKey(this.nativeHandle, path, contentKey, signingKey?.nativeHandle ?? null));
  }

  /** Creates lockbox contact. */
  createLockboxContact(path, contact, name, signingKey) {
    return new Lockbox(this.operations, this.operations.vaultCreateLockboxContact(this.nativeHandle, path, contact?.nativeHandle ?? null, name, signingKey?.nativeHandle ?? null));
  }

  /** Opens lockbox content key. */
  openLockboxContentKey(path, contentKey, signingKey) {
    return new Lockbox(this.operations, this.operations.vaultOpenLockboxContentKey(this.nativeHandle, path, contentKey, signingKey?.nativeHandle ?? null));
  }

  /** Stores lockbox password. */
  cacheLockboxPassword(path, password, ttlSeconds) {
    return this.operations.vaultCacheLockboxPassword(this.nativeHandle, path, password, ttlSeconds);
  }

  /** Releases the native resources held by lockbox. */
  closeLockbox(path) {
    return this.operations.vaultCloseLockbox(this.nativeHandle, path);
  }

  /** Releases the native resources held by all. */
  closeAll() {
    return this.operations.vaultCloseAll(this.nativeHandle);
  }

  /** Releases the native resources held by this object. */
  free() {
    this.operations.vaultFree(this.nativeHandle);
    this.nativeHandle = null;
  }

}
