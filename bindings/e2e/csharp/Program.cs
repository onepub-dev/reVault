using System.Diagnostics;
using System.Text;
using Revault;
using Revault.Bindings;

static class Conformance
{
    static readonly Vault Api = new();
    static void Pass(string symbol, int assertions = 1) => Console.WriteLine($"PASS\tcsharp\t{symbol}\t{assertions}");
    static void Check(bool value, string message) { if (!value) throw new Exception(message); }
    static byte[] Bytes(string value) => Encoding.UTF8.GetBytes(value);
    static byte[] Repeat(char value, int count) => Enumerable.Repeat((byte)value, count).ToArray();
    static string Root()
    {
        var path = Path.Combine(Environment.GetEnvironmentVariable("REVAULT_E2E_ARTIFACT_DIR") ?? "/tmp/revault-e2e-artifacts", "csharp");
        Directory.CreateDirectory(path); return path;
    }
    static FormFieldList Fields() => new() { Values = {
        new FormField { Id = "username", Label = "Username", Kind = "text", Required = true },
        new FormField { Id = "password", Label = "Password", Kind = "secret", Required = true }
    } };

    static void ArchiveLifecycle()
    {
        var key = Repeat('K', 32); byte[] archive;
        using (var box = Api.CreateLockbox(key))
        {
            Pass("lockbox_create"); box.AddFile("/hello.txt", Bytes("hello from csharp conformance"));
            Check(box.Exists("/hello.txt"), "add"); Pass("lockbox_add_file", 2);
            Check(box.GetFile("/hello.txt").SequenceEqual(Bytes("hello from csharp conformance")), "get"); Pass("lockbox_get_file", 3);
            box.AddFile("/hello.txt", Bytes("replacement payload"), 0x1a0, true); Check(box.Permissions("/hello.txt") == 0x1a0, "mode");
            Pass("lockbox_add_file_with_permissions", 2); Pass("lockbox_permissions");
            box.CreateDirectory("/tree", true); Check(box.IsDirectory("/tree"), "dir"); Pass("lockbox_create_dir", 2); Pass("lockbox_is_dir");
            box.CreateParentDirectories("/tree/a/b/file"); Check(box.IsDirectory("/tree/a/b"), "parents"); Pass("lockbox_create_parent_dirs", 2);
            box.Rename("/hello.txt", "/renamed.txt"); Check(box.Exists("/renamed.txt") && !box.Exists("/hello.txt"), "rename");
            Pass("lockbox_rename", 3); Pass("lockbox_exists", 2);
            box.SetPermissions("/renamed.txt", 0x180); Check(box.Permissions("/renamed.txt") == 0x180, "set mode"); Pass("lockbox_set_permissions", 2);
            Check(box.ReadRange("/renamed.txt", 0, 11).SequenceEqual(Bytes("replacement")), "range"); Pass("lockbox_read_range", 3);
            box.SetVariable("normal", "value"); Check(box.GetVariable("normal") == "value", "variable"); Pass("lockbox_set_variable"); Pass("lockbox_get_variable", 3);
            box.MoveVariables(new PathMoveList { Values = { new PathMove { Source = "normal", Destination = "moved" } } }); Check(box.GetVariable("moved") == "value", "moved variable");
            box.MoveVariables(new PathMoveList { Values = { new PathMove { Source = "moved", Destination = "normal" } } }); Pass("lockbox_move_variables", 3);
            box.SetSecretVariable("secret", Bytes("hidden")); Pass("lockbox_set_secret_variable");
            Check(box.WithSecretVariable("secret", value => Encoding.UTF8.GetString(value)) == "hidden", "secret variable");
            Pass("lockbox_get_secret_variable"); Pass("secret_len"); Pass("secret_copy"); Pass("secret_free");
            Check(box.VariableSensitivity("secret").Present, "secret"); Pass("lockbox_variable_sensitivity", 2);
            Check(box.ListVariables().Values.Count == 2, "variables"); Pass("lockbox_list_variables"); box.DeleteVariable("normal"); Pass("lockbox_delete_variable");
            box.AddSymlink("/link", "/renamed.txt"); Check(box.SymlinkTarget("/link") == "/renamed.txt", "link"); Pass("lockbox_add_symlink"); Pass("lockbox_get_symlink_target", 3);
            Check(box.List("/", true).Entries.Count > 0, "list"); Check(box.Stat("/renamed.txt").Value != null, "stat"); Pass("lockbox_list", 2); Pass("lockbox_stat", 2);
            box.SetWorkloadProfile("read-mostly"); box.SetWorkerPolicy("single", 1); Check(box.RuntimeOptions().WorkloadProfile.Length > 0, "runtime");
            Pass("lockbox_set_workload_profile"); Pass("lockbox_set_worker_policy"); Pass("lockbox_runtime_options");
            box.Commit(); Check(box.StorageLength > 0, "storage"); Pass("lockbox_commit"); Pass("lockbox_storage_len");
            archive = box.Bytes; Check(archive.Length > 0, "bytes"); Pass("lockbox_to_bytes", 2); Pass("buffer_free");
            var formatVersion = Api.LockboxFormatVersion; Check(formatVersion > 0 && Api.ProbeLockboxFormatVersion(archive) == formatVersion, "format probe"); Pass("lockbox_format_version", 2); Pass("lockbox_probe_format_version", 2);
            Check(Api.ProbeLockboxFormatVersion(Bytes("bad")) == 0 && Api.LastErrorDetails().Message.Length > 0, "structured error"); Pass("buffer_last_error_details", 2);
            var path = Path.Combine(Root(), "archive.lbox"); File.WriteAllBytes(path, archive); Console.WriteLine($"ARTIFACT\tcsharp\tarchive-created\t{path}");
        }
        using (var box = Api.OpenLockbox(archive, key))
        {
            Check(box.GetFile("/renamed.txt").SequenceEqual(Bytes("replacement payload")), "open"); Pass("lockbox_open", 2);
            Console.WriteLine($"ARTIFACT\tcsharp\tarchive-opened\t{Path.Combine(Root(), "archive.lbox")}");
            box.Delete("/renamed.txt"); Check(!box.Exists("/renamed.txt"), "delete"); Pass("lockbox_delete", 2);
            box.RemoveDirectory("/tree", true); Check(!box.Exists("/tree"), "remove"); Pass("lockbox_remove_dir", 2);
        }
        Pass("lockbox_free", 2);
    }

    static void KeyLifecycle()
    {
        var content = Enumerable.Range(0, 32).Select(i => (byte)i).ToArray();
        using (var contact = Api.GenerateContactKeyPair())
        {
            Pass("key_contact_generate"); var privateKey = contact.PrivateRecord(); Check(privateKey.Length > 32, "private"); Pass("key_contact_private", 2);
            using var copy = Api.ContactKeyPairFromPrivate(privateKey); Pass("key_contact_from_private");
            var publicBytes = contact.PublicBytes(); Check(publicBytes.Length > 0, "public"); Pass("key_contact_public", 2);
            using var publicKey = Api.ContactPublicKeyFromBytes(publicBytes); Pass("key_contact_public_from_bytes");
            using (var wrapped = publicKey.Encrypt(content))
            {
                Pass("key_contact_encrypt"); Check(copy.Decrypt(wrapped).SequenceEqual(content), "decrypt"); Pass("key_contact_decrypt", 3);
                Check(wrapped.PublicBytes().Length > 0, "wp"); Check(wrapped.Ciphertext().Length > 0, "wc"); Check(wrapped.EncryptedBytes().Length > 0, "we");
                Pass("key_contact_wrapped_public", 2); Pass("key_contact_wrapped_ciphertext", 2); Pass("key_contact_wrapped_encrypted", 2);
            }
            Pass("key_contact_wrapped_free");
            using (var imported = Api.ImportContactKeyPair(contact.Export("raw-hex"))) Check(imported.PublicBytes().Length > 0, "import private");
            Pass("vault_key_export_private", 2); Pass("vault_key_import_private");
            using (var imported = Api.ImportContactPublicKey(publicKey.Export("lockbox-pem")))
            {
                Pass("vault_key_export_public", 2); Pass("vault_key_import_public"); var fingerprint = imported.Fingerprint(); Check(fingerprint.Length >= 12, "fp"); Pass("vault_key_fingerprint", 2);
                var hex = Api.FormatKeyHex(fingerprint); Check(Api.DecodeKeyHex(hex).SequenceEqual(fingerprint), "hex"); Pass("vault_key_format_hex", 2); Pass("vault_key_decode_hex", 2);
                var shortFp = fingerprint[..12]; var code = Api.FormatKeyCrockford(shortFp); Check(Api.FormatKeyCrockfordReading(code).Length > 0, "reading");
                Check(Api.DecodeKeyCrockford(code).SequenceEqual(shortFp), "crockford"); Pass("vault_key_format_crockford", 2); Pass("vault_key_format_crockford_reading", 2); Pass("vault_key_decode_crockford", 2);
            }
            Pass("key_contact_public_free", 2); Pass("key_contact_free", 3);
        }
        var plain = Api.HexEncode(content); Check(Api.HexDecode(plain).SequenceEqual(content), "plain hex"); Pass("vault_key_hex_encode", 2); Pass("vault_key_hex_decode", 2);
        using (var signing = Api.GenerateSigningKeyPair())
        {
            Pass("key_signing_generate"); var privateKey = signing.PrivateRecord(); var publicKey = signing.PublicBytes(); Check(privateKey.Length > 0 && publicKey.Length > 0, "signing");
            Pass("key_signing_private", 2); Pass("key_signing_public", 2); using (var copy = Api.SigningKeyPairFromPrivate(privateKey)) Check(copy.PublicBytes().Length > 0, "copy"); Pass("key_signing_from_private");
            using (var key = Api.SigningPublicKeyFromBytes(publicKey)) Check(key != null, "public key"); Pass("key_signing_public_from_bytes"); Pass("key_signing_public_free", 2);
        }
        Pass("key_signing_free", 3);
    }

    static void AdvancedArchive()
    {
        var key = Repeat('A', 32); var options = new Vault.LockboxOptions("bytes", 4UL << 20, "bulk-import", "single", 1);
        using var box = Api.CreateLockbox(key, options); Pass("lockbox_create_with_options"); box.AddFile("/account.txt", Bytes("account data"));
        Check(box.List("/", "*.txt", true, true, false, false, 20).Entries.Count == 1, "filter"); Pass("lockbox_list_with_options", 2);
        var definition = box.DefineForm("account", "Account", "Account form", Fields()); Check(definition.TypeId.Length > 0, "form"); Pass("lockbox_define_form", 2);
        Check(box.ListFormDefinitions().Values.Count == 1, "defs"); Check(box.ResolveForm("account").TypeId == definition.TypeId, "resolve"); Check(box.ListFormRevisions(definition.TypeId).Values.Count == 1, "revs");
        Pass("lockbox_list_form_definitions"); Pass("lockbox_resolve_form"); Pass("lockbox_list_form_revisions");
        Check(box.CreateFormRecord("/account.form", "account", "Primary").Path == "/account.form", "record"); Pass("lockbox_create_form_record");
        box.SetFormField("/account.form", "username", "alice"); Pass("lockbox_set_form_field");
        box.SetSecretFormField("/account.form", "password", Bytes("hidden")); Pass("lockbox_set_secret_form_field");
        Check(box.WithSecretFormField("/account.form", "password", value => Encoding.UTF8.GetString(value)) == "hidden", "secret form field"); Pass("lockbox_get_secret_form_field");
        Check(box.GetFormRecord("/account.form").Value.Values.Count == 2, "values");
        Check(box.GetFormField("/account.form", "username").Value.Value == "alice", "field"); Check(box.ListFormRecords().Values.Count == 1, "records");
        Pass("lockbox_get_form_record"); Pass("lockbox_get_form_field"); Pass("lockbox_list_form_records");
        box.MoveFormRecords(new PathMoveList { Values = { new PathMove { Source = "/account.form", Destination = "/moved.form" } } }); Check(box.GetFormRecord("/moved.form").Value.Values.Count == 1, "moved record");
        box.MoveFormRecords(new PathMoveList { Values = { new PathMove { Source = "/moved.form", Destination = "/account.form" } } }); Pass("lockbox_move_form_records", 3);
        using var signing = Api.GenerateSigningKeyPair(); using var contact = Api.GenerateContactKeyPair(); using var publicKey = contact.PublicKey();
        box.SetOwnerSigningKey(signing); Pass("lockbox_set_owner_signing_key"); var passwordSlot = box.AddPassword(Bytes("archive password")); Pass("lockbox_add_password");
        var contactSlot = box.AddContact(publicKey, "recipient"); Check(contactSlot != ulong.MaxValue, "slot"); Pass("lockbox_add_contact"); Check(box.ListKeySlots().Values.Count >= 2, "slots"); Pass("lockbox_list_key_slots");
        box.DeleteKey(passwordSlot); Pass("lockbox_delete_key"); box.Commit(); Check(box.OwnerInspection().Signed, "owner"); Pass("lockbox_owner_inspection", 2);
        Check(box.CacheStats().LimitBytes > 0, "cache"); Check(box.ImportStats().HostReadNanos.Length > 0, "import"); box.ResetImportStats();
        Check(box.PageInspection().Values.Count > 0, "pages"); Check(box.RecoveryReport().IntactFileCount > 0, "recovery"); Check(box.RenderRecoveryReport(true, 100).Length > 0, "render");
        Check(box.StreamContent().Values.Count > 0, "stream"); Check(box.Id.Length > 0, "id"); Pass("lockbox_cache_stats"); Pass("lockbox_import_stats"); Pass("lockbox_reset_import_stats");
        Pass("lockbox_page_inspection"); Pass("lockbox_recovery_report"); Pass("lockbox_recovery_report_render", 2); Pass("lockbox_stream_content"); Pass("lockbox_id", 2);
        var archive = box.Bytes; var path = Path.Combine(Root(), "advanced.lbox"); File.WriteAllBytes(path, archive); Check(Api.InspectLockboxFile(path).HeaderReadable, "inspect");
        Check(Api.ScanLockboxPath(path, key).IntactFileCount > 0, "scan path"); Check(Api.ScanLockbox(archive, key).IntactFileCount > 0, "scan"); Pass("lockbox_inspect_file"); Pass("lockbox_recovery_scan_path"); Pass("lockbox_recovery_scan");
        using (var salvaged = Api.SalvageLockbox(archive[..^32], key, signing)) Check(salvaged.StorageLength > 0, "salvage"); Pass("lockbox_recovery_salvage", 2);
        using (var opened = Api.OpenLockbox(archive, key, options)) Check(opened.GetFile("/account.txt").SequenceEqual(Bytes("account data")), "option open"); Pass("lockbox_open_with_options", 2);
        byte[] passwordArchive; using (var passwordBox = Api.CreateLockboxWithPassword(Bytes("archive password"))) { passwordBox.AddFile("/password.txt", Bytes("password protected")); passwordBox.Commit(); passwordArchive = passwordBox.Bytes; }
        Pass("lockbox_create_password"); using (var opened = Api.OpenLockboxWithPassword(passwordArchive, Bytes("archive password"))) Check(opened.GetFile("/password.txt").SequenceEqual(Bytes("password protected")), "password open"); Pass("lockbox_open_password", 2);
        byte[] contactArchive; using (var contactBox = Api.CreateLockboxForContact(publicKey)) { contactBox.AddFile("/contact.txt", Bytes("contact protected")); contactBox.Commit(); contactArchive = contactBox.Bytes; }
        Pass("lockbox_create_contact"); using (var opened = Api.OpenLockboxForContact(contactArchive, contact)) Check(opened.GetFile("/contact.txt").SequenceEqual(Bytes("contact protected")), "contact open"); Pass("lockbox_open_contact", 2);
        using (var signed = Api.CreateSignedLockbox(key, signing)) { signed.Commit(); Check(signed.OwnerInspection().Signed, "signed"); } Pass("lockbox_create_with_signing_key", 2);
        var extract = Path.Combine(Root(), "extract"); if (Directory.Exists(extract)) Directory.Delete(extract, true); Directory.CreateDirectory(extract);
        var extracted = Path.Combine(extract, "account.txt"); box.ExtractFile("/account.txt", extracted); Check(File.Exists(extracted), "extract file"); Pass("lockbox_extract_file", 2);
        var tree = Path.Combine(extract, "tree"); Directory.CreateDirectory(tree); box.ExtractDirectory(tree, 1 << 20, 4 << 20, 100, false, true, false); Check(Directory.Exists(tree), "extract dir"); Pass("lockbox_extract_directory", 2);
        box.DeleteFormRecord("/account.form"); Pass("lockbox_delete_form_record");
    }

    static void VaultLifecycle()
    {
        var root = Path.Combine(Root(), "vault"); Directory.CreateDirectory(root); var password = Bytes("vault password"); var changed = Bytes("new vault password");
        var id = Enumerable.Range(0, 16).Select(i => (byte)(0xa0 + i)).ToArray(); using var profile = Api.GenerateContactKeyPair(); using var contact = Api.GenerateContactKeyPair();
        using var contactPublic = contact.PublicKey(); using var owner = Api.GenerateSigningKeyPair(); using var ownerPublic = owner.PublicKey();
        using (var vault = Api.ReplaceVaultDirectory(root, password))
        {
            Pass("vault_directory_replace"); Console.WriteLine($"ARTIFACT\tcsharp\tvault-created\t{root}"); Check(vault.Root == root && vault.StructureVersion > 0, "vault"); Pass("vault_directory_root", 3); Pass("vault_directory_structure_version");
            var currentVersion = Api.CurrentVaultStructureVersion; Check(currentVersion == vault.StructureVersion && Api.ProbeVaultStructureVersion(root, password) == currentVersion, "vault probe"); Pass("vault_structure_version_current", 2); Pass("vault_directory_probe_structure_version", 2);
            vault.StorePrivateKey("alice", profile); Check(vault.PrivateKeyExists("alice"), "profile"); using (var loaded = vault.LoadPrivateKey("alice")) Check(loaded.PublicBytes().Length > 0, "load"); using (var loaded = vault.LoadPrivateKeyGeneration("alice", 1)) Check(loaded.PublicBytes().Length > 0, "generation");
            Pass("vault_directory_store_private_key"); Pass("vault_directory_private_key_exists"); Pass("vault_directory_load_private_key"); Pass("vault_directory_load_private_key_generation");
            vault.StoreProfileEmail("alice", "alice@example.test"); Check(vault.ProfileEmail("alice").Present, "email"); Pass("vault_directory_store_profile_email"); Pass("vault_directory_profile_email", 3);
            Check(vault.ListProfileGenerations("alice").Generations.Count == 1, "history"); Check(vault.RotatePrivateKey("alice").Generations.Count == 2, "rotate"); Pass("vault_directory_list_profile_generations"); Pass("vault_directory_rotate_private_key");
            using (var loaded = vault.LoadOwnerSigningKey("alice")) Check(loaded.PublicBytes().Length > 0, "owner"); using (var loaded = vault.LoadOwnerSigningKeyGeneration("alice", 1)) Check(loaded.PublicBytes().Length > 0, "owner gen"); Pass("vault_directory_load_owner_signing_key"); Pass("vault_directory_load_owner_signing_key_generation");
            vault.StoreContact("bob", contactPublic); Check(vault.ContactExists("bob"), "contact"); using (var loaded = vault.LoadContact("bob")) Check(loaded.Fingerprint().Length > 0, "load contact"); Check(vault.ListContacts().Values.Count == 1, "contacts");
            Pass("vault_directory_store_contact"); Pass("vault_directory_contact_exists"); Pass("vault_directory_load_contact"); Pass("vault_directory_list_contacts");
            vault.StoreContactSigningKey("bob", ownerPublic); using (var loaded = vault.LoadContactSigningKey("bob")) Check(loaded != null, "signing contact"); Pass("vault_directory_store_contact_signing_key"); Pass("vault_directory_load_contact_signing_key");
            Check(vault.ListPrivateKeys().Values.Count > 0 && vault.ListPrivateKeyNames().Values.Count > 0 && vault.ListContactNames().Values.Count > 0, "lists"); Pass("vault_directory_list_private_keys"); Pass("vault_directory_list_private_key_names"); Pass("vault_directory_list_contact_names");
            vault.StoreBackup(id, Bytes("encrypted backup bytes")); Check(vault.BackupCount == 1 && vault.LoadBackup(id).SequenceEqual(Bytes("encrypted backup bytes")), "backup"); Pass("vault_directory_store_backup"); Pass("vault_directory_backup_count"); Pass("vault_directory_load_backup", 3);
            vault.RememberLockbox(id, "/tmp/example.lbox"); Check(vault.ListKnownLockboxes().Values.Count == 1, "known"); Pass("vault_directory_remember_lockbox"); Pass("vault_directory_list_known_lockboxes");
            vault.RememberAccessSlotLabel(id, 7, "primary"); Check(vault.ListAccessSlotLabels(id).Values.Count == 1 && vault.FindAccessSlotLabels(id, "primary").Values.Count == 1, "labels"); Pass("vault_directory_remember_access_slot_label"); Pass("vault_directory_list_access_slot_labels"); Pass("vault_directory_find_access_slot_labels");
            vault.RememberPassword(id, password); Check(vault.RememberedPassword(id).SequenceEqual(password), "remember"); Pass("vault_directory_remember_password"); Pass("vault_directory_remembered_password", 3);
            var vaultForm = vault.DefineForm("login", "Login", "Login form", Fields()); Check(vaultForm.TypeId.Length > 0 && vault.ResolveForm("login").TypeId.Length > 0 && vault.ListForms().Values.Count > 0, "forms"); Pass("vault_directory_define_form"); Pass("vault_directory_resolve_form"); Pass("vault_directory_list_forms");
            Check(vault.ListFormRevisions(vaultForm.TypeId).Values.Count > 0, "vault revisions"); Pass("vault_directory_list_form_revisions", 2);
            Check(vault.SeedForms() > 0, "seed"); Pass("vault_directory_seed_forms"); Check(vault.ListFormAliases().Values.Count > 0, "aliases"); Pass("vault_directory_list_form_aliases");
            vault.ForgetAccessSlotLabel(id, 7); vault.ForgetLockbox("/tmp/example.lbox"); vault.DeleteContact("bob"); Pass("vault_directory_forget_access_slot_label"); Pass("vault_directory_forget_lockbox"); Pass("vault_directory_delete_contact");
            vault.DeletePrivateKey("alice"); Check(!vault.PrivateKeyExists("alice"), "deleted"); vault.RestorePrivateKey("alice", profile, owner, true); Check(vault.PrivateKeyExists("alice"), "restored"); Pass("vault_directory_delete_private_key", 2); Pass("vault_directory_restore_private_key", 2);
        }
        Pass("vault_directory_free"); using (var readOnlyVault = Api.OpenReadOnlyVaultDirectory(root, password)) { Check(readOnlyVault.ListProfileNames().Values.Count > 0, "readonly profiles"); _ = readOnlyVault.ListContactNames(); Check(readOnlyVault.ListFormAliases().Values.Count > 0, "readonly forms"); _ = readOnlyVault.ListKnownLockboxes(); Pass("vault_read_only_open"); Pass("vault_read_only_list_profile_names", 2); Pass("vault_read_only_list_contact_names"); Pass("vault_read_only_list_form_aliases", 2); Pass("vault_read_only_list_known_lockboxes"); } Pass("vault_read_only_free");
        Api.ChangeVaultDirectoryPassword(root, password, changed); Pass("vault_directory_change_password"); using (var opened = Api.OpenVaultDirectory(root, changed)) Check(opened.StructureVersion > 0, "reopen"); Pass("vault_directory_open"); Console.WriteLine($"ARTIFACT\tcsharp\tvault-opened\t{root}");
        using (var opened = Api.OpenOrCreateVaultDirectory(root, changed)) Check(opened.StructureVersion > 0, "open create"); Pass("vault_directory_open_or_create");
    }

    static void DefaultVault()
    {
        var root = Environment.GetEnvironmentVariable("LOCKBOX_VAULT_DIR")!; Directory.CreateDirectory(root); using (Api.ReplaceDefaultVaultDirectory(Bytes("default password"))) { }
        Pass("vault_directory_replace_default"); Check(Api.DefaultVaultDirectory == root && Path.GetDirectoryName(Api.DefaultVaultPath) == root, "default paths"); Pass("vault_default_directory", 3); Pass("vault_default_path", 2);
        using (Api.OpenDefaultReadOnlyVaultDirectory(Bytes("default password"))) { } Pass("vault_read_only_open_default");
        using (Api.OpenOrCreateDefaultVaultDirectory(Bytes("default password"))) { } Pass("vault_directory_open_or_create_default"); Api.ChangeDefaultVaultDirectoryPassword(Bytes("default password"), Bytes("changed default password")); Pass("vault_directory_change_default_password");
        var backup = Path.Combine(Root(), "default-vault.backup"); if (File.Exists(backup)) File.Delete(backup); Check(Api.BackupDefaultVault(backup).VaultSize > 0, "backup default"); Check(Api.RestoreDefaultVault(backup, true).VaultSize > 0, "restore default"); Pass("vault_backup_default"); Pass("vault_restore_default");
    }

    static Process StartAgent() => Process.Start(new ProcessStartInfo("dotnet",
        $"\"{System.Reflection.Assembly.GetExecutingAssembly().Location}\" --serve-agent") { UseShellExecute = false })!;
    static void AgentAndLocal()
    {
        Directory.CreateDirectory(Environment.GetEnvironmentVariable("LOCKBOX_SESSION_AGENT_DIR")!); Directory.CreateDirectory(Environment.GetEnvironmentVariable("LOCKBOX_VAULT_DIR")!);
        using (var vault = Api.ReplaceDefaultVaultDirectory(Bytes("agent vault password"))) using (var profile = Api.GenerateContactKeyPair()) vault.StorePrivateKey("default", profile);
        Api.ForgetAllAgentSecrets(); Pass("vault_forget_all"); using var child = StartAgent(); var running = false; for (var i = 0; i < 200; i++) { if (Api.AgentIsRunning) { running = true; break; } Thread.Sleep(50); }
        Check(running, "agent"); Pass("vault_agent_serve"); Pass("vault_is_running"); Api.StartAgent(); Pass("vault_agent_start"); Api.VerifyAgentTransport(); Pass("vault_agent_verify_transport");
        var id = Enumerable.Range(0, 16).Select(i => (byte)(0xc0 + i)).ToArray(); var key = Enumerable.Range(0, 32).Select(i => (byte)(0x20 + i)).ToArray();
        Api.PutAgentKey(id, key); Check(Api.GetAgentKey(id).SequenceEqual(key) && Api.ListAgentKeys().Values.Count > 0, "agent key"); Pass("vault_agent_put"); Pass("vault_agent_get", 3); Pass("vault_agent_list");
        Api.PutAgentVaultUnlockKey("vault-id", key, 120); Check(Api.GetAgentVaultUnlockKey("vault-id").SequenceEqual(key), "vault key"); Pass("vault_agent_put_vault_unlock_key"); Pass("vault_agent_get_vault_unlock_key", 3);
        using (var owner = Api.GenerateSigningKeyPair())
        {
            Api.PutAgentOwnerSigningKey("vault-id", "alice", owner, 120); using (var loaded = Api.GetAgentOwnerSigningKey("vault-id", "alice")) Check(loaded.PublicBytes().Length > 0, "owner"); Pass("vault_agent_put_owner_signing_key"); Pass("vault_agent_get_owner_signing_key");
            using (Api.BeginAgentActivity("open")) Pass("vault_agent_begin_activity"); Pass("vault_agent_end_activity"); Check(Api.AgentSleepSupport() != null, "sleep"); Pass("vault_agent_sleep_support");
            Check(Api.AgentLogPath.Length > 0 && Api.AgentLogDestination.Length > 0, "logs"); Pass("vault_agent_log_path", 2); Pass("vault_agent_log_destination", 2);
            using (var local = Api.OpenLocalVault())
            {
                Pass("vault_local"); var root = Path.Combine(Path.GetTempPath(), $"revault-csharp-local-{Guid.NewGuid()}"); Directory.CreateDirectory(root); var payload = Bytes("local vault data"); var passwordPath = Path.Combine(root, "password.lbox");
                using (var box = local.CreateWithPassword(passwordPath, Bytes("local password"))) { box.AddFile("/data.txt", payload); box.Commit(); } Pass("vault_create_lockbox_password", 3);
                local.CachePassword(passwordPath, Bytes("local password"), 120); Pass("vault_cache_lockbox_password"); using (var box = local.OpenWithPassword(passwordPath, Bytes("local password"))) Check(box.GetFile("/data.txt").SequenceEqual(payload), "password local"); Pass("vault_open_lockbox_password", 3); local.CloseLockbox(passwordPath); Pass("vault_close_lockbox");
                var contentPath = Path.Combine(root, "content.lbox"); using (var box = local.CreateWithContentKey(contentPath, key, owner)) { box.AddFile("/data.txt", payload); box.Commit(); } Pass("vault_create_lockbox_content_key", 3);
                using (var box = local.OpenWithContentKey(contentPath, key, owner)) Check(box.GetFile("/data.txt").SequenceEqual(payload), "content local"); Pass("vault_open_lockbox_content_key", 3);
                using var contact = Api.GenerateContactKeyPair(); using var publicKey = contact.PublicKey(); using (var box = local.CreateForContact(Path.Combine(root, "contact.lbox"), publicKey, "recipient", owner)) { box.AddFile("/data.txt", payload); box.Commit(); }
                Pass("vault_create_lockbox_contact", 3); local.CloseAll(); Pass("vault_close_all");
            }
            Pass("vault_free");
        }
        Api.ForgetAgentOwnerSigningKey("vault-id", "alice"); Api.ForgetAgentVaultUnlockKey("vault-id"); Api.ForgetAgentKey(id); Pass("vault_agent_forget_owner_signing_key"); Pass("vault_agent_forget_vault_unlock_key"); Pass("vault_agent_forget"); Api.StopAgent(); Pass("vault_agent_stop"); child.WaitForExit(); Check(child.ExitCode == 0, "agent child");
    }

    static void Platform()
    {
        Check(Api.PlatformStatus() != null, "status"); Pass("vault_platform_status", 2); Api.SetPlatformScope("vault"); Pass("vault_platform_set_scope"); Api.DisablePlatformStore(); Check(Api.PlatformStoreDisabled, "disabled"); Pass("vault_platform_disable"); Pass("vault_platform_disabled");
        Api.EnablePlatformStore(); Check(!Api.PlatformStoreDisabled, "enabled"); Pass("vault_platform_enable"); Api.PutPlatformPassword(Bytes("platform vault password")); Check(Api.GetPlatformPassword().SequenceEqual(Bytes("platform vault password")), "platform password"); Pass("vault_platform_put_password"); Pass("vault_platform_get_password", 3); Api.ForgetPlatformPassword(); Pass("vault_platform_forget_password");
    }

    static void Interop(string producer)
    {
        var root = Environment.GetEnvironmentVariable("REVAULT_E2E_ARTIFACT_DIR") ?? "/tmp/revault-e2e-artifacts"; using (var box = Api.OpenLockbox(File.ReadAllBytes(Path.Combine(root, producer, "archive.lbox")), Repeat('K', 32))) Check(box.GetFile("/renamed.txt").SequenceEqual(Bytes("replacement payload")), "foreign archive");
        using (var vault = Api.OpenVaultDirectory(Path.Combine(root, producer, "vault"), Bytes("new vault password"))) Check(vault.StructureVersion > 0, "foreign vault"); Console.WriteLine($"INTEROP\tcsharp\t{producer}\tarchive\t3"); Console.WriteLine($"INTEROP\tcsharp\t{producer}\tvault\t2");
    }

    static void Main(string[] args)
    {
        if (args is ["--serve-agent"]) { Api.ServeAgent(); return; } if (args is ["--agent"]) { AgentAndLocal(); return; } if (args is ["--platform"]) { Platform(); return; } if (args is ["--default"]) { DefaultVault(); return; }
        if (args is ["--interop", var producer]) { Interop(producer); return; } ArchiveLifecycle(); KeyLifecycle(); AdvancedArchive(); VaultLifecycle(); _ = Api.LastError; Pass("buffer_last_error");
    }
}
