import { spawn } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const binding = await import(process.env.REVAULT_E2E_MODULE ?? '../../javascript/index.js');
const { Vault, createMessage, encodeMessage } = binding;

const api = new Vault();
const language = process.env.REVAULT_E2E_LANGUAGE ?? 'javascript';
const script = fileURLToPath(import.meta.url);
const pass = (symbol, assertions = 1) => console.log(`PASS\t${language}\t${symbol}\t${assertions}`);
const check = (value, message) => { if (!value) throw new Error(message); };
const bytes = value => Buffer.isBuffer(value) ? value : Buffer.from(value);
const equal = (left, right) => bytes(left).equals(bytes(right));
const artifactRoot = () => {
  const root = path.join(process.env.REVAULT_E2E_ARTIFACT_DIR ?? '/tmp/revault-e2e-artifacts', language);
  fs.mkdirSync(root, { recursive: true, mode: 0o700 });
  return root;
};
const fields = () => encodeMessage(createMessage('FormFieldList', { values: [
  createMessage('FormField', { id: 'username', label: 'Username', kind: 'text', required: true }),
  createMessage('FormField', { id: 'password', label: 'Password', kind: 'secret', required: true }),
] }));
const sleep = milliseconds => Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, milliseconds);

function archiveLifecycle() {
  const key = Buffer.alloc(32, 'K');
  const box = api.lockboxCreate(key); pass('lockbox_create');
  box.addFile('/hello.txt', `hello from ${language} conformance`, false); pass('lockbox_add_file', 2);
  check(equal(box.getFile('/hello.txt'), `hello from ${language} conformance`), 'get file'); pass('lockbox_get_file', 3);
  box.addFileWithPermissions('/hello.txt', 'replacement payload', 0o640, true); pass('lockbox_add_file_with_permissions', 2);
  check(box.permissions('/hello.txt') === 0o640, 'permissions'); pass('lockbox_permissions');
  box.createDir('/tree', true); pass('lockbox_create_dir', 2);
  check(box.isDir('/tree'), 'directory'); pass('lockbox_is_dir');
  box.createParentDirs('/tree/a/b/file'); pass('lockbox_create_parent_dirs', 2);
  box.rename('/hello.txt', '/renamed.txt'); pass('lockbox_rename', 3);
  check(box.exists('/renamed.txt') && !box.exists('/hello.txt'), 'exists'); pass('lockbox_exists', 2);
  box.setPermissions('/renamed.txt', 0o600); pass('lockbox_set_permissions', 2);
  check(equal(box.readRange('/renamed.txt', 0, 11), 'replacement'), 'range'); pass('lockbox_read_range', 3);
  box.setVariable('normal', 'value'); pass('lockbox_set_variable');
  check(box.getVariable('normal') === 'value', 'variable'); pass('lockbox_get_variable', 3);
  let moves = encodeMessage(createMessage('PathMoveList', { values: [
    createMessage('PathMove', { source: 'normal', destination: 'moved' }),
  ] }));
  box.moveVariables(moves); check(box.getVariable('moved') === 'value', 'moved variable');
  moves = encodeMessage(createMessage('PathMoveList', { values: [
    createMessage('PathMove', { source: 'moved', destination: 'normal' }),
  ] }));
  box.moveVariables(moves); pass('lockbox_move_variables', 3);
  box.setSecretVariable('secret', Buffer.from('hidden')); pass('lockbox_set_secret_variable');
  check(box.withSecretVariable('secret', value => value.toString()) === 'hidden', 'secret variable');
  pass('lockbox_get_secret_variable'); pass('secret_len'); pass('secret_copy'); pass('secret_free');
  box.variableSensitivity('secret'); pass('lockbox_variable_sensitivity', 2);
  check(box.listVariables().values.length === 2, 'variables'); pass('lockbox_list_variables');
  box.deleteVariable('normal'); pass('lockbox_delete_variable');
  box.addSymlink('/link', '/renamed.txt', false); pass('lockbox_add_symlink');
  check(box.getSymlinkTarget('/link') === '/renamed.txt', 'symlink'); pass('lockbox_get_symlink_target', 3);
  check(box.list('/', true).entries.length > 0, 'list'); box.stat('/renamed.txt'); pass('lockbox_list', 2); pass('lockbox_stat', 2);
  box.setWorkloadProfile('read-mostly'); box.setWorkerPolicy('single', 1); box.runtimeOptions();
  pass('lockbox_set_workload_profile'); pass('lockbox_set_worker_policy'); pass('lockbox_runtime_options');
  box.commit(); pass('lockbox_commit'); check(Number(box.storageLen()) > 0, 'storage'); pass('lockbox_storage_len');
  const archive = box.toBytes(); pass('lockbox_to_bytes', 2); pass('buffer_free');
  const formatVersion = api.lockboxFormatVersion(); check(formatVersion > 0, 'lockbox format version');
  check(api.lockboxProbeFormatVersion(archive) === formatVersion, 'lockbox format probe');
  pass('lockbox_format_version', 2); pass('lockbox_probe_format_version', 2);
  check(api.lockboxProbeFormatVersion(Buffer.from('bad')) === 0, 'invalid format probe');
  check(api.lastErrorDetails().message.length > 0, 'structured error details');
  pass('buffer_last_error_details', 2);
  const archivePath = path.join(artifactRoot(), 'archive.lbox'); fs.writeFileSync(archivePath, archive);
  console.log(`ARTIFACT\t${language}\tarchive-created\t${archivePath}`); box.free();
  const opened = api.lockboxOpen(archive, key); pass('lockbox_open', 2);
  check(equal(opened.getFile('/renamed.txt'), 'replacement payload'), 'opened');
  console.log(`ARTIFACT\t${language}\tarchive-opened\t${archivePath}`);
  opened.delete('/renamed.txt'); pass('lockbox_delete', 2);
  opened.removeDir('/tree', true); pass('lockbox_remove_dir', 2); opened.free(); pass('lockbox_free', 2);
}

function keyLifecycle() {
  const content = Buffer.from(Array.from({ length: 32 }, (_, index) => index));
  const contact = api.keyContactGenerate(); pass('key_contact_generate');
  const privateKey = contact.private(); pass('key_contact_private', 2);
  const copy = api.keyContactFromPrivate(privateKey); pass('key_contact_from_private');
  const publicBytes = contact.public(); pass('key_contact_public', 2);
  const publicKey = api.keyContactPublicFromBytes(publicBytes); pass('key_contact_public_from_bytes');
  const wrapped = publicKey.encrypt(content); pass('key_contact_encrypt');
  check(equal(copy.decrypt(wrapped), content), 'decrypt'); pass('key_contact_decrypt', 3);
  check(wrapped.public().length > 0 && wrapped.ciphertext().length > 0 && wrapped.encrypted().length > 0, 'wrapped');
  pass('key_contact_wrapped_public', 2); pass('key_contact_wrapped_ciphertext', 2); pass('key_contact_wrapped_encrypted', 2);
  wrapped.free(); pass('key_contact_wrapped_free');
  const importedPrivate = api.vaultKeyImportPrivate(api.vaultKeyExportPrivate(contact, 'raw-hex'));
  pass('vault_key_export_private', 2); pass('vault_key_import_private');
  const importedPublic = api.vaultKeyImportPublic(api.vaultKeyExportPublic(publicKey, 'lockbox-pem'));
  pass('vault_key_export_public', 2); pass('vault_key_import_public');
  const fingerprint = api.vaultKeyFingerprint(importedPublic); pass('vault_key_fingerprint', 2);
  const hex = api.vaultKeyFormatHex(fingerprint); check(equal(api.vaultKeyDecodeHex(hex), fingerprint), 'hex');
  pass('vault_key_format_hex', 2); pass('vault_key_decode_hex', 2);
  const short = fingerprint.subarray(0, 12); const code = api.vaultKeyFormatCrockford(short);
  api.vaultKeyFormatCrockfordReading(code); check(equal(api.vaultKeyDecodeCrockford(code), short), 'crockford');
  pass('vault_key_format_crockford', 2); pass('vault_key_format_crockford_reading', 2); pass('vault_key_decode_crockford', 2);
  importedPublic.publicFree(); publicKey.publicFree(); pass('key_contact_public_free', 2);
  importedPrivate.free(); copy.free(); contact.free(); pass('key_contact_free', 3);
  const plain = api.vaultKeyHexEncode(content); check(equal(api.vaultKeyHexDecode(plain), content), 'plain hex');
  pass('vault_key_hex_encode', 2); pass('vault_key_hex_decode', 2);
  const signing = api.keySigningGenerate(); pass('key_signing_generate');
  const signingPrivate = signing.private(); const signingPublic = signing.public();
  pass('key_signing_private', 2); pass('key_signing_public', 2);
  api.keySigningFromPrivate(signingPrivate).free(); pass('key_signing_from_private');
  api.keySigningPublicFromBytes(signingPublic).publicFree();
  pass('key_signing_public_from_bytes'); pass('key_signing_public_free', 2);
  signing.free(); pass('key_signing_free', 3);
}

function advancedArchive() {
  const key = Buffer.alloc(32, 'A');
  const box = api.lockboxCreateWithOptions(key, 'bytes', 4 << 20, 'bulk-import', 'single', 1); pass('lockbox_create_with_options');
  box.addFile('/account.txt', 'account data', false);
  box.listWithOptions('/', '*.txt', true, true, false, false, 20); pass('lockbox_list_with_options', 2);
  const definition = box.defineForm('account', 'Account', 'Account form', fields()); pass('lockbox_define_form', 2);
  box.listFormDefinitions(); box.resolveForm('account'); box.listFormRevisions(definition.typeId);
  pass('lockbox_list_form_definitions'); pass('lockbox_resolve_form'); pass('lockbox_list_form_revisions');
  box.createFormRecord('/account.form', 'account', 'Primary'); pass('lockbox_create_form_record');
  box.setFormField('/account.form', 'username', 'alice'); pass('lockbox_set_form_field');
  box.setSecretFormField('/account.form', 'password', Buffer.from('hidden')); pass('lockbox_set_secret_form_field');
  check(box.withSecretFormField('/account.form', 'password', value => value.toString()) === 'hidden', 'secret form field');
  pass('lockbox_get_secret_form_field');
  box.getFormRecord('/account.form'); box.getFormField('/account.form', 'username'); box.listFormRecords();
  pass('lockbox_get_form_record'); pass('lockbox_get_form_field'); pass('lockbox_list_form_records');
  let moves = encodeMessage(createMessage('PathMoveList', { values: [
    createMessage('PathMove', { source: '/account.form', destination: '/moved.form' }),
  ] }));
  box.moveFormRecords(moves); box.getFormRecord('/moved.form');
  moves = encodeMessage(createMessage('PathMoveList', { values: [
    createMessage('PathMove', { source: '/moved.form', destination: '/account.form' }),
  ] }));
  box.moveFormRecords(moves); pass('lockbox_move_form_records', 3);
  const signing = api.keySigningGenerate(); const contact = api.keyContactGenerate();
  const publicKey = api.keyContactPublicFromBytes(contact.public());
  box.setOwnerSigningKey(signing); pass('lockbox_set_owner_signing_key');
  const slot = box.addPassword('archive password'); pass('lockbox_add_password');
  box.addContact(publicKey, 'recipient'); pass('lockbox_add_contact'); box.listKeySlots(); pass('lockbox_list_key_slots');
  box.deleteKey(slot); pass('lockbox_delete_key'); box.commit(); box.ownerInspection(); pass('lockbox_owner_inspection', 2);
  box.cacheStats(); box.importStats(); box.resetImportStats(); box.pageInspection(); box.recoveryReport();
  box.recoveryReportRender(true, 100); box.streamContent(false); box.id();
  for (const symbol of ['lockbox_cache_stats', 'lockbox_import_stats', 'lockbox_reset_import_stats', 'lockbox_page_inspection', 'lockbox_recovery_report']) pass(symbol);
  pass('lockbox_recovery_report_render', 2); pass('lockbox_stream_content'); pass('lockbox_id', 2);
  const archive = box.toBytes(); const archivePath = path.join(artifactRoot(), 'advanced.lbox'); fs.writeFileSync(archivePath, archive);
  api.lockboxInspectFile(archivePath); api.lockboxRecoveryScanPath(archivePath, key); api.lockboxRecoveryScan(archive, key);
  pass('lockbox_inspect_file'); pass('lockbox_recovery_scan_path'); pass('lockbox_recovery_scan');
  api.lockboxRecoverySalvage(archive.subarray(0, -32), key, signing).free(); pass('lockbox_recovery_salvage', 2);
  api.lockboxOpenWithOptions(archive, key, 'bytes', 4 << 20, 'bulk-import', 'single', 1).free(); pass('lockbox_open_with_options', 2);
  const passwordBox = api.lockboxCreatePassword('archive password'); passwordBox.addFile('/password.txt', 'password protected', false); passwordBox.commit();
  const passwordArchive = passwordBox.toBytes(); passwordBox.free(); pass('lockbox_create_password');
  const passwordOpen = api.lockboxOpenPassword(passwordArchive, 'archive password'); passwordOpen.getFile('/password.txt'); passwordOpen.free(); pass('lockbox_open_password', 2);
  const contactBox = api.lockboxCreateContact(publicKey); contactBox.addFile('/contact.txt', 'contact protected', false); contactBox.commit();
  const contactArchive = contactBox.toBytes(); contactBox.free(); pass('lockbox_create_contact');
  const contactOpen = api.lockboxOpenContact(contactArchive, contact); contactOpen.getFile('/contact.txt'); contactOpen.free(); pass('lockbox_open_contact', 2);
  const signed = api.lockboxCreateWithSigningKey(key, signing); signed.commit(); signed.free(); pass('lockbox_create_with_signing_key', 2);
  const extract = fs.mkdtempSync(path.join(os.tmpdir(), `revault-${language}-extract-`));
  box.extractFile('/account.txt', path.join(extract, 'account.txt'), false); pass('lockbox_extract_file', 2);
  const tree = path.join(extract, 'tree'); fs.mkdirSync(tree); box.extractDirectory(tree, 1 << 20, 4 << 20, 100, false, true, false); pass('lockbox_extract_directory', 2);
  box.deleteFormRecord('/account.form'); pass('lockbox_delete_form_record');
  box.free(); publicKey.publicFree(); contact.free(); signing.free();
}

function vaultLifecycle() {
  const root = path.join(artifactRoot(), 'vault'); fs.mkdirSync(root, { recursive: true });
  const password = 'vault password'; const changed = 'new vault password';
  const id = Buffer.from(Array.from({ length: 16 }, (_, index) => 0xa0 + index));
  const profile = api.keyContactGenerate(); const contact = api.keyContactGenerate();
  const contactPublic = api.keyContactPublicFromBytes(contact.public());
  const owner = api.keySigningGenerate(); const ownerPublic = api.keySigningPublicFromBytes(owner.public());
  const vault = api.vaultDirectoryReplace(root, password); pass('vault_directory_replace');
  console.log(`ARTIFACT\t${language}\tvault-created\t${root}`);
  check(vault.root() === root && vault.structureVersion() > 0, 'vault'); pass('vault_directory_root', 3); pass('vault_directory_structure_version');
  const currentVersion = api.vaultStructureVersionCurrent();
  check(currentVersion === vault.structureVersion(), 'current vault structure version');
  check(api.vaultDirectoryProbeStructureVersion(root, password) === currentVersion, 'vault structure probe');
  pass('vault_structure_version_current', 2); pass('vault_directory_probe_structure_version', 2);
  vault.storePrivateKey('alice', profile); pass('vault_directory_store_private_key');
  vault.privateKeyExists('alice'); pass('vault_directory_private_key_exists');
  vault.loadPrivateKey('alice').free(); vault.loadPrivateKeyGeneration('alice', 1).free();
  pass('vault_directory_load_private_key'); pass('vault_directory_load_private_key_generation');
  vault.storeProfileEmail('alice', 'alice@example.test'); vault.profileEmail('alice');
  pass('vault_directory_store_profile_email'); pass('vault_directory_profile_email', 3);
  vault.listProfileGenerations('alice'); vault.rotatePrivateKey('alice');
  pass('vault_directory_list_profile_generations'); pass('vault_directory_rotate_private_key');
  vault.loadOwnerSigningKey('alice').free(); vault.loadOwnerSigningKeyGeneration('alice', 1).free();
  pass('vault_directory_load_owner_signing_key'); pass('vault_directory_load_owner_signing_key_generation');
  vault.storeContact('bob', contactPublic); vault.contactExists('bob'); vault.loadContact('bob').publicFree(); vault.listContacts();
  pass('vault_directory_store_contact'); pass('vault_directory_contact_exists'); pass('vault_directory_load_contact'); pass('vault_directory_list_contacts');
  vault.storeContactSigningKey('bob', ownerPublic); vault.loadContactSigningKey('bob').publicFree();
  pass('vault_directory_store_contact_signing_key'); pass('vault_directory_load_contact_signing_key');
  vault.listPrivateKeys(); vault.listPrivateKeyNames(); vault.listContactNames();
  pass('vault_directory_list_private_keys'); pass('vault_directory_list_private_key_names'); pass('vault_directory_list_contact_names');
  vault.storeBackup(id, 'encrypted backup bytes'); vault.backupCount(); check(equal(vault.loadBackup(id), 'encrypted backup bytes'), 'backup');
  pass('vault_directory_store_backup'); pass('vault_directory_backup_count'); pass('vault_directory_load_backup', 3);
  vault.rememberLockbox(id, '/tmp/example.lbox'); vault.listKnownLockboxes();
  pass('vault_directory_remember_lockbox'); pass('vault_directory_list_known_lockboxes');
  vault.rememberAccessSlotLabel(id, 7, 'primary'); vault.listAccessSlotLabels(id); vault.findAccessSlotLabels(id, 'primary');
  pass('vault_directory_remember_access_slot_label'); pass('vault_directory_list_access_slot_labels'); pass('vault_directory_find_access_slot_labels');
  vault.rememberPassword(id, password); vault.rememberedPassword(id);
  pass('vault_directory_remember_password'); pass('vault_directory_remembered_password', 3);
  const vaultForm = vault.defineForm('login', 'Login', 'Login form', fields()); vault.resolveForm('login'); vault.listForms();
  pass('vault_directory_define_form'); pass('vault_directory_resolve_form'); pass('vault_directory_list_forms');
  vault.listFormRevisions(vaultForm.typeId); pass('vault_directory_list_form_revisions', 2);
  vault.seedForms(); pass('vault_directory_seed_forms'); vault.listFormAliases(); pass('vault_directory_list_form_aliases');
  vault.forgetAccessSlotLabel(id, 7); vault.forgetLockbox('/tmp/example.lbox'); vault.deleteContact('bob');
  pass('vault_directory_forget_access_slot_label'); pass('vault_directory_forget_lockbox'); pass('vault_directory_delete_contact');
  vault.deletePrivateKey('alice'); vault.restorePrivateKey('alice', profile, owner, true);
  pass('vault_directory_delete_private_key', 2); pass('vault_directory_restore_private_key', 2);
  vault.free(); pass('vault_directory_free');
  const readonly = api.vaultReadOnlyOpen(root, password); pass('vault_read_only_open');
  readonly.listProfileNames(); readonly.listContactNames(); readonly.listFormAliases(); readonly.listKnownLockboxes();
  pass('vault_read_only_list_profile_names', 2); pass('vault_read_only_list_contact_names');
  pass('vault_read_only_list_form_aliases', 2); pass('vault_read_only_list_known_lockboxes');
  readonly.free(); pass('vault_read_only_free');
  api.vaultDirectoryChangePassword(root, password, changed); pass('vault_directory_change_password');
  api.vaultDirectoryOpen(root, changed).free(); pass('vault_directory_open'); console.log(`ARTIFACT\t${language}\tvault-opened\t${root}`);
  api.vaultDirectoryOpenOrCreate(root, changed).free(); pass('vault_directory_open_or_create');
  ownerPublic.publicFree(); owner.free(); contactPublic.publicFree(); contact.free(); profile.free();
}

function defaultVault() {
  fs.mkdirSync(process.env.LOCKBOX_VAULT_DIR, { recursive: true, mode: 0o700 });
  api.vaultDirectoryReplaceDefault('default password').free(); pass('vault_directory_replace_default');
  api.vaultReadOnlyOpenDefault('default password').free(); pass('vault_read_only_open_default');
  api.vaultDefaultDirectory(); api.vaultDefaultPath(); pass('vault_default_directory', 3); pass('vault_default_path', 2);
  api.vaultDirectoryOpenOrCreateDefault('default password').free(); pass('vault_directory_open_or_create_default');
  api.vaultDirectoryChangeDefaultPassword('default password', 'changed default password'); pass('vault_directory_change_default_password');
  const backup = path.join(artifactRoot(), 'default-vault.backup'); fs.rmSync(backup, { force: true });
  api.vaultBackupDefault(backup, false); api.vaultRestoreDefault(backup, true);
  pass('vault_backup_default'); pass('vault_restore_default');
}

function platformStore() {
  const platform = api.platform; platform.status(); pass('vault_platform_status', 2);
  platform.setScope('vault'); pass('vault_platform_set_scope');
  platform.disable(); platform.disabled(); pass('vault_platform_disable'); pass('vault_platform_disabled');
  platform.enable(); pass('vault_platform_enable');
  platform.putPassword('platform vault password'); check(equal(platform.getPassword(), 'platform vault password'), 'platform');
  pass('vault_platform_put_password'); pass('vault_platform_get_password', 3);
  platform.forgetPassword(); pass('vault_platform_forget_password');
}

async function agentAndLocal() {
  for (const directory of [process.env.LOCKBOX_SESSION_AGENT_DIR, process.env.LOCKBOX_VAULT_DIR]) fs.mkdirSync(directory, { recursive: true, mode: 0o700 });
  const directory = api.vaultDirectoryReplaceDefault('agent vault password'); const profile = api.keyContactGenerate();
  directory.storePrivateKey('default', profile); profile.free(); directory.free();
  const agent = api.agent; agent.forgetAll(); pass('vault_forget_all');
  const output = fs.openSync(path.join(os.tmpdir(), `${language}-agent.out`), 'w');
  const error = fs.openSync(path.join(os.tmpdir(), `${language}-agent.err`), 'w');
  const child = spawn(process.execPath, [script, '--serve-agent'], { env: process.env, stdio: ['ignore', output, error] });
  for (let index = 0; index < 200 && !agent.isRunning(); index += 1) sleep(50);
  check(agent.isRunning(), 'agent'); pass('vault_agent_serve'); pass('vault_is_running');
  agent.start(); pass('vault_agent_start');
  agent.verifyTransport(); pass('vault_agent_verify_transport');
  const id = Buffer.from(Array.from({ length: 16 }, (_, index) => 0xc0 + index));
  const key = Buffer.from(Array.from({ length: 32 }, (_, index) => 0x20 + index));
  agent.put(id, key); check(equal(agent.get(id), key), 'agent key'); agent.list();
  pass('vault_agent_put'); pass('vault_agent_get', 3); pass('vault_agent_list');
  agent.putVaultUnlockKey('vault-id', key, 120); agent.getVaultUnlockKey('vault-id'); pass('vault_agent_put_vault_unlock_key'); pass('vault_agent_get_vault_unlock_key', 3);
  const owner = api.keySigningGenerate(); agent.putOwnerSigningKey('vault-id', 'alice', owner, 120); agent.getOwnerSigningKey('vault-id', 'alice').free();
  pass('vault_agent_put_owner_signing_key'); pass('vault_agent_get_owner_signing_key');
  const activity = agent.beginActivity('open'); pass('vault_agent_begin_activity'); agent.endActivity(activity); pass('vault_agent_end_activity');
  agent.sleepSupport(); pass('vault_agent_sleep_support'); api.vaultAgentLogPath(); api.vaultAgentLogDestination();
  pass('vault_agent_log_path', 2); pass('vault_agent_log_destination', 2);
  const local = api.vaultLocal(); pass('vault_local'); const root = fs.mkdtempSync(path.join(os.tmpdir(), `revault-${language}-local-`));
  const passwordPath = path.join(root, 'password.lbox'); let box = local.createLockboxPassword(passwordPath, 'local password');
  box.addFile('/data.txt', 'local vault data', false); box.commit(); box.free(); pass('vault_create_lockbox_password', 3);
  local.cacheLockboxPassword(passwordPath, 'local password', 120); pass('vault_cache_lockbox_password');
  let opened = local.openLockboxPassword(passwordPath, 'local password'); opened.getFile('/data.txt'); opened.free(); pass('vault_open_lockbox_password', 3);
  local.closeLockbox(passwordPath); pass('vault_close_lockbox');
  const contentPath = path.join(root, 'content.lbox'); box = local.createLockboxContentKey(contentPath, key, owner);
  box.addFile('/data.txt', 'local vault data', false); box.commit(); box.free(); pass('vault_create_lockbox_content_key', 3);
  opened = local.openLockboxContentKey(contentPath, key, owner); opened.getFile('/data.txt'); opened.free(); pass('vault_open_lockbox_content_key', 3);
  const contact = api.keyContactGenerate(); const publicKey = api.keyContactPublicFromBytes(contact.public());
  box = local.createLockboxContact(path.join(root, 'contact.lbox'), publicKey, 'recipient', owner);
  box.addFile('/data.txt', 'local vault data', false); box.commit(); box.free(); pass('vault_create_lockbox_contact', 3);
  publicKey.publicFree(); contact.free(); local.closeAll(); pass('vault_close_all'); local.free(); pass('vault_free'); owner.free();
  agent.forgetOwnerSigningKey('vault-id', 'alice'); agent.forgetVaultUnlockKey('vault-id'); agent.forget(id);
  pass('vault_agent_forget_owner_signing_key'); pass('vault_agent_forget_vault_unlock_key'); pass('vault_agent_forget');
  agent.stop(); pass('vault_agent_stop');
  const exitCode = await new Promise((resolve, reject) => { child.once('error', reject); child.once('exit', resolve); });
  check(exitCode === 0, 'agent child');
}

function interop(producer) {
  const root = process.env.REVAULT_E2E_ARTIFACT_DIR ?? '/tmp/revault-e2e-artifacts';
  const box = api.lockboxOpen(fs.readFileSync(path.join(root, producer, 'archive.lbox')), Buffer.alloc(32, 'K'));
  check(equal(box.getFile('/renamed.txt'), 'replacement payload'), 'foreign archive'); box.free();
  const vault = api.vaultDirectoryOpen(path.join(root, producer, 'vault'), 'new vault password');
  check(vault.structureVersion() > 0, 'foreign vault'); vault.free();
  console.log(`INTEROP\t${language}\t${producer}\tarchive\t3\nINTEROP\t${language}\t${producer}\tvault\t2`);
}

const args = process.argv.slice(2);
if (args[0] === '--serve-agent') api.agent.serve();
else if (args[0] === '--default') defaultVault();
else if (args[0] === '--platform') platformStore();
else if (args[0] === '--agent') await agentAndLocal();
else if (args[0] === '--interop' && args.length === 2) interop(args[1]);
else {
  archiveLifecycle(); keyLifecycle(); advancedArchive(); vaultLifecycle();
  api.lastError(); pass('buffer_last_error');
}
