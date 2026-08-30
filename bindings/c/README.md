# reVault C API

reVault is a fast, local toolkit for creating secure portable archives called
Lockboxes. Each Lockbox is encrypted, compressed, and signed. It can store
files and directory trees, variables such as API keys, and forms such as login
details.

Lockboxes are easy to copy, share, and back up, and they do not require a
hosted service. The engine is designed for speed and effective compression.
Applications can read, write, and seek within stored files without extracting
the archive, and recover data from partial corruption. reVault provides a
command line tool for everyday work and APIs for application code.

Read the [reVault manual](https://docs.revault.onepub.dev/) for the quick start,
core concepts, and security model.

Your Vault holds your profile and contacts. The CLI protects a new Lockbox for
your profile by default, and you can grant access to contacts using their
public keys. Use password access when you do not have a recipient's contact
(public key) details.

The stable C ABI provides Lockbox, Vault, key, and session agent operations for
applications and other language packages.

## Native library distribution

Use `revault_api.h` with the matching `revault_api` shared or static library
from the GitHub release SDK, Debian/RPM package, Homebrew, vcpkg, or Conan.
The build system or package manager supplies the include and linker paths. An
application linked to the shared library must also make it available through
the operating system's runtime library search path. Static builds contain the
engine in the application and do not load it at runtime. In either case,
`api_abi_version()` must return `3`.

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

The [complete method example index](https://github.com/onepub-dev/reVault/blob/main/bindings/API_EXAMPLES.md)
and [executable C conformance example](https://github.com/onepub-dev/reVault/blob/main/bindings/e2e/c/conformance.c)
are maintained in the source repository. The conformance example runs against
the installed SDK and releases every returned buffer and handle.

Every returned `RevaultBuffer` must be released with `buffer_free`; every
owned handle has a corresponding `*_free`. Secret handles are opaque and must
be copied only for the duration of the operation, cleared, then freed.

## Core API concepts

The runtime, Vault, Lockbox, and agent are separate handles. Loading the ABI
does not open persistent state. Freeing an agent entry does not delete a
Lockbox or Vault credential.

## API documentation and support

The installed `revault_api.h` is the signature authority for its matching
library; `api_abi_version()` must return `3`. Do not combine headers and
libraries from different releases. The linked method index and executable
conformance program provide ownership and error examples for every common
operation.

## Vaults, Lockboxes, and credentials

A `Vault` is the persistent encrypted local store for profiles, private keys,
contacts, signing keys, and remembered Lockbox access. A `Lockbox` is a
portable encrypted `.lbox` archive. Opening either resource never creates it;
use the distinct create and open or create functions when creation is intended and
the replace functions only for deliberate destructive replacement.

A vault passphrase, Lockbox password, and 32-byte Lockbox content key are
different secrets. Pass the exact byte length for each value and clear buffers
owned by the caller promptly. A profile signing key identifies a Vault
profile; it becomes an owner key only after assignment to a Lockbox.

Every function that returns a handle transfers ownership to the caller. Release
it with the matching `*_free` function on success and on every later error
path. `lockbox_free` releases the handle and content key held by this process;
it does not delete the archive. `lockbox_commit` persists pending changes.

## Errors and secret outputs

Check every Boolean result and every returned pointer. On failure,
`buffer_last_error_details` returns structured category, format version, and
recovery guidance; release the returned buffer with `buffer_free`. The
error stored for the current thread is diagnostic state, not a substitute for checking the
operation result.

Secret getters return opaque secret handles. Copy their content into mutable
memory only for the required operation, overwrite that copy, and call
`secret_free`. Do not log secret buffers or convert them to immutable strings.

## Optional session agent

Ordinary Lockbox functions keep their state in this process and never start or
contact the agent. Use the agent when Lockbox keys need to be shared across
processes or remain available after the process that opened the Lockbox exits.
Closing an agent entry
forgets that cached key; it does not delete a Lockbox file or remove a
persistent credential from the Vault.

## Platform credential store

The operating system credential store can hold the Vault passphrase. The
user's operating system login normally unlocks that store. After login,
another process running as that user may be able to retrieve the passphrase if
the access policy applied to the saved Vault passphrase does not require
approval for each retrieval. Exact access depends on the operating system, the
credential store configuration, and that access policy.

A process that retrieves the Vault passphrase can open the Vault. The Vault
can then provide access to Lockboxes through profile keys or remembered
Lockbox passwords. Both remain encrypted inside the Vault; they are not copied
to the operating system credential store.

Agent expiry improves memory hygiene. It is not an authentication boundary
after login if the saved Vault passphrase can be retrieved without approval.

## CLI interoperability

Use the path based open functions with the appropriate password, profile
key, or Vault credential. Use inspection functions when only public header and
key slot metadata is required.

## Distribution and exact version documentation

Install the header and library from the same release artifact. The
[C conformance program](../e2e/c/conformance.c) is an executable ownership and
error handling reference, while the [cross language example index](../API_EXAMPLES.md)
tracks operation coverage. Report ABI or documentation defects in the
[reVault issue tracker](https://github.com/onepub-dev/reVault/issues).
