# reVault C++ API

reVault is an encrypted `Lockbox` archive and persistent `Vault` for files,
credentials, keys, and typed records. This C++20 facade provides RAII ownership over the
stable native API. See the
[reVault manual](https://docs.revault.onepub.dev/).

Install the SDK with a GitHub release package, Homebrew, vcpkg, Conan, Debian,
or RPM, then link `revault_api_cpp` and `revault_api`.

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
content keys to the optional session agent. `revault::ProfileSigningKeyPair`
names a profile identity; `owner` is used only for the role assigned by
`Lockbox::set_owner_signing_key`. Every lockbox handle owns its content key and
must be destroyed (or moved) when the operation ends. Host-path lockbox
workflows use `Lockbox::create_path_*` and `Lockbox::open_path_*`; password
caching and forgetting remain explicit `AgentSession` operations.

The [complete method-example index](https://github.com/onepub-dev/reVault/blob/main/bindings/API_EXAMPLES.md)
and [executable C++ conformance example](https://github.com/onepub-dev/reVault/blob/main/bindings/e2e/cpp/conformance.cpp)
are maintained in the source repository. Each public
operation is exercised there against an installed native carrier.

Facade objects own their native handles. Secret values are deliberately read
only through scoped callbacks; do not copy them into long-lived strings.
