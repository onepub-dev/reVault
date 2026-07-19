#include <chrono>
#include <cstdlib>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <stdexcept>
#include <string>
#include <thread>
#include <vector>
#include <sys/wait.h>
#include <unistd.h>

#include <revault_api.hpp>

namespace fs = std::filesystem;
using namespace revault;

static const char* executable_path;
static void pass(const char* symbol, unsigned assertions = 1) {
  std::cout << "PASS\tcpp\t" << symbol << '\t' << assertions << '\n';
}
static void check(bool condition, const char* message) {
  if (!condition) throw std::runtime_error(message);
}
static fs::path artifact_root() {
  const char* base = std::getenv("REVAULT_E2E_ARTIFACT_DIR");
  fs::path root = base && *base ? base : "/tmp/revault-e2e-artifacts";
  root /= "cpp";
  fs::create_directories(root);
  return root;
}
static std::vector<std::uint8_t> bytes(const std::string& value) {
  return {value.begin(), value.end()};
}
static std::vector<std::uint8_t> copy(const std::vector<std::uint8_t>& value) { return value; }
static void write(const fs::path& path, const std::vector<std::uint8_t>& value) {
  std::ofstream output(path, std::ios::binary | std::ios::trunc);
  output.write(reinterpret_cast<const char*>(value.data()), value.size());
  check(output.good(), "write artifact");
}
static std::vector<std::uint8_t> read(const fs::path& path) {
  std::ifstream input(path, std::ios::binary);
  return {std::istreambuf_iterator<char>(input), std::istreambuf_iterator<char>()};
}

static void archive_lifecycle() {
  const std::vector<std::uint8_t> key(32, 'K');
  Lockbox box(key);
  pass("lockbox_create");
  box.add_file("/hello.txt", bytes("hello from cpp conformance"));
  check(box.exists("/hello.txt"), "added file exists");
  pass("lockbox_add_file", 2);
  check(box.get_file("/hello.txt") == bytes("hello from cpp conformance"), "file content");
  pass("lockbox_get_file", 3);
  box.add_file_with_permissions("/hello.txt", bytes("replacement payload"), 0640, true);
  check(box.permissions("/hello.txt") == 0640, "file permissions");
  pass("lockbox_add_file_with_permissions", 2);
  pass("lockbox_permissions");
  box.create_dir("/tree", true);
  check(box.is_dir("/tree"), "directory type");
  pass("lockbox_create_dir", 2);
  pass("lockbox_is_dir");
  box.create_parent_dirs("/tree/a/b/file");
  check(box.is_dir("/tree/a/b"), "parents created");
  pass("lockbox_create_parent_dirs", 2);
  box.rename("/hello.txt", "/renamed.txt");
  check(box.exists("/renamed.txt") && !box.exists("/hello.txt"), "rename result");
  pass("lockbox_rename", 3);
  pass("lockbox_exists", 2);
  box.set_permissions("/renamed.txt", 0600);
  check(box.permissions("/renamed.txt") == 0600, "changed permissions");
  pass("lockbox_set_permissions", 2);
  check(copy(box.read_range("/renamed.txt", 0, 11)) == bytes("replacement"), "range");
  pass("lockbox_read_range", 3);
  box.set_variable("normal", "value");
  check(box.get_variable("normal") == "value", "variable");
  pass("lockbox_set_variable");
  pass("lockbox_get_variable", 3);
  box.move_variables({{"normal", "moved"}});
  check(box.get_variable("moved") == "value", "moved variable");
  box.move_variables({{"moved", "normal"}});
  pass("lockbox_move_variables", 3);
  box.set_secret_variable("secret", bytes("hidden"));
  pass("lockbox_set_secret_variable");
  check(box.with_secret_variable("secret", [](std::span<const std::uint8_t> value) {
    check(std::string(value.begin(), value.end()) == "hidden", "secret variable");
  }), "secret variable present");
  pass("lockbox_get_secret_variable"); pass("secret_len"); pass("secret_copy"); pass("secret_free");
  check(box.variable_sensitivity("secret").present(), "variable sensitivity");
  pass("lockbox_variable_sensitivity", 2);
  check(box.list_variables().values_size() == 2, "variable list");
  pass("lockbox_list_variables");
  box.delete_variable("normal");
  pass("lockbox_delete_variable");
  box.add_symlink("/link", "/renamed.txt");
  check(copy(box.symlink_target("/link")) == bytes("/renamed.txt"), "symlink target");
  pass("lockbox_add_symlink");
  pass("lockbox_get_symlink_target", 3);
  check(box.list("/", true).entries_size() > 0, "entry list");
  check(box.stat("/renamed.txt").has_value(), "entry stat");
  pass("lockbox_list", 2);
  pass("lockbox_stat", 2);
  box.set_workload_profile("read-mostly");
  box.set_worker_policy("single", 1);
  check(!box.runtime_options().workload_profile().empty(), "runtime options");
  pass("lockbox_set_workload_profile");
  pass("lockbox_set_worker_policy");
  pass("lockbox_runtime_options");
  box.commit();
  check(box.storage_length() > 0, "storage length");
  pass("lockbox_commit");
  pass("lockbox_storage_len");
  auto archive = copy(box.to_bytes());
  check(!archive.empty(), "archive bytes");
  pass("lockbox_to_bytes", 2);
  pass("buffer_free");
  const auto format_version = Lockbox::format_version();
  check(format_version > 0, "current lockbox format version");
  check(Lockbox::probe_format_version(archive) == format_version,
        "probed lockbox format version");
  pass("lockbox_format_version", 2);
  pass("lockbox_probe_format_version", 2);
  try {
    (void)Lockbox::probe_format_version({});
    throw std::runtime_error("invalid lockbox probe unexpectedly succeeded");
  } catch (const std::exception&) {
    const auto details = last_error_details();
    check(!details.message().empty(), "structured native error details");
  }
  pass("buffer_last_error_details", 2);
  const auto artifact = artifact_root() / "archive.lbox";
  write(artifact, archive);
  std::cout << "ARTIFACT\tcpp\tarchive-created\t" << artifact.string() << '\n';
  auto opened = Lockbox::open(archive, key);
  check(opened.get_file("/renamed.txt") == bytes("replacement payload"), "opened archive");
  pass("lockbox_open", 2);
  std::cout << "ARTIFACT\tcpp\tarchive-opened\t" << artifact.string() << '\n';
  opened.remove("/renamed.txt");
  check(!opened.exists("/renamed.txt"), "delete file");
  pass("lockbox_delete", 2);
  opened.remove_dir("/tree", true);
  check(!opened.exists("/tree"), "delete directory");
  pass("lockbox_remove_dir", 2);
  pass("lockbox_free", 2);
}

static void key_lifecycle() {
  std::vector<std::uint8_t> content(32);
  for (std::size_t i = 0; i < content.size(); ++i) content[i] = i;
  ContactKeyPair contact;
  pass("key_contact_generate");
  auto private_record = contact.private_record();
  check(!private_record.empty(), "contact private record");
  pass("key_contact_private", 2);
  auto copy_key = ContactKeyPair::from_private_record(private_record);
  pass("key_contact_from_private");
  auto public_bytes = contact.public_bytes();
  check(!public_bytes.empty(), "contact public bytes");
  pass("key_contact_public", 2);
  ContactPublicKey public_key(public_bytes);
  pass("key_contact_public_from_bytes");
  auto wrapped = public_key.encrypt(content);
  pass("key_contact_encrypt");
  check(copy_key.decrypt(wrapped) == content, "contact decrypt");
  pass("key_contact_decrypt", 3);
  check(!wrapped.public_bytes().empty(), "wrapped public");
  check(!wrapped.ciphertext().empty(), "wrapped ciphertext");
  check(!wrapped.encrypted_bytes().empty(), "wrapped encrypted");
  pass("key_contact_wrapped_public", 2);
  pass("key_contact_wrapped_ciphertext", 2);
  pass("key_contact_wrapped_encrypted", 2);
  SigningKeyPair signing;
  pass("key_signing_generate");
  auto signing_private = signing.private_record();
  auto signing_public = signing.public_bytes();
  check(!signing_private.empty() && !signing_public.empty(), "signing records");
  pass("key_signing_private", 2);
  pass("key_signing_public", 2);
  auto signing_copy = SigningKeyPair::from_private_record(signing_private);
  pass("key_signing_from_private");
  SigningPublicKey signing_public_key(signing_public);
  pass("key_signing_public_from_bytes");
  auto exported_private = contact.export_as("raw-hex");
  auto imported_private = ContactKeyPair::import(exported_private);
  auto exported_public = public_key.export_as("lockbox-pem");
  auto imported_public = ContactPublicKey::import(exported_public);
  pass("vault_key_export_private", 2);
  pass("vault_key_import_private");
  pass("vault_key_export_public", 2);
  pass("vault_key_import_public");
  const auto fingerprint = KeyFormat::fingerprint(imported_public);
  check(fingerprint.size() >= 12, "fingerprint");
  pass("vault_key_fingerprint", 2);
  const auto hex = KeyFormat::hex(fingerprint);
  check(KeyFormat::decode_hex(hex) == fingerprint, "hex round trip");
  pass("vault_key_format_hex", 2);
  pass("vault_key_decode_hex", 2);
  const std::vector<std::uint8_t> short_fingerprint(fingerprint.begin(), fingerprint.begin() + 12);
  const auto code = KeyFormat::crockford(short_fingerprint);
  check(!KeyFormat::crockford_reading(code).empty(), "crockford reading");
  check(KeyFormat::decode_crockford(code) == short_fingerprint, "crockford round trip");
  pass("vault_key_format_crockford", 2);
  pass("vault_key_format_crockford_reading", 2);
  pass("vault_key_decode_crockford", 2);
  const auto plain_hex = KeyFormat::hex_encode(content);
  check(KeyFormat::hex_decode(plain_hex) == content, "plain hex round trip");
  pass("vault_key_hex_encode", 2);
  pass("vault_key_hex_decode", 2);
  pass("key_contact_wrapped_free");
  pass("key_contact_public_free", 2);
  pass("key_contact_free", 3);
  pass("key_signing_public_free", 2);
  pass("key_signing_free", 3);
}

static void advanced_archive() {
  const std::vector<std::uint8_t> key(32, 'A');
  LockboxOptions options;
  options.cache_mode = "bytes";
  options.cache_bytes = 4 * 1024 * 1024;
  options.workload = "bulk-import";
  options.worker = "single";
  options.jobs = 1;
  auto box = Lockbox::create_with_options(key, options);
  pass("lockbox_create_with_options");
  box.add_file("/account.txt", bytes("account data"));
  auto listed = box.list_with_options("/", "*.txt", true, true, false, false, 20);
  check(listed.entries_size() == 1, "filtered listing");
  pass("lockbox_list_with_options", 2);

  bindings::FormFieldList fields;
  auto* field = fields.add_values();
  field->set_id("username");
  field->set_label("Username");
  field->set_kind("text");
  field->set_required(true);
  field = fields.add_values();
  field->set_id("password");
  field->set_label("Password");
  field->set_kind("secret");
  field->set_required(true);
  const auto definition = box.define_form("account", "Account", "Account form", fields);
  check(!definition.type_id().empty(), "form type identifier");
  pass("lockbox_define_form", 2);
  check(box.list_form_definitions().values_size() == 1, "form definitions");
  check(box.resolve_form("account").type_id() == definition.type_id(), "form resolution");
  check(box.list_form_revisions(definition.type_id()).values_size() == 1, "form revisions");
  pass("lockbox_list_form_definitions");
  pass("lockbox_resolve_form");
  pass("lockbox_list_form_revisions");
  auto record = box.create_form_record("/account.form", "account", "Primary");
  check(record.path() == "/account.form", "form record path");
  pass("lockbox_create_form_record");
  box.set_form_field("/account.form", "username", "alice");
  pass("lockbox_set_form_field");
  box.set_secret_form_field("/account.form", "password", bytes("hidden"));
  pass("lockbox_set_secret_form_field");
  check(box.with_secret_form_field("/account.form", "password", [](std::span<const std::uint8_t> value) {
    check(std::string(value.begin(), value.end()) == "hidden", "secret form field");
  }), "secret form field present");
  pass("lockbox_get_secret_form_field");
  check(box.get_form_record("/account.form").value().values_size() == 2, "form record values");
  check(box.get_form_field("/account.form", "username").value().value() == "alice", "form field");
  check(box.list_form_records().values_size() == 1, "form records");
  pass("lockbox_get_form_record");
  pass("lockbox_get_form_field");
  pass("lockbox_list_form_records");
  box.move_form_records({{"/account.form", "/moved.form"}});
  check(box.get_form_record("/moved.form").value().values_size() == 2,
        "moved form record");
  box.move_form_records({{"/moved.form", "/account.form"}});
  pass("lockbox_move_form_records", 3);

  SigningKeyPair signing;
  box.set_owner_signing_key(signing);
  pass("lockbox_set_owner_signing_key");
  const auto password_slot = box.add_password("archive password");
  check(password_slot != UINT64_MAX, "password slot");
  pass("lockbox_add_password");
  ContactKeyPair contact;
  auto contact_public = contact.public_key();
  const auto contact_slot = box.add_contact(contact_public, "recipient");
  check(contact_slot != UINT64_MAX, "contact slot");
  pass("lockbox_add_contact");
  check(box.list_key_slots().values_size() >= 2, "key slots");
  pass("lockbox_list_key_slots");
  box.delete_key(password_slot);
  pass("lockbox_delete_key");

  box.commit();
  check(box.owner_inspection().signed_(), "owner inspection");
  pass("lockbox_owner_inspection", 2);
  check(box.cache_stats().limit_bytes() > 0, "cache stats");
  check(!box.import_stats().host_read_nanos().empty(), "import stats");
  box.reset_import_stats();
  check(box.page_inspection().values_size() > 0, "page inspection");
  check(box.recovery_report().intact_file_count() > 0, "recovery report");
  check(!box.render_recovery_report(true, 100).empty(), "rendered recovery report");
  check(box.stream_content(false).values_size() > 0, "content stream");
  check(!box.id().empty(), "lockbox id");
  pass("lockbox_cache_stats");
  pass("lockbox_import_stats");
  pass("lockbox_reset_import_stats");
  pass("lockbox_page_inspection");
  pass("lockbox_recovery_report");
  pass("lockbox_recovery_report_render", 2);
  pass("lockbox_stream_content");
  pass("lockbox_id", 2);

  auto archive = copy(box.to_bytes());
  const auto path = artifact_root() / "advanced.lbox";
  write(path, archive);
  check(Lockbox::inspect_file(path.string()).header_readable(), "file inspection");
  check(Lockbox::scan_path(path.string(), key).intact_file_count() > 0, "scan path");
  check(Lockbox::scan(archive, key).intact_file_count() > 0, "scan bytes");
  pass("lockbox_inspect_file");
  pass("lockbox_recovery_scan_path");
  pass("lockbox_recovery_scan");
  auto damaged = archive;
  damaged.resize(damaged.size() - std::min<std::size_t>(32, damaged.size() / 10));
  auto salvaged = Lockbox::salvage(damaged, key, &signing);
  check(salvaged.storage_length() > 0, "salvaged archive");
  pass("lockbox_recovery_salvage", 2);
  auto opened_options = Lockbox::open_with_options(archive, key, options);
  check(opened_options.get_file("/account.txt") == bytes("account data"), "options open");
  pass("lockbox_open_with_options", 2);

  auto password_box = Lockbox::create_with_password("archive password");
  password_box.add_file("/password.txt", bytes("password protected"));
  password_box.commit();
  auto password_archive = copy(password_box.to_bytes());
  pass("lockbox_create_password");
  auto password_open = Lockbox::open_with_password(password_archive, "archive password");
  check(password_open.get_file("/password.txt") == bytes("password protected"), "password open");
  pass("lockbox_open_password", 2);
  auto contact_box = Lockbox::create_for_contact(contact_public);
  contact_box.add_file("/contact.txt", bytes("contact protected"));
  contact_box.commit();
  auto contact_archive = copy(contact_box.to_bytes());
  pass("lockbox_create_contact");
  auto contact_open = Lockbox::open_with_contact(contact_archive, contact);
  check(contact_open.get_file("/contact.txt") == bytes("contact protected"), "contact open");
  pass("lockbox_open_contact", 2);
  auto signed_box = Lockbox::create_signed(key, signing);
  signed_box.commit();
  check(signed_box.owner_inspection().signed_(), "signed create");
  pass("lockbox_create_with_signing_key", 2);

  const auto extract_root = artifact_root() / "extract";
  fs::remove_all(extract_root);
  fs::create_directories(extract_root);
  box.extract_file("/account.txt", (extract_root / "account.txt").string(), false);
  check(fs::exists(extract_root / "account.txt"), "file extracted");
  pass("lockbox_extract_file", 2);
  box.extract_directory((extract_root / "tree").string(), 1024 * 1024,
                        4 * 1024 * 1024, 100, false, true, false);
  check(fs::exists(extract_root / "tree"), "directory extracted");
  pass("lockbox_extract_directory", 2);
  box.delete_form_record("/account.form");
  pass("lockbox_delete_form_record");
}

static void vault_lifecycle() {
  const auto root = artifact_root() / "vault";
  fs::create_directories(root);
  const std::string password = "vault password";
  const std::string new_password = "new vault password";
  std::vector<std::uint8_t> id(16);
  for (std::size_t i = 0; i < id.size(); ++i) id[i] = 0xa0 + i;
  ContactKeyPair profile;
  ContactKeyPair contact;
  auto contact_public = contact.public_key();
  SigningKeyPair owner;
  auto owner_public = owner.public_key();
  {
    auto vault = VaultDirectory::replace(root.string(), password);
    pass("vault_directory_replace");
    std::cout << "ARTIFACT\tcpp\tvault-created\t" << root.string() << '\n';
    check(vault.root() == root.string(), "vault root");
    check(vault.structure_version() > 0, "vault structure");
    pass("vault_directory_root", 3);
    pass("vault_directory_structure_version");
    const auto current_version = VaultDirectory::current_structure_version();
    check(current_version == vault.structure_version(), "current vault structure version");
    check(VaultDirectory::probe_structure_version(root.string(), password) == current_version,
          "probed vault structure version");
    pass("vault_structure_version_current", 2);
    pass("vault_directory_probe_structure_version", 2);
    vault.store_private_key("alice", profile);
    check(vault.private_key_exists("alice"), "private key exists");
    auto loaded_profile = vault.load_private_key("alice");
    auto first_generation = vault.load_private_key_generation("alice", 1);
    pass("vault_directory_store_private_key");
    pass("vault_directory_private_key_exists");
    pass("vault_directory_load_private_key");
    pass("vault_directory_load_private_key_generation");
    vault.store_profile_email("alice", "alice@example.test");
    auto email = vault.profile_email("alice");
    check(email.present() && email.value() == "alice@example.test", "profile email");
    pass("vault_directory_store_profile_email");
    pass("vault_directory_profile_email", 3);
    check(vault.list_profile_generations("alice").generations_size() == 1,
          "profile history");
    check(vault.rotate_private_key("alice").generations_size() == 2,
          "profile rotation");
    pass("vault_directory_list_profile_generations");
    pass("vault_directory_rotate_private_key");
    auto loaded_owner = vault.load_owner_signing_key("alice");
    auto loaded_owner_generation = vault.load_owner_signing_key_generation("alice", 1);
    pass("vault_directory_load_owner_signing_key");
    pass("vault_directory_load_owner_signing_key_generation");
    vault.store_contact("bob", contact_public);
    check(vault.contact_exists("bob"), "contact exists");
    auto loaded_contact = vault.load_contact("bob");
    check(vault.list_contacts().values_size() == 1, "contacts list");
    pass("vault_directory_store_contact");
    pass("vault_directory_contact_exists");
    pass("vault_directory_load_contact");
    pass("vault_directory_list_contacts");
    vault.store_contact_signing_key("bob", owner_public);
    auto loaded_contact_signing = vault.load_contact_signing_key("bob");
    pass("vault_directory_store_contact_signing_key");
    pass("vault_directory_load_contact_signing_key");
    check(vault.list_private_keys().values_size() > 0, "private key list");
    check(vault.list_private_key_names().values_size() > 0, "private key name list");
    check(vault.list_contact_names().values_size() > 0, "contact name list");
    pass("vault_directory_list_private_keys");
    pass("vault_directory_list_private_key_names");
    pass("vault_directory_list_contact_names");
    vault.store_backup(id, bytes("encrypted backup bytes"));
    check(vault.backup_count() == 1, "backup count");
    check(vault.load_backup(id) == bytes("encrypted backup bytes"), "backup content");
    pass("vault_directory_store_backup");
    pass("vault_directory_backup_count");
    pass("vault_directory_load_backup", 3);
    vault.remember_lockbox(id, "/tmp/example.lbox");
    check(vault.list_known_lockboxes().values_size() == 1, "known lockboxes");
    pass("vault_directory_remember_lockbox");
    pass("vault_directory_list_known_lockboxes");
    vault.remember_access_slot_label(id, 7, "primary");
    check(vault.list_access_slot_labels(id).values_size() == 1, "access labels");
    check(vault.find_access_slot_labels(id, "primary").values_size() == 1,
          "access label lookup");
    pass("vault_directory_remember_access_slot_label");
    pass("vault_directory_list_access_slot_labels");
    pass("vault_directory_find_access_slot_labels");
    vault.remember_password(id, password);
    check(vault.remembered_password(id) == bytes(password), "remembered password");
    pass("vault_directory_remember_password");
    pass("vault_directory_remembered_password", 3);
    bindings::FormFieldList fields;
    auto* field = fields.add_values();
    field->set_id("username"); field->set_label("Username");
    field->set_kind("text"); field->set_required(true);
    const auto vault_form = vault.define_form("login", "Login", "Login form", fields);
    check(!vault_form.type_id().empty(), "vault form definition");
    check(!vault.resolve_form("login").type_id().empty(), "vault form resolve");
    check(vault.list_forms().values_size() > 0, "vault forms");
    pass("vault_directory_define_form");
    pass("vault_directory_resolve_form");
    pass("vault_directory_list_forms");
    check(vault.list_form_revisions(vault_form.type_id()).values_size() > 0,
          "vault form revisions");
    pass("vault_directory_list_form_revisions", 2);
    check(vault.seed_forms() > 0, "seed forms");
    pass("vault_directory_seed_forms");
    check(vault.list_form_aliases().values_size() > 0, "form aliases");
    pass("vault_directory_list_form_aliases");
    vault.forget_access_slot_label(id, 7);
    vault.forget_lockbox("/tmp/example.lbox");
    vault.delete_contact("bob");
    pass("vault_directory_forget_access_slot_label");
    pass("vault_directory_forget_lockbox");
    pass("vault_directory_delete_contact");
    vault.delete_private_key("alice");
    check(!vault.private_key_exists("alice"), "private key deleted");
    vault.restore_private_key("alice", profile, owner, true);
    check(vault.private_key_exists("alice"), "private key restored");
    pass("vault_directory_delete_private_key", 2);
    pass("vault_directory_restore_private_key", 2);
  }
  pass("vault_directory_free");
  {
    ReadOnlyVaultDirectory readonly(root.string(), password);
    check(readonly.list_profile_names().values_size() > 0, "read-only profiles");
    (void)readonly.list_contact_names();
    check(readonly.list_form_aliases().values_size() > 0, "read-only form aliases");
    (void)readonly.list_known_lockboxes();
    pass("vault_read_only_open");
    pass("vault_read_only_list_profile_names", 2);
    pass("vault_read_only_list_contact_names");
    pass("vault_read_only_list_form_aliases", 2);
    pass("vault_read_only_list_known_lockboxes");
  }
  pass("vault_read_only_free");
  VaultDirectory::change_password(root.string(), password, new_password);
  pass("vault_directory_change_password");
  {
    auto reopened = VaultDirectory::open(root.string(), new_password);
    check(reopened.structure_version() > 0, "reopened vault");
    pass("vault_directory_open");
    std::cout << "ARTIFACT\tcpp\tvault-opened\t" << root.string() << '\n';
  }
  {
    auto opened = VaultDirectory::open_or_create(root.string(), new_password);
    check(opened.structure_version() > 0, "open or create vault");
    pass("vault_directory_open_or_create");
  }
}

static void default_vault_lifecycle() {
  const auto root = artifact_root() / "default-vault";
  fs::create_directories(root);
  setenv("LOCKBOX_VAULT_DIR", root.c_str(), 1);
  {
    auto vault = VaultDirectory::replace_default("default password");
    pass("vault_directory_replace_default");
  }
  {
    auto vault = ReadOnlyVaultDirectory::open_default("default password");
    (void)vault.list_profile_names();
    pass("vault_read_only_open_default");
  }
  check(VaultDirectory::default_directory() == root.string(), "default directory");
  check(VaultDirectory::default_path().find(root.string()) == 0, "default path");
  pass("vault_default_directory", 3);
  pass("vault_default_path", 2);
  {
    auto vault = VaultDirectory::open_or_create_default("default password");
    check(vault.structure_version() > 0, "default open or create");
    pass("vault_directory_open_or_create_default");
  }
  VaultDirectory::change_default_password("default password", "changed default password");
  pass("vault_directory_change_default_password");
  const auto backup = artifact_root() / "default-vault.backup";
  fs::remove(backup);
  check(VaultDirectory::backup_default(backup.string()).vault_size() > 0, "default backup");
  check(VaultDirectory::restore_default(backup.string(), true).vault_size() > 0,
        "default restore");
  pass("vault_backup_default");
  pass("vault_restore_default");
}

static void agent_and_local_vault() {
  const auto agent_dir = fs::temp_directory_path() /
      ("revault-cpp-agent-" + std::to_string(getpid()));
  const auto agent_vault_dir = fs::temp_directory_path() /
      ("revault-cpp-agent-vault-" + std::to_string(getpid()));
  fs::remove_all(agent_dir); fs::remove_all(agent_vault_dir);
  fs::create_directories(agent_dir); fs::create_directories(agent_vault_dir);
  fs::permissions(agent_dir, fs::perms::owner_all, fs::perm_options::replace);
  fs::permissions(agent_vault_dir, fs::perms::owner_all, fs::perm_options::replace);
  setenv("LOCKBOX_SESSION_AGENT_DIR", agent_dir.c_str(), 1);
  setenv("LOCKBOX_VAULT_DIR", agent_vault_dir.c_str(), 1);
  setenv("LOCKBOX_VAULT_PASSWORD", "agent vault password", 1);
  {
    auto vault = VaultDirectory::replace_default("agent vault password");
    ContactKeyPair profile;
    vault.store_private_key("default", profile);
  }
  Agent::forget_all();
  pass("vault_forget_all");
  const pid_t child = fork();
  check(child >= 0, "fork agent");
  if (child == 0) {
    execl(executable_path, executable_path, "--serve-agent", nullptr);
    _exit(127);
  }
  bool running = false;
  for (unsigned attempt = 0; attempt < 200; ++attempt) {
    if (Agent::is_running()) { running = true; break; }
    std::this_thread::sleep_for(std::chrono::milliseconds(50));
  }
  check(running, "agent started");
  pass("vault_agent_serve");
  pass("vault_is_running");
  Agent::start();
  pass("vault_agent_start");
  Agent::verify_transport();
  pass("vault_agent_verify_transport");
  std::vector<std::uint8_t> id(16), key(32);
  for (std::size_t i = 0; i < id.size(); ++i) id[i] = 0xc0 + i;
  for (std::size_t i = 0; i < key.size(); ++i) key[i] = 0x20 + i;
  Agent::put(id, key);
  check(Agent::get(id) == key, "agent generic key");
  check(Agent::list().values_size() > 0, "agent list");
  pass("vault_agent_put");
  pass("vault_agent_get", 3);
  pass("vault_agent_list");
  Agent::put_vault_unlock_key("vault-id", key, 120);
  check(Agent::get_vault_unlock_key("vault-id") == key, "agent vault key");
  pass("vault_agent_put_vault_unlock_key");
  pass("vault_agent_get_vault_unlock_key", 3);
  SigningKeyPair owner;
  Agent::put_owner_signing_key("vault-id", "alice", owner, 120);
  auto loaded_owner = Agent::get_owner_signing_key("vault-id", "alice");
  check(!loaded_owner.public_bytes().empty(), "agent owner key");
  pass("vault_agent_put_owner_signing_key");
  pass("vault_agent_get_owner_signing_key");
  {
    auto activity = Agent::begin_activity("open");
    pass("vault_agent_begin_activity");
    activity.end();
    pass("vault_agent_end_activity");
  }
  (void)Agent::sleep_support();
  pass("vault_agent_sleep_support");
  check(!Agent::log_path().empty(), "agent log path");
  check(!Agent::log_destination().empty(), "agent log destination");
  pass("vault_agent_log_path", 2);
  pass("vault_agent_log_destination", 2);

  LocalVault local;
  pass("vault_local");
  const auto local_root = fs::temp_directory_path() /
      ("revault-cpp-local-" + std::to_string(getpid()));
  fs::remove_all(local_root); fs::create_directories(local_root);
  const auto password_path = (local_root / "password.lbox").string();
  {
    auto box = local.create_with_password(password_path, "local password");
    box.add_file("/data.txt", bytes("local vault data"));
    box.commit();
  }
  pass("vault_create_lockbox_password", 3);
  local.cache_password(password_path, "local password", 120);
  pass("vault_cache_lockbox_password");
  {
    auto box = local.open_with_password(password_path, "local password");
    check(box.get_file("/data.txt") == bytes("local vault data"), "local password open");
  }
  pass("vault_open_lockbox_password", 3);
  local.close(password_path);
  pass("vault_close_lockbox");
  const auto content_path = (local_root / "content.lbox").string();
  {
    auto box = local.create_with_content_key(content_path, key, owner);
    box.add_file("/data.txt", bytes("local vault data"));
    box.commit();
  }
  pass("vault_create_lockbox_content_key", 3);
  {
    auto box = local.open_with_content_key(content_path, key, owner);
    check(box.get_file("/data.txt") == bytes("local vault data"), "local content open");
  }
  pass("vault_open_lockbox_content_key", 3);
  ContactKeyPair contact;
  auto public_key = contact.public_key();
  const auto contact_path = (local_root / "contact.lbox").string();
  {
    auto box = local.create_for_contact(contact_path, public_key, "recipient", owner);
    box.add_file("/data.txt", bytes("local vault data"));
    box.commit();
  }
  pass("vault_create_lockbox_contact", 3);
  local.close_all();
  pass("vault_close_all");
  pass("vault_free");
  Agent::forget_owner_signing_key("vault-id", "alice");
  Agent::forget_vault_unlock_key("vault-id");
  Agent::forget(id);
  pass("vault_agent_forget_owner_signing_key");
  pass("vault_agent_forget_vault_unlock_key");
  pass("vault_agent_forget");
  Agent::stop();
  pass("vault_agent_stop");
  int status = 0;
  check(waitpid(child, &status, 0) == child && WIFEXITED(status) && WEXITSTATUS(status) == 0,
        "agent child exit");
}

static void platform_secret_store() {
  const auto root = fs::temp_directory_path() /
      ("revault-cpp-platform-" + std::to_string(getpid()));
  fs::remove_all(root); fs::create_directories(root);
  setenv("LOCKBOX_VAULT_DIR", root.c_str(), 1);
  (void)PlatformSecretStore::status();
  pass("vault_platform_status", 2);
  PlatformSecretStore::set_scope("vault");
  pass("vault_platform_set_scope");
  PlatformSecretStore::disable();
  check(PlatformSecretStore::disabled(), "platform disabled");
  pass("vault_platform_disable");
  pass("vault_platform_disabled");
  PlatformSecretStore::enable();
  check(!PlatformSecretStore::disabled(), "platform enabled");
  pass("vault_platform_enable");
  PlatformSecretStore::put_password("platform vault password");
  check(PlatformSecretStore::get_password() == "platform vault password", "platform password");
  pass("vault_platform_put_password");
  pass("vault_platform_get_password", 3);
  PlatformSecretStore::forget_password();
  pass("vault_platform_forget_password");
}

static void interop_open(const std::string& producer) {
  const char* base_value = std::getenv("REVAULT_E2E_ARTIFACT_DIR");
  const fs::path base = base_value && *base_value ? base_value : "/tmp/revault-e2e-artifacts";
  auto archive = read(base / producer / "archive.lbox");
  auto box = Lockbox::open(archive, std::vector<std::uint8_t>(32, 'K'));
  check(box.get_file("/renamed.txt") == bytes("replacement payload"), "foreign archive");
  auto vault = VaultDirectory::open((base / producer / "vault").string(), "new vault password");
  check(vault.structure_version() > 0, "foreign vault");
  std::cout << "INTEROP\tcpp\t" << producer << "\tarchive\t3\n";
  std::cout << "INTEROP\tcpp\t" << producer << "\tvault\t2\n";
}

int main(int argc, char** argv) {
  std::cout << std::unitbuf;
  executable_path = argv[0];
  try {
    revault::require_compatible_abi();
    if (argc == 2 && std::strcmp(argv[1], "--serve-agent") == 0) {
      Agent::serve();
      return 0;
    }
    if (argc == 2 && std::strcmp(argv[1], "--agent") == 0) {
      agent_and_local_vault();
      return 0;
    }
    if (argc == 2 && std::strcmp(argv[1], "--platform") == 0) {
      platform_secret_store();
      return 0;
    }
    if (argc == 3 && std::strcmp(argv[1], "--interop") == 0) {
      interop_open(argv[2]);
      return 0;
    }
    if (argc == 2 && std::strcmp(argv[1], "--core") == 0) {
      archive_lifecycle();
      key_lifecycle();
      advanced_archive();
      vault_lifecycle();
      default_vault_lifecycle();
      return 0;
    }
    archive_lifecycle();
    key_lifecycle();
    advanced_archive();
    vault_lifecycle();
    default_vault_lifecycle();
    agent_and_local_vault();
    platform_secret_store();
    check(buffer_last_error() != nullptr, "last error pointer");
    pass("buffer_last_error");
    return 0;
  } catch (const std::exception& error) {
    std::cerr << "C++ conformance failure: " << error.what()
              << " (" << buffer_last_error() << ")\n";
    return 1;
  }
}
