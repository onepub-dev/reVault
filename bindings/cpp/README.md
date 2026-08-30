# reVault C++ API

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

This C++20 facade provides Lockbox, Vault, key, and session agent classes with
RAII ownership.

## Native library distribution

Install the SDK with a GitHub release package, Homebrew, vcpkg, Conan, Debian,
or RPM, then link `revault_api_cpp` and `revault_api`. The C++ facade is linked
with the matching C ABI library from the same SDK. Shared builds require that
library on the operating system's runtime search path; static builds include
the engine in the application.

```cpp
#include <revault_api.hpp>

auto runtime = revault::Revault::load();
std::vector<std::uint8_t> key(32); // load a real content key securely
revault::Lockbox box(key);
const std::vector<std::uint8_t> hello{'h', 'e', 'l', 'l', 'o', '\n'};
box.add_file("/hello.txt", hello, false);
box.set_variable("owner", "alice");
const std::vector<std::uint8_t> token{'s', 'e', 'c', 'r', 'e', 't'};
box.set_secret_variable("token", token);
box.with_secret_variable("token", [](std::span<const std::uint8_t> token) {
  // Use token only inside this callback; the temporary copy is cleared.
});
box.commit();
```

Use `revault::Vault` for the persistent encrypted store and
`revault::AgentSession` only when explicitly delegating selected lockbox
content keys to the optional Session Agent. `revault::ProfileSigningKeyPair`
names a profile identity; `owner` is used only for the role assigned by
`Lockbox::set_owner_signing_key`. Every lockbox handle owns its content key and
must be destroyed (or moved) when the operation ends. Workflows that use files
on the host use `Lockbox::create_path_*` and `Lockbox::open_path_*`; password
caching and forgetting remain explicit `AgentSession` operations.

The [complete method example index](https://github.com/onepub-dev/reVault/blob/main/bindings/API_EXAMPLES.md)
and [executable C++ conformance example](https://github.com/onepub-dev/reVault/blob/main/bindings/e2e/cpp/conformance.cpp)
are maintained in the source repository. Each public
operation is exercised there against the installed native library.

Facade objects own their native handles. Secret values are deliberately read
only through scoped callbacks; do not copy them into retained strings.

## Core API concepts

- `Revault` loads the runtime.
- `Vault` owns persistent local state.
- `Lockbox` owns an open archive.
- `AgentSession` delegates selected content keys.

Each object has an independent lifetime.

## API documentation and support

The installed `revault_api.hpp` documents the facade belonging to the linked
SDK. The method index and executable C++ conformance program demonstrate class
ownership, errors, and every common operation.

## Create, open, and replace

`Lockbox::open_path_*` and `Vault::open` require existing resources. They
never create or replace missing state. Use `Lockbox::create_path_*` or the
Vault open or create APIs when creation is intended, and use replacement APIs
only after the caller has explicitly chosen destructive replacement.

A C++ application can open the resulting archive through its path based
Lockbox API with a password, profile key, or credential resolved from a Vault.

## Credentials and secret ownership

A vault passphrase, Lockbox password, and 32-byte content key are distinct
secrets. Keep each in mutable storage owned by the caller, avoid copies into
`std::string`, and overwrite it when the operation finishes. Profile signing
keys identify Vault profiles; the owner role begins only when
`set_owner_signing_key` assigns one to a Lockbox.

Facade objects use RAII. Destroying a `Lockbox` releases the
content key but does not delete the archive or forget an independent agent
entry. `commit` persists changes; destruction must not be treated as an
implicit commit.

Secret getters accept callbacks with a temporary `std::span`. Do not retain
the span or its pointer. The backing copy is cleared when the callback returns.
Native failures are translated to the facade's exception/error contract rather
than exposed through global error state.

## Optional session agent

Ordinary Lockbox opens never consult the session agent. Use `AgentSession`
when Lockbox keys need to be shared across processes or remain available after
the process that opened the Lockbox exits.
`close_lockbox` and `close_all` forget agent cache entries; they do not
delete files or persistent Vault credentials.

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

Agent expiry and cache closure improve memory hygiene. They are not
authentication boundaries after login if the saved Vault passphrase can be
retrieved without approval.

## Version specific references

Use the header installed with the linked SDK. The
[C++ conformance program](../e2e/cpp/conformance.cpp) demonstrates the complete
facade with cleanup and failures, and the [example index](../API_EXAMPLES.md)
tracks operation coverage. Report missing method documentation as a binding
defect.
