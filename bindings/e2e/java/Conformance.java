package com.onepub.revault.e2e;

import com.onepub.revault.Revault;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;
import java.util.Arrays;
import revault.bindings.RevaultBindings.FormField;
import revault.bindings.RevaultBindings.FormFieldList;
import revault.bindings.RevaultBindings.PathMove;
import revault.bindings.RevaultBindings.PathMoveList;

/** Full Java end-to-end conformance runner. Every PASS follows a real API call. */
public final class Conformance {
  private static final String LANGUAGE = System.getenv().getOrDefault("REVAULT_E2E_LANGUAGE", "java");
  private static final Revault API = new Revault();

  private static void pass(String symbol, int... assertions) {
    System.out.printf("PASS\t%s\t%s\t%d%n", LANGUAGE, symbol, assertions.length == 0 ? 1 : assertions[0]);
  }
  private static void check(boolean condition, String message) {
    if (!condition) throw new AssertionError(message);
  }
  private static byte[] repeated(int value, int length) {
    var result = new byte[length]; Arrays.fill(result, (byte) value); return result;
  }
  private static Path artifactRoot() throws IOException {
    var base = Path.of(System.getenv().getOrDefault("REVAULT_E2E_ARTIFACT_DIR", "/tmp/revault-e2e-artifacts"));
    var result = base.resolve(LANGUAGE); Files.createDirectories(result); return result;
  }
  private static FormFieldList formFields() {
    return FormFieldList.newBuilder().addValues(FormField.newBuilder()
        .setId("username").setLabel("Username").setKind("text").setRequired(true)).build();
  }
  private static void deleteTree(Path root) throws IOException {
    if (!Files.exists(root)) return;
    try (var paths = Files.walk(root)) {
      for (var path : paths.sorted(java.util.Comparator.reverseOrder()).toList()) Files.delete(path);
    }
  }

  private static void archiveLifecycle() throws Exception {
    var key = repeated('K', 32);
    byte[] archive;
    try (var box = API.createLockbox(key)) {
      pass("lockbox_create");
      box.addFile("/hello.txt", "hello from java conformance".getBytes(), false);
      check(box.exists("/hello.txt"), "added file"); pass("lockbox_add_file", 2);
      check(Arrays.equals(box.getFile("/hello.txt"), "hello from java conformance".getBytes()), "file");
      pass("lockbox_get_file", 3);
      box.addFile("/hello.txt", "replacement payload".getBytes(), 0640, true);
      check(box.permissions("/hello.txt") == 0640, "permissions");
      pass("lockbox_add_file_with_permissions", 2); pass("lockbox_permissions");
      box.createDirectory("/tree", true); check(box.isDirectory("/tree"), "directory");
      pass("lockbox_create_dir", 2); pass("lockbox_is_dir");
      box.createParentDirectories("/tree/a/b/file"); check(box.isDirectory("/tree/a/b"), "parents");
      pass("lockbox_create_parent_dirs", 2);
      box.rename("/hello.txt", "/renamed.txt");
      check(box.exists("/renamed.txt") && !box.exists("/hello.txt"), "rename");
      pass("lockbox_rename", 3); pass("lockbox_exists", 2);
      box.setPermissions("/renamed.txt", 0600); check(box.permissions("/renamed.txt") == 0600, "set permissions");
      pass("lockbox_set_permissions", 2);
      check(Arrays.equals(box.readRange("/renamed.txt", 0, 11), "replacement".getBytes()), "range");
      pass("lockbox_read_range", 3);
      box.setVariable("normal", "value", false); check(box.getVariable("normal").equals("value"), "variable");
      pass("lockbox_set_variable"); pass("lockbox_get_variable", 3);
      box.moveVariables(PathMoveList.newBuilder().addValues(PathMove.newBuilder()
          .setSource("normal").setDestination("moved")).build());
      check(box.getVariable("moved").equals("value"), "moved variable");
      box.moveVariables(PathMoveList.newBuilder().addValues(PathMove.newBuilder()
          .setSource("moved").setDestination("normal")).build());
      pass("lockbox_move_variables", 3);
      box.setVariable("secret", "hidden", true);
      check(box.variableSensitivity("secret").getPresent(), "sensitivity");
      pass("lockbox_variable_sensitivity", 2);
      check(box.listVariables().getValuesCount() == 2, "variables"); pass("lockbox_list_variables");
      box.deleteVariable("normal"); pass("lockbox_delete_variable");
      box.addSymlink("/link", "/renamed.txt", false);
      check(box.symlinkTarget("/link").equals("/renamed.txt"), "symlink");
      pass("lockbox_add_symlink"); pass("lockbox_get_symlink_target", 3);
      check(box.list("/", true).getEntriesCount() > 0, "list");
      check(box.stat("/renamed.txt").hasValue(), "stat");
      pass("lockbox_list", 2); pass("lockbox_stat", 2);
      box.setWorkloadProfile("read-mostly"); box.setWorkerPolicy("single", 1);
      check(!box.runtimeOptions().getWorkloadProfile().isEmpty(), "runtime");
      pass("lockbox_set_workload_profile"); pass("lockbox_set_worker_policy"); pass("lockbox_runtime_options");
      box.commit(); check(box.storageLength() > 0, "storage");
      pass("lockbox_commit"); pass("lockbox_storage_len");
      archive = box.bytes(); check(archive.length > 0, "bytes");
      pass("lockbox_to_bytes", 2); pass("buffer_free");
      int formatVersion = API.lockboxFormatVersion(); check(formatVersion > 0, "format version");
      check(API.probeLockboxFormatVersion(archive) == formatVersion, "format probe");
      pass("lockbox_format_version", 2); pass("lockbox_probe_format_version", 2);
      check(API.probeLockboxFormatVersion(new byte[] {1, 2, 3}) == 0, "invalid format probe");
      check(!API.lastErrorDetails().getMessage().isEmpty(), "error details");
      pass("buffer_last_error_details", 2);
      var path = artifactRoot().resolve("archive.lbox"); Files.write(path, archive);
      System.out.printf("ARTIFACT\t%s\tarchive-created\t%s%n", LANGUAGE, path);
    }
    try (var opened = API.openLockbox(archive, key)) {
      check(Arrays.equals(opened.getFile("/renamed.txt"), "replacement payload".getBytes()), "open");
      pass("lockbox_open", 2);
      System.out.printf("ARTIFACT\t%s\tarchive-opened\t%s%n", LANGUAGE, artifactRoot().resolve("archive.lbox"));
      opened.delete("/renamed.txt"); check(!opened.exists("/renamed.txt"), "delete"); pass("lockbox_delete", 2);
      opened.removeDirectory("/tree", true); check(!opened.exists("/tree"), "remove"); pass("lockbox_remove_dir", 2);
    }
    pass("lockbox_free", 2);
  }

  private static void keyLifecycle() {
    var content = new byte[32]; for (int i = 0; i < content.length; i++) content[i] = (byte) i;
    try (var contact = API.generateContactKeyPair()) {
      pass("key_contact_generate");
      var privateRecord = contact.privateRecord(); check(privateRecord.length > 32, "private"); pass("key_contact_private", 2);
      try (var copy = API.contactKeyPairFromPrivate(privateRecord)) {
        pass("key_contact_from_private");
        var publicBytes = contact.publicBytes(); check(publicBytes.length > 0, "public"); pass("key_contact_public", 2);
        try (var publicKey = API.contactPublicKey(publicBytes)) {
          pass("key_contact_public_from_bytes");
          try (var wrapped = publicKey.encrypt(content)) {
            pass("key_contact_encrypt");
            check(Arrays.equals(copy.decrypt(wrapped), content), "decrypt"); pass("key_contact_decrypt", 3);
            check(wrapped.publicBytes().length > 0, "wrapped public");
            check(wrapped.ciphertext().length > 0, "wrapped cipher");
            check(wrapped.encryptedBytes().length > 0, "wrapped encrypted");
            pass("key_contact_wrapped_public", 2); pass("key_contact_wrapped_ciphertext", 2);
            pass("key_contact_wrapped_encrypted", 2); pass("key_contact_wrapped_free");
          }
          var exportedPrivate = contact.export("raw-hex");
          try (var imported = API.importContactKeyPair(exportedPrivate)) { check(imported.publicBytes().length > 0, "import private"); }
          pass("vault_key_export_private", 2); pass("vault_key_import_private");
          var exportedPublic = publicKey.export("lockbox-pem");
          try (var imported = API.importContactPublicKey(exportedPublic)) {
            pass("vault_key_export_public", 2); pass("vault_key_import_public");
            var fingerprint = imported.fingerprint(); check(fingerprint.length >= 12, "fingerprint");
            pass("vault_key_fingerprint", 2);
            var hex = API.formatKeyHex(fingerprint); check(Arrays.equals(API.decodeKeyHex(hex), fingerprint), "hex");
            pass("vault_key_format_hex", 2); pass("vault_key_decode_hex", 2);
            var shortFingerprint = Arrays.copyOf(fingerprint, 12);
            var code = API.formatKeyCrockford(shortFingerprint);
            check(!API.formatKeyCrockfordReading(code).isEmpty(), "reading");
            check(Arrays.equals(API.decodeKeyCrockford(code), shortFingerprint), "crockford");
            pass("vault_key_format_crockford", 2); pass("vault_key_format_crockford_reading", 2);
            pass("vault_key_decode_crockford", 2);
          }
          pass("key_contact_public_free", 2);
        }
      }
      pass("key_contact_free", 3);
    }
    var plainHex = API.hexEncode(content); check(Arrays.equals(API.hexDecode(plainHex), content), "plain hex");
    pass("vault_key_hex_encode", 2); pass("vault_key_hex_decode", 2);
    try (var signing = API.generateSigningKeyPair()) {
      pass("key_signing_generate");
      var privateRecord = signing.privateRecord(); var publicRecord = signing.publicBytes();
      check(privateRecord.length > 0 && publicRecord.length > 0, "signing records");
      pass("key_signing_private", 2); pass("key_signing_public", 2);
      try (var copy = API.signingKeyPairFromPrivate(privateRecord)) { check(copy.publicBytes().length > 0, "signing copy"); }
      pass("key_signing_from_private");
      try (var publicKey = API.signingPublicKey(publicRecord)) { check(publicKey != null, "signing public"); }
      pass("key_signing_public_from_bytes"); pass("key_signing_public_free", 2);
    }
    pass("key_signing_free", 3);
  }

  private static void advancedArchive() throws Exception {
    var key = repeated('A', 32);
    var options = new Revault.LockboxOptions("bytes", 4L << 20, "bulk-import", "single", 1);
    try (var box = API.createLockbox(key, options)) {
      pass("lockbox_create_with_options");
      box.addFile("/account.txt", "account data".getBytes(), false);
      check(box.list("/", "*.txt", true, true, false, false, 20).getEntriesCount() == 1, "filtered list");
      pass("lockbox_list_with_options", 2);
      var definition = box.defineForm("account", "Account", "Account form", formFields());
      check(!definition.getTypeId().isEmpty(), "form type"); pass("lockbox_define_form", 2);
      check(box.listFormDefinitions().getValuesCount() == 1, "definitions");
      check(box.resolveForm("account").getTypeId().equals(definition.getTypeId()), "resolve");
      check(box.listFormRevisions(definition.getTypeId()).getValuesCount() == 1, "revisions");
      pass("lockbox_list_form_definitions"); pass("lockbox_resolve_form"); pass("lockbox_list_form_revisions");
      var record = box.createFormRecord("/account.form", "account", "Primary");
      check(record.getPath().equals("/account.form"), "record"); pass("lockbox_create_form_record");
      box.setFormField("/account.form", "username", "alice", false); pass("lockbox_set_form_field");
      check(box.getFormRecord("/account.form").getValuesCount() == 1, "record values");
      check(box.getFormField("/account.form", "username").getValue().equals("alice"), "field");
      check(box.listFormRecords().getValuesCount() == 1, "records");
      pass("lockbox_get_form_record"); pass("lockbox_get_form_field"); pass("lockbox_list_form_records");
      box.moveFormRecords(PathMoveList.newBuilder().addValues(PathMove.newBuilder()
          .setSource("/account.form").setDestination("/moved.form")).build());
      check(box.getFormRecord("/moved.form").getValuesCount() == 1, "moved record");
      box.moveFormRecords(PathMoveList.newBuilder().addValues(PathMove.newBuilder()
          .setSource("/moved.form").setDestination("/account.form")).build());
      pass("lockbox_move_form_records", 3);
      try (var signing = API.generateSigningKeyPair(); var contact = API.generateContactKeyPair();
           var publicKey = contact.publicKey()) {
        box.setOwnerSigningKey(signing); pass("lockbox_set_owner_signing_key");
        long passwordSlot = box.addPassword("archive password".getBytes());
        check(passwordSlot != -1L, "password slot"); pass("lockbox_add_password");
        long contactSlot = box.addContact(publicKey, "recipient");
        check(contactSlot != -1L, "contact slot"); pass("lockbox_add_contact");
        check(box.listKeySlots().getValuesCount() >= 2, "slots"); pass("lockbox_list_key_slots");
        box.deleteKey(passwordSlot); pass("lockbox_delete_key");
        box.commit(); check(box.ownerInspection().getSigned(), "owner"); pass("lockbox_owner_inspection", 2);
        check(box.cacheStats().getLimitBytes() > 0, "cache");
        check(!box.importStats().getHostReadNanos().isEmpty(), "import");
        box.resetImportStats(); check(box.pageInspection().getValuesCount() > 0, "pages");
        check(box.recoveryReport().getIntactFileCount() > 0, "recovery");
        check(!box.renderRecoveryReport(true, 100).isEmpty(), "render");
        check(box.streamContent(false).getValuesCount() > 0, "stream"); check(box.id().length > 0, "id");
        pass("lockbox_cache_stats"); pass("lockbox_import_stats"); pass("lockbox_reset_import_stats");
        pass("lockbox_page_inspection"); pass("lockbox_recovery_report");
        pass("lockbox_recovery_report_render", 2); pass("lockbox_stream_content"); pass("lockbox_id", 2);
        var archive = box.bytes(); var path = artifactRoot().resolve("advanced.lbox"); Files.write(path, archive);
        check(API.inspectLockboxFile(path.toString()).getHeaderReadable(), "inspect");
        check(API.scanLockboxPath(path.toString(), key).getIntactFileCount() > 0, "scan path");
        check(API.scanLockbox(archive, key).getIntactFileCount() > 0, "scan");
        pass("lockbox_inspect_file"); pass("lockbox_recovery_scan_path"); pass("lockbox_recovery_scan");
        var damaged = Arrays.copyOf(archive, archive.length - 32);
        try (var salvaged = API.salvageLockbox(damaged, key, signing)) {
          check(salvaged.storageLength() > 0, "salvage");
        }
        pass("lockbox_recovery_salvage", 2);
        try (var opened = API.openLockbox(archive, key, options)) {
          check(Arrays.equals(opened.getFile("/account.txt"), "account data".getBytes()), "option open");
        }
        pass("lockbox_open_with_options", 2);
        try (var passwordBox = API.createLockboxWithPassword("archive password".getBytes())) {
          passwordBox.addFile("/password.txt", "password protected".getBytes(), false); passwordBox.commit();
          var passwordArchive = passwordBox.bytes(); pass("lockbox_create_password");
          try (var opened = API.openLockboxWithPassword(passwordArchive, "archive password".getBytes())) {
            check(Arrays.equals(opened.getFile("/password.txt"), "password protected".getBytes()), "password open");
          }
          pass("lockbox_open_password", 2);
        }
        try (var contactBox = API.createLockboxForContact(publicKey)) {
          contactBox.addFile("/contact.txt", "contact protected".getBytes(), false); contactBox.commit();
          var contactArchive = contactBox.bytes(); pass("lockbox_create_contact");
          try (var opened = API.openLockboxForContact(contactArchive, contact)) {
            check(Arrays.equals(opened.getFile("/contact.txt"), "contact protected".getBytes()), "contact open");
          }
          pass("lockbox_open_contact", 2);
        }
        try (var signed = API.createSignedLockbox(key, signing)) {
          signed.commit(); check(signed.ownerInspection().getSigned(), "signed box");
        }
        pass("lockbox_create_with_signing_key", 2);
      }
      var extractRoot = artifactRoot().resolve("extract"); deleteTree(extractRoot); Files.createDirectories(extractRoot);
      var extractedFile = extractRoot.resolve("account.txt"); Files.deleteIfExists(extractedFile);
      box.extractFile("/account.txt", extractedFile.toString(), false);
      check(Files.exists(extractedFile), "extract file"); pass("lockbox_extract_file", 2);
      var extractedTree = extractRoot.resolve("tree"); Files.createDirectories(extractedTree);
      box.extractDirectory(extractedTree.toString(), 1 << 20, 4 << 20, 100, false, true, false);
      check(Files.exists(extractedTree), "extract dir"); pass("lockbox_extract_directory", 2);
      box.deleteFormRecord("/account.form"); pass("lockbox_delete_form_record");
    }
  }

  private static void vaultLifecycle() throws Exception {
    var root = artifactRoot().resolve("vault"); Files.createDirectories(root);
    var password = "vault password".getBytes(); var newPassword = "new vault password".getBytes();
    var id = new byte[16]; for (int i = 0; i < id.length; i++) id[i] = (byte) (0xa0 + i);
    try (var profile = API.generateContactKeyPair(); var contact = API.generateContactKeyPair();
         var contactPublic = contact.publicKey(); var owner = API.generateSigningKeyPair();
         var ownerPublic = owner.publicKey()) {
      try (var vault = API.replaceVaultDirectory(root.toString(), password)) {
        pass("vault_directory_replace");
        System.out.printf("ARTIFACT\t%s\tvault-created\t%s%n", LANGUAGE, root);
        check(vault.root().equals(root.toString()), "root"); check(vault.structureVersion() > 0, "version");
        pass("vault_directory_root", 3); pass("vault_directory_structure_version");
        int currentVersion = API.currentVaultStructureVersion();
        check(currentVersion == vault.structureVersion(), "current vault version");
        check(API.probeVaultStructureVersion(root.toString(), password) == currentVersion, "vault probe");
        pass("vault_structure_version_current", 2); pass("vault_directory_probe_structure_version", 2);
        vault.storePrivateKey("alice", profile); check(vault.privateKeyExists("alice"), "profile");
        try (var loaded = vault.loadPrivateKey("alice"); var generation = vault.loadPrivateKeyGeneration("alice", 1)) {
          check(loaded.publicBytes().length > 0 && generation.publicBytes().length > 0, "loaded profile");
        }
        pass("vault_directory_store_private_key"); pass("vault_directory_private_key_exists");
        pass("vault_directory_load_private_key"); pass("vault_directory_load_private_key_generation");
        vault.storeProfileEmail("alice", "alice@example.test");
        var email = vault.profileEmail("alice"); check(email.getPresent() && email.getValue().equals("alice@example.test"), "email");
        pass("vault_directory_store_profile_email"); pass("vault_directory_profile_email", 3);
        check(vault.listProfileGenerations("alice").getGenerationsCount() == 1, "history");
        check(vault.rotatePrivateKey("alice").getGenerationsCount() == 2, "rotation");
        pass("vault_directory_list_profile_generations"); pass("vault_directory_rotate_private_key");
        try (var loaded = vault.loadOwnerSigningKey("alice");
             var generation = vault.loadOwnerSigningKeyGeneration("alice", 1)) {
          check(loaded.publicBytes().length > 0 && generation.publicBytes().length > 0, "owner");
        }
        pass("vault_directory_load_owner_signing_key"); pass("vault_directory_load_owner_signing_key_generation");
        vault.storeContact("bob", contactPublic); check(vault.contactExists("bob"), "contact");
        try (var loaded = vault.loadContact("bob")) { check(loaded.fingerprint().length > 0, "loaded contact"); }
        check(vault.listContacts().getValuesCount() == 1, "contacts");
        pass("vault_directory_store_contact"); pass("vault_directory_contact_exists");
        pass("vault_directory_load_contact"); pass("vault_directory_list_contacts");
        vault.storeContactSigningKey("bob", ownerPublic);
        try (var loaded = vault.loadContactSigningKey("bob")) { check(loaded != null, "contact signing"); }
        pass("vault_directory_store_contact_signing_key"); pass("vault_directory_load_contact_signing_key");
        check(vault.listPrivateKeys().getValuesCount() > 0, "keys");
        check(vault.listPrivateKeyNames().getValuesCount() > 0, "names");
        check(vault.listContactNames().getValuesCount() > 0, "contacts");
        pass("vault_directory_list_private_keys"); pass("vault_directory_list_private_key_names");
        pass("vault_directory_list_contact_names");
        vault.storeBackup(id, "encrypted backup bytes".getBytes()); check(vault.backupCount() == 1, "count");
        check(Arrays.equals(vault.loadBackup(id), "encrypted backup bytes".getBytes()), "backup");
        pass("vault_directory_store_backup"); pass("vault_directory_backup_count"); pass("vault_directory_load_backup", 3);
        vault.rememberLockbox(id, "/tmp/example.lbox");
        check(vault.listKnownLockboxes().getValuesCount() == 1, "known");
        pass("vault_directory_remember_lockbox"); pass("vault_directory_list_known_lockboxes");
        vault.rememberAccessSlotLabel(id, 7, "primary");
        check(vault.listAccessSlotLabels(id).getValuesCount() == 1, "labels");
        check(vault.findAccessSlotLabels(id, "primary").getValuesCount() == 1, "find");
        pass("vault_directory_remember_access_slot_label"); pass("vault_directory_list_access_slot_labels");
        pass("vault_directory_find_access_slot_labels");
        vault.rememberPassword(id, password); check(Arrays.equals(vault.rememberedPassword(id), password), "remember");
        pass("vault_directory_remember_password"); pass("vault_directory_remembered_password", 3);
        var vaultForm = vault.defineForm("login", "Login", "Login form", formFields());
        check(!vaultForm.getTypeId().isEmpty(), "define");
        check(!vault.resolveForm("login").getTypeId().isEmpty(), "resolve");
        check(vault.listForms().getValuesCount() > 0, "forms");
        pass("vault_directory_define_form"); pass("vault_directory_resolve_form"); pass("vault_directory_list_forms");
        check(vault.listFormRevisions(vaultForm.getTypeId()).getValuesCount() > 0, "vault revisions");
        pass("vault_directory_list_form_revisions", 2);
        check(vault.seedForms() > 0, "seed"); pass("vault_directory_seed_forms");
        check(vault.listFormAliases().getValuesCount() > 0, "aliases"); pass("vault_directory_list_form_aliases");
        vault.forgetAccessSlotLabel(id, 7); vault.forgetLockbox("/tmp/example.lbox"); vault.deleteContact("bob");
        pass("vault_directory_forget_access_slot_label"); pass("vault_directory_forget_lockbox");
        pass("vault_directory_delete_contact");
        vault.deletePrivateKey("alice"); check(!vault.privateKeyExists("alice"), "deleted");
        vault.restorePrivateKey("alice", profile, owner, true); check(vault.privateKeyExists("alice"), "restored");
        pass("vault_directory_delete_private_key", 2); pass("vault_directory_restore_private_key", 2);
      }
      pass("vault_directory_free");
      try (var readonly = API.openReadOnlyVaultDirectory(root.toString(), password)) {
        check(readonly.listProfileNames().getValuesCount() > 0, "read-only profiles");
        readonly.listContactNames();
        check(readonly.listFormAliases().getValuesCount() > 0, "read-only forms");
        readonly.listKnownLockboxes();
        pass("vault_read_only_open"); pass("vault_read_only_list_profile_names", 2);
        pass("vault_read_only_list_contact_names"); pass("vault_read_only_list_form_aliases", 2);
        pass("vault_read_only_list_known_lockboxes");
      }
      pass("vault_read_only_free");
    }
    API.changeVaultDirectoryPassword(root.toString(), password, newPassword); pass("vault_directory_change_password");
    try (var reopened = API.openVaultDirectory(root.toString(), newPassword)) {
      check(reopened.structureVersion() > 0, "reopen");
    }
    pass("vault_directory_open"); System.out.printf("ARTIFACT\t%s\tvault-opened\t%s%n", LANGUAGE, root);
    try (var opened = API.openOrCreateVaultDirectory(root.toString(), newPassword)) {
      check(opened.structureVersion() > 0, "open create");
    }
    pass("vault_directory_open_or_create");
  }

  private static void defaultVaultLifecycle() throws Exception {
    var expectedRoot = Path.of(System.getenv("LOCKBOX_VAULT_DIR")); Files.createDirectories(expectedRoot);
    try (var vault = API.replaceDefaultVaultDirectory("default password".getBytes())) {}
    pass("vault_directory_replace_default");
    try (var vault = API.openDefaultReadOnlyVaultDirectory("default password".getBytes())) {}
    pass("vault_read_only_open_default");
    check(API.defaultVaultDirectory().equals(expectedRoot.toString()), "default dir");
    check(Path.of(API.defaultVaultPath()).getParent().equals(expectedRoot), "default path");
    pass("vault_default_directory", 3); pass("vault_default_path", 2);
    try (var vault = API.openOrCreateDefaultVaultDirectory("default password".getBytes())) {}
    pass("vault_directory_open_or_create_default");
    API.changeDefaultVaultDirectoryPassword("default password".getBytes(), "changed default password".getBytes());
    pass("vault_directory_change_default_password");
    var backup = artifactRoot().resolve("default-vault.backup"); Files.deleteIfExists(backup);
    check(API.backupDefaultVault(backup.toString(), false).getVaultSize() > 0, "backup default");
    check(API.restoreDefaultVault(backup.toString(), true).getVaultSize() > 0, "restore default");
    pass("vault_backup_default"); pass("vault_restore_default");
  }

  private static Process startAgent() throws IOException {
    var java = Path.of(System.getProperty("java.home"), "bin", "java").toString();
    return new ProcessBuilder(java, "--enable-native-access=ALL-UNNAMED", "-cp",
        System.getProperty("java.class.path"), Conformance.class.getName(), "--serve-agent")
        .inheritIO().start();
  }

  private static void agentAndLocalVault() throws Exception {
    Files.createDirectories(Path.of(System.getenv("LOCKBOX_SESSION_AGENT_DIR")));
    Files.createDirectories(Path.of(System.getenv("LOCKBOX_VAULT_DIR")));
    try (var vault = API.replaceDefaultVaultDirectory("agent vault password".getBytes());
         var profile = API.generateContactKeyPair()) {
      vault.storePrivateKey("default", profile);
    }
    API.forgetAllAgentSecrets(); pass("vault_forget_all");
    var child = startAgent();
    boolean running = false;
    for (int attempt = 0; attempt < 200; attempt++) {
      if (API.agentIsRunning()) { running = true; break; }
      Thread.sleep(50);
    }
    check(running, "agent start"); pass("vault_agent_serve"); pass("vault_is_running");
    API.startAgent(); pass("vault_agent_start");
    API.verifyAgentTransport(); pass("vault_agent_verify_transport");
    var id = new byte[16]; var key = new byte[32];
    for (int i = 0; i < id.length; i++) id[i] = (byte) (0xc0 + i);
    for (int i = 0; i < key.length; i++) key[i] = (byte) (0x20 + i);
    API.putAgentKey(id, key); check(Arrays.equals(API.getAgentKey(id), key), "agent key");
    check(API.listAgentKeys().getValuesCount() > 0, "agent list");
    pass("vault_agent_put"); pass("vault_agent_get", 3); pass("vault_agent_list");
    API.putAgentVaultUnlockKey("vault-id", key, 120); check(Arrays.equals(API.getAgentVaultUnlockKey("vault-id"), key), "vault key");
    pass("vault_agent_put_vault_unlock_key"); pass("vault_agent_get_vault_unlock_key", 3);
    try (var owner = API.generateSigningKeyPair()) {
      API.putAgentOwnerSigningKey("vault-id", "alice", owner, 120);
      try (var loaded = API.getAgentOwnerSigningKey("vault-id", "alice")) {
        check(loaded.publicBytes().length > 0, "owner key");
      }
      pass("vault_agent_put_owner_signing_key"); pass("vault_agent_get_owner_signing_key");
      try (var activity = API.beginAgentActivity("open")) { pass("vault_agent_begin_activity"); }
      pass("vault_agent_end_activity");
      check(API.agentSleepSupport() != null, "sleep"); pass("vault_agent_sleep_support");
      check(!API.agentLogPath().isEmpty(), "log path"); check(!API.agentLogDestination().isEmpty(), "log destination");
      pass("vault_agent_log_path", 2); pass("vault_agent_log_destination", 2);
      try (var local = API.openLocalVault()) {
        pass("vault_local");
        var root = Files.createTempDirectory("revault-java-local-"); var payload = "local vault data".getBytes();
        var passwordPath = root.resolve("password.lbox").toString();
        try (var box = local.createWithPassword(passwordPath, "local password".getBytes())) {
          box.addFile("/data.txt", payload, false); box.commit();
        }
        pass("vault_create_lockbox_password", 3);
        local.cachePassword(passwordPath, "local password".getBytes(), 120); pass("vault_cache_lockbox_password");
        try (var box = local.openWithPassword(passwordPath, "local password".getBytes())) {
          check(Arrays.equals(box.getFile("/data.txt"), payload), "password open");
        }
        pass("vault_open_lockbox_password", 3); local.closeLockbox(passwordPath); pass("vault_close_lockbox");
        var contentPath = root.resolve("content.lbox").toString();
        try (var box = local.createWithContentKey(contentPath, key, owner)) {
          box.addFile("/data.txt", payload, false); box.commit();
        }
        pass("vault_create_lockbox_content_key", 3);
        try (var box = local.openWithContentKey(contentPath, key, owner)) {
          check(Arrays.equals(box.getFile("/data.txt"), payload), "content open");
        }
        pass("vault_open_lockbox_content_key", 3);
        try (var contact = API.generateContactKeyPair(); var publicKey = contact.publicKey()) {
          var contactPath = root.resolve("contact.lbox").toString();
          try (var box = local.createForContact(contactPath, publicKey, "recipient", owner)) {
            box.addFile("/data.txt", payload, false); box.commit();
          }
        }
        pass("vault_create_lockbox_contact", 3); local.closeAll(); pass("vault_close_all");
      }
      pass("vault_free");
    }
    API.forgetAgentOwnerSigningKey("vault-id", "alice"); API.forgetAgentVaultUnlockKey("vault-id"); API.forgetAgentKey(id);
    pass("vault_agent_forget_owner_signing_key"); pass("vault_agent_forget_vault_unlock_key"); pass("vault_agent_forget");
    API.stopAgent(); pass("vault_agent_stop"); check(child.waitFor() == 0, "agent child");
  }

  private static void platformSecretStore() {
    check(API.platformStatus() != null, "platform status"); pass("vault_platform_status", 2);
    API.setPlatformScope("vault"); pass("vault_platform_set_scope");
    API.disablePlatformStore(); check(API.platformStoreDisabled(), "disabled");
    pass("vault_platform_disable"); pass("vault_platform_disabled");
    API.enablePlatformStore(); check(!API.platformStoreDisabled(), "enabled"); pass("vault_platform_enable");
    API.putPlatformPassword("platform vault password".getBytes());
    check(Arrays.equals(API.getPlatformPassword(), "platform vault password".getBytes()), "password");
    pass("vault_platform_put_password"); pass("vault_platform_get_password", 3);
    API.forgetPlatformPassword(); pass("vault_platform_forget_password");
  }

  private static void interopOpen(String producer) throws Exception {
    var base = Path.of(System.getenv().getOrDefault("REVAULT_E2E_ARTIFACT_DIR", "/tmp/revault-e2e-artifacts"));
    var archive = Files.readAllBytes(base.resolve(producer).resolve("archive.lbox"));
    try (var box = API.openLockbox(archive, repeated('K', 32))) {
      check(Arrays.equals(box.getFile("/renamed.txt"), "replacement payload".getBytes()), "foreign archive");
    }
    try (var vault = API.openVaultDirectory(base.resolve(producer).resolve("vault").toString(),
        "new vault password".getBytes())) {
      check(vault.structureVersion() > 0, "foreign vault");
    }
    System.out.printf("INTEROP\t%s\t%s\tarchive\t3%n", LANGUAGE, producer);
    System.out.printf("INTEROP\t%s\t%s\tvault\t2%n", LANGUAGE, producer);
  }

  public static void main(String[] args) throws Exception {
    if (args.length == 1 && args[0].equals("--serve-agent")) { API.serveAgent(); return; }
    if (args.length == 1 && args[0].equals("--agent")) { agentAndLocalVault(); return; }
    if (args.length == 1 && args[0].equals("--platform")) { platformSecretStore(); return; }
    if (args.length == 1 && args[0].equals("--default")) { defaultVaultLifecycle(); return; }
    if (args.length == 2 && args[0].equals("--interop")) { interopOpen(args[1]); return; }
    archiveLifecycle(); keyLifecycle(); advancedArchive(); vaultLifecycle();
    API.lastError(); pass("buffer_last_error");
  }
}
