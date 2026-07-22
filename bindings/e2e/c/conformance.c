#ifdef _WIN32
#define _CRT_SECURE_NO_WARNINGS
#endif

#define _GNU_SOURCE
#include <inttypes.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#ifdef _WIN32
#include <direct.h>
#include <io.h>
#include <process.h>
#include <windows.h>
#define access _access
#define F_OK 0
#define mkdir(path, mode) _mkdir(path)
#define unlink _unlink
#define PATH_SEPARATOR "\\"
typedef HANDLE AgentProcess;
#else
#include <sys/wait.h>
#include <unistd.h>
#define PATH_SEPARATOR "/"
typedef pid_t AgentProcess;
#endif

#include <revault_api.h>

#define CHECK(condition, message)                                                \
  do {                                                                           \
    if (!(condition)) {                                                          \
      fprintf(stderr, "conformance failure: %s (%s)\n", message,               \
              buffer_last_error());                                              \
      exit(1);                                                                   \
    }                                                                            \
  } while (0)

#define PASS(symbol, assertions) \
  printf("PASS\t%s\t%s\t%d\n", language_name(), #symbol, assertions)

typedef struct {
  const uint8_t *ptr;
  size_t len;
} Bytes;

static const char *executable_path;
static void write_file(const char *path, const uint8_t *bytes, size_t len);
static unsigned temporary_counter;

static unsigned long process_id(void) {
#ifdef _WIN32
  return (unsigned long)GetCurrentProcessId();
#else
  return (unsigned long)getpid();
#endif
}

static const char *temporary_root(void) {
  const char *root = getenv("TMPDIR");
#ifdef _WIN32
  if (root == NULL || *root == '\0') root = getenv("TEMP");
  if (root == NULL || *root == '\0') root = ".";
#else
  if (root == NULL || *root == '\0') root = "/tmp";
#endif
  return root;
}

static void temporary_path(char *out, size_t out_len, const char *stem) {
  snprintf(out, out_len, "%s%s%s-%lu-%u", temporary_root(), PATH_SEPARATOR,
           stem, process_id(), ++temporary_counter);
}

static void make_temp_dir(char *out, size_t out_len, const char *stem) {
  for (unsigned attempt = 0; attempt < 100; ++attempt) {
    temporary_path(out, out_len, stem);
    if (mkdir(out, 0700) == 0) return;
  }
  CHECK(false, "create temporary directory");
}

static void make_temp_file(char *out, size_t out_len, const char *stem) {
  for (unsigned attempt = 0; attempt < 100; ++attempt) {
    temporary_path(out, out_len, stem);
#ifdef _WIN32
    HANDLE file = CreateFileA(out, GENERIC_WRITE, 0, NULL, CREATE_NEW,
                              FILE_ATTRIBUTE_NORMAL, NULL);
    if (file != INVALID_HANDLE_VALUE) {
      CloseHandle(file);
      return;
    }
#else
    FILE *file = fopen(out, "wx");
    if (file != NULL) {
      fclose(file);
      return;
    }
#endif
  }
  CHECK(false, "create temporary file");
}

static int set_environment(const char *name, const char *value) {
#ifdef _WIN32
  return _putenv_s(name, value);
#else
  return setenv(name, value, 1);
#endif
}

static AgentProcess spawn_agent(void) {
#ifdef _WIN32
  char command[4096];
  snprintf(command, sizeof(command), "\"%s\" --serve-agent", executable_path);
  STARTUPINFOA startup = {0};
  PROCESS_INFORMATION process = {0};
  startup.cb = sizeof(startup);
  CHECK(CreateProcessA(NULL, command, NULL, NULL, FALSE, 0, NULL, NULL,
                       &startup, &process),
        "start agent process");
  CloseHandle(process.hThread);
  return process.hProcess;
#else
  pid_t child = fork();
  CHECK(child >= 0, "fork agent process");
  if (child == 0) {
    execl(executable_path, executable_path, "--serve-agent", NULL);
    _exit(127);
  }
  return child;
#endif
}

static void wait_for_agent(AgentProcess child) {
#ifdef _WIN32
  CHECK(WaitForSingleObject(child, 10000) == WAIT_OBJECT_0,
        "agent stopped within ten seconds");
  DWORD status = 1;
  CHECK(GetExitCodeProcess(child, &status) && status == 0,
        "agent exit status");
  CloseHandle(child);
#else
  int status = 0;
  CHECK(waitpid(child, &status, 0) == child, "wait for agent");
  CHECK(WIFEXITED(status) && WEXITSTATUS(status) == 0, "agent exit status");
#endif
}

static void trace_phase(const char *phase) {
  fprintf(stderr, "native conformance: %s\n", phase);
  fflush(stderr);
}

static void pause_for_agent(void) {
#ifdef _WIN32
  Sleep(50);
#else
  usleep(50000);
#endif
}

static const char *language_name(void) {
  const char *value = getenv("REVAULT_E2E_LANGUAGE");
  return value && *value ? value : "c";
}

static const char *artifact_directory(void) {
  const char *value = getenv("REVAULT_E2E_ARTIFACT_DIR");
  return value && *value ? value : "/tmp/revault-e2e-artifacts";
}

static void artifact_path(char *out, size_t out_len, const char *name) {
  const char *base = artifact_directory();
  const char *binding = language_name();
  size_t base_len = strlen(base);
  size_t binding_len = strlen(binding);
  size_t name_len = strlen(name);
  (void)mkdir(base, 0700);
  char language[512];
  CHECK(base_len + 1 + binding_len < sizeof(language),
        "artifact language path is too long");
  memcpy(language, base, base_len);
  language[base_len] = '/';
  memcpy(language + base_len + 1, binding, binding_len + 1);
  (void)mkdir(language, 0700);
  CHECK(base_len + 1 + binding_len + 1 + name_len < out_len,
        "artifact path is too long");
  memcpy(out, language, base_len + 1 + binding_len);
  out[base_len + 1 + binding_len] = '/';
  memcpy(out + base_len + 1 + binding_len + 1, name, name_len + 1);
}

static uint8_t *read_artifact(const char *path, size_t *length) {
  FILE *file = fopen(path, "rb");
  CHECK(file != NULL, "open foreign artifact");
  CHECK(fseek(file, 0, SEEK_END) == 0, "seek foreign artifact");
  long size = ftell(file);
  CHECK(size > 0 && fseek(file, 0, SEEK_SET) == 0, "size foreign artifact");
  uint8_t *bytes = malloc((size_t)size);
  CHECK(bytes != NULL, "allocate foreign artifact");
  CHECK(fread(bytes, 1, (size_t)size, file) == (size_t)size,
        "read foreign artifact");
  CHECK(fclose(file) == 0, "close foreign artifact");
  *length = (size_t)size;
  return bytes;
}

static void interop_open(const char *producer) {
  char archive_path[512];
  char vault_path[512];
  snprintf(archive_path, sizeof(archive_path), "%s/%s/archive.lbox",
           artifact_directory(), producer);
  snprintf(vault_path, sizeof(vault_path), "%s/%s/vault",
           artifact_directory(), producer);
  size_t archive_len = 0;
  uint8_t *archive = read_artifact(archive_path, &archive_len);
  uint8_t key[32];
  memset(key, 'K', sizeof(key));
  void *box = lockbox_open(archive, archive_len, key, sizeof(key));
  CHECK(box != NULL, "open foreign archive");
  const char path[] = "/renamed.txt";
  const char expected[] = "replacement payload";
  RevaultBuffer value = lockbox_get_file(box, path, sizeof(path) - 1);
  CHECK(value.ptr != NULL && value.len == sizeof(expected) - 1 &&
            memcmp(value.ptr, expected, value.len) == 0,
        "read foreign archive content");
  buffer_free(value);
  lockbox_free(box);
  free(archive);

  const uint8_t password[] = "new vault password";
  void *vault = vault_directory_open(vault_path, strlen(vault_path), password,
                                     sizeof(password) - 1);
  CHECK(vault != NULL, "open foreign vault");
  CHECK(vault_directory_structure_version(vault) > 0,
        "inspect foreign vault structure");
  vault_directory_free(vault);
  printf("INTEROP\t%s\t%s\tarchive\t3\n", language_name(), producer);
  printf("INTEROP\t%s\t%s\tvault\t2\n", language_name(), producer);
}

static Bytes framed_payload(RevaultBuffer value) {
  CHECK(value.ptr != NULL, "expected returned buffer");
  CHECK(value.len >= 8, "FlatBuffer is too short");
  return (Bytes){value.ptr, value.len};
}

static uint16_t little_u16(const uint8_t *value) {
  return (uint16_t)value[0] | (uint16_t)((uint16_t)value[1] << 8);
}

static uint32_t little_u32(const uint8_t *value) {
  return (uint32_t)value[0] | ((uint32_t)value[1] << 8) |
         ((uint32_t)value[2] << 16) | ((uint32_t)value[3] << 24);
}

static const uint8_t *flatbuffer_field(Bytes message, uint32_t wanted) {
  CHECK(wanted > 0 && message.len >= 8, "invalid FlatBuffer field");
  uint32_t root = little_u32(message.ptr);
  CHECK(root + 4 <= message.len, "invalid FlatBuffer root");
  const uint8_t *table = message.ptr + root;
  uint32_t back = little_u32(table);
  CHECK(back <= root, "invalid FlatBuffer vtable");
  const uint8_t *vtable = table - back;
  uint16_t vtable_len = little_u16(vtable);
  size_t entry = 4 + (size_t)(wanted - 1) * 2;
  if (entry + 2 > vtable_len) return NULL;
  uint16_t offset = little_u16(vtable + entry);
  return offset == 0 ? NULL : table + offset;
}

static Bytes flatbuffer_bytes(Bytes message, uint32_t wanted) {
  const uint8_t *field = flatbuffer_field(message, wanted);
  if (field == NULL) return (Bytes){NULL, 0};
  const uint8_t *value = field + little_u32(field);
  CHECK(value + 4 <= message.ptr + message.len, "invalid FlatBuffer offset");
  size_t len = little_u32(value);
  CHECK(value + 4 + len <= message.ptr + message.len,
        "invalid FlatBuffer vector");
  return (Bytes){value + 4, len};
}

static uint64_t flatbuffer_scalar(Bytes message, uint32_t wanted, bool *found) {
  const uint8_t *field = flatbuffer_field(message, wanted);
  *found = field != NULL;
  return field == NULL ? 0 : field[0];
}

static void expect_raw(RevaultBuffer value, const uint8_t *expected,
                       size_t expected_len) {
  CHECK(value.ptr != NULL, "expected raw buffer");
  CHECK(value.len == expected_len, "raw buffer length");
  CHECK(memcmp(value.ptr, expected, expected_len) == 0, "raw buffer value");
  buffer_free(value);
  PASS(buffer_free, 1);
}

static void expect_optional_string(RevaultBuffer value, const char *expected) {
  Bytes payload = framed_payload(value);
  bool found = false;
  CHECK(flatbuffer_scalar(payload, 1, &found) == 1 && found,
        "optional string presence");
  Bytes text = flatbuffer_bytes(payload, 2);
  CHECK(text.ptr != NULL && text.len == strlen(expected),
        "optional string length");
  CHECK(memcmp(text.ptr, expected, text.len) == 0, "optional string value");
  buffer_free(value);
}

static void archive_lifecycle(void) {
  uint8_t key[32];
  memset(key, 0x4b, sizeof(key));
  const uint8_t hello[] = "hello from c conformance";
  const uint8_t replacement[] = "replacement payload";

  void *box = lockbox_create(key, sizeof(key));
  CHECK(box != NULL, "lockbox_create");
  PASS(lockbox_create, 1);

  CHECK(lockbox_add_file(box, "/hello.txt", 10, hello, sizeof(hello) - 1,
                         false),
        "lockbox_add_file");
  CHECK(lockbox_exists(box, "/hello.txt", 10), "file visible after add");
  PASS(lockbox_add_file, 2);

  expect_raw(lockbox_get_file(box, "/hello.txt", 10), hello,
             sizeof(hello) - 1);
  PASS(lockbox_get_file, 3);

  CHECK(lockbox_add_file_with_permissions(
            box, "/hello.txt", 10, replacement, sizeof(replacement) - 1,
            0640, true),
        "replace with permissions");
  CHECK(lockbox_permissions(box, "/hello.txt", 10) == 0640,
        "replacement permissions");
  PASS(lockbox_add_file_with_permissions, 2);
  PASS(lockbox_permissions, 1);

  CHECK(lockbox_create_dir(box, "/tree", 5, true), "create directory");
  CHECK(lockbox_is_dir(box, "/tree", 5), "created path is directory");
  PASS(lockbox_create_dir, 2);
  PASS(lockbox_is_dir, 1);

  CHECK(lockbox_create_parent_dirs(box, "/tree/a/b/file", 14),
        "create parent directories");
  CHECK(lockbox_is_dir(box, "/tree/a/b", 9), "parents created");
  PASS(lockbox_create_parent_dirs, 2);

  CHECK(lockbox_rename(box, "/hello.txt", 10, "/renamed.txt", 12),
        "rename file");
  CHECK(lockbox_exists(box, "/renamed.txt", 12), "renamed file exists");
  CHECK(!lockbox_exists(box, "/hello.txt", 10), "old path absent");
  PASS(lockbox_rename, 3);
  PASS(lockbox_exists, 2);

  CHECK(lockbox_set_permissions(box, "/renamed.txt", 12, 0600),
        "set permissions");
  CHECK(lockbox_permissions(box, "/renamed.txt", 12) == 0600,
        "permissions changed");
  PASS(lockbox_set_permissions, 2);

  RevaultBuffer range = lockbox_read_range(box, "/renamed.txt", 12, 0, 11);
  CHECK(range.ptr != NULL && range.len == 11 &&
            memcmp(range.ptr, replacement, 11) == 0,
        "range read value");
  buffer_free(range);
  PASS(lockbox_read_range, 3);

  CHECK(lockbox_set_variable(box, "normal", 6, "value", 5),
        "normal variable");
  expect_optional_string(lockbox_get_variable(box, "normal", 6), "value");
  PASS(lockbox_set_variable, 1);
  PASS(lockbox_get_variable, 3);
  static const uint8_t move_normal[] = {
      0x0c,0x00,0x00,0x00,0x00,0x00,0x06,0x00,0x08,0x00,0x04,0x00,
      0x06,0x00,0x00,0x00,0x04,0x00,0x00,0x00,0x01,0x00,0x00,0x00,
      0x0c,0x00,0x00,0x00,0x08,0x00,0x0c,0x00,0x08,0x00,0x04,0x00,
      0x08,0x00,0x00,0x00,0x08,0x00,0x00,0x00,0x10,0x00,0x00,0x00,
      0x05,0x00,0x00,0x00,'m','o','v','e','d',0x00,0x00,0x00,
      0x06,0x00,0x00,0x00,'n','o','r','m','a','l',0x00,0x00};
  static const uint8_t move_back[] = {
      0x0c,0x00,0x00,0x00,0x00,0x00,0x06,0x00,0x08,0x00,0x04,0x00,
      0x06,0x00,0x00,0x00,0x04,0x00,0x00,0x00,0x01,0x00,0x00,0x00,
      0x0c,0x00,0x00,0x00,0x08,0x00,0x0c,0x00,0x08,0x00,0x04,0x00,
      0x08,0x00,0x00,0x00,0x08,0x00,0x00,0x00,0x10,0x00,0x00,0x00,
      0x06,0x00,0x00,0x00,'n','o','r','m','a','l',0x00,0x00,
      0x05,0x00,0x00,0x00,'m','o','v','e','d',0x00,0x00,0x00};
  CHECK(lockbox_move_variables(box, move_normal, sizeof(move_normal)), "move variable");
  expect_optional_string(lockbox_get_variable(box, "moved", 5), "value");
  CHECK(lockbox_move_variables(box, move_back, sizeof(move_back)), "move variable back");
  PASS(lockbox_move_variables, 3);

  CHECK(lockbox_set_secret_variable(box, "secret", 6,
                                    (const uint8_t *)"hidden", 6),
        "secret variable");
  PASS(lockbox_set_secret_variable, 1);
  void *secret_handle = NULL;
  CHECK(lockbox_get_secret_variable(box, "secret", 6, &secret_handle) &&
            secret_handle != NULL,
        "get secret variable");
  size_t secret_length = 0;
  CHECK(secret_len(secret_handle, &secret_length) && secret_length == 6,
        "secret length");
  uint8_t secret_bytes[6] = {0};
  CHECK(secret_copy(secret_handle, secret_bytes, sizeof(secret_bytes)) &&
            memcmp(secret_bytes, "hidden", sizeof(secret_bytes)) == 0,
        "secret copy");
  memset(secret_bytes, 0, sizeof(secret_bytes));
  secret_free(secret_handle);
  PASS(lockbox_get_secret_variable, 1);
  PASS(secret_len, 1);
  PASS(secret_copy, 1);
  PASS(secret_free, 1);
  RevaultBuffer sensitivity = lockbox_variable_sensitivity(box, "secret", 6);
  Bytes sensitivity_payload = framed_payload(sensitivity);
  Bytes sensitivity_text = flatbuffer_bytes(sensitivity_payload, 2);
  CHECK(sensitivity_text.len == 6 &&
            memcmp(sensitivity_text.ptr, "secret", 6) == 0,
        "secret sensitivity");
  buffer_free(sensitivity);
  PASS(lockbox_variable_sensitivity, 2);

  RevaultBuffer variables = lockbox_list_variables(box);
  CHECK(framed_payload(variables).len > 0, "variable list payload");
  buffer_free(variables);
  PASS(lockbox_list_variables, 1);

  CHECK(lockbox_delete_variable(box, "normal", 6), "delete variable");
  PASS(lockbox_delete_variable, 1);

  CHECK(lockbox_add_symlink(box, "/link", 5, "/renamed.txt", 12, false),
        "add symlink");
  RevaultBuffer target = lockbox_get_symlink_target(box, "/link", 5);
  expect_raw(target, (const uint8_t *)"/renamed.txt", 12);
  PASS(lockbox_add_symlink, 1);
  PASS(lockbox_get_symlink_target, 3);

  RevaultBuffer listing = lockbox_list(box, "/", 1, true);
  Bytes listing_payload = framed_payload(listing);
  CHECK(flatbuffer_bytes(listing_payload, 1).ptr != NULL, "listing entry");
  buffer_free(listing);
  PASS(lockbox_list, 2);

  RevaultBuffer stat = lockbox_stat(box, "/renamed.txt", 12);
  Bytes stat_payload = framed_payload(stat);
  CHECK(flatbuffer_bytes(stat_payload, 1).ptr != NULL, "stat entry");
  buffer_free(stat);
  PASS(lockbox_stat, 2);

  CHECK(lockbox_set_workload_profile(box, "read-mostly", 11),
        "workload profile");
  CHECK(lockbox_set_worker_policy(box, "single", 6, 1), "worker policy");
  RevaultBuffer options = lockbox_runtime_options(box);
  CHECK(framed_payload(options).len > 0, "runtime options");
  buffer_free(options);
  PASS(lockbox_set_workload_profile, 1);
  PASS(lockbox_set_worker_policy, 1);
  PASS(lockbox_runtime_options, 1);

  CHECK(lockbox_commit(box), "commit archive");
  CHECK(lockbox_storage_len(box) > 0, "storage length after commit");
  PASS(lockbox_commit, 1);
  PASS(lockbox_storage_len, 1);

  RevaultBuffer archive = lockbox_to_bytes(box);
  CHECK(archive.ptr != NULL && archive.len > 0, "archive bytes");
  PASS(lockbox_to_bytes, 2);
  CHECK(lockbox_format_version() > 0, "current lockbox format version");
  CHECK(lockbox_probe_format_version(archive.ptr, archive.len) == lockbox_format_version(), "probe lockbox format");
  PASS(lockbox_format_version, 1);
  PASS(lockbox_probe_format_version, 2);
  (void)lockbox_probe_format_version(NULL, 1);
  RevaultBuffer error_details = buffer_last_error_details();
  CHECK(error_details.ptr != NULL && error_details.len > 12, "typed error details");
  buffer_free(error_details);
  PASS(buffer_last_error_details, 1);
  char artifact[512];
  artifact_path(artifact, sizeof(artifact), "archive.lbox");
  write_file(artifact, archive.ptr, archive.len);
  printf("ARTIFACT\t%s\tarchive-created\t%s\n", language_name(), artifact);

  void *opened = lockbox_open(archive.ptr, archive.len, key, sizeof(key));
  CHECK(opened != NULL, "reopen archive");
  expect_raw(lockbox_get_file(opened, "/renamed.txt", 12), replacement,
             sizeof(replacement) - 1);
  PASS(lockbox_open, 2);
  printf("ARTIFACT\t%s\tarchive-opened\t%s\n", language_name(), artifact);

  CHECK(lockbox_delete(opened, "/renamed.txt", 12), "delete file");
  CHECK(!lockbox_exists(opened, "/renamed.txt", 12), "deleted file absent");
  PASS(lockbox_delete, 2);
  CHECK(lockbox_remove_dir(opened, "/tree", 5, true), "remove tree");
  CHECK(!lockbox_exists(opened, "/tree", 5), "tree removed");
  PASS(lockbox_remove_dir, 2);

  lockbox_free(opened);
  lockbox_free(box);
  buffer_free(archive);
  PASS(lockbox_free, 2);
}

static void key_lifecycle(void) {
  uint8_t content_key[32];
  for (size_t i = 0; i < sizeof(content_key); ++i) content_key[i] = (uint8_t)i;

  void *contact = key_contact_generate();
  CHECK(contact != NULL, "generate contact key");
  PASS(key_contact_generate, 1);

  RevaultBuffer private_record = key_contact_private(contact);
  CHECK(private_record.ptr != NULL && private_record.len > 32,
        "contact private record");
  PASS(key_contact_private, 2);
  void *contact_copy =
      key_contact_from_private(private_record.ptr, private_record.len);
  CHECK(contact_copy != NULL, "import contact private record");
  PASS(key_contact_from_private, 1);
  buffer_free(private_record);

  RevaultBuffer public_record = key_contact_public(contact);
  CHECK(public_record.ptr != NULL && public_record.len > 0,
        "contact public record");
  PASS(key_contact_public, 2);
  void *public_key =
      key_contact_public_from_bytes(public_record.ptr, public_record.len);
  CHECK(public_key != NULL, "import contact public record");
  PASS(key_contact_public_from_bytes, 1);

  void *wrapped = key_contact_encrypt(public_key, content_key, sizeof(content_key));
  CHECK(wrapped != NULL, "wrap content key");
  PASS(key_contact_encrypt, 1);
  RevaultBuffer decrypted = key_contact_decrypt(contact_copy, wrapped);
  CHECK(decrypted.ptr != NULL && decrypted.len == sizeof(content_key) &&
            memcmp(decrypted.ptr, content_key, sizeof(content_key)) == 0,
        "unwrap content key");
  buffer_free(decrypted);
  PASS(key_contact_decrypt, 3);

  RevaultBuffer wrapped_public = key_contact_wrapped_public(wrapped);
  CHECK(wrapped_public.ptr != NULL && wrapped_public.len > 0,
        "wrapped public component");
  buffer_free(wrapped_public);
  PASS(key_contact_wrapped_public, 2);
  RevaultBuffer wrapped_ciphertext = key_contact_wrapped_ciphertext(wrapped);
  CHECK(wrapped_ciphertext.ptr != NULL && wrapped_ciphertext.len > 0,
        "wrapped ciphertext");
  buffer_free(wrapped_ciphertext);
  PASS(key_contact_wrapped_ciphertext, 2);
  RevaultBuffer wrapped_encrypted = key_contact_wrapped_encrypted(wrapped);
  CHECK(wrapped_encrypted.ptr != NULL && wrapped_encrypted.len > 0,
        "wrapped encrypted key");
  buffer_free(wrapped_encrypted);
  PASS(key_contact_wrapped_encrypted, 2);

  RevaultBuffer exported_private =
      vault_key_export_private(contact, "raw-hex", 7);
  CHECK(exported_private.ptr != NULL && exported_private.len > 0,
        "export private key");
  PASS(vault_key_export_private, 2);
  void *imported_private =
      vault_key_import_private(exported_private.ptr, exported_private.len);
  CHECK(imported_private != NULL, "import exported private key");
  PASS(vault_key_import_private, 1);
  buffer_free(exported_private);

  RevaultBuffer exported_public =
      vault_key_export_public(public_key, "lockbox-pem", 11);
  CHECK(exported_public.ptr != NULL && exported_public.len > 0,
        "export public key");
  PASS(vault_key_export_public, 2);
  void *imported_public =
      vault_key_import_public(exported_public.ptr, exported_public.len);
  CHECK(imported_public != NULL, "import exported public key");
  PASS(vault_key_import_public, 1);
  buffer_free(exported_public);

  RevaultBuffer fingerprint = vault_key_fingerprint(imported_public);
  CHECK(fingerprint.ptr != NULL && fingerprint.len >= 12, "key fingerprint");
  PASS(vault_key_fingerprint, 2);

  RevaultBuffer fingerprint_hex =
      vault_key_format_hex(fingerprint.ptr, fingerprint.len);
  CHECK(fingerprint_hex.ptr != NULL && fingerprint_hex.len > fingerprint.len,
        "format fingerprint hex");
  PASS(vault_key_format_hex, 2);
  RevaultBuffer decoded_hex = vault_key_decode_hex(
      (const char *)fingerprint_hex.ptr, fingerprint_hex.len);
  CHECK(decoded_hex.len == fingerprint.len &&
            memcmp(decoded_hex.ptr, fingerprint.ptr, fingerprint.len) == 0,
        "decode fingerprint hex");
  buffer_free(decoded_hex);
  buffer_free(fingerprint_hex);
  PASS(vault_key_decode_hex, 2);

  RevaultBuffer crockford = vault_key_format_crockford(fingerprint.ptr, 12);
  CHECK(crockford.ptr != NULL && crockford.len > 0, "format crockford");
  PASS(vault_key_format_crockford, 2);
  RevaultBuffer reading = vault_key_format_crockford_reading(
      (const char *)crockford.ptr, crockford.len);
  CHECK(reading.ptr != NULL && reading.len >= crockford.len,
        "format crockford reading");
  PASS(vault_key_format_crockford_reading, 2);
  RevaultBuffer decoded_crockford =
      vault_key_decode_crockford((const char *)crockford.ptr, crockford.len);
  CHECK(decoded_crockford.len == 12 &&
            memcmp(decoded_crockford.ptr, fingerprint.ptr, 12) == 0,
        "decode crockford");
  buffer_free(decoded_crockford);
  buffer_free(reading);
  buffer_free(crockford);
  PASS(vault_key_decode_crockford, 2);

  RevaultBuffer encoded = vault_key_hex_encode(content_key, sizeof(content_key));
  CHECK(encoded.ptr != NULL && encoded.len == sizeof(content_key) * 2,
        "hex encode");
  PASS(vault_key_hex_encode, 2);
  RevaultBuffer decoded =
      vault_key_hex_decode((const char *)encoded.ptr, encoded.len);
  CHECK(decoded.len == sizeof(content_key) &&
            memcmp(decoded.ptr, content_key, sizeof(content_key)) == 0,
        "hex decode");
  buffer_free(decoded);
  buffer_free(encoded);
  PASS(vault_key_hex_decode, 2);
  buffer_free(fingerprint);

  void *signing = key_signing_generate();
  CHECK(signing != NULL, "generate signing key");
  PASS(key_signing_generate, 1);
  RevaultBuffer signing_private = key_signing_private(signing);
  CHECK(signing_private.ptr != NULL && signing_private.len > 0,
        "signing private record");
  PASS(key_signing_private, 2);
  void *signing_copy =
      key_signing_from_private(signing_private.ptr, signing_private.len);
  CHECK(signing_copy != NULL, "import signing private record");
  PASS(key_signing_from_private, 1);
  buffer_free(signing_private);
  RevaultBuffer signing_public = key_signing_public(signing);
  CHECK(signing_public.ptr != NULL && signing_public.len > 0,
        "signing public record");
  PASS(key_signing_public, 2);
  void *signing_public_copy =
      key_signing_public_from_bytes(signing_public.ptr, signing_public.len);
  CHECK(signing_public_copy != NULL, "import signing public record");
  PASS(key_signing_public_from_bytes, 1);
  buffer_free(signing_public);

  key_signing_public_free(signing_public_copy);
  key_signing_free(signing_copy);
  key_signing_free(signing);
  PASS(key_signing_public_free, 1);
  PASS(key_signing_free, 2);

  key_contact_wrapped_free(wrapped);
  key_contact_public_free(imported_public);
  key_contact_free(imported_private);
  key_contact_public_free(public_key);
  key_contact_free(contact_copy);
  key_contact_free(contact);
  PASS(key_contact_wrapped_free, 1);
  PASS(key_contact_public_free, 2);
  PASS(key_contact_free, 3);
  buffer_free(public_record);
}

static void expect_framed_nonempty(RevaultBuffer value, const char *message) {
  Bytes payload = framed_payload(value);
  CHECK(payload.len > 0, message);
  buffer_free(value);
}

static void vault_lifecycle(void) {
  char root[512];
  artifact_path(root, sizeof(root), "vault");
  (void)mkdir(root, 0700);
  const uint8_t password[] = "vault password";
  const uint8_t new_password[] = "new vault password";
  uint8_t id[16];
  for (size_t i = 0; i < sizeof(id); ++i) id[i] = (uint8_t)(0xa0 + i);

  void *vault = vault_directory_replace(root, strlen(root), password,
                                        sizeof(password) - 1);
  CHECK(vault != NULL, "replace vault directory");
  PASS(vault_directory_replace, 1);
  printf("ARTIFACT\t%s\tvault-created\t%s\n", language_name(), root);

  RevaultBuffer returned_root = vault_directory_root(vault);
  CHECK(returned_root.ptr != NULL && returned_root.len == strlen(root) &&
            memcmp(returned_root.ptr, root, returned_root.len) == 0,
        "vault root");
  buffer_free(returned_root);
  PASS(vault_directory_root, 3);
  CHECK(vault_directory_structure_version(vault) > 0,
        "vault structure version");
  PASS(vault_directory_structure_version, 1);
  CHECK(vault_structure_version_current() == vault_directory_structure_version(vault), "current vault structure version");
  CHECK(vault_directory_probe_structure_version(root, strlen(root), password, sizeof(password) - 1) == vault_structure_version_current(), "probe vault structure version");
  PASS(vault_structure_version_current, 1);
  PASS(vault_directory_probe_structure_version, 2);

  void *profile_key = key_contact_generate();
  void *contact_key = key_contact_generate();
  void *contact_public = NULL;
  void *contact_signing = key_signing_generate();
  void *contact_signing_public = NULL;
  CHECK(profile_key && contact_key && contact_signing, "vault fixture keys");
  RevaultBuffer contact_bytes = key_contact_public(contact_key);
  contact_public =
      key_contact_public_from_bytes(contact_bytes.ptr, contact_bytes.len);
  CHECK(contact_public != NULL, "vault contact public fixture");
  RevaultBuffer contact_signing_bytes = key_signing_public(contact_signing);
  contact_signing_public = key_signing_public_from_bytes(
      contact_signing_bytes.ptr, contact_signing_bytes.len);
  CHECK(contact_signing_public != NULL, "vault contact signing public fixture");

  CHECK(vault_directory_store_private_key(vault, "alice", 5, profile_key),
        "store profile private key");
  CHECK(vault_directory_private_key_exists(vault, "alice", 5),
        "profile private key exists");
  PASS(vault_directory_store_private_key, 1);
  PASS(vault_directory_private_key_exists, 1);
  void *loaded_profile =
      vault_directory_load_private_key(vault, "alice", 5);
  CHECK(loaded_profile != NULL, "load profile private key");
  PASS(vault_directory_load_private_key, 1);
  void *generation_zero =
      vault_directory_load_private_key_generation(vault, "alice", 5, 1);
  CHECK(generation_zero != NULL, "load profile generation one");
  PASS(vault_directory_load_private_key_generation, 1);

  CHECK(vault_directory_store_profile_email(
            vault, "alice", 5, "alice@example.test", 18),
        "store profile email");
  expect_optional_string(vault_directory_profile_email(vault, "alice", 5),
                         "alice@example.test");
  PASS(vault_directory_store_profile_email, 1);
  PASS(vault_directory_profile_email, 3);

  expect_framed_nonempty(vault_directory_list_profile_generations(
                             vault, "alice", 5),
                         "profile generation history");
  PASS(vault_directory_list_profile_generations, 1);
  expect_framed_nonempty(vault_directory_rotate_private_key(vault, "alice", 5),
                         "rotated profile history");
  PASS(vault_directory_rotate_private_key, 1);

  void *owner_key =
      vault_directory_load_owner_signing_key(vault, "alice", 5);
  CHECK(owner_key != NULL, "load profile owner signing key");
  PASS(vault_directory_load_owner_signing_key, 1);
  void *owner_generation =
      vault_directory_load_owner_signing_key_generation(vault, "alice", 5, 1);
  CHECK(owner_generation != NULL, "load owner signing generation");
  PASS(vault_directory_load_owner_signing_key_generation, 1);

  CHECK(vault_directory_store_contact(vault, "bob", 3, contact_public),
        "store contact");
  CHECK(vault_directory_contact_exists(vault, "bob", 3), "contact exists");
  PASS(vault_directory_store_contact, 1);
  PASS(vault_directory_contact_exists, 1);
  void *loaded_contact = vault_directory_load_contact(vault, "bob", 3);
  CHECK(loaded_contact != NULL, "load contact");
  PASS(vault_directory_load_contact, 1);
  expect_framed_nonempty(vault_directory_list_contacts(vault), "contact list");
  PASS(vault_directory_list_contacts, 1);

  CHECK(vault_directory_store_contact_signing_key(
            vault, "bob", 3, contact_signing_public),
        "store contact signing key");
  void *loaded_contact_signing =
      vault_directory_load_contact_signing_key(vault, "bob", 3);
  CHECK(loaded_contact_signing != NULL, "load contact signing key");
  PASS(vault_directory_store_contact_signing_key, 1);
  PASS(vault_directory_load_contact_signing_key, 1);

  expect_framed_nonempty(vault_directory_list_private_keys(vault),
                         "private key list");
  expect_framed_nonempty(vault_directory_list_private_key_names(vault),
                         "private key name list");
  expect_framed_nonempty(vault_directory_list_contact_names(vault),
                         "contact name list");
  PASS(vault_directory_list_private_keys, 1);
  PASS(vault_directory_list_private_key_names, 1);
  PASS(vault_directory_list_contact_names, 1);

  const uint8_t backup[] = "encrypted backup bytes";
  CHECK(vault_directory_store_backup(vault, id, sizeof(id), backup,
                                     sizeof(backup) - 1),
        "store backup");
  CHECK(vault_directory_backup_count(vault) == 1, "backup count");
  expect_raw(vault_directory_load_backup(vault, id, sizeof(id)), backup,
             sizeof(backup) - 1);
  PASS(vault_directory_store_backup, 1);
  PASS(vault_directory_backup_count, 1);
  PASS(vault_directory_load_backup, 3);

  CHECK(vault_directory_remember_lockbox(vault, id, sizeof(id),
                                         "/tmp/example.lbox", 17),
        "remember lockbox");
  expect_framed_nonempty(vault_directory_list_known_lockboxes(vault),
                         "known lockboxes");
  PASS(vault_directory_remember_lockbox, 1);
  PASS(vault_directory_list_known_lockboxes, 1);

  CHECK(vault_directory_remember_access_slot_label(vault, id, sizeof(id), 7,
                                              "primary", 7),
        "remember access label");
  expect_framed_nonempty(
      vault_directory_list_access_slot_labels(vault, id, sizeof(id)),
      "access label list");
  expect_framed_nonempty(vault_directory_find_access_slot_labels(
                             vault, id, sizeof(id), "primary", 7),
                         "access label lookup");
  PASS(vault_directory_remember_access_slot_label, 1);
  PASS(vault_directory_list_access_slot_labels, 1);
  PASS(vault_directory_find_access_slot_labels, 1);

  CHECK(vault_directory_remember_password(vault, id, sizeof(id), password,
                                          sizeof(password) - 1),
        "remember lockbox password");
  expect_raw(vault_directory_remembered_password(vault, id, sizeof(id)),
             password, sizeof(password) - 1);
  PASS(vault_directory_remember_password, 1);
  PASS(vault_directory_remembered_password, 3);

  static const uint8_t form_fields[] = {
      0x0c,0x00,0x00,0x00,0x00,0x00,0x06,0x00,0x08,0x00,0x04,0x00,
      0x06,0x00,0x00,0x00,0x04,0x00,0x00,0x00,0x02,0x00,0x00,0x00,
      0x54,0x00,0x00,0x00,0x04,0x00,0x00,0x00,0xc0,0xff,0xff,0xff,
      0x00,0x00,0x00,0x01,0x0c,0x00,0x00,0x00,0x14,0x00,0x00,0x00,
      0x20,0x00,0x00,0x00,0x06,0x00,0x00,0x00,'s','e','c','r','e','t',
      0x00,0x00,0x08,0x00,0x00,0x00,'P','a','s','s','w','o','r','d',
      0x00,0x00,0x00,0x00,0x08,0x00,0x00,0x00,'p','a','s','s','w','o','r','d',
      0x00,0x00,0x00,0x00,0x0c,0x00,0x14,0x00,0x10,0x00,0x0c,0x00,
      0x08,0x00,0x07,0x00,0x0c,0x00,0x00,0x00,0x00,0x00,0x00,0x01,
      0x0c,0x00,0x00,0x00,0x14,0x00,0x00,0x00,0x20,0x00,0x00,0x00,
      0x04,0x00,0x00,0x00,'t','e','x','t',0x00,0x00,0x00,0x00,
      0x08,0x00,0x00,0x00,'U','s','e','r','n','a','m','e',0x00,0x00,0x00,0x00,
      0x08,0x00,0x00,0x00,'u','s','e','r','n','a','m','e',0x00,0x00,0x00,0x00};
  RevaultBuffer form = vault_directory_define_form(
      vault, "login", 5, "Login", 5, "Login form", 10, form_fields,
      sizeof(form_fields));
  Bytes vault_form_payload = framed_payload(form);
  CHECK(vault_form_payload.len > 0, "define vault form");
  Bytes vault_form_type = flatbuffer_bytes(vault_form_payload, 1);
  CHECK(vault_form_type.ptr != NULL, "vault form type id");
  char vault_form_type_id[128] = {0};
  CHECK(vault_form_type.len < sizeof(vault_form_type_id), "vault form type id length");
  memcpy(vault_form_type_id, vault_form_type.ptr, vault_form_type.len);
  buffer_free(form);
  PASS(vault_directory_define_form, 1);
  expect_framed_nonempty(vault_directory_resolve_form(vault, "login", 5),
                         "resolve vault form");
  expect_framed_nonempty(vault_directory_list_forms(vault), "vault form list");
  expect_framed_nonempty(vault_directory_list_form_revisions(vault, vault_form_type_id, vault_form_type.len), "vault form revisions");
  PASS(vault_directory_resolve_form, 1);
  PASS(vault_directory_list_forms, 1);
  PASS(vault_directory_list_form_revisions, 1);
  CHECK(vault_directory_seed_forms(vault) > 0, "seed default forms");
  PASS(vault_directory_seed_forms, 1);
  expect_framed_nonempty(vault_directory_list_form_aliases(vault),
                         "vault form alias list");
  PASS(vault_directory_list_form_aliases, 1);

  CHECK(vault_directory_forget_access_slot_label(vault, id, sizeof(id), 7),
        "forget access label");
  CHECK(vault_directory_forget_lockbox(vault, "/tmp/example.lbox", 17),
        "forget lockbox");
  CHECK(vault_directory_delete_contact(vault, "bob", 3), "delete contact");
  PASS(vault_directory_forget_access_slot_label, 1);
  PASS(vault_directory_forget_lockbox, 1);
  PASS(vault_directory_delete_contact, 1);

  CHECK(vault_directory_restore_private_key(
            vault, "restored", 8, generation_zero, owner_generation, false),
        "restore private key");
  CHECK(vault_directory_private_key_exists(vault, "restored", 8),
        "restored private key exists");
  PASS(vault_directory_restore_private_key, 2);
  CHECK(vault_directory_delete_private_key(vault, "restored", 8),
        "delete restored private key");
  CHECK(!vault_directory_private_key_exists(vault, "restored", 8),
        "deleted private key absent");
  PASS(vault_directory_delete_private_key, 2);

  key_contact_free(loaded_profile);
  key_contact_free(generation_zero);
  key_contact_free(profile_key);
  key_contact_public_free(loaded_contact);
  key_contact_public_free(contact_public);
  key_contact_free(contact_key);
  key_signing_free(owner_key);
  key_signing_free(owner_generation);
  key_signing_public_free(loaded_contact_signing);
  key_signing_public_free(contact_signing_public);
  key_signing_free(contact_signing);
  buffer_free(contact_bytes);
  buffer_free(contact_signing_bytes);

  vault_directory_free(vault);
  PASS(vault_directory_free, 1);
  void *read_only = vault_read_only_open(root, strlen(root), password, sizeof(password) - 1);
  CHECK(read_only != NULL, "open read-only vault");
  expect_framed_nonempty(vault_read_only_list_profile_names(read_only), "read-only profiles");
  RevaultBuffer read_only_contacts = vault_read_only_list_contact_names(read_only);
  CHECK(read_only_contacts.ptr != NULL, "read-only contacts"); buffer_free(read_only_contacts);
  expect_framed_nonempty(vault_read_only_list_form_aliases(read_only), "read-only forms");
  RevaultBuffer read_only_known = vault_read_only_list_known_lockboxes(read_only);
  CHECK(read_only_known.ptr != NULL, "read-only known lockboxes"); buffer_free(read_only_known);
  vault_read_only_free(read_only);
  PASS(vault_read_only_open, 1); PASS(vault_read_only_list_profile_names, 1);
  PASS(vault_read_only_list_contact_names, 1); PASS(vault_read_only_list_form_aliases, 1);
  PASS(vault_read_only_list_known_lockboxes, 1); PASS(vault_read_only_free, 1);
  CHECK(vault_directory_change_password(root, strlen(root), password,
                                        sizeof(password) - 1, new_password,
                                        sizeof(new_password) - 1),
        "change vault password");
  PASS(vault_directory_change_password, 1);
  void *reopened = vault_directory_open(root, strlen(root), new_password,
                                        sizeof(new_password) - 1);
  CHECK(reopened != NULL, "reopen changed-password vault");
  PASS(vault_directory_open, 1);
  printf("ARTIFACT\t%s\tvault-opened\t%s\n", language_name(), root);
  vault_directory_free(reopened);
  void *opened_or_created = vault_directory_open_or_create(
      root, strlen(root), new_password, sizeof(new_password) - 1);
  CHECK(opened_or_created != NULL, "open or create existing vault");
  PASS(vault_directory_open_or_create, 1);
  vault_directory_free(opened_or_created);
}

static void default_vault_lifecycle(void) {
  char root[512];
  make_temp_dir(root, sizeof(root), "revault-c-default-vault");
  CHECK(set_environment("LOCKBOX_VAULT_DIR", root) == 0,
        "set default vault directory");
  const uint8_t password[] = "default password";
  const uint8_t changed[] = "changed default password";

  void *vault =
      vault_directory_replace_default(password, sizeof(password) - 1);
  CHECK(vault != NULL, "replace default vault");
  PASS(vault_directory_replace_default, 1);
  vault_directory_free(vault);

  RevaultBuffer directory = vault_default_directory();
  CHECK(directory.ptr != NULL && directory.len == strlen(root) &&
            memcmp(directory.ptr, root, directory.len) == 0,
        "default directory");
  buffer_free(directory);
  PASS(vault_default_directory, 3);
  RevaultBuffer path = vault_default_path();
  CHECK(path.ptr != NULL && path.len > strlen(root), "default vault path");
  buffer_free(path);
  PASS(vault_default_path, 2);

  void *opened =
      vault_directory_open_or_create_default(password, sizeof(password) - 1);
  CHECK(opened != NULL, "open default vault");
  PASS(vault_directory_open_or_create_default, 1);
  vault_directory_free(opened);

  void *read_only = vault_read_only_open_default(password, sizeof(password) - 1);
  CHECK(read_only != NULL, "open default read-only vault");
  vault_read_only_free(read_only);
  PASS(vault_read_only_open_default, 1);

  CHECK(vault_directory_change_default_password(
            password, sizeof(password) - 1, changed, sizeof(changed) - 1),
        "change default password");
  PASS(vault_directory_change_default_password, 1);

  char backup_path[512];
  make_temp_file(backup_path, sizeof(backup_path), "revault-c-default-backup");
  unlink(backup_path);
  expect_framed_nonempty(
      vault_backup_default(backup_path, strlen(backup_path), false),
      "default vault backup manifest");
  PASS(vault_backup_default, 1);
  expect_framed_nonempty(
      vault_restore_default(backup_path, strlen(backup_path), true),
      "default vault restore manifest");
  PASS(vault_restore_default, 1);
}

static void agent_and_local_vault(void) {
  char agent_dir[512];
  make_temp_dir(agent_dir, sizeof(agent_dir), "revault-c-agent");
  CHECK(set_environment("LOCKBOX_SESSION_AGENT_DIR", agent_dir) == 0,
        "set agent socket directory");
  char agent_vault_dir[512];
  make_temp_dir(agent_vault_dir, sizeof(agent_vault_dir),
                "revault-c-agent-vault");
  CHECK(set_environment("LOCKBOX_VAULT_DIR", agent_vault_dir) == 0,
        "set agent vault directory");
  const uint8_t agent_vault_password[] = "agent vault password";
  CHECK(set_environment("LOCKBOX_VAULT_PASSWORD", "agent vault password") == 0,
        "set agent vault password");
  void *agent_vault = vault_directory_replace_default(
      agent_vault_password, sizeof(agent_vault_password) - 1);
  CHECK(agent_vault != NULL, "initialize agent vault");
  void *default_profile = key_contact_generate();
  CHECK(default_profile != NULL, "generate default profile");
  CHECK(vault_directory_store_private_key(agent_vault, "default", 7,
                                          default_profile),
        "store default profile");
  key_contact_free(default_profile);
  vault_directory_free(agent_vault);
  CHECK(vault_forget_all(), "clear agent before start");
  PASS(vault_forget_all, 1);
  AgentProcess child = spawn_agent();

  bool running = false;
  for (unsigned attempt = 0; attempt < 200; ++attempt) {
    if (vault_is_running()) {
      running = true;
      break;
    }
    pause_for_agent();
  }
  CHECK(running, "agent started");
  PASS(vault_agent_serve, 1);
  PASS(vault_is_running, 1);
  CHECK(vault_agent_start(), "ensure agent started");
  PASS(vault_agent_start, 1);
  CHECK(vault_agent_verify_transport(), "agent transport security");
  PASS(vault_agent_verify_transport, 1);

  uint8_t id[16];
  uint8_t key[32];
  for (size_t i = 0; i < sizeof(id); ++i) id[i] = (uint8_t)(0xc0 + i);
  for (size_t i = 0; i < sizeof(key); ++i) key[i] = (uint8_t)(0x20 + i);
  CHECK(vault_agent_put(id, sizeof(id), key, sizeof(key)), "agent put key");
  expect_raw(vault_agent_get(id, sizeof(id)), key, sizeof(key));
  expect_framed_nonempty(vault_agent_list(), "agent key list");
  PASS(vault_agent_put, 1);
  PASS(vault_agent_get, 3);
  PASS(vault_agent_list, 1);

  CHECK(vault_agent_put_vault_unlock_key("vault-id", 8, key, sizeof(key), 120),
        "agent put vault key");
  expect_raw(vault_agent_get_vault_unlock_key("vault-id", 8), key, sizeof(key));
  PASS(vault_agent_put_vault_unlock_key, 1);
  PASS(vault_agent_get_vault_unlock_key, 3);

  void *owner = key_signing_generate();
  CHECK(owner != NULL, "agent owner fixture");
  CHECK(vault_agent_put_owner_signing_key("vault-id", 8, "alice", 5, owner, 120),
        "agent put owner key");
  void *loaded_owner =
      vault_agent_get_owner_signing_key("vault-id", 8, "alice", 5);
  CHECK(loaded_owner != NULL, "agent get owner key");
  PASS(vault_agent_put_owner_signing_key, 1);
  PASS(vault_agent_get_owner_signing_key, 1);
  key_signing_free(loaded_owner);

  void *activity = vault_agent_begin_activity("open", 4);
  CHECK(activity != NULL, "begin agent activity");
  PASS(vault_agent_begin_activity, 1);
  vault_agent_end_activity(activity);
  PASS(vault_agent_end_activity, 1);
  expect_framed_nonempty(vault_agent_sleep_support(), "agent sleep support");
  PASS(vault_agent_sleep_support, 1);

  RevaultBuffer log_path = vault_agent_log_path();
  CHECK(log_path.ptr != NULL && log_path.len > 0, "agent log path");
  buffer_free(log_path);
  RevaultBuffer log_destination = vault_agent_log_destination();
  CHECK(log_destination.ptr != NULL && log_destination.len > 0,
        "agent log destination");
  buffer_free(log_destination);
  PASS(vault_agent_log_path, 2);
  PASS(vault_agent_log_destination, 2);

  void *local = vault_local();
  CHECK(local != NULL, "local vault handle");
  PASS(vault_local, 1);
  char local_root[512];
  make_temp_dir(local_root, sizeof(local_root), "revault-c-local");
  char password_path[1024];
  char content_path[1024];
  char contact_path[1024];
  snprintf(password_path, sizeof(password_path), "%s/password.lbox", local_root);
  snprintf(content_path, sizeof(content_path), "%s/content.lbox", local_root);
  snprintf(contact_path, sizeof(contact_path), "%s/contact.lbox", local_root);
  const uint8_t password[] = "local password";
  const uint8_t data[] = "local vault data";

  void *password_box = vault_create_lockbox_password(
      local, password_path, strlen(password_path), password,
      sizeof(password) - 1);
  CHECK(password_box != NULL, "local password lockbox create");
  CHECK(lockbox_add_file(password_box, "/data.txt", 9, data,
                         sizeof(data) - 1, false),
        "local password data");
  CHECK(lockbox_commit(password_box), "local password commit");
  lockbox_free(password_box);
  PASS(vault_create_lockbox_password, 3);
  CHECK(vault_cache_lockbox_password(local, password_path,
                                     strlen(password_path), password,
                                     sizeof(password) - 1, 120),
        "cache local lockbox password");
  PASS(vault_cache_lockbox_password, 1);
  void *opened_password = vault_open_lockbox_password(
      local, password_path, strlen(password_path), password,
      sizeof(password) - 1);
  CHECK(opened_password != NULL, "local password lockbox open");
  expect_raw(lockbox_get_file(opened_password, "/data.txt", 9), data,
             sizeof(data) - 1);
  lockbox_free(opened_password);
  PASS(vault_open_lockbox_password, 3);
  CHECK(vault_close_lockbox(local, password_path, strlen(password_path)),
        "close local lockbox");
  PASS(vault_close_lockbox, 1);

  void *content_box = vault_create_lockbox_content_key(
      local, content_path, strlen(content_path), key, sizeof(key), owner);
  CHECK(content_box != NULL, "local content-key lockbox create");
  CHECK(lockbox_add_file(content_box, "/data.txt", 9, data,
                         sizeof(data) - 1, false),
        "local content-key data");
  CHECK(lockbox_commit(content_box), "local content-key commit");
  lockbox_free(content_box);
  PASS(vault_create_lockbox_content_key, 3);
  void *opened_content = vault_open_lockbox_content_key(
      local, content_path, strlen(content_path), key, sizeof(key), owner);
  CHECK(opened_content != NULL, "local content-key lockbox open");
  expect_raw(lockbox_get_file(opened_content, "/data.txt", 9), data,
             sizeof(data) - 1);
  lockbox_free(opened_content);
  PASS(vault_open_lockbox_content_key, 3);

  void *contact = key_contact_generate();
  RevaultBuffer public_bytes = key_contact_public(contact);
  void *contact_public =
      key_contact_public_from_bytes(public_bytes.ptr, public_bytes.len);
  CHECK(contact_public != NULL, "local contact fixture");
  void *contact_box = vault_create_lockbox_contact(
      local, contact_path, strlen(contact_path), contact_public, "recipient", 9,
      owner);
  CHECK(contact_box != NULL, "local contact lockbox create");
  CHECK(lockbox_add_file(contact_box, "/data.txt", 9, data,
                         sizeof(data) - 1, false),
        "local contact data");
  CHECK(lockbox_commit(contact_box), "local contact commit");
  lockbox_free(contact_box);
  PASS(vault_create_lockbox_contact, 3);

  CHECK(vault_close_all(local), "close all local lockboxes");
  PASS(vault_close_all, 1);
  vault_free(local);
  PASS(vault_free, 1);

  CHECK(vault_agent_forget_owner_signing_key("vault-id", 8, "alice", 5),
        "forget agent owner key");
  CHECK(vault_agent_forget_vault_unlock_key("vault-id", 8), "forget agent vault key");
  CHECK(vault_agent_forget(id, sizeof(id)), "forget agent key");
  PASS(vault_agent_forget_owner_signing_key, 1);
  PASS(vault_agent_forget_vault_unlock_key, 1);
  PASS(vault_agent_forget, 1);

  key_contact_public_free(contact_public);
  key_contact_free(contact);
  buffer_free(public_bytes);
  key_signing_free(owner);
  CHECK(vault_agent_stop(), "stop agent");
  PASS(vault_agent_stop, 1);
  wait_for_agent(child);
}

static void platform_secret_store(void) {
  char root[512];
  make_temp_dir(root, sizeof(root), "revault-c-platform-vault");
  CHECK(set_environment("LOCKBOX_VAULT_DIR", root) == 0,
        "set platform vault directory");
  const uint8_t password[] = "platform vault password";

  RevaultBuffer status = vault_platform_status();
  CHECK(status.ptr != NULL && status.len >= 12, "platform status frame");
  (void)framed_payload(status);
  buffer_free(status);
  PASS(vault_platform_status, 2);
  CHECK(vault_platform_set_scope("vault", 5), "set platform scope");
  PASS(vault_platform_set_scope, 1);
  CHECK(vault_platform_disable(), "disable platform store");
  CHECK(vault_platform_disabled(), "platform store disabled marker");
  PASS(vault_platform_disable, 1);
  PASS(vault_platform_disabled, 1);
  CHECK(vault_platform_enable(), "enable platform store");
  CHECK(!vault_platform_disabled(), "platform store enabled marker");
  PASS(vault_platform_enable, 1);
  CHECK(vault_platform_put_password(password, sizeof(password) - 1),
        "put platform vault password");
  expect_raw(vault_platform_get_password(), password, sizeof(password) - 1);
  PASS(vault_platform_put_password, 1);
  PASS(vault_platform_get_password, 3);
  CHECK(vault_platform_forget_password(), "forget platform password");
  RevaultBuffer missing = vault_platform_get_password();
  CHECK(missing.ptr == NULL && missing.len == 0,
        "platform password absent after forget");
  PASS(vault_platform_forget_password, 1);
}

static void write_file(const char *path, const uint8_t *bytes, size_t len) {
  FILE *file = fopen(path, "wb");
  CHECK(file != NULL, "open fixture output");
  CHECK(fwrite(bytes, 1, len, file) == len, "write fixture output");
  CHECK(fclose(file) == 0, "close fixture output");
}

static void archive_advanced(void) {
  uint8_t key[32];
  memset(key, 0x31, sizeof(key));
  const uint8_t password[] = "archive password";
  const uint8_t payload[] = "advanced archive payload";
  static const uint8_t form_fields[] = {
      0x0c,0x00,0x00,0x00,0x00,0x00,0x06,0x00,0x08,0x00,0x04,0x00,
      0x06,0x00,0x00,0x00,0x04,0x00,0x00,0x00,0x02,0x00,0x00,0x00,
      0x54,0x00,0x00,0x00,0x04,0x00,0x00,0x00,0xc0,0xff,0xff,0xff,
      0x00,0x00,0x00,0x01,0x0c,0x00,0x00,0x00,0x14,0x00,0x00,0x00,
      0x20,0x00,0x00,0x00,0x06,0x00,0x00,0x00,'s','e','c','r','e','t',
      0x00,0x00,0x08,0x00,0x00,0x00,'P','a','s','s','w','o','r','d',
      0x00,0x00,0x00,0x00,0x08,0x00,0x00,0x00,'p','a','s','s','w','o','r','d',
      0x00,0x00,0x00,0x00,0x0c,0x00,0x14,0x00,0x10,0x00,0x0c,0x00,
      0x08,0x00,0x07,0x00,0x0c,0x00,0x00,0x00,0x00,0x00,0x00,0x01,
      0x0c,0x00,0x00,0x00,0x14,0x00,0x00,0x00,0x20,0x00,0x00,0x00,
      0x04,0x00,0x00,0x00,'t','e','x','t',0x00,0x00,0x00,0x00,
      0x08,0x00,0x00,0x00,'U','s','e','r','n','a','m','e',0x00,0x00,0x00,0x00,
      0x08,0x00,0x00,0x00,'u','s','e','r','n','a','m','e',0x00,0x00,0x00,0x00};

  void *box = lockbox_create_with_options(
      key, sizeof(key), "bytes", 5, 1024 * 1024, "bulk-import", 11,
      "single", 6, 1);
  CHECK(box != NULL, "create archive with options");
  PASS(lockbox_create_with_options, 1);
  CHECK(lockbox_add_file(box, "/advanced.txt", 13, payload,
                         sizeof(payload) - 1, false),
        "advanced file");

  RevaultBuffer listed = lockbox_list_with_options(
      box, "/", 1, "*.txt", 5, true, true, false, false, 10);
  CHECK(flatbuffer_bytes(framed_payload(listed), 1).ptr != NULL,
        "filtered listing");
  buffer_free(listed);
  PASS(lockbox_list_with_options, 2);

  RevaultBuffer definition = lockbox_define_form(
      box, "account", 7, "Account", 7, "Account form", 12, form_fields,
      sizeof(form_fields));
  Bytes definition_payload = framed_payload(definition);
  Bytes type_id_value = flatbuffer_bytes(definition_payload, 1);
  CHECK(type_id_value.ptr != NULL && type_id_value.len < 128,
        "form type identifier");
  char type_id[128] = {0};
  memcpy(type_id, type_id_value.ptr, type_id_value.len);
  size_t type_id_len = type_id_value.len;
  buffer_free(definition);
  PASS(lockbox_define_form, 2);

  expect_framed_nonempty(lockbox_list_form_definitions(box),
                         "form definition list");
  expect_framed_nonempty(lockbox_resolve_form(box, "account", 7),
                         "resolve archive form");
  expect_framed_nonempty(
      lockbox_list_form_revisions(box, type_id, type_id_len),
      "form revision list");
  PASS(lockbox_list_form_definitions, 1);
  PASS(lockbox_resolve_form, 1);
  PASS(lockbox_list_form_revisions, 1);

  expect_framed_nonempty(lockbox_create_form_record(
                             box, "/account.form", 13, "account", 7,
                             "Primary account", 15),
                         "create form record");
  PASS(lockbox_create_form_record, 1);
  CHECK(lockbox_set_form_field(box, "/account.form", 13, "username", 8,
                               "alice", 5),
        "set form field");
  PASS(lockbox_set_form_field, 1);
  CHECK(lockbox_set_secret_form_field(box, "/account.form", 13, "password", 8,
                                      (const uint8_t *)"hidden", 6),
        "set secret form field");
  PASS(lockbox_set_secret_form_field, 1);
  void *form_secret_handle = NULL;
  CHECK(lockbox_get_secret_form_field(box, "/account.form", 13, "password", 8,
                                      &form_secret_handle) &&
            form_secret_handle != NULL,
        "get secret form field");
  size_t form_secret_length = 0;
  CHECK(secret_len(form_secret_handle, &form_secret_length) &&
            form_secret_length == 6,
        "secret form field length");
  uint8_t form_secret_bytes[6] = {0};
  CHECK(secret_copy(form_secret_handle, form_secret_bytes,
                    sizeof(form_secret_bytes)) &&
            memcmp(form_secret_bytes, "hidden", sizeof(form_secret_bytes)) == 0,
        "secret form field value");
  memset(form_secret_bytes, 0, sizeof(form_secret_bytes));
  secret_free(form_secret_handle);
  PASS(lockbox_get_secret_form_field, 1);
  static const uint8_t move_form[] = {
      0x0c,0x00,0x00,0x00,0x00,0x00,0x06,0x00,0x08,0x00,0x04,0x00,
      0x06,0x00,0x00,0x00,0x04,0x00,0x00,0x00,0x01,0x00,0x00,0x00,
      0x0c,0x00,0x00,0x00,0x08,0x00,0x0c,0x00,0x08,0x00,0x04,0x00,
      0x08,0x00,0x00,0x00,0x08,0x00,0x00,0x00,0x14,0x00,0x00,0x00,
      0x0b,0x00,0x00,0x00,'/','m','o','v','e','d','.','f','o','r','m',0x00,
      0x0d,0x00,0x00,0x00,'/','a','c','c','o','u','n','t','.','f','o','r','m',0x00,0x00,0x00};
  static const uint8_t move_form_back[] = {
      0x0c,0x00,0x00,0x00,0x00,0x00,0x06,0x00,0x08,0x00,0x04,0x00,
      0x06,0x00,0x00,0x00,0x04,0x00,0x00,0x00,0x01,0x00,0x00,0x00,
      0x0c,0x00,0x00,0x00,0x08,0x00,0x0c,0x00,0x08,0x00,0x04,0x00,
      0x08,0x00,0x00,0x00,0x08,0x00,0x00,0x00,0x18,0x00,0x00,0x00,
      0x0d,0x00,0x00,0x00,'/','a','c','c','o','u','n','t','.','f','o','r','m',0x00,0x00,0x00,
      0x0b,0x00,0x00,0x00,'/','m','o','v','e','d','.','f','o','r','m',0x00};
  CHECK(lockbox_move_form_records(box, move_form, sizeof(move_form)), "move form record");
  CHECK(lockbox_move_form_records(box, move_form_back, sizeof(move_form_back)), "move form record back");
  PASS(lockbox_move_form_records, 2);
  expect_framed_nonempty(lockbox_get_form_record(box, "/account.form", 13),
                         "get form record");
  expect_framed_nonempty(lockbox_get_form_field(
                             box, "/account.form", 13, "username", 8),
                         "get form field");
  expect_framed_nonempty(lockbox_list_form_records(box), "form record list");
  PASS(lockbox_get_form_record, 1);
  PASS(lockbox_get_form_field, 1);
  PASS(lockbox_list_form_records, 1);

  void *signing = key_signing_generate();
  CHECK(signing != NULL, "advanced signing key");
  CHECK(lockbox_set_owner_signing_key(box, signing), "set owner signing key");
  PASS(lockbox_set_owner_signing_key, 1);

  uint64_t password_slot =
      lockbox_add_password(box, password, sizeof(password) - 1);
  CHECK(password_slot != UINT64_MAX, "add password slot");
  PASS(lockbox_add_password, 1);
  void *contact_pair = key_contact_generate();
  RevaultBuffer contact_public_bytes = key_contact_public(contact_pair);
  void *contact_public = key_contact_public_from_bytes(
      contact_public_bytes.ptr, contact_public_bytes.len);
  CHECK(contact_public != NULL, "advanced contact public key");
  uint64_t contact_slot =
      lockbox_add_contact(box, contact_public, "recipient", 9);
  CHECK(contact_slot != UINT64_MAX, "add contact slot");
  PASS(lockbox_add_contact, 1);
  expect_framed_nonempty(lockbox_list_key_slots(box), "key slot list");
  PASS(lockbox_list_key_slots, 1);
  CHECK(lockbox_delete_key(box, password_slot), "delete password slot");
  PASS(lockbox_delete_key, 1);

  CHECK(lockbox_commit(box), "advanced archive commit");
  RevaultBuffer owner = lockbox_owner_inspection(box);
  Bytes owner_payload = framed_payload(owner);
  bool owner_found = false;
  CHECK(flatbuffer_scalar(owner_payload, 1, &owner_found) == 1 && owner_found,
        "owner-signed archive inspection");
  buffer_free(owner);
  PASS(lockbox_owner_inspection, 2);
  expect_framed_nonempty(lockbox_cache_stats(box), "cache statistics");
  expect_framed_nonempty(lockbox_import_stats(box), "import statistics");
  CHECK(lockbox_reset_import_stats(box), "reset import statistics");
  expect_framed_nonempty(lockbox_page_inspection(box), "page inspection");
  expect_framed_nonempty(lockbox_recovery_report(box), "recovery report");
  RevaultBuffer rendered = lockbox_recovery_report_render(box, true, 100);
  CHECK(rendered.ptr != NULL && rendered.len > 0, "rendered recovery report");
  buffer_free(rendered);
  expect_framed_nonempty(lockbox_stream_content(box, false),
                         "logical content stream");
  PASS(lockbox_cache_stats, 1);
  PASS(lockbox_import_stats, 1);
  PASS(lockbox_reset_import_stats, 1);
  PASS(lockbox_page_inspection, 1);
  PASS(lockbox_recovery_report, 1);
  PASS(lockbox_recovery_report_render, 2);
  PASS(lockbox_stream_content, 1);

  RevaultBuffer id = lockbox_id(box);
  CHECK(id.ptr != NULL && id.len == 16, "lockbox identifier");
  buffer_free(id);
  PASS(lockbox_id, 2);

  RevaultBuffer archive = lockbox_to_bytes(box);
  CHECK(archive.ptr != NULL && archive.len > 0, "advanced archive bytes");
  char temp[512];
  make_temp_file(temp, sizeof(temp), "revault-c-archive");
  write_file(temp, archive.ptr, archive.len);

  expect_framed_nonempty(lockbox_inspect_file(temp, strlen(temp)),
                         "archive file inspection");
  expect_framed_nonempty(
      lockbox_recovery_scan_path(temp, strlen(temp), key, sizeof(key)),
      "path recovery scan");
  expect_framed_nonempty(
      lockbox_recovery_scan(archive.ptr, archive.len, key, sizeof(key)),
      "memory recovery scan");
  PASS(lockbox_inspect_file, 1);
  PASS(lockbox_recovery_scan_path, 1);
  PASS(lockbox_recovery_scan, 1);

  void *salvaged = lockbox_recovery_salvage(
      archive.ptr, archive.len, key, sizeof(key), signing);
  CHECK(salvaged != NULL, "recovery salvage");
  expect_raw(lockbox_get_file(salvaged, "/advanced.txt", 13), payload,
             sizeof(payload) - 1);
  PASS(lockbox_recovery_salvage, 2);
  lockbox_free(salvaged);

  void *opened_options = lockbox_open_with_options(
      archive.ptr, archive.len, key, sizeof(key), "disabled", 8, 0,
      "read-mostly", 11, "single", 6, 1);
  CHECK(opened_options != NULL, "open archive with options");
  PASS(lockbox_open_with_options, 1);
  lockbox_free(opened_options);

  void *password_box =
      lockbox_create_password(password, sizeof(password) - 1);
  CHECK(password_box != NULL, "create password archive");
  CHECK(lockbox_add_file(password_box, "/password.txt", 13, payload,
                         sizeof(payload) - 1, false),
        "password archive file");
  CHECK(lockbox_commit(password_box), "password archive commit");
  RevaultBuffer password_archive = lockbox_to_bytes(password_box);
  void *opened_password = lockbox_open_password(
      password_archive.ptr, password_archive.len, password,
      sizeof(password) - 1);
  CHECK(opened_password != NULL, "open password archive");
  expect_raw(lockbox_get_file(opened_password, "/password.txt", 13), payload,
             sizeof(payload) - 1);
  PASS(lockbox_create_password, 1);
  PASS(lockbox_open_password, 2);
  lockbox_free(opened_password);
  lockbox_free(password_box);
  buffer_free(password_archive);

  void *contact_box = lockbox_create_contact(contact_public);
  CHECK(contact_box != NULL, "create contact archive");
  CHECK(lockbox_add_file(contact_box, "/contact.txt", 12, payload,
                         sizeof(payload) - 1, false),
        "contact archive file");
  CHECK(lockbox_commit(contact_box), "contact archive commit");
  RevaultBuffer contact_archive = lockbox_to_bytes(contact_box);
  void *opened_contact = lockbox_open_contact(
      contact_archive.ptr, contact_archive.len, contact_pair);
  CHECK(opened_contact != NULL, "open contact archive");
  expect_raw(lockbox_get_file(opened_contact, "/contact.txt", 12), payload,
             sizeof(payload) - 1);
  PASS(lockbox_create_contact, 1);
  PASS(lockbox_open_contact, 2);
  lockbox_free(opened_contact);
  lockbox_free(contact_box);
  buffer_free(contact_archive);

  void *signed_box =
      lockbox_create_with_signing_key(key, sizeof(key), signing);
  CHECK(signed_box != NULL, "create signed archive");
  CHECK(lockbox_commit(signed_box), "commit signed archive");
  RevaultBuffer signed_owner = lockbox_owner_inspection(signed_box);
  Bytes signed_owner_payload = framed_payload(signed_owner);
  bool signed_found = false;
  CHECK(flatbuffer_scalar(signed_owner_payload, 1, &signed_found) == 1 &&
            signed_found,
        "signed owner inspection");
  buffer_free(signed_owner);
  PASS(lockbox_create_with_signing_key, 2);
  lockbox_free(signed_box);

  char extract_root[512];
  make_temp_dir(extract_root, sizeof(extract_root), "revault-c-extract");
  char extracted_file[1024];
  snprintf(extracted_file, sizeof(extracted_file), "%s/file.txt", extract_root);
  CHECK(lockbox_extract_file(box, "/advanced.txt", 13, extracted_file,
                             strlen(extracted_file), false),
        "extract one file");
  CHECK(access(extracted_file, F_OK) == 0, "extracted file exists");
  PASS(lockbox_extract_file, 2);
  char extracted_tree[1024];
  snprintf(extracted_tree, sizeof(extracted_tree), "%s/tree", extract_root);
  CHECK(lockbox_extract_directory(box, extracted_tree, strlen(extracted_tree),
                                  1024 * 1024, 4 * 1024 * 1024, 100, false,
                                  true, false),
        "extract archive directory");
  CHECK(access(extracted_tree, F_OK) == 0, "extracted tree exists");
  PASS(lockbox_extract_directory, 2);

  CHECK(lockbox_delete_form_record(box, "/account.form", 13),
        "delete form record");
  PASS(lockbox_delete_form_record, 1);

  key_contact_public_free(contact_public);
  key_contact_free(contact_pair);
  key_signing_free(signing);
  buffer_free(contact_public_bytes);
  buffer_free(archive);
  lockbox_free(box);
  unlink(temp);
}

int main(int argc, char **argv) {
  CHECK(api_abi_version() == 3, "revault-api ABI version");
  executable_path = argv[0];
  if (argc == 2 &&
      (strcmp(argv[1], "--serve-agent") == 0 ||
       strcmp(argv[1], "__agent") == 0)) {
    if (vault_agent_serve()) return 0;
    fprintf(stderr, "agent server failed: %s\n", buffer_last_error());
    return 1;
  }
  if (argc == 2 && strcmp(argv[1], "--agent") == 0) {
    trace_phase("agent and local vault");
    agent_and_local_vault();
    return 0;
  }
  if (argc == 2 && strcmp(argv[1], "--platform") == 0) {
    trace_phase("platform secret store");
    platform_secret_store();
    return 0;
  }
  if (argc == 3 && strcmp(argv[1], "--interop") == 0) {
    interop_open(argv[2]);
    return 0;
  }
  if (argc == 2 && strcmp(argv[1], "--core") == 0) {
    archive_lifecycle();
    key_lifecycle();
    archive_advanced();
    vault_lifecycle();
    default_vault_lifecycle();
    return 0;
  }
  if (argc == 2 && strcmp(argv[1], "--last-error") == 0) {
    CHECK(buffer_last_error() != NULL, "last-error pointer");
    PASS(buffer_last_error, 1);
    return 0;
  }
  trace_phase("archive lifecycle");
  archive_lifecycle();
  trace_phase("key lifecycle");
  key_lifecycle();
  trace_phase("advanced archive");
  archive_advanced();
  trace_phase("vault lifecycle");
  vault_lifecycle();
  trace_phase("default vault lifecycle");
  default_vault_lifecycle();
  trace_phase("agent and local vault");
  agent_and_local_vault();
  trace_phase("platform secret store");
  platform_secret_store();
  const char *error = buffer_last_error();
  CHECK(error != NULL, "last-error pointer");
  PASS(buffer_last_error, 1);
  return 0;
}
