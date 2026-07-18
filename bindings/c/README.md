# reVault C API

reVault is an encrypted archive and local-vault library for storing files,
metadata, credentials, keys, and typed form records. The stable C ABI is the
foundation for the native language packages. See the
[reVault documentation](https://github.com/onepub-dev/reVault/tree/main/docs)
for archive, vault, key-management, and security concepts.

Use `revault_api.h` with the matching `revault_api` shared or static library
from the GitHub release SDK, Debian/RPM package, Homebrew, vcpkg, or Conan.
`api_abi_version()` must return `2`.

```c
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <revault_api.h>

uint8_t key[32] = {0}; /* load a real content key securely */
void *box = lockbox_create(key, sizeof key);
lockbox_add_file(box, "/hello.txt", 10,
                 (const uint8_t *)"hello\n", 6, false);
lockbox_set_variable(box, "owner", 5, "alice", 5);
lockbox_set_secret_variable(box, "token", 5,
                            (const uint8_t *)"secret", 6);

void *secret = NULL;
if (lockbox_get_secret_variable(box, "token", 5, &secret) && secret) {
  size_t length = 0;
  secret_len(secret, &length);
  uint8_t *bytes = malloc(length);
  secret_copy(secret, bytes, length);
  /* consume bytes without retaining them */
  memset(bytes, 0, length);
  free(bytes);
  secret_free(secret);
}
lockbox_commit(box);
lockbox_free(box);
```

Every returned `RevaultBuffer` must be released with `buffer_free`; every
owned handle has a corresponding `*_free`. Secret handles are opaque and must
be copied only for the duration of the operation, cleared, then freed.
