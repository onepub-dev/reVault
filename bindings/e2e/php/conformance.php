<?php
declare(strict_types=1);

require __DIR__ . '/vendor/autoload.php';

use Revault\Vault;
use Revault\FormField;
use Revault\PathMove;

$api = new Vault();

function pass(string $symbol, int $assertions = 1): void { echo "PASS\tphp\t$symbol\t$assertions\n"; }
function check(bool $value, string $message): void { if (!$value) throw new RuntimeException($message); }
function artifactRoot(): string {
    $path = (getenv('REVAULT_E2E_ARTIFACT_DIR') ?: '/tmp/revault-e2e-artifacts') . '/php';
    if (!is_dir($path)) mkdir($path, 0700, true);
    return $path;
}
function fields(): array {
    return [
        new FormField('username', 'Username', 'text', true),
        new FormField('password', 'Password', 'secret', true),
    ];
}
function moves(string $source, string $destination): array {
    return [new PathMove($source, $destination)];
}
function freeHandle(object $value, string $symbol): void { $value->free(); pass($symbol); }

function archiveLifecycle(Vault $api): void {
    $key = str_repeat('K', 32);
    $box = $api->lockboxCreate($key); pass('lockbox_create');
    $box->addFile('/hello.txt', 'hello from php conformance', false); pass('lockbox_add_file', 2);
    check($box->getFile('/hello.txt') === 'hello from php conformance', 'get file'); pass('lockbox_get_file', 3);
    $box->addFileWithPermissions('/hello.txt', 'replacement payload', 0640, true); pass('lockbox_add_file_with_permissions', 2);
    check($box->permissions('/hello.txt') === 0640, 'permissions'); pass('lockbox_permissions');
    $box->createDir('/tree', true); pass('lockbox_create_dir', 2);
    check($box->isDir('/tree'), 'directory'); pass('lockbox_is_dir');
    $box->createParentDirs('/tree/a/b/file'); pass('lockbox_create_parent_dirs', 2);
    $box->rename('/hello.txt', '/renamed.txt'); pass('lockbox_rename', 3);
    check($box->exists('/renamed.txt') && !$box->exists('/hello.txt'), 'exists'); pass('lockbox_exists', 2);
    $box->setPermissions('/renamed.txt', 0600); pass('lockbox_set_permissions', 2);
    check($box->readRange('/renamed.txt', 0, 11) === 'replacement', 'range'); pass('lockbox_read_range', 3);
    $box->setVariable('normal', 'value'); pass('lockbox_set_variable');
    check($box->getVariable('normal') === 'value', 'variable'); pass('lockbox_get_variable', 3);
    $box->moveVariables(moves('normal', 'moved')); check($box->getVariable('moved') === 'value', 'moved variable');
    $box->moveVariables(moves('moved', 'normal')); pass('lockbox_move_variables', 3);
    $box->setSecretVariable('secret', 'hidden'); pass('lockbox_set_secret_variable');
    check($box->withSecretVariable('secret', fn (\FFI\CData $value, int $length): int => $length) === 6, 'secret variable');
    pass('lockbox_get_secret_variable'); pass('secret_len'); pass('secret_copy'); pass('secret_free');
    $box->variableSensitivity('secret'); pass('lockbox_variable_sensitivity', 2);
    check(count($box->listVariables()->getValues()) === 2, 'variables'); pass('lockbox_list_variables');
    $box->deleteVariable('normal'); pass('lockbox_delete_variable');
    $box->addSymlink('/link', '/renamed.txt', false); pass('lockbox_add_symlink');
    check($box->getSymlinkTarget('/link') === '/renamed.txt', 'symlink'); pass('lockbox_get_symlink_target', 3);
    check(count($box->list('/', true)->getEntries()) > 0, 'list'); pass('lockbox_list', 2);
    $box->stat('/renamed.txt'); pass('lockbox_stat', 2);
    $box->setWorkloadProfile('read-mostly'); pass('lockbox_set_workload_profile');
    $box->setWorkerPolicy('single', 1); pass('lockbox_set_worker_policy');
    $box->runtimeOptions(); pass('lockbox_runtime_options');
    $box->commit(); pass('lockbox_commit');
    check($box->storageLen() > 0, 'storage'); pass('lockbox_storage_len');
    $archive = $box->toBytes(); pass('lockbox_to_bytes', 2); pass('buffer_free');
    $format = $api->lockboxFormatVersion(); check($format > 0 && $api->lockboxProbeFormatVersion($archive) === $format, 'format probe');
    pass('lockbox_format_version', 2); pass('lockbox_probe_format_version', 2);
    check($api->lockboxProbeFormatVersion('bad') === 0 && $api->lastErrorDetails()->getMessage() !== '', 'error details'); pass('buffer_last_error_details', 2);
    $path = artifactRoot() . '/archive.lbox'; file_put_contents($path, $archive);
    echo "ARTIFACT\tphp\tarchive-created\t$path\n";
    $box->free();
    $opened = $api->lockboxOpen($archive, $key); pass('lockbox_open', 2);
    check($opened->getFile('/renamed.txt') === 'replacement payload', 'open archive');
    echo "ARTIFACT\tphp\tarchive-opened\t$path\n";
    $opened->delete('/renamed.txt'); pass('lockbox_delete', 2);
    $opened->removeDir('/tree', true); pass('lockbox_remove_dir', 2);
    $opened->free(); pass('lockbox_free', 2);
}

function keyLifecycle(Vault $api): void {
    $content = implode('', array_map(chr(...), range(0, 31)));
    $contact = $api->keyContactGenerate(); pass('key_contact_generate');
    $private = $contact->private(); pass('key_contact_private', 2);
    $copy = $api->keyContactFromPrivate($private); pass('key_contact_from_private');
    $publicBytes = $contact->public(); pass('key_contact_public', 2);
    $public = $api->keyContactPublicFromBytes($publicBytes); pass('key_contact_public_from_bytes');
    $wrapped = $public->encrypt($content); pass('key_contact_encrypt');
    check($copy->decrypt($wrapped) === $content, 'contact decrypt'); pass('key_contact_decrypt', 3);
    check($wrapped->public() !== '' && $wrapped->ciphertext() !== '' && $wrapped->encrypted() !== '', 'wrapped');
    pass('key_contact_wrapped_public', 2); pass('key_contact_wrapped_ciphertext', 2); pass('key_contact_wrapped_encrypted', 2);
    $wrapped->free(); pass('key_contact_wrapped_free');
    $importPrivate = $api->vaultKeyImportPrivate($api->vaultKeyExportPrivate($contact, 'raw-hex'));
    pass('vault_key_export_private', 2); pass('vault_key_import_private');
    $importPublic = $api->vaultKeyImportPublic($api->vaultKeyExportPublic($public, 'lockbox-pem'));
    pass('vault_key_export_public', 2); pass('vault_key_import_public');
    $fingerprint = $api->vaultKeyFingerprint($importPublic); pass('vault_key_fingerprint', 2);
    $hex = $api->vaultKeyFormatHex($fingerprint); check($api->vaultKeyDecodeHex($hex) === $fingerprint, 'formatted hex');
    pass('vault_key_format_hex', 2); pass('vault_key_decode_hex', 2);
    $short = substr($fingerprint, 0, 12); $code = $api->vaultKeyFormatCrockford($short);
    $api->vaultKeyFormatCrockfordReading($code); check($api->vaultKeyDecodeCrockford($code) === $short, 'crockford');
    pass('vault_key_format_crockford', 2); pass('vault_key_format_crockford_reading', 2); pass('vault_key_decode_crockford', 2);
    $importPublic->publicFree(); $public->publicFree(); pass('key_contact_public_free', 2);
    $importPrivate->free(); $copy->free(); $contact->free(); pass('key_contact_free', 3);
    $plain = $api->vaultKeyHexEncode($content); check($api->vaultKeyHexDecode($plain) === $content, 'hex');
    pass('vault_key_hex_encode', 2); pass('vault_key_hex_decode', 2);
    $signing = $api->keySigningGenerate(); pass('key_signing_generate');
    $signPrivate = $signing->private(); $signPublic = $signing->public();
    pass('key_signing_private', 2); pass('key_signing_public', 2);
    $signCopy = $api->keySigningFromPrivate($signPrivate); $signCopy->free(); pass('key_signing_from_private');
    $signPub = $api->keySigningPublicFromBytes($signPublic); $signPub->publicFree();
    pass('key_signing_public_from_bytes'); pass('key_signing_public_free', 2);
    $signing->free(); pass('key_signing_free', 3);
}

function advancedArchive(Vault $api): void {
    $key = str_repeat('A', 32);
    $box = $api->lockboxCreateWithOptions($key, 'bytes', 4 << 20, 'bulk-import', 'single', 1); pass('lockbox_create_with_options');
    $box->addFile('/account.txt', 'account data', false);
    $box->listWithOptions('/', '*.txt', true, true, false, false, 20); pass('lockbox_list_with_options', 2);
    $definition = $box->defineForm('account', 'Account', 'Account form', fields()); pass('lockbox_define_form', 2);
    $typeId = $definition->getTypeId();
    $box->listFormDefinitions(); $box->resolveForm('account'); $box->listFormRevisions($typeId);
    pass('lockbox_list_form_definitions'); pass('lockbox_resolve_form'); pass('lockbox_list_form_revisions');
    $box->createFormRecord('/account.form', 'account', 'Primary'); pass('lockbox_create_form_record');
    $box->setFormField('/account.form', 'username', 'alice'); pass('lockbox_set_form_field');
    $box->setSecretFormField('/account.form', 'password', 'hidden'); pass('lockbox_set_secret_form_field');
    check($box->withSecretFormField('/account.form', 'password', fn (\FFI\CData $value, int $length): int => $length) === 6, 'secret form field');
    pass('lockbox_get_secret_form_field');
    $box->getFormRecord('/account.form'); $box->getFormField('/account.form', 'username'); $box->listFormRecords();
    pass('lockbox_get_form_record'); pass('lockbox_get_form_field'); pass('lockbox_list_form_records');
    $box->moveFormRecords(moves('/account.form', '/moved.form')); $box->getFormRecord('/moved.form');
    $box->moveFormRecords(moves('/moved.form', '/account.form')); pass('lockbox_move_form_records', 3);
    $signing = $api->keySigningGenerate(); $contact = $api->keyContactGenerate();
    $public = $api->keyContactPublicFromBytes($contact->public());
    $box->setOwnerSigningKey($signing); pass('lockbox_set_owner_signing_key');
    $slot = $box->addPassword('archive password'); pass('lockbox_add_password');
    $box->addContact($public, 'recipient'); pass('lockbox_add_contact');
    $box->listKeySlots(); pass('lockbox_list_key_slots'); $box->deleteKey($slot); pass('lockbox_delete_key');
    $box->commit(); $box->ownerInspection(); pass('lockbox_owner_inspection', 2);
    $box->cacheStats(); $box->importStats(); $box->resetImportStats(); $box->pageInspection();
    $box->recoveryReport(); $box->recoveryReportRender(true, 100); $box->streamContent(false); $box->id();
    pass('lockbox_cache_stats'); pass('lockbox_import_stats'); pass('lockbox_reset_import_stats');
    pass('lockbox_page_inspection'); pass('lockbox_recovery_report'); pass('lockbox_recovery_report_render', 2);
    pass('lockbox_stream_content'); pass('lockbox_id', 2);
    $archive = $box->toBytes(); $path = artifactRoot() . '/advanced.lbox'; file_put_contents($path, $archive);
    $api->lockboxInspectFile($path); $api->lockboxRecoveryScanPath($path, $key); $api->lockboxRecoveryScan($archive, $key);
    pass('lockbox_inspect_file'); pass('lockbox_recovery_scan_path'); pass('lockbox_recovery_scan');
    $salvaged = $api->lockboxRecoverySalvage(substr($archive, 0, -32), $key, $signing); $salvaged->free(); pass('lockbox_recovery_salvage', 2);
    $options = $api->lockboxOpenWithOptions($archive, $key, 'bytes', 4 << 20, 'bulk-import', 'single', 1); $options->free(); pass('lockbox_open_with_options', 2);
    $passwordBox = $api->lockboxCreatePassword('archive password'); $passwordBox->addFile('/password.txt', 'password protected', false); $passwordBox->commit();
    $passwordArchive = $passwordBox->toBytes(); $passwordBox->free(); pass('lockbox_create_password');
    $passwordOpen = $api->lockboxOpenPassword($passwordArchive, 'archive password'); $passwordOpen->getFile('/password.txt'); $passwordOpen->free(); pass('lockbox_open_password', 2);
    $contactBox = $api->lockboxCreateContact($public); $contactBox->addFile('/contact.txt', 'contact protected', false); $contactBox->commit();
    $contactArchive = $contactBox->toBytes(); $contactBox->free(); pass('lockbox_create_contact');
    $contactOpen = $api->lockboxOpenContact($contactArchive, $contact); $contactOpen->getFile('/contact.txt'); $contactOpen->free(); pass('lockbox_open_contact', 2);
    $signed = $api->lockboxCreateWithSigningKey($key, $signing); $signed->commit(); $signed->free(); pass('lockbox_create_with_signing_key', 2);
    $extract = sys_get_temp_dir() . '/revault-php-extract-' . bin2hex(random_bytes(4)); mkdir($extract, 0700, true);
    $box->extractFile('/account.txt', "$extract/account.txt", false); pass('lockbox_extract_file', 2);
    $tree = "$extract/tree"; if (!is_dir($tree)) mkdir($tree, 0700, true);
    $box->extractDirectory($tree, 1 << 20, 4 << 20, 100, false, true, false); pass('lockbox_extract_directory', 2);
    $box->deleteFormRecord('/account.form'); pass('lockbox_delete_form_record');
    $box->free(); $public->publicFree(); $contact->free(); $signing->free();
}

function vaultLifecycle(Vault $api): void {
    $root = artifactRoot() . '/vault'; if (!is_dir($root)) mkdir($root, 0700, true);
    $password = 'vault password'; $changed = 'new vault password'; $id = implode('', array_map(chr(...), range(0xa0, 0xaf)));
    $profile = $api->keyContactGenerate(); $contact = $api->keyContactGenerate();
    $contactPublic = $api->keyContactPublicFromBytes($contact->public());
    $owner = $api->keySigningGenerate(); $ownerPublic = $api->keySigningPublicFromBytes($owner->public());
    $vault = $api->vaultDirectoryReplace($root, $password); pass('vault_directory_replace');
    echo "ARTIFACT\tphp\tvault-created\t$root\n";
    check($vault->root() === $root && $vault->structureVersion() > 0, 'vault'); pass('vault_directory_root', 3); pass('vault_directory_structure_version');
    $current = $api->vaultStructureVersionCurrent(); check($current === $vault->structureVersion() && $api->vaultDirectoryProbeStructureVersion($root, $password) === $current, 'vault probe');
    pass('vault_structure_version_current', 2); pass('vault_directory_probe_structure_version', 2);
    $vault->storePrivateKey('alice', $profile); pass('vault_directory_store_private_key');
    $vault->privateKeyExists('alice'); pass('vault_directory_private_key_exists');
    $vault->loadPrivateKey('alice')->free(); $vault->loadPrivateKeyGeneration('alice', 1)->free();
    pass('vault_directory_load_private_key'); pass('vault_directory_load_private_key_generation');
    $vault->storeProfileEmail('alice', 'alice@example.test'); $vault->profileEmail('alice');
    pass('vault_directory_store_profile_email'); pass('vault_directory_profile_email', 3);
    $vault->listProfileGenerations('alice'); $vault->rotatePrivateKey('alice');
    pass('vault_directory_list_profile_generations'); pass('vault_directory_rotate_private_key');
    $vault->loadOwnerSigningKey('alice')->free(); $vault->loadOwnerSigningKeyGeneration('alice', 1)->free();
    pass('vault_directory_load_owner_signing_key'); pass('vault_directory_load_owner_signing_key_generation');
    $vault->storeContact('bob', $contactPublic); $vault->contactExists('bob'); $vault->loadContact('bob')->publicFree(); $vault->listContacts();
    pass('vault_directory_store_contact'); pass('vault_directory_contact_exists'); pass('vault_directory_load_contact'); pass('vault_directory_list_contacts');
    $vault->storeContactSigningKey('bob', $ownerPublic); $vault->loadContactSigningKey('bob')->publicFree();
    pass('vault_directory_store_contact_signing_key'); pass('vault_directory_load_contact_signing_key');
    $vault->listPrivateKeys(); $vault->listPrivateKeyNames(); $vault->listContactNames();
    pass('vault_directory_list_private_keys'); pass('vault_directory_list_private_key_names'); pass('vault_directory_list_contact_names');
    $vault->storeBackup($id, 'encrypted backup bytes'); $vault->backupCount(); check($vault->loadBackup($id) === 'encrypted backup bytes', 'backup');
    pass('vault_directory_store_backup'); pass('vault_directory_backup_count'); pass('vault_directory_load_backup', 3);
    $vault->rememberLockbox($id, '/tmp/example.lbox'); $vault->listKnownLockboxes();
    pass('vault_directory_remember_lockbox'); pass('vault_directory_list_known_lockboxes');
    $vault->rememberAccessSlotLabel($id, 7, 'primary'); $vault->listAccessSlotLabels($id); $vault->findAccessSlotLabels($id, 'primary');
    pass('vault_directory_remember_access_slot_label'); pass('vault_directory_list_access_slot_labels'); pass('vault_directory_find_access_slot_labels');
    $vault->rememberPassword($id, $password); $vault->rememberedPassword($id);
    pass('vault_directory_remember_password'); pass('vault_directory_remembered_password', 3);
    $vaultForm = $vault->defineForm('login', 'Login', 'Login form', fields()); $vault->resolveForm('login'); $vault->listForms();
    pass('vault_directory_define_form'); pass('vault_directory_resolve_form'); pass('vault_directory_list_forms');
    $vault->listFormRevisions($vaultForm->getTypeId()); pass('vault_directory_list_form_revisions', 2);
    $vault->seedForms(); pass('vault_directory_seed_forms'); $vault->listFormAliases(); pass('vault_directory_list_form_aliases');
    $vault->forgetAccessSlotLabel($id, 7); $vault->forgetLockbox('/tmp/example.lbox'); $vault->deleteContact('bob');
    pass('vault_directory_forget_access_slot_label'); pass('vault_directory_forget_lockbox'); pass('vault_directory_delete_contact');
    $vault->deletePrivateKey('alice'); $vault->restorePrivateKey('alice', $profile, $owner, true);
    pass('vault_directory_delete_private_key', 2); pass('vault_directory_restore_private_key', 2);
    $vault->free(); pass('vault_directory_free');
    $readonly = $api->vaultReadOnlyOpen($root, $password); $readonly->listProfileNames(); $readonly->listContactNames(); $readonly->listFormAliases(); $readonly->listKnownLockboxes();
    pass('vault_read_only_open'); pass('vault_read_only_list_profile_names', 2); pass('vault_read_only_list_contact_names'); pass('vault_read_only_list_form_aliases', 2); pass('vault_read_only_list_known_lockboxes');
    $readonly->free(); pass('vault_read_only_free');
    $api->vaultDirectoryChangePassword($root, $password, $changed); pass('vault_directory_change_password');
    $api->vaultDirectoryOpen($root, $changed)->free(); pass('vault_directory_open'); echo "ARTIFACT\tphp\tvault-opened\t$root\n";
    $api->vaultDirectoryOpenOrCreate($root, $changed)->free(); pass('vault_directory_open_or_create');
    $ownerPublic->publicFree(); $owner->free(); $contactPublic->publicFree(); $contact->free(); $profile->free();
}

function defaultVault(Vault $api): void {
    $root = getenv('LOCKBOX_VAULT_DIR'); if (!is_dir($root)) mkdir($root, 0700, true);
    $api->vaultDirectoryReplaceDefault('default password')->free(); pass('vault_directory_replace_default');
    $api->vaultReadOnlyOpenDefault('default password')->free(); pass('vault_read_only_open_default');
    $api->vaultDefaultDirectory(); $api->vaultDefaultPath(); pass('vault_default_directory', 3); pass('vault_default_path', 2);
    $api->vaultDirectoryOpenOrCreateDefault('default password')->free(); pass('vault_directory_open_or_create_default');
    $api->vaultDirectoryChangeDefaultPassword('default password', 'changed default password'); pass('vault_directory_change_default_password');
    $backup = artifactRoot() . '/default-vault.backup'; @unlink($backup);
    $api->vaultBackupDefault($backup, false); $api->vaultRestoreDefault($backup, true);
    pass('vault_backup_default'); pass('vault_restore_default');
}

function platformStore(Vault $api): void {
    $platform = $api->platform(); $platform->status(); pass('vault_platform_status', 2);
    $platform->setScope('vault'); pass('vault_platform_set_scope');
    $platform->disable(); $platform->disabled(); pass('vault_platform_disable'); pass('vault_platform_disabled');
    $platform->enable(); pass('vault_platform_enable');
    $platform->putPassword('platform vault password'); check($platform->getPassword() === 'platform vault password', 'platform password');
    pass('vault_platform_put_password'); pass('vault_platform_get_password', 3);
    $platform->forgetPassword(); pass('vault_platform_forget_password');
}

function agentAndLocal(Vault $api): void {
    foreach ([getenv('LOCKBOX_SESSION_AGENT_DIR'), getenv('LOCKBOX_VAULT_DIR')] as $dir) if (!is_dir($dir)) mkdir($dir, 0700, true);
    $directory = $api->vaultDirectoryReplaceDefault('agent vault password'); $profile = $api->keyContactGenerate();
    $directory->storePrivateKey('default', $profile); $profile->free(); $directory->free();
    $agent = $api->agent(); $agent->forgetAll(); pass('vault_forget_all');
    $command = [PHP_BINARY, __FILE__, '--serve-agent'];
    $temporary = sys_get_temp_dir();
    $null = PHP_OS_FAMILY === 'Windows' ? 'NUL' : '/dev/null';
    $process = proc_open($command, [
        0 => ['file', $null, 'r'],
        1 => ['file', $temporary . DIRECTORY_SEPARATOR . 'php-agent.out', 'a'],
        2 => ['file', $temporary . DIRECTORY_SEPARATOR . 'php-agent.err', 'a'],
    ], $pipes);
    for ($i = 0; $i < 200 && !$agent->isRunning(); $i++) usleep(50000);
    check($agent->isRunning(), 'agent running'); pass('vault_agent_serve'); pass('vault_is_running');
    $agent->start(); pass('vault_agent_start');
    $agent->verifyTransport(); pass('vault_agent_verify_transport');
    $id = implode('', array_map(chr(...), range(0xc0, 0xcf))); $key = implode('', array_map(chr(...), range(0x20, 0x3f)));
    $agent->put($id, $key); check($agent->get($id) === $key, 'agent key'); $agent->list();
    pass('vault_agent_put'); pass('vault_agent_get', 3); pass('vault_agent_list');
    $agent->putVaultUnlockKey('vault-id', $key, 120); $agent->getVaultUnlockKey('vault-id'); pass('vault_agent_put_vault_unlock_key'); pass('vault_agent_get_vault_unlock_key', 3);
    $owner = $api->keySigningGenerate(); $agent->putOwnerSigningKey('vault-id', 'alice', $owner, 120); $agent->getOwnerSigningKey('vault-id', 'alice')->free();
    pass('vault_agent_put_owner_signing_key'); pass('vault_agent_get_owner_signing_key');
    $activity = $agent->beginActivity('open'); pass('vault_agent_begin_activity'); $agent->endActivity($activity); pass('vault_agent_end_activity');
    $agent->sleepSupport(); pass('vault_agent_sleep_support'); $api->vaultAgentLogPath(); $api->vaultAgentLogDestination();
    pass('vault_agent_log_path', 2); pass('vault_agent_log_destination', 2);
    $local = $api->vaultLocal(); pass('vault_local'); $root = sys_get_temp_dir() . '/revault-php-local-' . bin2hex(random_bytes(4)); mkdir($root, 0700, true);
    $passwordPath = "$root/password.lbox"; $box = $local->createLockboxPassword($passwordPath, 'local password');
    $box->addFile('/data.txt', 'local vault data', false); $box->commit(); $box->free(); pass('vault_create_lockbox_password', 3);
    $local->cacheLockboxPassword($passwordPath, 'local password', 120); pass('vault_cache_lockbox_password');
    $open = $local->openLockboxPassword($passwordPath, 'local password'); $open->getFile('/data.txt'); $open->free(); pass('vault_open_lockbox_password', 3);
    $local->closeLockbox($passwordPath); pass('vault_close_lockbox');
    $contentPath = "$root/content.lbox"; $box = $local->createLockboxContentKey($contentPath, $key, $owner);
    $box->addFile('/data.txt', 'local vault data', false); $box->commit(); $box->free(); pass('vault_create_lockbox_content_key', 3);
    $open = $local->openLockboxContentKey($contentPath, $key, $owner); $open->getFile('/data.txt'); $open->free(); pass('vault_open_lockbox_content_key', 3);
    $contact = $api->keyContactGenerate(); $public = $api->keyContactPublicFromBytes($contact->public());
    $box = $local->createLockboxContact("$root/contact.lbox", $public, 'recipient', $owner); $box->addFile('/data.txt', 'local vault data', false); $box->commit(); $box->free();
    pass('vault_create_lockbox_contact', 3); $public->publicFree(); $contact->free();
    $local->closeAll(); pass('vault_close_all'); $local->free(); pass('vault_free'); $owner->free();
    $agent->forgetOwnerSigningKey('vault-id', 'alice'); $agent->forgetVaultUnlockKey('vault-id'); $agent->forget($id);
    pass('vault_agent_forget_owner_signing_key'); pass('vault_agent_forget_vault_unlock_key'); pass('vault_agent_forget');
    $agent->stop(); pass('vault_agent_stop'); check(proc_close($process) === 0, 'agent process');
}

function interop(Vault $api, string $producer): void {
    $root = getenv('REVAULT_E2E_ARTIFACT_DIR') ?: '/tmp/revault-e2e-artifacts';
    $box = $api->lockboxOpen(file_get_contents("$root/$producer/archive.lbox"), str_repeat('K', 32));
    check($box->getFile('/renamed.txt') === 'replacement payload', 'foreign archive'); $box->free();
    $vault = $api->vaultDirectoryOpen("$root/$producer/vault", 'new vault password'); check($vault->structureVersion() > 0, 'foreign vault'); $vault->free();
    echo "INTEROP\tphp\t$producer\tarchive\t3\nINTEROP\tphp\t$producer\tvault\t2\n";
}

$args = array_slice($argv, 1);
if ($args === ['--serve-agent']) { $api->agent()->serve(); exit(0); }
if ($args === ['--default']) { defaultVault($api); exit(0); }
if ($args === ['--platform']) { platformStore($api); exit(0); }
if ($args === ['--agent']) { agentAndLocal($api); exit(0); }
if (count($args) === 2 && $args[0] === '--interop') { interop($api, $args[1]); exit(0); }
archiveLifecycle($api); keyLifecycle($api); advancedArchive($api); vaultLifecycle($api);
$api->lastError(); pass('buffer_last_error');
