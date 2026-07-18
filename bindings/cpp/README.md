# reVault C++ API

reVault is an encrypted archive and local-vault library for files, credentials,
keys, and typed records. This C++20 facade provides RAII ownership over the
stable native API. See the
[reVault documentation](https://github.com/onepub-dev/reVault/tree/main/docs).

Install the SDK with a GitHub release package, Homebrew, vcpkg, Conan, Debian,
or RPM, then link `revault_api_cpp` and `revault_api`.

```cpp
#include <revault_api.hpp>

std::vector<std::uint8_t> key(32); // load a real key securely
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

Facade objects own their native handles. Secret values are deliberately read
only through scoped callbacks; do not copy them into long-lived strings.
