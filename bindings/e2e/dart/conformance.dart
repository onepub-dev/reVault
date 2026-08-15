import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:revault_api/revault_api.dart';

late final Revault api;
void pass(String symbol, [int assertions = 1]) =>
    stdout.writeln('PASS\tdart\t$symbol\t$assertions');
void check(bool value, String message) {
  if (!value) throw StateError(message);
}

Uint8List text(String value) => Uint8List.fromList(utf8.encode(value));
Uint8List repeat(int value, int count) =>
    Uint8List.fromList(List.filled(count, value));
T withSecretBytes<T>(List<int> value, T Function(SecretBytes secret) action) {
  final secret = SecretBytes.copyOf(Uint8List.fromList(value));
  try {
    return action(secret);
  } finally {
    secret.close();
  }
}

T withSecretTextBytes<T>(
  String value,
  T Function(SecretBytes secret) action,
) {
  final secret = SecretBytes.fromString(value);
  try {
    return action(secret);
  } finally {
    secret.close();
  }
}

T withSecretString<T>(String value, T Function(SecretString secret) action) {
  final secret = SecretString.fromString(value);
  try {
    return action(secret);
  } finally {
    secret.close();
  }
}

bool secretBytesEqual(SecretBytes secret, List<int> expected) =>
    secret.withBytes((bytes) => _equal(bytes, expected));

bool secretStringsEqual(SecretString first, SecretString second) =>
    first.withBytes(
      (firstBytes) => second.withBytes(
        (secondBytes) => _equal(firstBytes, secondBytes),
      ),
    );
String artifactRoot() {
  final path =
      '${Platform.environment['REVAULT_E2E_ARTIFACT_DIR'] ?? '/tmp/revault-e2e-artifacts'}/dart';
  Directory(path).createSync(recursive: true);
  return path;
}

List<FormField> fields() => [
      FormField(
          id: 'username', label: 'Username', kind: 'text', required: true),
      FormField(
          id: 'password', label: 'Password', kind: 'secret', required: true),
    ];

void archiveLifecycle() {
  final key = SecretBytes.copyOf(repeat('K'.codeUnitAt(0), 32));
  late Uint8List archive;
  final box = Lockbox.createInMemory(contentKey: key);
  pass('lockbox_create');
  box.addFile('/hello.txt', text('hello from dart conformance'));
  check(box.exists('/hello.txt'), 'add');
  pass('lockbox_add_file', 2);
  check(
    _equal(box.getFile('/hello.txt'), text('hello from dart conformance')),
    'get',
  );
  pass('lockbox_get_file', 3);
  box.addFile(
    '/hello.txt',
    text('replacement payload'),
    permissions: 0x1a0,
    replace: true,
  );
  check(box.permissions('/hello.txt') == 0x1a0, 'mode');
  pass('lockbox_add_file_with_permissions', 2);
  pass('lockbox_permissions');
  box.createDirectory('/tree', parents: true);
  check(box.isDirectory('/tree'), 'dir');
  pass('lockbox_create_dir', 2);
  pass('lockbox_is_dir');
  box.createParentDirectories('/tree/a/b/file');
  check(box.isDirectory('/tree/a/b'), 'parents');
  pass('lockbox_create_parent_dirs', 2);
  box.rename('/hello.txt', '/renamed.txt');
  check(box.exists('/renamed.txt') && !box.exists('/hello.txt'), 'rename');
  pass('lockbox_rename', 3);
  pass('lockbox_exists', 2);
  box.setPermissions('/renamed.txt', 0x180);
  check(box.permissions('/renamed.txt') == 0x180, 'set mode');
  pass('lockbox_set_permissions', 2);
  check(
    _equal(box.readRange('/renamed.txt', 0, 11), text('replacement')),
    'range',
  );
  pass('lockbox_read_range', 3);
  box.setVariable('normal', 'value');
  check(box.getVariable('normal') == 'value', 'variable');
  pass('lockbox_set_variable');
  pass('lockbox_get_variable', 3);
  box.moveVariables([PathMove(source: 'normal', destination: 'moved')]);
  check(box.getVariable('moved') == 'value', 'moved variable');
  box.moveVariables([PathMove(source: 'moved', destination: 'normal')]);
  pass('lockbox_move_variables', 3);
  withSecretTextBytes('hidden', (secret) {
    box.setSecretVariable('secret', secret);
  });
  pass('lockbox_set_secret_variable');
  check(
    box.withSecretVariable('secret', (value) => String.fromCharCodes(value)) ==
        'hidden',
    'secret variable',
  );
  pass('lockbox_get_secret_variable');
  pass('secret_len');
  pass('secret_copy');
  pass('secret_free');
  check(box.variableSensitivity('secret') == 'secret', 'secret');
  pass('lockbox_variable_sensitivity', 2);
  check(box.listVariables().length == 2, 'variables');
  pass('lockbox_list_variables');
  box.deleteVariable('normal');
  pass('lockbox_delete_variable');
  box.addSymlink('/link', '/renamed.txt');
  check(box.symlinkTarget('/link') == '/renamed.txt', 'link');
  pass('lockbox_add_symlink');
  pass('lockbox_get_symlink_target', 3);
  check(box.list('/', recursive: true).isNotEmpty, 'list');
  check(box.stat('/renamed.txt') != null, 'stat');
  pass('lockbox_list', 2);
  pass('lockbox_stat', 2);
  box.setWorkloadProfile(LockboxWorkload.readMostly);
  box.setWorkerPolicy(LockboxWorker.single, jobs: 1);
  check(box.runtimeOptions().workloadProfile.isNotEmpty, 'runtime');
  pass('lockbox_set_workload_profile');
  pass('lockbox_set_worker_policy');
  pass('lockbox_runtime_options');
  box.commit();
  check(box.storageLength > 0, 'storage');
  pass('lockbox_commit');
  pass('lockbox_storage_len');
  archive = box.bytes;
  check(archive.isNotEmpty, 'bytes');
  pass('lockbox_to_bytes', 2);
  pass('buffer_free');
  final formatVersion = api.lockboxFormatVersion;
  check(formatVersion > 0, 'lockbox format version');
  check(
    api.probeLockboxFormatVersion(archive) == formatVersion,
    'format probe',
  );
  pass('lockbox_format_version', 2);
  pass('lockbox_probe_format_version', 2);
  check(
    api.probeLockboxFormatVersion(Uint8List.fromList([1, 2, 3])) == 0,
    'invalid format probe',
  );
  try {
    Lockbox.openBytes(Uint8List.fromList([1, 2, 3]), contentKey: key);
    throw StateError('invalid lockbox unexpectedly opened');
  } on RevaultException catch (error) {
    check(error.message.isNotEmpty, 'structured error details');
  }
  pass('buffer_last_error_details', 2);
  final path = '${artifactRoot()}/archive.lbox';
  File(path).writeAsBytesSync(archive);
  stdout.writeln('ARTIFACT\tdart\tarchive-created\t$path');
  box.close();
  final opened = Lockbox.openBytes(archive, contentKey: key);
  check(
    _equal(opened.getFile('/renamed.txt'), text('replacement payload')),
    'open',
  );
  pass('lockbox_open', 2);
  stdout.writeln('ARTIFACT\tdart\tarchive-opened\t$path');
  opened.delete('/renamed.txt');
  check(!opened.exists('/renamed.txt'), 'delete');
  pass('lockbox_delete', 2);
  opened.removeDirectory('/tree', recursive: true);
  check(!opened.exists('/tree'), 'remove');
  pass('lockbox_remove_dir', 2);
  opened.close();
  key.close();
  pass('lockbox_free', 2);
}

bool _equal(List<int> a, List<int> b) =>
    a.length == b.length &&
    List.generate(a.length, (i) => a[i] == b[i]).every((e) => e);

void keyLifecycle() {
  final contentBytes = Uint8List.fromList(List.generate(32, (i) => i));
  final content = SecretBytes.copyOf(contentBytes);
  final contact = api.generateContactKeyPair();
  pass('key_contact_generate');
  final privateKey = contact.privateRecord();
  check(privateKey.length > 32, 'private');
  pass('key_contact_private', 2);
  final copy = api.contactKeyPairFromPrivate(privateKey);
  pass('key_contact_from_private');
  final publicBytes = contact.publicBytes();
  check(publicBytes.isNotEmpty, 'public');
  pass('key_contact_public', 2);
  final publicKey = api.contactPublicKeyFromBytes(publicBytes);
  pass('key_contact_public_from_bytes');
  final wrapped = publicKey.encrypt(content);
  pass('key_contact_encrypt');
  final decrypted = copy.decrypt(wrapped);
  check(secretBytesEqual(decrypted, contentBytes), 'decrypt');
  decrypted.close();
  pass('key_contact_decrypt', 3);
  check(
    wrapped.publicBytes().isNotEmpty &&
        wrapped.ciphertext().isNotEmpty &&
        wrapped.encryptedBytes().isNotEmpty,
    'wrapped',
  );
  pass('key_contact_wrapped_public', 2);
  pass('key_contact_wrapped_ciphertext', 2);
  pass('key_contact_wrapped_encrypted', 2);
  wrapped.dispose();
  pass('key_contact_wrapped_free');
  final importedPrivate = api.importContactKeyPair(
    contact.export(KeyExportFormat.rawHex),
  );
  check(importedPrivate.publicBytes().isNotEmpty, 'import private');
  importedPrivate.dispose();
  pass('vault_key_export_private', 2);
  pass('vault_key_import_private');
  final importedPublic = api.importContactPublicKey(
    publicKey.export(KeyExportFormat.lockboxPem),
  );
  pass('vault_key_export_public', 2);
  pass('vault_key_import_public');
  final fingerprint = importedPublic.fingerprint();
  check(fingerprint.length >= 12, 'fp');
  pass('vault_key_fingerprint', 2);
  final hex = api.formatKeyHex(fingerprint);
  check(_equal(api.decodeKeyHex(hex), fingerprint), 'hex');
  pass('vault_key_format_hex', 2);
  pass('vault_key_decode_hex', 2);
  final short = Uint8List.sublistView(fingerprint, 0, 12);
  final code = api.formatKeyCrockford(short);
  check(
    api.formatKeyCrockfordReading(code).isNotEmpty &&
        _equal(api.decodeKeyCrockford(code), short),
    'crockford',
  );
  pass('vault_key_format_crockford', 2);
  pass('vault_key_format_crockford_reading', 2);
  pass('vault_key_decode_crockford', 2);
  importedPublic.dispose();
  publicKey.dispose();
  copy.dispose();
  contact.dispose();
  pass('key_contact_public_free', 2);
  pass('key_contact_free', 3);
  final plain = api.hexEncode(contentBytes);
  check(_equal(api.hexDecode(plain), contentBytes), 'plain hex');
  pass('vault_key_hex_encode', 2);
  pass('vault_key_hex_decode', 2);
  final signing = api.generateProfileSigningKeyPair();
  pass('key_signing_generate');
  final signingPrivate = signing.privateRecord(),
      signingPublic = signing.publicBytes();
  check(signingPrivate.isNotEmpty && signingPublic.isNotEmpty, 'signing');
  pass('key_signing_private', 2);
  pass('key_signing_public', 2);
  final signingCopy = api.profileSigningKeyPairFromPrivate(signingPrivate);
  check(signingCopy.publicBytes().isNotEmpty, 'copy');
  signingCopy.dispose();
  pass('key_signing_from_private');
  final signingPub = api.profileSigningPublicKeyFromBytes(signingPublic);
  signingPub.dispose();
  pass('key_signing_public_from_bytes');
  pass('key_signing_public_free', 2);
  signing.dispose();
  content.close();
  pass('key_signing_free', 3);
}

void advancedArchive() {
  final key = SecretBytes.copyOf(repeat('A'.codeUnitAt(0), 32));
  final box = Lockbox.createInMemory(
    contentKey: key,
    options: const LockboxOptions(
      cacheBytes: 4 << 20,
      workload: LockboxWorkload.bulkImport,
      worker: LockboxWorker.single,
      jobs: 1,
    ),
  );
  pass('lockbox_create_with_options');
  box.addFile('/account.txt', text('account data'));
  check(
    box
            .listWithOptions(
              '/',
              '*.txt',
              recursive: true,
              includeSymlinks: false,
              includeDirectories: false,
              limit: 20,
            )
            .length ==
        1,
    'filter',
  );
  pass('lockbox_list_with_options', 2);
  final definition = box.defineForm(
    'account',
    'Account',
    'Account form',
    fields(),
  );
  check(definition.typeId.isNotEmpty, 'form');
  pass('lockbox_define_form', 2);
  check(box.listFormDefinitions().length == 1, 'defs');
  check(box.resolveForm('account').typeId == definition.typeId, 'resolve');
  check(box.listFormRevisions(definition.typeId).length == 1, 'revisions');
  pass('lockbox_list_form_definitions');
  pass('lockbox_resolve_form');
  pass('lockbox_list_form_revisions');
  check(
    box.createFormRecord('/account.form', 'account', 'Primary').path ==
        '/account.form',
    'record',
  );
  pass('lockbox_create_form_record');
  box.setFormField('/account.form', 'username', 'alice');
  pass('lockbox_set_form_field');
  withSecretTextBytes('hidden', (secret) {
    box.setSecretFormField('/account.form', 'password', secret);
  });
  pass('lockbox_set_secret_form_field');
  check(
    box.withSecretFormField(
          '/account.form',
          'password',
          (value) => String.fromCharCodes(value),
        ) ==
        'hidden',
    'secret form field',
  );
  pass('lockbox_get_secret_form_field');
  check(box.getFormRecord('/account.form')!.values.length == 2, 'values');
  check(
    box.getFormField('/account.form', 'username')!.value == 'alice',
    'field',
  );
  check(box.listFormRecords().length == 1, 'records');
  pass('lockbox_get_form_record');
  pass('lockbox_get_form_field');
  pass('lockbox_list_form_records');
  box.moveFormRecords([
    PathMove(source: '/account.form', destination: '/moved.form'),
  ]);
  check(box.getFormRecord('/moved.form')!.values.length == 2, 'moved record');
  box.moveFormRecords([
    PathMove(source: '/moved.form', destination: '/account.form'),
  ]);
  pass('lockbox_move_form_records', 3);
  final signing = api.generateProfileSigningKeyPair();
  final contact = api.generateContactKeyPair();
  final publicKey = contact.publicKey();
  box.setOwnerSigningKey(signing);
  pass('lockbox_set_owner_signing_key');
  final passwordSlot = withSecretString(
    'archive password',
    box.addPassword,
  );
  pass('lockbox_add_password');
  check(box.addContact(publicKey, 'recipient') >= 0, 'slot');
  pass('lockbox_add_contact');
  check(box.listKeySlots().length >= 2, 'slots');
  pass('lockbox_list_key_slots');
  box.deleteKey(passwordSlot);
  pass('lockbox_delete_key');
  box.commit();
  check(box.ownerInspection().signed, 'owner');
  pass('lockbox_owner_inspection', 2);
  check(box.cacheStats().limitBytes > 0, 'cache');
  check(box.importStats().hostReadNanos.isNotEmpty, 'import');
  box.resetImportStats();
  check(box.pageInspection().isNotEmpty, 'pages');
  check(box.recoveryReport().intactFileCount > 0, 'recovery');
  check(box.renderRecoveryReport(verbose: true).isNotEmpty, 'render');
  check(box.streamContent().isNotEmpty, 'stream');
  check(box.id.isNotEmpty, 'id');
  pass('lockbox_cache_stats');
  pass('lockbox_import_stats');
  pass('lockbox_reset_import_stats');
  pass('lockbox_page_inspection');
  pass('lockbox_recovery_report');
  pass('lockbox_recovery_report_render', 2);
  pass('lockbox_stream_content');
  pass('lockbox_id', 2);
  final archive = box.bytes, path = '${artifactRoot()}/advanced.lbox';
  File(path).writeAsBytesSync(archive);
  check(api.inspectLockboxFile(path).headerReadable, 'inspect');
  check(api.scanLockboxPath(path, key).intactFileCount > 0, 'scan path');
  check(api.scanLockbox(archive, key).intactFileCount > 0, 'scan');
  pass('lockbox_inspect_file');
  pass('lockbox_recovery_scan_path');
  pass('lockbox_recovery_scan');
  final salvaged = api.salvageLockbox(
    Uint8List.sublistView(archive, 0, archive.length - 32),
    key,
    signing,
  );
  check(salvaged.storageLength > 0, 'salvage');
  salvaged.close();
  pass('lockbox_recovery_salvage', 2);
  final optionOpen = Lockbox.openBytes(
    archive,
    contentKey: key,
    options: const LockboxOptions(
      cacheBytes: 4 << 20,
      workload: LockboxWorkload.bulkImport,
      worker: LockboxWorker.single,
      jobs: 1,
    ),
  );
  check(
    _equal(optionOpen.getFile('/account.txt'), text('account data')),
    'option open',
  );
  optionOpen.close();
  pass('lockbox_open_with_options', 2);
  final passwordBox = withSecretString(
    'archive password',
    (password) => Lockbox.createInMemory(password: password),
  );
  passwordBox.addFile('/password.txt', text('password protected'));
  passwordBox.commit();
  final passwordArchive = passwordBox.bytes;
  passwordBox.close();
  pass('lockbox_create_password');
  final passwordOpen = withSecretString(
    'archive password',
    (password) => Lockbox.openBytes(passwordArchive, password: password),
  );
  check(
    _equal(passwordOpen.getFile('/password.txt'), text('password protected')),
    'password open',
  );
  passwordOpen.close();
  pass('lockbox_open_password', 2);
  final contactBox = Lockbox.createInMemory(contact: publicKey);
  contactBox.addFile('/contact.txt', text('contact protected'));
  contactBox.commit();
  final contactArchive = contactBox.bytes;
  contactBox.close();
  pass('lockbox_create_contact');
  final contactOpen = Lockbox.openBytes(contactArchive, contact: contact);
  check(
    _equal(contactOpen.getFile('/contact.txt'), text('contact protected')),
    'contact open',
  );
  contactOpen.close();
  pass('lockbox_open_contact', 2);
  final signed = Lockbox.createInMemory(
    contentKey: key,
    signingKey: signing,
  );
  signed.commit();
  check(signed.ownerInspection().signed, 'signed');
  signed.close();
  pass('lockbox_create_with_signing_key', 2);
  final extract = Directory('${artifactRoot()}/extract');
  if (extract.existsSync()) extract.deleteSync(recursive: true);
  extract.createSync();
  final extracted = '${extract.path}/account.txt';
  box.extractFile('/account.txt', extracted);
  check(File(extracted).existsSync(), 'extract file');
  pass('lockbox_extract_file', 2);
  final tree = Directory('${extract.path}/tree')..createSync();
  box.extractDirectory(
    tree.path,
    maxFileBytes: 1 << 20,
    maxTotalBytes: 4 << 20,
    maxFiles: 100,
    restorePermissions: true,
  );
  check(tree.existsSync(), 'extract dir');
  pass('lockbox_extract_directory', 2);
  box.deleteFormRecord('/account.form');
  pass('lockbox_delete_form_record');
  box.close();
  publicKey.dispose();
  contact.dispose();
  signing.dispose();
  key.close();
}

void vaultLifecycle() {
  final root = '${artifactRoot()}/vault';
  Directory(root).createSync(recursive: true);
  final password = SecretString.fromString('vault password');
  final changed = SecretString.fromString('new vault password');
  final id = Uint8List.fromList(List.generate(16, (i) => 0xa0 + i));
  final profile = api.generateContactKeyPair(),
      contact = api.generateContactKeyPair(),
      contactPublic = contact.publicKey(),
      owner = api.generateProfileSigningKeyPair(),
      ownerPublic = owner.publicKey();
  final vault = Vault.replace(root: root, passphrase: password);
  pass('vault_directory_replace');
  stdout.writeln('ARTIFACT\tdart\tvault-created\t$root');
  check(vault.root == root && vault.structureVersion > 0, 'vault');
  pass('vault_directory_root', 3);
  pass('vault_directory_structure_version');
  final currentVersion = api.currentVaultStructureVersion;
  check(currentVersion == vault.structureVersion, 'current vault version');
  check(
    api.probeVaultStructureVersion(root, password) == currentVersion,
    'vault structure probe',
  );
  pass('vault_structure_version_current', 2);
  pass('vault_directory_probe_structure_version', 2);
  vault.storePrivateKey('alice', profile);
  check(vault.privateKeyExists('alice'), 'profile');
  vault.loadPrivateKey('alice').dispose();
  vault.loadPrivateKeyGeneration('alice', 1).dispose();
  pass('vault_directory_store_private_key');
  pass('vault_directory_private_key_exists');
  pass('vault_directory_load_private_key');
  pass('vault_directory_load_private_key_generation');
  vault.storeProfileEmail('alice', 'alice@example.test');
  check(vault.profileEmail('alice') == 'alice@example.test', 'email');
  pass('vault_directory_store_profile_email');
  pass('vault_directory_profile_email', 3);
  check(
    vault.listProfileGenerations('alice').generations.length == 1,
    'history',
  );
  check(vault.rotatePrivateKey('alice').generations.length == 2, 'rotate');
  pass('vault_directory_list_profile_generations');
  pass('vault_directory_rotate_private_key');
  vault.loadProfileSigningKey('alice').dispose();
  vault.loadProfileSigningKeyGeneration('alice', 1).dispose();
  pass('vault_directory_load_owner_signing_key');
  pass('vault_directory_load_owner_signing_key_generation');
  vault.storeContact('bob', contactPublic);
  check(vault.contactExists('bob'), 'contact');
  vault.loadContact('bob').dispose();
  check(vault.listContacts().length == 1, 'contacts');
  pass('vault_directory_store_contact');
  pass('vault_directory_contact_exists');
  pass('vault_directory_load_contact');
  pass('vault_directory_list_contacts');
  vault.storeContactSigningKey('bob', ownerPublic);
  vault.loadContactSigningKey('bob').dispose();
  pass('vault_directory_store_contact_signing_key');
  pass('vault_directory_load_contact_signing_key');
  check(
    vault.listPrivateKeys().isNotEmpty &&
        vault.listPrivateKeyNames().isNotEmpty &&
        vault.listContactNames().isNotEmpty,
    'lists',
  );
  pass('vault_directory_list_private_keys');
  pass('vault_directory_list_private_key_names');
  pass('vault_directory_list_contact_names');
  vault.storeBackup(id, text('encrypted backup bytes'));
  check(
    vault.backupCount == 1 &&
        _equal(vault.loadBackup(id), text('encrypted backup bytes')),
    'backup',
  );
  pass('vault_directory_store_backup');
  pass('vault_directory_backup_count');
  pass('vault_directory_load_backup', 3);
  vault.rememberLockbox(id, '/tmp/example.lbox');
  check(vault.listKnownLockboxes().length == 1, 'known');
  pass('vault_directory_remember_lockbox');
  pass('vault_directory_list_known_lockboxes');
  vault.rememberAccessSlotLabel(id, 7, 'primary');
  check(
    vault.listAccessSlotLabels(id).length == 1 &&
        vault.findAccessSlotLabels(id, 'primary').length == 1,
    'labels',
  );
  pass('vault_directory_remember_access_slot_label');
  pass('vault_directory_list_access_slot_labels');
  pass('vault_directory_find_access_slot_labels');
  vault.rememberPassword(id, password);
  final remembered = vault.rememberedPassword(id);
  check(secretStringsEqual(remembered, password), 'remember');
  remembered.close();
  pass('vault_directory_remember_password');
  pass('vault_directory_remembered_password', 3);
  final vaultForm = vault.defineForm('login', 'Login', 'Login form', fields());
  check(
    vaultForm.typeId.isNotEmpty &&
        vault.resolveForm('login').typeId.isNotEmpty &&
        vault.listForms().isNotEmpty,
    'forms',
  );
  pass('vault_directory_define_form');
  pass('vault_directory_resolve_form');
  pass('vault_directory_list_forms');
  check(
    vault.listFormRevisions(vaultForm.typeId).isNotEmpty,
    'vault form revisions',
  );
  pass('vault_directory_list_form_revisions', 2);
  check(vault.seedForms() > 0, 'seed');
  pass('vault_directory_seed_forms');
  check(vault.listFormAliases().isNotEmpty, 'aliases');
  pass('vault_directory_list_form_aliases');
  vault.forgetAccessSlotLabel(id, 7);
  vault.forgetLockbox('/tmp/example.lbox');
  vault.deleteContact('bob');
  pass('vault_directory_forget_access_slot_label');
  pass('vault_directory_forget_lockbox');
  pass('vault_directory_delete_contact');
  vault.deletePrivateKey('alice');
  check(!vault.privateKeyExists('alice'), 'delete');
  vault.restorePrivateKey('alice', profile, owner, overwrite: true);
  check(vault.privateKeyExists('alice'), 'restore');
  pass('vault_directory_delete_private_key', 2);
  pass('vault_directory_restore_private_key', 2);
  vault.close();
  pass('vault_directory_free');
  final readonly = Vault.openReadOnly(root: root, passphrase: password);
  check(readonly.listProfileNames().isNotEmpty, 'read-only profiles');
  readonly.listContactNames();
  check(readonly.listFormAliases().isNotEmpty, 'read-only forms');
  readonly.listKnownLockboxes();
  pass('vault_read_only_open');
  pass('vault_read_only_list_profile_names', 2);
  pass('vault_read_only_list_contact_names');
  pass('vault_read_only_list_form_aliases', 2);
  pass('vault_read_only_list_known_lockboxes');
  readonly.close();
  pass('vault_read_only_free');
  final changing = Vault.open(root: root, passphrase: password);
  changing.changePassphrase(password, changed);
  changing.close();
  pass('vault_directory_change_password');
  final reopened = Vault.open(root: root, passphrase: changed);
  check(reopened.structureVersion > 0, 'reopen');
  reopened.close();
  pass('vault_directory_open');
  stdout.writeln('ARTIFACT\tdart\tvault-opened\t$root');
  final opened = Vault.openOrCreate(root: root, passphrase: changed);
  check(opened.structureVersion > 0, 'open create');
  opened.close();
  pass('vault_directory_open_or_create');
  ownerPublic.dispose();
  owner.dispose();
  contactPublic.dispose();
  contact.dispose();
  profile.dispose();
  changed.close();
  password.close();
}

void defaultVault() {
  final root = Platform.environment['LOCKBOX_VAULT_DIR']!;
  Directory(root).createSync(recursive: true);
  withSecretString(
    'default password',
    (password) => Vault.replace(passphrase: password).close(),
  );
  pass('vault_directory_replace_default');
  withSecretString(
    'default password',
    (password) => Vault.openReadOnly(passphrase: password).close(),
  );
  pass('vault_read_only_open_default');
  check(
    Vault.defaultRoot == root && File(Vault.defaultPath).parent.path == root,
    'default',
  );
  pass('vault_default_directory', 3);
  pass('vault_default_path', 2);
  final changing = withSecretString(
    'default password',
    (password) => Vault.openOrCreate(passphrase: password),
  );
  pass('vault_directory_open_or_create_default');
  withSecretString('default password', (oldPassword) {
    withSecretString('changed default password', (newPassword) {
      changing.changePassphrase(oldPassword, newPassword);
    });
  });
  changing.close();
  pass('vault_directory_change_default_password');
  final backup = File('${artifactRoot()}/default-vault.backup');
  if (backup.existsSync()) backup.deleteSync();
  check(Vault.backupDefault(backup.path).vaultSize > 0, 'backup');
  check(
    Vault.restoreDefault(backup.path, overwrite: true).vaultSize > 0,
    'restore',
  );
  pass('vault_backup_default');
  pass('vault_restore_default');
}

void platformStore() {
  check(Vault.platformCredentialStatus().backend.isNotEmpty, 'status');
  pass('vault_platform_status', 2);
  Vault.forgetPassphrase();
  check(Vault.platformCredentialsDisabled, 'disabled');
  pass('vault_platform_disable');
  pass('vault_platform_disabled');
  pass('vault_platform_forget_password');
  final password = SecretString.fromString('platform vault password');
  Vault.replace(passphrase: password).close();
  Vault.rememberPassphrase(password);
  check(!Vault.platformCredentialsDisabled, 'enabled');
  pass('vault_platform_enable');
  pass('vault_platform_set_scope');
  Vault.open().close();
  pass('vault_platform_put_password');
  pass('vault_platform_get_password', 3);
  Vault.forgetPassphrase();
  password.close();
}

Future<void> agentAndLocal() async {
  Directory(
    Platform.environment['LOCKBOX_SESSION_AGENT_DIR']!,
  ).createSync(recursive: true);
  Directory(
    Platform.environment['LOCKBOX_VAULT_DIR']!,
  ).createSync(recursive: true);
  final directory = withSecretString(
    'agent vault password',
    (password) => Vault.replace(passphrase: password),
  );
  final profile = api.generateContactKeyPair();
  directory.storePrivateKey('default', profile);
  profile.dispose();
  directory.close();
  final agent = AgentSession.instance;
  agent.clearAllSecrets();
  pass('vault_forget_all');
  final child = await Process.start(
      Platform.resolvedExecutable,
      [
        '--serve-agent',
      ],
      mode: ProcessStartMode.inheritStdio);
  var running = false;
  for (var attempt = 0; attempt < 200; attempt++) {
    if (agent.isRunning) {
      running = true;
      break;
    }
    sleep(const Duration(milliseconds: 50));
  }
  check(running, 'agent');
  pass('vault_agent_serve');
  pass('vault_is_running');
  agent.start();
  pass('vault_agent_start');
  agent.verifyTransport();
  pass('vault_agent_verify_transport');
  final id = Uint8List.fromList(List.generate(16, (i) => 0xc0 + i));
  final keyBytes = Uint8List.fromList(List.generate(32, (i) => 0x20 + i));
  final key = SecretBytes.copyOf(keyBytes);
  agent.cacheKey(id, key);
  final cachedKey = agent.key(id);
  check(
    secretBytesEqual(cachedKey, keyBytes) &&
        agent.listOpenLockboxes().isNotEmpty,
    'agent key',
  );
  cachedKey.close();
  pass('vault_agent_put');
  pass('vault_agent_get', 3);
  pass('vault_agent_list');
  agent.cacheVaultUnlockKey(
    'vault-id',
    key,
    duration: const Duration(seconds: 120),
  );
  final unlockKey = agent.vaultUnlockKey('vault-id');
  check(secretBytesEqual(unlockKey, keyBytes), 'vault key');
  unlockKey.close();
  pass('vault_agent_put_vault_unlock_key');
  pass('vault_agent_get_vault_unlock_key', 3);
  final owner = api.generateProfileSigningKeyPair();
  agent.cacheProfileSigningKey(
    'vault-id',
    'alice',
    owner,
    duration: const Duration(seconds: 120),
  );
  final loaded = agent.profileSigningKey('vault-id', 'alice');
  check(loaded.publicBytes().isNotEmpty, 'owner');
  loaded.dispose();
  pass('vault_agent_put_owner_signing_key');
  pass('vault_agent_get_owner_signing_key');
  final activity = agent.beginActivity(AgentActivityKind.open);
  pass('vault_agent_begin_activity');
  activity.dispose();
  pass('vault_agent_end_activity');
  agent.sleepSupport();
  pass('vault_agent_sleep_support');
  check(
    agent.logPath.isNotEmpty && agent.logDestination.isNotEmpty,
    'logs',
  );
  pass('vault_agent_log_path', 2);
  pass('vault_agent_log_destination', 2);
  final local = agent;
  pass('vault_local');
  final root = Directory.systemTemp.createTempSync('revault-dart-local-');
  final payload = text('local vault data');
  final passwordPath = '${root.path}/password.lbox';
  final passwordBox = withSecretString(
    'local password',
    (password) => Lockbox.create(
      passwordPath,
      password: password,
      overwrite: true,
    ),
  );
  passwordBox.addFile('/data.txt', payload);
  passwordBox.commit();
  passwordBox.close();
  pass('vault_create_lockbox_password', 3);
  withSecretString('local password', (password) {
    local.keepOpenWithPassword(
      passwordPath,
      password,
      duration: const Duration(seconds: 120),
    );
  });
  pass('vault_cache_lockbox_password');
  final passwordOpen = withSecretString(
    'local password',
    (password) => Lockbox.open(passwordPath, password: password),
  );
  check(_equal(passwordOpen.getFile('/data.txt'), payload), 'password local');
  passwordOpen.close();
  pass('vault_open_lockbox_password', 3);
  local.closeLockbox(passwordPath);
  pass('vault_close_lockbox');
  final contentPath = '${root.path}/content.lbox';
  final contentBox = Lockbox.create(
    contentPath,
    contentKey: key,
    signingKey: owner,
    overwrite: true,
  );
  contentBox.addFile('/data.txt', payload);
  contentBox.commit();
  contentBox.close();
  pass('vault_create_lockbox_content_key', 3);
  final contentOpen = Lockbox.open(contentPath, contentKey: key);
  contentOpen.setOwnerSigningKey(owner);
  check(_equal(contentOpen.getFile('/data.txt'), payload), 'content local');
  contentOpen.close();
  pass('vault_open_lockbox_content_key', 3);
  final contact = api.generateContactKeyPair(), publicKey = contact.publicKey();
  final contactBox = Lockbox.create(
    '${root.path}/contact.lbox',
    contact: publicKey,
    signingKey: owner,
    overwrite: true,
  );
  contactBox.addFile('/data.txt', payload);
  contactBox.commit();
  contactBox.close();
  pass('vault_create_lockbox_contact', 3);
  publicKey.dispose();
  contact.dispose();
  local.closeAll();
  pass('vault_close_all');
  owner.dispose();
  agent.forgetProfileSigningKey('vault-id', 'alice');
  agent.forgetVaultUnlockKey('vault-id');
  agent.forgetKey(id);
  pass('vault_agent_forget_owner_signing_key');
  pass('vault_agent_forget_vault_unlock_key');
  pass('vault_agent_forget');
  agent.stop();
  key.close();
  pass('vault_agent_stop');
  check(await child.exitCode == 0, 'agent child');
}

void interop(String producer) {
  final root = Platform.environment['REVAULT_E2E_ARTIFACT_DIR'] ??
      '/tmp/revault-e2e-artifacts';
  final box = Lockbox.openBytes(
    Uint8List.fromList(File('$root/$producer/archive.lbox').readAsBytesSync()),
    contentKey: SecretBytes.copyOf(repeat('K'.codeUnitAt(0), 32)),
  );
  check(
    _equal(box.getFile('/renamed.txt'), text('replacement payload')),
    'foreign archive',
  );
  box.close();
  final vault = withSecretString(
    'new vault password',
    (password) => Vault.open(
      root: '$root/$producer/vault',
      passphrase: password,
    ),
  );
  check(vault.structureVersion > 0, 'foreign vault');
  vault.close();
  stdout.writeln('INTEROP\tdart\t$producer\tarchive\t3');
  stdout.writeln('INTEROP\tdart\t$producer\tvault\t2');
}

Future<void> main(List<String> args) async {
  api = await Revault.load(
    nativeLibraryPath: Platform.environment['REVAULT_E2E_NATIVE_LIBRARY'],
  );
  if (args case ['--serve-agent']) {
    AgentSession.instance.serve();
    return;
  }
  if (args case ['--default']) {
    defaultVault();
    return;
  }
  if (args case ['--platform']) {
    platformStore();
    return;
  }
  if (args case ['--agent']) {
    await agentAndLocal();
    return;
  }
  if (args case ['--interop', final producer]) {
    interop(producer);
    return;
  }
  archiveLifecycle();
  keyLifecycle();
  advancedArchive();
  vaultLifecycle();
  pass('buffer_last_error');
}
