/**
 * Owned JavaScript API for encrypted reVault lockboxes and local vaults.
 * See https://github.com/onepub-dev/reVault#readme for installation, security
 * guidance, and complete examples.
 * @module @onepub-dev/revault-api
 */
import { BindingOperations, RevaultError, createMessage, encodeMessage } from './native.js';
import fs from 'node:fs';

/** Error raised when a native operation is rejected; details preserve ABI diagnostics. */
export { RevaultError };
/** Cache policies for an open Lockbox. */
export const LockboxCacheMode = Object.freeze({ BYTES: 'bytes', DISABLED: 'disabled', AUTOMATIC: 'automatic' });
/** Workload profiles used to tune archive operations. */
export const LockboxWorkload = Object.freeze({ INTERACTIVE: 'interactive', BULK_IMPORT: 'bulk-import', READ_MOSTLY: 'read-mostly' });
/** Worker policies used by archive operations. */
export const LockboxWorker = Object.freeze({ AUTO: 'auto', SINGLE: 'single', THREADS: 'threads' });
/** Categories recorded by the explicit Session Agent. */
export const AgentActivityKind = Object.freeze({ OPEN: 'open', CLOSE: 'close', VARIABLES: 'variables', FORM: 'form', RECOVERY: 'recovery', VAULT: 'vault' });
/** Stable key export encodings. */
export const KeyExportFormat = Object.freeze({ LOCKBOX_PEM: 'lockbox-pem', JWK: 'jwk', JWKS: 'jwks', RAW_HEX: 'raw-hex' });

/** Mutable UTF-8 secret owned by the caller and wiped by close(). */
export class SecretBytes extends Uint8Array {
  /** Wipe the mutable secret buffer in place. */
  close() { this.fill(0); }
}
/** Mutable UTF-8 password owned by the caller and wiped by close(). */
export class SecretString extends SecretBytes {
  /** Copy a UTF-8 passphrase into a mutable, wipeable buffer. */
  constructor(value) { super(Buffer.from(value, 'utf8')); }
  /** Decode the current UTF-8 value for a native call. */
  toString() { return Buffer.from(this).toString('utf8'); }
}

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
  /** Release this handle when its concrete type supplies a native free method. */
  close() { if (this.nativeHandle != null && typeof this.free === 'function') this.free(); }
}

/** Primary API used to open lockboxes, manage keys and metadata, use the
 * Session Agent, and access the platform credential store. */
/** Native runtime loader and archive/key factory. */
export class Revault {
  /** Load an explicit, inherited, or installed native carrier asynchronously. */
  static async load(nativeLibraryPath = undefined) { return new Revault(nativeLibraryPath); }
  /** Return a synchronous runtime facade for factory operations. */
  static get runtime() { return new Revault(); }
  /** Creates a new facade over the bundled native library. */
  constructor(nativeLibraryPath = undefined) { this.operations = new BindingOperations(nativeLibraryPath); this.agent = new Agent(this.operations); this.platform = new Platform(this.operations); }
  /** Returns the last error. */
  lastError() { return this.operations.lastErrorMessage(); }
  /** Returns the last error details. */
  lastErrorDetails() { return this.operations.bufferLastErrorDetails(); }

  /** Returns the newest Lockbox archive format version supported by this engine. */
  lockboxFormatVersion() {
    return this.operations.lockboxFormatVersion();
  }

  /** Reads the format version from serialized Lockbox bytes without opening them. */
  lockboxProbeFormatVersion(bytes) {
    return this.operations.lockboxProbeFormatVersion(bytes);
  }

  /** Creates an in memory Lockbox protected by a 32 byte content key. */
  lockboxCreate(key) {
    return new Lockbox(this.operations, this.operations.lockboxCreate(key));
  }

  /** Creates a lockbox with explicit cache capacity, workload, worker policy, and job count. */
  lockboxCreateWithOptions(key, cacheMode, cacheBytes, workload, worker, jobs) {
    return new Lockbox(this.operations, this.operations.lockboxCreateWithOptions(key, cacheMode, cacheBytes, workload, worker, jobs));
  }

  /** Creates an in memory Lockbox protected by the supplied password. */
  lockboxCreatePassword(password) {
    return new Lockbox(this.operations, this.operations.lockboxCreatePassword(password));
  }

  /** Creates an in memory Lockbox that the supplied contact can open. */
  lockboxCreateContact(contact) {
    return new Lockbox(this.operations, this.operations.lockboxCreateContact(contact?.nativeHandle ?? null));
  }

  /** Creates an in memory Lockbox and assigns its profile signing key. */
  lockboxCreateWithSigningKey(contentKey, signingKey) {
    return new Lockbox(this.operations, this.operations.lockboxCreateWithSigningKey(contentKey, signingKey?.nativeHandle ?? null));
  }

  /** Opens serialized Lockbox bytes with a 32 byte content key. */
  lockboxOpen(archive, key) {
    return new Lockbox(this.operations, this.operations.lockboxOpen(archive, key));
  }

  /** Opens a lockbox with explicit cache capacity, workload, worker policy, and job count. */
  lockboxOpenWithOptions(archive, key, cacheMode, cacheBytes, workload, worker, jobs) {
    return new Lockbox(this.operations, this.operations.lockboxOpenWithOptions(archive, key, cacheMode, cacheBytes, workload, worker, jobs));
  }

  /** Opens serialized Lockbox bytes with the supplied password. */
  lockboxOpenPassword(archive, password) {
    return new Lockbox(this.operations, this.operations.lockboxOpenPassword(archive, password));
  }

  /** Opens serialized Lockbox bytes with the supplied contact private key. */
  lockboxOpenContact(archive, contact) {
    return new Lockbox(this.operations, this.operations.lockboxOpenContact(archive, contact?.nativeHandle ?? null));
  }

  /** Reads public header, signature, and access slot metadata from a Lockbox file. */
  lockboxInspectFile(path) {
    return this.operations.lockboxInspectFile(path);
  }

  /** Scans a damaged Lockbox file with its 32 byte content key. */
  lockboxRecoveryScanPath(path, key) {
    return this.operations.lockboxRecoveryScanPath(path, key);
  }

  /** Scans damaged serialized Lockbox bytes with their 32 byte content key. */
  lockboxRecoveryScan(bytes, key) {
    return this.operations.lockboxRecoveryScan(bytes, key);
  }

  /** Builds a new Lockbox from recoverable records without changing the source. */
  lockboxRecoverySalvage(bytes, key, signingKey) {
    return new Lockbox(this.operations, this.operations.lockboxRecoverySalvage(bytes, key, signingKey?.nativeHandle ?? null));
  }

  /** Generates a contact encryption key pair using secure random data. */
  keyContactGenerate() {
    return new ContactKeyPair(this.operations, this.operations.keyContactGenerate());
  }

  /** Imports a contact key pair from its private binary record. */
  keyContactFromPrivate(bytes) {
    return new ContactKeyPair(this.operations, this.operations.keyContactFromPrivate(bytes));
  }

  /** Imports a contact public key from its binary representation. */
  keyContactPublicFromBytes(bytes) {
    return new ContactPublicKey(this.operations, this.operations.keyContactPublicFromBytes(bytes));
  }

  /** Generates a signing identity owned by a Vault Profile. */
  generateProfileSigningKeyPair() {
    return new ProfileSigningKeyPair(this.operations, this.operations.keySigningGenerate());
  }

  /** Imports a Vault Profile signing identity from its private record. */
  profileSigningKeyPairFromPrivate(bytes) {
    return new ProfileSigningKeyPair(this.operations, this.operations.keySigningFromPrivate(bytes));
  }

  /** Imports the public half of a Vault Profile signing identity. */
  profileSigningPublicKeyFromBytes(bytes) {
    return new ProfileSigningPublicKey(this.operations, this.operations.keySigningPublicFromBytes(bytes));
  }

  /** Exports a private key in the requested key format. */
  vaultKeyExportPrivate(key, format) {
    return this.operations.vaultKeyExportPrivate(key?.nativeHandle ?? null, format);
  }

  /** Exports a public key in the requested key format. */
  vaultKeyExportPublic(key, format) {
    return this.operations.vaultKeyExportPublic(key?.nativeHandle ?? null, format);
  }

  /** Imports a private contact key from a detected supported encoding. */
  vaultKeyImportPrivate(bytes) {
    return new ContactKeyPair(this.operations, this.operations.vaultKeyImportPrivate(bytes));
  }

  /** Imports a public contact key from a detected supported encoding. */
  vaultKeyImportPublic(bytes) {
    return new ContactPublicKey(this.operations, this.operations.vaultKeyImportPublic(bytes));
  }

  /** Returns the stable fingerprint used to verify a public key. */
  vaultKeyFingerprint(key) {
    return this.operations.vaultKeyFingerprint(key?.nativeHandle ?? null);
  }

  /** Encodes key bytes as hexadecimal text. */
  vaultKeyFormatHex(bytes) {
    return this.operations.vaultKeyFormatHex(bytes);
  }

  /** Decodes hexadecimal key text and rejects malformed input. */
  vaultKeyDecodeHex(text) {
    return this.operations.vaultKeyDecodeHex(text);
  }

  /** Encodes key bytes using Crockford Base32. */
  vaultKeyFormatCrockford(bytes) {
    return this.operations.vaultKeyFormatCrockford(bytes);
  }

  /** Groups a Crockford code for easier reading and transcription. */
  vaultKeyFormatCrockfordReading(code) {
    return this.operations.vaultKeyFormatCrockfordReading(code);
  }

  /** Decodes Crockford Base32 key text and rejects malformed input. */
  vaultKeyDecodeCrockford(code) {
    return this.operations.vaultKeyDecodeCrockford(code);
  }

  /** Encodes arbitrary bytes as hexadecimal text. */
  vaultKeyHexEncode(bytes) {
    return this.operations.vaultKeyHexEncode(bytes);
  }

  /** Decodes arbitrary hexadecimal text and rejects malformed input. */
  vaultKeyHexDecode(text) {
    return this.operations.vaultKeyHexDecode(text);
  }

  /** Opens an existing Vault directory with its passphrase. */
  vaultDirectoryOpen(root, password) {
    return new Vault(this.operations, this.operations.vaultDirectoryOpen(root, password));
  }

  /** Returns the newest Vault structure version supported by this engine. */
  vaultStructureVersionCurrent() {
    return this.operations.vaultStructureVersionCurrent();
  }

  /** Reads an existing Vault structure version without changing it. */
  vaultDirectoryProbeStructureVersion(root, password) {
    return this.operations.vaultDirectoryProbeStructureVersion(root, password);
  }

  /** Opens or creates the default Vault without replacing existing state. */
  vaultDirectoryOpenOrCreateDefault(password) {
    return new Vault(this.operations, this.operations.vaultDirectoryOpenOrCreateDefault(password));
  }

  /** Replaces the default Vault and all persistent data it contains. */
  vaultDirectoryReplaceDefault(password) {
    return new Vault(this.operations, this.operations.vaultDirectoryReplaceDefault(password));
  }

  /** Changes the passphrase for an existing Vault. */
  vaultDirectoryChangePassword(root, oldPassword, newPassword) {
    return this.operations.vaultDirectoryChangePassword(root, oldPassword, newPassword);
  }

  /** Changes the passphrase for the default Vault. */
  vaultDirectoryChangeDefaultPassword(oldPassword, newPassword) {
    return this.operations.vaultDirectoryChangeDefaultPassword(oldPassword, newPassword);
  }

  /** Replaces the selected Vault and all persistent data it contains. */
  vaultDirectoryReplace(root, password) {
    return new Vault(this.operations, this.operations.vaultDirectoryReplace(root, password));
  }

  /** Opens the selected Vault, creating it only when absent. */
  vaultDirectoryOpenOrCreate(root, password) {
    return new Vault(this.operations, this.operations.vaultDirectoryOpenOrCreate(root, password));
  }

  /** Writes a backup of the default Vault to the selected path. */
  vaultBackupDefault(path, overwrite) {
    return this.operations.vaultBackupDefault(path, overwrite);
  }

  /** Restores the default Vault from the selected backup. */
  vaultRestoreDefault(path, overwrite) {
    return this.operations.vaultRestoreDefault(path, overwrite);
  }

  /** Opens an existing Vault metadata view that cannot load private keys. */
  vaultReadOnlyOpen(root, password) {
    return new ReadOnlyVault(this.operations, this.operations.vaultReadOnlyOpen(root, password));
  }

  /** Opens the default Vault metadata view without loading private keys. */
  vaultReadOnlyOpenDefault(password) {
    return new ReadOnlyVault(this.operations, this.operations.vaultReadOnlyOpenDefault(password));
  }

  /** Returns the platform default Vault directory. */
  vaultDefaultDirectory() {
    return this.operations.vaultDefaultDirectory();
  }

  /** Returns the path of the default Vault file. */
  vaultDefaultPath() {
    return this.operations.vaultDefaultPath();
  }

  /** Returns the session agent log path. */
  vaultAgentLogPath() {
    return this.operations.vaultAgentLogPath();
  }

  /** Returns the configured session agent log destination. */
  vaultAgentLogDestination() {
    return this.operations.vaultAgentLogDestination();
  }

}

/** An open encrypted archive containing files, variables, secrets, and forms.
 * Commit pending changes and release it when finished with decrypted content. */
export class Lockbox extends OwnedHandle {
  /** Create an in-memory archive using exactly one credential. */
  static createInMemory({ password, contentKey, contact, signingKey, options } = {}) {
    const credentials = [password, contentKey, contact].filter((value) => value != null);
    if (credentials.length !== 1) throw new TypeError('Supply exactly one of password, contentKey, or contact.');
    const runtime = Revault.runtime;
    let lockbox;
    if (password != null) lockbox = runtime.lockboxCreatePassword(password);
    else if (contact != null) lockbox = runtime.lockboxCreateContact(contact);
    else if (options != null) lockbox = runtime.lockboxCreateWithOptions(contentKey, options.cacheMode, options.cacheBytes ?? 0, options.workload, options.worker, options.jobs ?? 0);
    else lockbox = runtime.lockboxCreate(contentKey);
    if (signingKey != null) lockbox.setOwnerSigningKey(signingKey);
    return lockbox;
  }

  /** Open serialized archive bytes with exactly one credential. */
  static openBytes(archive, { password, contentKey, contact, options } = {}) {
    const credentials = [password, contentKey, contact].filter((value) => value != null);
    if (credentials.length !== 1) throw new TypeError('Supply exactly one of password, contentKey, or contact.');
    const runtime = Revault.runtime;
    if (password != null) return runtime.lockboxOpenPassword(archive, password);
    if (contact != null) return runtime.lockboxOpenContact(archive, contact);
    return options == null
      ? runtime.lockboxOpen(archive, contentKey)
      : runtime.lockboxOpenWithOptions(archive, contentKey, options.cacheMode, options.cacheBytes ?? 0, options.workload, options.worker, options.jobs ?? 0);
  }

  /** Create an archive file and return its process-local handle. */
  static create(path, options = {}) {
    if (fs.existsSync(path) && !options.overwrite) throw new Error(`Lockbox already exists: ${path}`);
    const lockbox = Lockbox.createInMemory(options);
    fs.writeFileSync(path, lockbox.toBytes());
    lockbox._backingPath = path;
    return lockbox;
  }

  /** Open an archive file without consulting the Session Agent. */
  static open(path, options = {}) {
    const lockbox = Lockbox.openBytes(fs.readFileSync(path), options);
    lockbox._backingPath = path;
    return lockbox;
  }

  /** Stages a file at the Lockbox path; replace controls an existing entry. */
  addFile(path, data, replace) {
    return this.operations.lockboxAddFile(this.nativeHandle, path, data, replace);
  }

  /** Stages a file and its portable Unix permission bits. */
  addFileWithPermissions(path, data, permissions, replace) {
    return this.operations.lockboxAddFileWithPermissions(this.nativeHandle, path, data, permissions, replace);
  }

  /** Reads the complete file stored at the Lockbox path. */
  getFile(path) {
    return this.operations.lockboxGetFile(this.nativeHandle, path);
  }

  /** Writes one Lockbox file to the host filesystem. */
  extractFile(source, destination, replace) {
    return this.operations.lockboxExtractFile(this.nativeHandle, source, destination, replace);
  }

  /** Extracts the Lockbox with explicit size, count, link, and permission limits. */
  extractDirectory(destination, maxFileBytes, maxTotalBytes, maxFiles, restoreSymlinks, restorePermissions, overwrite) {
    return this.operations.lockboxExtractDirectory(this.nativeHandle, destination, maxFileBytes, maxTotalBytes, maxFiles, restoreSymlinks, restorePermissions, overwrite);
  }

  /** Lists logical or physical content chunks for streaming diagnostics. */
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

  /** Returns page metadata for diagnostics without exposing plaintext secrets. */
  pageInspection() {
    return this.operations.lockboxPageInspection(this.nativeHandle);
  }

  /** Scans the open archive and returns its structured recovery report. */
  recoveryReport() {
    return this.operations.lockboxRecoveryReport(this.nativeHandle);
  }

  /** Renders the recovery report for a person, capped at maxEntries. */
  recoveryReportRender(verbose, maxEntries) {
    return this.operations.lockboxRecoveryReportRender(this.nativeHandle, verbose, maxEntries);
  }

  /** Returns the current serialized archive size in bytes. */
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

  /** Returns the cache, workload, and worker settings used by this Lockbox. */
  runtimeOptions() {
    return this.operations.lockboxRuntimeOptions(this.nativeHandle);
  }

  /** Authenticates and publishes the staged changes. */
  commit() {
    const result = this.operations.lockboxCommit(this.nativeHandle);
    // The byte-oriented factory keeps a process-local handle backed by the
    // requested host path. Persist only after native authentication succeeds.
    if (this._backingPath != null) fs.writeFileSync(this._backingPath, this.toBytes());
    return result;
  }

  /** Stages a directory entry and optionally creates missing parents. */
  createDir(path, createParents) {
    return this.operations.lockboxCreateDir(this.nativeHandle, path, createParents);
  }

  /** Stages removal of a file, link, or empty directory at path. */
  delete(path) {
    return this.operations.lockboxDelete(this.nativeHandle, path);
  }

  /** Stages removal of a directory, optionally including its descendants. */
  removeDir(path, recursive) {
    return this.operations.lockboxRemoveDir(this.nativeHandle, path, recursive);
  }

  /** Stages every missing parent directory for path. */
  createParentDirs(path) {
    return this.operations.lockboxCreateParentDirs(this.nativeHandle, path);
  }

  /** Stages an atomic move from one Lockbox path to another. */
  rename(from, to) {
    return this.operations.lockboxRename(this.nativeHandle, from, to);
  }

  /** Lists entries below path, optionally including descendants. */
  list(path, recursive) {
    return this.operations.lockboxList(this.nativeHandle, path, recursive);
  }

  /** Lists entries using glob, type, recursion, and result limit filters. */
  listWithOptions(path, glob, recursive, includeFiles, includeSymlinks, includeDirectories, limit) {
    return this.operations.lockboxListWithOptions(this.nativeHandle, path, glob, recursive, includeFiles, includeSymlinks, includeDirectories, limit);
  }

  /** Returns metadata for the selected lockbox entry. */
  stat(path) {
    return this.operations.lockboxStat(this.nativeHandle, path);
  }

  /** Stages a plain text variable; commit to publish the change. */
  setVariable(name, value) {
    return this.operations.lockboxSetVariable(this.nativeHandle, name, value);
  }

  /** Sets secret variable. */
  setSecretVariable(name, value) {
    return this.operations.lockboxSetSecretVariable(this.nativeHandle, name, value);
  }

  /** Returns a plain variable when it is present. */
  getVariable(name) {
    return this.operations.lockboxGetVariable(this.nativeHandle, name);
  }

  /** Returns the encrypted Lockbox description, or undefined when unset.
   * Example lifecycle: set "Production credentials", commit, then print this property.
   */
  get description() {
    return this.getVariable('/.revault/description');
  }

  /** Stages an encrypted Lockbox description; call commit to publish it.
   * Example lifecycle: set "Production credentials", then commit the box.
   */
  setDescription(description) {
    return this.setVariable('/.revault/description', description);
  }

  /** Stages removal of the encrypted Lockbox description; call commit.
   * Example lifecycle: clear the description, then commit the box.
   */
  clearDescription() {
    return this.deleteVariable('/.revault/description');
  }

  /** Invokes a callback with temporary secret variable bytes, then clears them. */
  withSecretVariable(name, callback) {
    return this.operations.lockboxWithSecretVariable(this.nativeHandle, name, callback);
  }

  /** Stages removal of a variable. */
  deleteVariable(name) {
    return this.operations.lockboxDeleteVariable(this.nativeHandle, name);
  }

  /** Updates variables. */
  moveVariables(moves) {
    return this.operations.lockboxMoveVariables(this.nativeHandle, encodePathMoves(moves));
  }

  /** Lists variable names and metadata without exposing secret values. */
  listVariables() {
    return this.operations.lockboxListVariables(this.nativeHandle);
  }

  /** Returns whether a variable is plain or secret. */
  variableSensitivity(name) {
    return this.operations.lockboxVariableSensitivity(this.nativeHandle, name);
  }

  /** Stages a symbolic link with its stored target text. */
  addSymlink(path, target, replace) {
    return this.operations.lockboxAddSymlink(this.nativeHandle, path, target, replace);
  }

  /** Returns the target text stored for a symbolic link. */
  getSymlinkTarget(path) {
    return this.operations.lockboxGetSymlinkTarget(this.nativeHandle, path);
  }

  /** Returns the stable public identifier stored in the Lockbox header. */
  id() {
    return this.operations.lockboxId(this.nativeHandle);
  }

  /** Reports whether an entry exists at path. */
  exists(path) {
    return this.operations.lockboxExists(this.nativeHandle, path);
  }

  /** Reports whether path names a directory entry. */
  isDir(path) {
    return this.operations.lockboxIsDir(this.nativeHandle, path);
  }

  /** Returns the portable Unix permission bits stored for path. */
  permissions(path) {
    return this.operations.lockboxPermissions(this.nativeHandle, path);
  }

  /** Stages portable Unix permission bits for path. */
  setPermissions(path, permissions) {
    return this.operations.lockboxSetPermissions(this.nativeHandle, path, permissions);
  }

  /** Reads the requested byte range from a stored file. */
  readRange(path, offset, len) {
    return this.operations.lockboxReadRange(this.nativeHandle, path, offset, len);
  }

  /** Adds a password access slot and returns its slot identifier. */
  addPassword(password) {
    return this.operations.lockboxAddPassword(this.nativeHandle, password);
  }

  /** Grants a named contact access and returns the new slot identifier. */
  addContact(contact, name) {
    return this.operations.lockboxAddContact(this.nativeHandle, contact?.nativeHandle ?? null, name);
  }

  /** Removes an access slot; at least one usable slot must remain. */
  deleteKey(id) {
    return this.operations.lockboxDeleteKey(this.nativeHandle, id);
  }

  /** Lists public access slot metadata without returning credentials. */
  listKeySlots() {
    return this.operations.lockboxListKeySlots(this.nativeHandle);
  }

  /** Assigns a profile signing key to the Lockbox owner role. */
  setOwnerSigningKey(key) {
    return this.operations.lockboxSetOwnerSigningKey(this.nativeHandle, key?.nativeHandle ?? null);
  }

  /** Returns public signing and ownership metadata for the current revision. */
  ownerInspection() {
    return this.operations.lockboxOwnerInspection(this.nativeHandle);
  }

  /** Defines and stores a reusable versioned form. */
  defineForm(alias, name, description, fields) {
    return this.operations.lockboxDefineForm(this.nativeHandle, alias, name, description, encodeFormFields(fields));
  }

  /** Lists the form definitions stored in this Lockbox. */
  listFormDefinitions() {
    return this.operations.lockboxListFormDefinitions(this.nativeHandle);
  }

  /** Resolves a form alias, type identifier, or revision. */
  resolveForm(reference) {
    return this.operations.lockboxResolveForm(this.nativeHandle, reference);
  }

  /** Lists every stored revision for a form type identifier. */
  listFormRevisions(typeId) {
    return this.operations.lockboxListFormRevisions(this.nativeHandle, typeId);
  }

  /** Stages a form record at path using the referenced definition. */
  createFormRecord(path, typeReference, name) {
    return this.operations.lockboxCreateFormRecord(this.nativeHandle, path, typeReference, name);
  }

  /** Stages a plain field value in a form record. */
  setFormField(path, field, value) {
    return this.operations.lockboxSetFormField(this.nativeHandle, path, field, value);
  }

  /** Sets secret form field. */
  setSecretFormField(path, field, value) {
    return this.operations.lockboxSetSecretFormField(this.nativeHandle, path, field, value);
  }

  /** Lists form records without exposing secret field values. */
  listFormRecords() {
    return this.operations.lockboxListFormRecords(this.nativeHandle);
  }

  /** Returns the form record at path when present. */
  getFormRecord(path) {
    return this.operations.lockboxGetFormRecord(this.nativeHandle, path);
  }

  /** Stages removal of a form record. */
  deleteFormRecord(path) {
    return this.operations.lockboxDeleteFormRecord(this.nativeHandle, path);
  }

  /** Updates form records. */
  moveFormRecords(moves) {
    return this.operations.lockboxMoveFormRecords(this.nativeHandle, encodePathMoves(moves));
  }

  /** Returns a plain form field when it exists. */
  getFormField(path, field) {
    return this.operations.lockboxGetFormField(this.nativeHandle, path, field);
  }

  /** Invokes a callback with temporary secret field bytes, then clears them. */
  withSecretFormField(path, field, callback) {
    return this.operations.lockboxWithSecretFormField(this.nativeHandle, path, field, callback);
  }

  /** Serializes the current Lockbox, including committed changes. */
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
  /** Releases this public contact key. */
  publicFree() {
    this.operations.keyContactPublicFree(this.nativeHandle);
    this.nativeHandle = null;
  }
  /** Release the public key handle. */
  close() { this.publicFree(); }

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

  /** Returns the encrypted content key bytes. */
  ciphertext() {
    return this.operations.keyContactWrappedCiphertext(this.nativeHandle);
  }

  /** Returns the complete wrapped key record for storage or transport. */
  encrypted() {
    return this.operations.keyContactWrappedEncrypted(this.nativeHandle);
  }

  /** Releases the native resources held by this object. */
  free() {
    this.operations.keyContactWrappedFree(this.nativeHandle);
    this.nativeHandle = null;
  }

}

/** A Vault Profile signing identity used to authorize mutable Lockbox revisions. */
export class ProfileSigningKeyPair extends OwnedHandle {
  /** Returns the canonical public bytes paired with this identity. */
  publicBytes() {
    return this.operations.keySigningPublic(this.nativeHandle);
  }

  /** Returns the private signing-key record for secure binary backup. */
  privateRecord() {
    return this.operations.keySigningPrivate(this.nativeHandle);
  }

  /** Creates an independently owned public verification-key handle. */
  publicKey() {
    return new ProfileSigningPublicKey(
      this.operations,
      this.operations.keySigningPublicFromBytes(this.publicBytes()),
    );
  }

  /** Wipes and releases the native signing-key handle. */
  dispose() {
    this.operations.keySigningFree(this.nativeHandle);
    this.nativeHandle = null;
  }

}

/** The shareable half of a Vault Profile signing identity. */
export class ProfileSigningPublicKey extends OwnedHandle {
  /** Releases the native verification-key handle. */
  dispose() {
    this.operations.keySigningPublicFree(this.nativeHandle);
    this.nativeHandle = null;
  }

}

/** A password-protected local store for Profile keys, contacts, forms, backups,
 * and remembered lockbox paths; it does not contain lockbox file contents. */
/** Persistent encrypted local store for profiles, keys, contacts and metadata. */
export class Vault extends OwnedHandle {
  /** Open an existing persistent Vault without creating or replacing it. */
  static open(root, vaultPassphrase) { return new Revault().vaultDirectoryOpen(root, vaultPassphrase); }
  /** Open a persistent Vault or create it when absent. */
  static openOrCreate(root, vaultPassphrase) { return new Revault().vaultDirectoryOpenOrCreate(root, vaultPassphrase); }
  /** Create a new persistent Vault. */
  static create(root, vaultPassphrase) { return new Revault().vaultDirectoryReplace(root, vaultPassphrase); }
  /** Replace a persistent Vault explicitly; existing contents are discarded. */
  static replace(root, vaultPassphrase) { return new Revault().vaultDirectoryReplace(root, vaultPassphrase); }
  /** Returns the canonical root directory of this Vault. */
  root() {
    return this.operations.vaultDirectoryRoot(this.nativeHandle);
  }

  /** Returns the persistent structure version of this Vault. */
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

  /** Reports whether the named profile private key exists. */
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

  /** Reports whether the named contact exists. */
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

  /** Returns the email recorded for a profile, when present. */
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

  /** Returns the number of stored key recovery backups. */
  backupCount() {
    return this.operations.vaultDirectoryBackupCount(this.nativeHandle);
  }

  /** Restores a profile private key and signing key from recovery material. */
  restorePrivateKey(name, key, signingKey, overwrite) {
    return this.operations.vaultDirectoryRestorePrivateKey(this.nativeHandle, name, key?.nativeHandle ?? null, signingKey?.nativeHandle ?? null, overwrite);
  }

  /** Loads owner signing key. */
  loadProfileSigningKey(name) {
    return new ProfileSigningKeyPair(this.operations, this.operations.vaultDirectoryLoadOwnerSigningKey(this.nativeHandle, name));
  }

  /** Loads owner signing key generation. */
  loadProfileSigningKeyGeneration(name, index) {
    return new ProfileSigningKeyPair(this.operations, this.operations.vaultDirectoryLoadOwnerSigningKeyGeneration(this.nativeHandle, name, index));
  }

  /** Stores contact signing key. */
  storeContactSigningKey(name, key) {
    return this.operations.vaultDirectoryStoreContactSigningKey(this.nativeHandle, name, key?.nativeHandle ?? null);
  }

  /** Loads contact signing key. */
  loadContactSigningKey(name) {
    return new ProfileSigningPublicKey(this.operations, this.operations.vaultDirectoryLoadContactSigningKey(this.nativeHandle, name));
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

  /** Finds access slot labels with the supplied name for one Lockbox. */
  findAccessSlotLabels(id, name) {
    return this.operations.vaultDirectoryFindAccessSlotLabels(this.nativeHandle, id, name);
  }

  /** Removes access slot label. */
  forgetAccessSlotLabel(id, slotId) {
    return this.operations.vaultDirectoryForgetAccessSlotLabel(this.nativeHandle, id, slotId);
  }

  /** Defines and stores a reusable versioned form. */
  defineForm(alias, name, description, fields) {
    return this.operations.vaultDirectoryDefineForm(this.nativeHandle, alias, name, description, encodeFormFields(fields));
  }

  /** Resolves a form alias, type identifier, or revision. */
  resolveForm(reference) {
    return this.operations.vaultDirectoryResolveForm(this.nativeHandle, reference);
  }

  /** Lists forms. */
  listForms() {
    return this.operations.vaultDirectoryListForms(this.nativeHandle);
  }

  /** Lists every stored revision for a form type identifier. */
  listFormRevisions(typeId) {
    return this.operations.vaultDirectoryListFormRevisions(this.nativeHandle, typeId);
  }

  /** Adds missing standard form definitions and returns the number added. */
  seedForms() {
    return this.operations.vaultDirectorySeedForms(this.nativeHandle);
  }

  /** Stores password. */
  rememberPassword(id, password) {
    return this.operations.vaultDirectoryRememberPassword(this.nativeHandle, id, password);
  }

  /** Returns the Lockbox password encrypted inside this Vault. */
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
/** Read-only persistent Vault view for discovery and diagnostics. */
export class ReadOnlyVault extends OwnedHandle {
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
    if (this.nativeHandle != null) {
      this.operations.vaultReadOnlyFree(this.nativeHandle);
      this.nativeHandle = null;
    }
  }

}

/** Client for the session service that temporarily caches vault unlock and
 * owner signing keys across application operations. */
class Agent {
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

  /** Runs the session agent server until it is stopped. */
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

  /** Lists entries below path, optionally including descendants. */
  list() {
    return this.operations.vaultAgentList();
  }

  /** Reports how the platform handles agent expiry during system sleep. */
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

  /** Returns the cached signing identity for a Vault Profile. */
  profileSigningKey(vaultId, profile) {
    return new ProfileSigningKeyPair(this.operations, this.operations.vaultAgentGetOwnerSigningKey(vaultId, profile));
  }

  /** Caches a signing identity for a Vault Profile. */
  cacheProfileSigningKey(vaultId, profile, key, ttlSeconds) {
    return this.operations.vaultAgentPutOwnerSigningKey(vaultId, profile, key?.nativeHandle ?? null, ttlSeconds);
  }

  /** Removes a cached Vault Profile signing identity. */
  forgetProfileSigningKey(vaultId, profile) {
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

/** Explicit Session Agent controller; it caches content keys only when asked. */
export class AgentSession extends Agent {
  /** Create an explicit session controller over a native runtime. */
  constructor(operations) { super(operations); this._vaultHandle = operations.vaultLocal(); }
  /** Return the process-wide explicit session controller. */
  static get instance() { if (!this._instance) this._instance = new AgentSession(new BindingOperations()); return this._instance; }
  /** Remove one cached lockbox key from this session. */
  closeLockbox(lockboxPath) { return this.operations.vaultCloseLockbox(this._vaultHandle, lockboxPath); }
  /** Remove every lockbox key cached by this session. */
  closeAll() { return this.operations.vaultCloseAll(this._vaultHandle); }
  /** Create a password-protected lockbox file through this session. */
  createLockboxPassword(path, password) { return new Lockbox(this.operations, this.operations.vaultCreateLockboxPassword(this._vaultHandle, path, password)); }
  /** Open a password-protected lockbox file through this session. */
  openLockboxPassword(path, password) { return new Lockbox(this.operations, this.operations.vaultOpenLockboxPassword(this._vaultHandle, path, password)); }
  /** Create a content-key lockbox file through this session. */
  createLockboxContentKey(path, contentKey, signingKey) { return new Lockbox(this.operations, this.operations.vaultCreateLockboxContentKey(this._vaultHandle, path, contentKey, signingKey?.nativeHandle ?? null)); }
  /** Create a contact-addressed lockbox file through this session. */
  createLockboxContact(path, contact, name, signingKey) { return new Lockbox(this.operations, this.operations.vaultCreateLockboxContact(this._vaultHandle, path, contact?.nativeHandle ?? null, name, signingKey?.nativeHandle ?? null)); }
  /** Open a content-key lockbox file through this session. */
  openLockboxContentKey(path, contentKey, signingKey) { return new Lockbox(this.operations, this.operations.vaultOpenLockboxContentKey(this._vaultHandle, path, contentKey, signingKey?.nativeHandle ?? null)); }
  /** Cache a password-derived key for the requested number of seconds. */
  cacheLockboxPassword(path, password, ttlSeconds) { return this.operations.vaultCacheLockboxPassword(this._vaultHandle, path, password, ttlSeconds); }
  /** Release the process-local session handle. */
  free() { if (this._vaultHandle != null) { this.operations.vaultFree(this._vaultHandle); this._vaultHandle = null; } }
}

/** A token kept alive while an operation needs secrets cached by the agent. */
export class AgentActivity extends OwnedHandle {
  /** End the activity and release its native lifetime token. */
  free() {
    if (this.nativeHandle != null) {
      this.operations.vaultAgentEndActivity(this.nativeHandle);
      this.nativeHandle = null;
    }
  }
}

/** Access to the platform credential store for a scoped Vault passphrase. */
class Platform {
  /** Creates a new facade over the bundled native library. */
  constructor(operations) { this.operations = operations; }

  /** Returns availability and user presence guarantees for platform storage. */
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

  /** Enables storage of the Vault passphrase in platform credentials. */
  enable() {
    return this.operations.vaultPlatformEnable();
  }

  /** Disables platform credential use without deleting the stored value. */
  disable() {
    return this.operations.vaultPlatformDisable();
  }

  /** Reports whether platform credential use is disabled. */
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
class LocalVault extends OwnedHandle {
  /** Creates Lockbox password. */
  createLockboxPassword(path, password) {
    return new Lockbox(this.operations, this.operations.vaultCreateLockboxPassword(this.nativeHandle, path, password));
  }

  /** Opens Lockbox password. */
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

  /** Stores Lockbox password. */
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
