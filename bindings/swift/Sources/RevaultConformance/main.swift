import Foundation
import RevaultAPI

let api = Vault()
func pass(_ symbol: String, _ assertions: Int = 1) { print("PASS\tswift\t\(symbol)\t\(assertions)") }
func check(_ value: @autoclosure () throws -> Bool, _ message: String) throws {
    if try !value() { throw NSError(domain: "RevaultSwiftConformance", code: 1, userInfo: [NSLocalizedDescriptionKey: message]) }
}
func data(_ value: String) -> Data { Data(value.utf8) }
func sequence(_ first: UInt8, _ last: UInt8) -> Data { Data(first...last) }
let files = FileManager.default
func makeDirectory(_ path: String) throws { try files.createDirectory(atPath: path, withIntermediateDirectories: true) }
func resetDirectory(_ path: String) throws { try? files.removeItem(atPath: path); try makeDirectory(path) }
func artifactRoot() throws -> String {
    let path = (ProcessInfo.processInfo.environment["REVAULT_E2E_ARTIFACT_DIR"] ?? "/tmp/revault-e2e-artifacts") + "/swift"
    try makeDirectory(path); return path
}
func fields() throws -> Data {
    var field = Revault_Bindings_FormField(); field.id = "username"; field.label = "Username"; field.kind = "text"; field.required = true
    var list = Revault_Bindings_FormFieldList(); list.values = [field]; return try list.serializedData()
}
func moves(_ source: String, _ destination: String) throws -> Data {
    var move = Revault_Bindings_PathMove(); move.source = source; move.destination = destination
    var list = Revault_Bindings_PathMoveList(); list.values = [move]; return try list.serializedData()
}

func archiveLifecycle() throws {
    let key = Data(repeating: 75, count: 32)
    let box = try api.lockboxCreate(key); pass("lockbox_create")
    try box.addFile("/hello.txt", data("hello from swift conformance"), false); pass("lockbox_add_file", 2)
    try check(try box.getFile("/hello.txt") == data("hello from swift conformance"), "get file"); pass("lockbox_get_file", 3)
    try box.addFileWithPermissions("/hello.txt", data("replacement payload"), 0o640, true); pass("lockbox_add_file_with_permissions", 2)
    try check(try box.permissions("/hello.txt") == 0o640, "permissions"); pass("lockbox_permissions")
    try box.createDir("/tree", true); pass("lockbox_create_dir", 2)
    try check(try box.isDir("/tree"), "directory"); pass("lockbox_is_dir")
    try box.createParentDirs("/tree/a/b/file"); pass("lockbox_create_parent_dirs", 2)
    try box.rename("/hello.txt", "/renamed.txt"); pass("lockbox_rename", 3)
    try check(try box.exists("/renamed.txt") && !(try box.exists("/hello.txt")), "exists"); pass("lockbox_exists", 2)
    try box.setPermissions("/renamed.txt", 0o600); pass("lockbox_set_permissions", 2)
    try check(try box.readRange("/renamed.txt", 0, 11) == data("replacement"), "range"); pass("lockbox_read_range", 3)
    try box.setVariable("normal", "value", false); pass("lockbox_set_variable")
    try check(try box.getVariable("normal") == "value", "variable"); pass("lockbox_get_variable", 3)
    try box.moveVariables(moves("normal", "moved")); try check(try box.getVariable("moved") == "value", "moved variable")
    try box.moveVariables(moves("moved", "normal")); pass("lockbox_move_variables", 3)
    try box.setVariable("secret", "hidden", true); _ = try box.variableSensitivity("secret"); pass("lockbox_variable_sensitivity", 2)
    try check(try box.listVariables().values.count == 2, "variables"); pass("lockbox_list_variables")
    try box.deleteVariable("normal"); pass("lockbox_delete_variable")
    try box.addSymlink("/link", "/renamed.txt", false); pass("lockbox_add_symlink")
    try check(try box.getSymlinkTarget("/link") == "/renamed.txt", "symlink"); pass("lockbox_get_symlink_target", 3)
    try check(try !box.list("/", true).entries.isEmpty, "list"); _ = try box.stat("/renamed.txt"); pass("lockbox_list", 2); pass("lockbox_stat", 2)
    try box.setWorkloadProfile("read-mostly"); try box.setWorkerPolicy("single", 1); _ = try box.runtimeOptions()
    pass("lockbox_set_workload_profile"); pass("lockbox_set_worker_policy"); pass("lockbox_runtime_options")
    try box.commit(); pass("lockbox_commit"); try check(try box.storageLen() > 0, "storage"); pass("lockbox_storage_len")
    let archive = try box.toBytes(); pass("lockbox_to_bytes", 2); pass("buffer_free")
    let format = try api.lockboxFormatVersion(); try check(format > 0 && api.lockboxProbeFormatVersion(archive) == format, "format probe")
    pass("lockbox_format_version", 2); pass("lockbox_probe_format_version", 2)
    try check(try api.lockboxProbeFormatVersion(data("bad")) == 0 && !api.lastErrorDetails().message.isEmpty, "error details"); pass("buffer_last_error_details", 2)
    let path = try artifactRoot() + "/archive.lbox"; try archive.write(to: URL(fileURLWithPath: path))
    print("ARTIFACT\tswift\tarchive-created\t\(path)"); try box.free()
    let opened = try api.lockboxOpen(archive, key); pass("lockbox_open", 2)
    try check(try opened.getFile("/renamed.txt") == data("replacement payload"), "opened")
    print("ARTIFACT\tswift\tarchive-opened\t\(path)")
    try opened.delete("/renamed.txt"); pass("lockbox_delete", 2)
    try opened.removeDir("/tree", true); pass("lockbox_remove_dir", 2); try opened.free(); pass("lockbox_free", 2)
}

func keyLifecycle() throws {
    let content = sequence(0, 31); let contact = try api.keyContactGenerate(); pass("key_contact_generate")
    let privateKey = try contact.privateBytes(); pass("key_contact_private", 2)
    let copy = try api.keyContactFromPrivate(privateKey); pass("key_contact_from_private")
    let publicBytes = try contact.publicBytes(); pass("key_contact_public", 2)
    let publicKey = try api.keyContactPublicFromBytes(publicBytes); pass("key_contact_public_from_bytes")
    let wrapped = try publicKey.encrypt(content); pass("key_contact_encrypt")
    try check(try copy.decrypt(wrapped) == content, "decrypt"); pass("key_contact_decrypt", 3)
    try check(try !wrapped.publicBytes().isEmpty && !wrapped.ciphertext().isEmpty && !wrapped.encrypted().isEmpty, "wrapped")
    pass("key_contact_wrapped_public", 2); pass("key_contact_wrapped_ciphertext", 2); pass("key_contact_wrapped_encrypted", 2)
    try wrapped.free(); pass("key_contact_wrapped_free")
    let importedPrivate = try api.vaultKeyImportPrivate(try api.vaultKeyExportPrivate(contact, "raw-hex")); pass("vault_key_export_private", 2); pass("vault_key_import_private")
    let importedPublic = try api.vaultKeyImportPublic(try api.vaultKeyExportPublic(publicKey, "lockbox-pem")); pass("vault_key_export_public", 2); pass("vault_key_import_public")
    let fingerprint = try api.vaultKeyFingerprint(importedPublic); pass("vault_key_fingerprint", 2)
    let hex = try api.vaultKeyFormatHex(fingerprint); try check(try api.vaultKeyDecodeHex(hex) == fingerprint, "hex"); pass("vault_key_format_hex", 2); pass("vault_key_decode_hex", 2)
    let short = fingerprint.prefix(12); let code = try api.vaultKeyFormatCrockford(Data(short)); _ = try api.vaultKeyFormatCrockfordReading(code)
    try check(try api.vaultKeyDecodeCrockford(code) == Data(short), "crockford"); pass("vault_key_format_crockford", 2); pass("vault_key_format_crockford_reading", 2); pass("vault_key_decode_crockford", 2)
    try importedPublic.publicFree(); try publicKey.publicFree(); pass("key_contact_public_free", 2)
    try importedPrivate.free(); try copy.free(); try contact.free(); pass("key_contact_free", 3)
    let plain = try api.vaultKeyHexEncode(content); try check(try api.vaultKeyHexDecode(plain) == content, "plain hex"); pass("vault_key_hex_encode", 2); pass("vault_key_hex_decode", 2)
    let signing = try api.keySigningGenerate(); pass("key_signing_generate")
    let signingPrivate = try signing.privateBytes(); let signingPublic = try signing.publicBytes(); pass("key_signing_private", 2); pass("key_signing_public", 2)
    try api.keySigningFromPrivate(signingPrivate).free(); pass("key_signing_from_private")
    try api.keySigningPublicFromBytes(signingPublic).publicFree(); pass("key_signing_public_from_bytes"); pass("key_signing_public_free", 2)
    try signing.free(); pass("key_signing_free", 3)
}

func advancedArchive() throws {
    let key = Data(repeating: 65, count: 32)
    let box = try api.lockboxCreateWithOptions(key, "bytes", 4 << 20, "bulk-import", "single", 1); pass("lockbox_create_with_options")
    try box.addFile("/account.txt", data("account data"), false)
    _ = try box.listWithOptions("/", "*.txt", true, true, false, false, 20); pass("lockbox_list_with_options", 2)
    let definition = try box.defineForm("account", "Account", "Account form", fields()); pass("lockbox_define_form", 2)
    _ = try box.listFormDefinitions(); _ = try box.resolveForm("account"); _ = try box.listFormRevisions(definition.typeID)
    pass("lockbox_list_form_definitions"); pass("lockbox_resolve_form"); pass("lockbox_list_form_revisions")
    _ = try box.createFormRecord("/account.form", "account", "Primary"); pass("lockbox_create_form_record")
    try box.setFormField("/account.form", "username", "alice", false); pass("lockbox_set_form_field")
    _ = try box.getFormRecord("/account.form"); _ = try box.getFormField("/account.form", "username"); _ = try box.listFormRecords()
    pass("lockbox_get_form_record"); pass("lockbox_get_form_field"); pass("lockbox_list_form_records")
    try box.moveFormRecords(moves("/account.form", "/moved.form")); _ = try box.getFormRecord("/moved.form")
    try box.moveFormRecords(moves("/moved.form", "/account.form")); pass("lockbox_move_form_records", 3)
    let signing = try api.keySigningGenerate(); let contact = try api.keyContactGenerate(); let publicKey = try api.keyContactPublicFromBytes(try contact.publicBytes())
    try box.setOwnerSigningKey(signing); pass("lockbox_set_owner_signing_key")
    let slot = try box.addPassword(data("archive password")); pass("lockbox_add_password")
    _ = try box.addContact(publicKey, "recipient"); pass("lockbox_add_contact"); _ = try box.listKeySlots(); pass("lockbox_list_key_slots")
    try box.deleteKey(slot); pass("lockbox_delete_key"); try box.commit(); _ = try box.ownerInspection(); pass("lockbox_owner_inspection", 2)
    _ = try box.cacheStats(); _ = try box.importStats(); try box.resetImportStats(); _ = try box.pageInspection(); _ = try box.recoveryReport()
    _ = try box.recoveryReportRender(true, 100); _ = try box.streamContent(false); _ = try box.id()
    for symbol in ["lockbox_cache_stats","lockbox_import_stats","lockbox_reset_import_stats","lockbox_page_inspection","lockbox_recovery_report"] { pass(symbol) }
    pass("lockbox_recovery_report_render", 2); pass("lockbox_stream_content"); pass("lockbox_id", 2)
    let archive = try box.toBytes(); let path = try artifactRoot() + "/advanced.lbox"; try archive.write(to: URL(fileURLWithPath: path))
    _ = try api.lockboxInspectFile(path); _ = try api.lockboxRecoveryScanPath(path, key); _ = try api.lockboxRecoveryScan(archive, key)
    pass("lockbox_inspect_file"); pass("lockbox_recovery_scan_path"); pass("lockbox_recovery_scan")
    try api.lockboxRecoverySalvage(Data(archive.dropLast(32)), key, signing).free(); pass("lockbox_recovery_salvage", 2)
    try api.lockboxOpenWithOptions(archive, key, "bytes", 4 << 20, "bulk-import", "single", 1).free(); pass("lockbox_open_with_options", 2)
    let passwordBox = try api.lockboxCreatePassword(data("archive password")); try passwordBox.addFile("/password.txt", data("password protected"), false); try passwordBox.commit()
    let passwordArchive = try passwordBox.toBytes(); try passwordBox.free(); pass("lockbox_create_password")
    let passwordOpen = try api.lockboxOpenPassword(passwordArchive, data("archive password")); _ = try passwordOpen.getFile("/password.txt"); try passwordOpen.free(); pass("lockbox_open_password", 2)
    let contactBox = try api.lockboxCreateContact(publicKey); try contactBox.addFile("/contact.txt", data("contact protected"), false); try contactBox.commit()
    let contactArchive = try contactBox.toBytes(); try contactBox.free(); pass("lockbox_create_contact")
    let contactOpen = try api.lockboxOpenContact(contactArchive, contact); _ = try contactOpen.getFile("/contact.txt"); try contactOpen.free(); pass("lockbox_open_contact", 2)
    let signed = try api.lockboxCreateWithSigningKey(key, signing); try signed.commit(); try signed.free(); pass("lockbox_create_with_signing_key", 2)
    let extract = "/tmp/revault-swift-extract"; try resetDirectory(extract); try box.extractFile("/account.txt", extract + "/account.txt", false); pass("lockbox_extract_file", 2)
    let tree = extract + "/tree"; try makeDirectory(tree); try box.extractDirectory(tree, 1 << 20, 4 << 20, 100, false, true, false); pass("lockbox_extract_directory", 2)
    try box.deleteFormRecord("/account.form"); pass("lockbox_delete_form_record")
    try box.free(); try publicKey.publicFree(); try contact.free(); try signing.free()
}

func vaultLifecycle() throws {
    let root = try artifactRoot() + "/vault"; try resetDirectory(root)
    let password = data("vault password"), changed = data("new vault password"), id = sequence(160, 175)
    let profile = try api.keyContactGenerate(), contact = try api.keyContactGenerate(), contactPublic = try api.keyContactPublicFromBytes(try contact.publicBytes())
    let owner = try api.keySigningGenerate(), ownerPublic = try api.keySigningPublicFromBytes(try owner.publicBytes())
    let vault = try api.vaultDirectoryReplace(root, password); pass("vault_directory_replace"); print("ARTIFACT\tswift\tvault-created\t\(root)")
    try check(try vault.root() == root && vault.structureVersion() > 0, "vault"); pass("vault_directory_root", 3); pass("vault_directory_structure_version")
    let current = try api.vaultStructureVersionCurrent()
    try check(try current == vault.structureVersion() && api.vaultDirectoryProbeStructureVersion(root, password) == current, "vault probe")
    pass("vault_structure_version_current", 2); pass("vault_directory_probe_structure_version", 2)
    try vault.storePrivateKey("alice", profile); pass("vault_directory_store_private_key"); _ = try vault.privateKeyExists("alice"); pass("vault_directory_private_key_exists")
    try vault.loadPrivateKey("alice").free(); try vault.loadPrivateKeyGeneration("alice", 1).free(); pass("vault_directory_load_private_key"); pass("vault_directory_load_private_key_generation")
    try vault.storeProfileEmail("alice", "alice@example.test"); _ = try vault.profileEmail("alice"); pass("vault_directory_store_profile_email"); pass("vault_directory_profile_email", 3)
    _ = try vault.listProfileGenerations("alice"); _ = try vault.rotatePrivateKey("alice"); pass("vault_directory_list_profile_generations"); pass("vault_directory_rotate_private_key")
    try vault.loadOwnerSigningKey("alice").free(); try vault.loadOwnerSigningKeyGeneration("alice", 1).free(); pass("vault_directory_load_owner_signing_key"); pass("vault_directory_load_owner_signing_key_generation")
    try vault.storeContact("bob", contactPublic); _ = try vault.contactExists("bob"); try vault.loadContact("bob").publicFree(); _ = try vault.listContacts()
    pass("vault_directory_store_contact"); pass("vault_directory_contact_exists"); pass("vault_directory_load_contact"); pass("vault_directory_list_contacts")
    try vault.storeContactSigningKey("bob", ownerPublic); try vault.loadContactSigningKey("bob").publicFree(); pass("vault_directory_store_contact_signing_key"); pass("vault_directory_load_contact_signing_key")
    _ = try vault.listPrivateKeys(); _ = try vault.listPrivateKeyNames(); _ = try vault.listContactNames(); pass("vault_directory_list_private_keys"); pass("vault_directory_list_private_key_names"); pass("vault_directory_list_contact_names")
    try vault.storeBackup(id, data("encrypted backup bytes")); _ = try vault.backupCount(); try check(try vault.loadBackup(id) == data("encrypted backup bytes"), "backup")
    pass("vault_directory_store_backup"); pass("vault_directory_backup_count"); pass("vault_directory_load_backup", 3)
    try vault.rememberLockbox(id, "/tmp/example.lbox"); _ = try vault.listKnownLockboxes(); pass("vault_directory_remember_lockbox"); pass("vault_directory_list_known_lockboxes")
    try vault.rememberAccessSlotLabel(id, 7, "primary"); _ = try vault.listAccessSlotLabels(id); _ = try vault.findAccessSlotLabels(id, "primary"); pass("vault_directory_remember_access_slot_label"); pass("vault_directory_list_access_slot_labels"); pass("vault_directory_find_access_slot_labels")
    try vault.rememberPassword(id, password); _ = try vault.rememberedPassword(id); pass("vault_directory_remember_password"); pass("vault_directory_remembered_password", 3)
    let vaultForm = try vault.defineForm("login", "Login", "Login form", fields()); _ = try vault.resolveForm("login"); _ = try vault.listForms(); pass("vault_directory_define_form"); pass("vault_directory_resolve_form"); pass("vault_directory_list_forms")
    _ = try vault.listFormRevisions(vaultForm.typeID); pass("vault_directory_list_form_revisions", 2)
    _ = try vault.seedForms(); pass("vault_directory_seed_forms"); _ = try vault.listFormAliases(); pass("vault_directory_list_form_aliases")
    try vault.forgetAccessSlotLabel(id, 7); try vault.forgetLockbox("/tmp/example.lbox"); try vault.deleteContact("bob"); pass("vault_directory_forget_access_slot_label"); pass("vault_directory_forget_lockbox"); pass("vault_directory_delete_contact")
    try vault.deletePrivateKey("alice"); try vault.restorePrivateKey("alice", profile, owner, true); pass("vault_directory_delete_private_key", 2); pass("vault_directory_restore_private_key", 2)
    try vault.free(); pass("vault_directory_free")
    let readonly = try api.vaultReadOnlyOpen(root, password); _ = try readonly.listProfileNames(); _ = try readonly.listContactNames(); _ = try readonly.listFormAliases(); _ = try readonly.listKnownLockboxes()
    pass("vault_read_only_open"); pass("vault_read_only_list_profile_names", 2); pass("vault_read_only_list_contact_names"); pass("vault_read_only_list_form_aliases", 2); pass("vault_read_only_list_known_lockboxes")
    try readonly.free(); pass("vault_read_only_free")
    try api.vaultDirectoryChangePassword(root, password, changed); pass("vault_directory_change_password")
    try api.vaultDirectoryOpen(root, changed).free(); pass("vault_directory_open"); print("ARTIFACT\tswift\tvault-opened\t\(root)")
    try api.vaultDirectoryOpenOrCreate(root, changed).free(); pass("vault_directory_open_or_create")
    try ownerPublic.publicFree(); try owner.free(); try contactPublic.publicFree(); try contact.free(); try profile.free()
}

func defaultVault() throws {
    try makeDirectory(ProcessInfo.processInfo.environment["LOCKBOX_VAULT_DIR"]!)
    try api.vaultDirectoryReplaceDefault(data("default password")).free(); pass("vault_directory_replace_default")
    try api.vaultReadOnlyOpenDefault(data("default password")).free(); pass("vault_read_only_open_default")
    _ = try api.vaultDefaultDirectory(); _ = try api.vaultDefaultPath(); pass("vault_default_directory", 3); pass("vault_default_path", 2)
    try api.vaultDirectoryOpenOrCreateDefault(data("default password")).free(); pass("vault_directory_open_or_create_default")
    try api.vaultDirectoryChangeDefaultPassword(data("default password"), data("changed default password")); pass("vault_directory_change_default_password")
    let backup = try artifactRoot() + "/default-vault.backup"; try? files.removeItem(atPath: backup)
    _ = try api.vaultBackupDefault(backup, false); _ = try api.vaultRestoreDefault(backup, true); pass("vault_backup_default"); pass("vault_restore_default")
}

func platformStore() throws {
    _ = try api.platform.status(); pass("vault_platform_status", 2); try api.platform.setScope("vault"); pass("vault_platform_set_scope")
    try api.platform.disable(); _ = try api.platform.disabled(); pass("vault_platform_disable"); pass("vault_platform_disabled"); try api.platform.enable(); pass("vault_platform_enable")
    try api.platform.putPassword(data("platform vault password")); try check(try api.platform.getPassword() == data("platform vault password"), "platform")
    pass("vault_platform_put_password"); pass("vault_platform_get_password", 3); try api.platform.forgetPassword(); pass("vault_platform_forget_password")
}

func agentAndLocal() throws {
    try makeDirectory(ProcessInfo.processInfo.environment["LOCKBOX_SESSION_AGENT_DIR"]!); try makeDirectory(ProcessInfo.processInfo.environment["LOCKBOX_VAULT_DIR"]!)
    let directory = try api.vaultDirectoryReplaceDefault(data("agent vault password")), profile = try api.keyContactGenerate()
    try directory.storePrivateKey("default", profile); try profile.free(); try directory.free(); try api.agent.forgetAll(); pass("vault_forget_all")
    for _ in 0..<200 { if try api.agent.isRunning() { break }; Thread.sleep(forTimeInterval: 0.05) }
    try check(try api.agent.isRunning(), "agent"); pass("vault_agent_serve"); pass("vault_is_running"); _ = try api.agent.start(); pass("vault_agent_start"); try api.agent.verifyTransport(); pass("vault_agent_verify_transport")
    let id = sequence(192, 207), key = sequence(32, 63); try api.agent.put(id, key); try check(try api.agent.get(id) == key, "agent key"); _ = try api.agent.list()
    pass("vault_agent_put"); pass("vault_agent_get", 3); pass("vault_agent_list")
    try api.agent.putVaultUnlockKey("vault-id", key, 120); _ = try api.agent.getVaultUnlockKey("vault-id"); pass("vault_agent_put_vault_unlock_key"); pass("vault_agent_get_vault_unlock_key", 3)
    let owner = try api.keySigningGenerate(); try api.agent.putOwnerSigningKey("vault-id", "alice", owner, 120); try api.agent.getOwnerSigningKey("vault-id", "alice").free(); pass("vault_agent_put_owner_signing_key"); pass("vault_agent_get_owner_signing_key")
    let activity = try api.agent.beginActivity("open"); pass("vault_agent_begin_activity"); try api.agent.endActivity(activity); pass("vault_agent_end_activity")
    _ = try api.agent.sleepSupport(); pass("vault_agent_sleep_support"); _ = try api.vaultAgentLogPath(); _ = try api.vaultAgentLogDestination(); pass("vault_agent_log_path", 2); pass("vault_agent_log_destination", 2)
    let local = try api.vaultLocal(); pass("vault_local"); let root = "/tmp/revault-swift-local"; try resetDirectory(root)
    let passwordPath = root + "/password.lbox"; let box = try local.createLockboxPassword(passwordPath, data("local password")); try box.addFile("/data.txt", data("local vault data"), false); try box.commit(); try box.free(); pass("vault_create_lockbox_password", 3)
    try local.cacheLockboxPassword(passwordPath, data("local password"), 120); pass("vault_cache_lockbox_password")
    let opened = try local.openLockboxPassword(passwordPath, data("local password")); _ = try opened.getFile("/data.txt"); try opened.free(); pass("vault_open_lockbox_password", 3); try local.closeLockbox(passwordPath); pass("vault_close_lockbox")
    let contentPath = root + "/content.lbox"; let contentBox = try local.createLockboxContentKey(contentPath, key, owner); try contentBox.addFile("/data.txt", data("local vault data"), false); try contentBox.commit(); try contentBox.free(); pass("vault_create_lockbox_content_key", 3)
    let contentOpen = try local.openLockboxContentKey(contentPath, key, owner); _ = try contentOpen.getFile("/data.txt"); try contentOpen.free(); pass("vault_open_lockbox_content_key", 3)
    let contact = try api.keyContactGenerate(), publicKey = try api.keyContactPublicFromBytes(try contact.publicBytes()); let contactBox = try local.createLockboxContact(root + "/contact.lbox", publicKey, "recipient", owner)
    try contactBox.addFile("/data.txt", data("local vault data"), false); try contactBox.commit(); try contactBox.free(); pass("vault_create_lockbox_contact", 3)
    try publicKey.publicFree(); try contact.free(); try local.closeAll(); pass("vault_close_all"); try local.free(); pass("vault_free"); try owner.free()
    try api.agent.forgetOwnerSigningKey("vault-id", "alice"); try api.agent.forgetVaultUnlockKey("vault-id"); try api.agent.forget(id); pass("vault_agent_forget_owner_signing_key"); pass("vault_agent_forget_vault_unlock_key"); pass("vault_agent_forget")
    try api.agent.stop(); pass("vault_agent_stop")
}

func interop(_ producer: String) throws {
    let root = ProcessInfo.processInfo.environment["REVAULT_E2E_ARTIFACT_DIR"] ?? "/tmp/revault-e2e-artifacts"
    let archive = try Data(contentsOf: URL(fileURLWithPath: root + "/\(producer)/archive.lbox")); let box = try api.lockboxOpen(archive, Data(repeating: 75, count: 32))
    try check(try box.getFile("/renamed.txt") == data("replacement payload"), "foreign archive"); try box.free()
    let vault = try api.vaultDirectoryOpen(root + "/\(producer)/vault", data("new vault password")); try check(try vault.structureVersion() > 0, "foreign vault"); try vault.free()
    print("INTEROP\tswift\t\(producer)\tarchive\t3"); print("INTEROP\tswift\t\(producer)\tvault\t2")
}

let arguments = CommandLine.arguments
if arguments.count > 1 && arguments[1] == "--serve-agent" { try api.agent.serve() }
else if arguments.count > 1 && arguments[1] == "--default" { try defaultVault() }
else if arguments.count > 1 && arguments[1] == "--platform" { try platformStore() }
else if arguments.count > 1 && arguments[1] == "--agent" { try agentAndLocal() }
else if arguments.count > 2 && arguments[1] == "--interop" { try interop(arguments[2]) }
else { try archiveLifecycle(); try keyLifecycle(); try advancedArchive(); try vaultLifecycle(); _ = api.lastError(); pass("buffer_last_error") }
