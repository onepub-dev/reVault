# reVault for Lua

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

The LuaJIT package provides Lockbox, Vault, key, and session agent APIs through
FFI and includes the matching reVault engine.

## Installation and native runtime

```shell
luarocks install revault_api
```

LuaRocks publishes a binary rock for each supported operating system and
architecture. The rock installs the matching library on Lua's C module search
path, where LuaJIT FFI can load it. Install the rock matching the LuaJIT
process architecture.

The [complete method example index](https://github.com/onepub-dev/reVault/blob/main/bindings/API_EXAMPLES.md)
is maintained in the source repository.

```lua
local revault = require('revault_api')
local runtime = revault.Revault.load() -- loading does not open a Vault
local signing = runtime:generate_profile_signing_key_pair()
local public_signing_key = signing:public_key()
local box = runtime:lockbox_create(string.rep('\0', 32))
box:set_owner_signing_key(signing) -- Profile becomes this Lockbox's owner
box:add_file('/hello.txt', 'hello\n', false)
box:set_variable('owner', 'alice')
box:set_secret_variable('token', 'secret')
box:with_secret_variable('token', function(token, length)
  -- Consume token[0..length-1] only inside this callback.
end)
box:commit()
box:free()
public_signing_key:free(); signing:free()

local persistent = revault.Vault.open_or_create('/tmp/revault-vault', 'Vault passphrase')
persistent:close()
```

Pass a path to `revault.Revault.load(native_library_path)` to use a native
library supplied by the application. Otherwise, reVault checks
`REVAULT_LIBRARY` and then the library installed by LuaRocks. A bare name uses
the operating system library search path.

The callback receives temporary FFI memory that is cleared after it returns.
Lua strings are immutable, so avoid putting secrets in retained strings.
Ordinary Lockbox operations never contact the agent.

## Core API concepts

- `Revault` loads the runtime.
- `Vault` owns persistent local state.
- `Lockbox` owns an open archive.
- `AgentSession` caches selected content keys.

Free or close each object independently.

## API documentation and support

The rock source documents the public tables and methods for its release. The
[method examples](https://github.com/onepub-dev/reVault/blob/main/bindings/API_EXAMPLES.md)
and [Lua conformance program](https://github.com/onepub-dev/reVault/blob/main/bindings/e2e/lua/conformance.lua)
cover the common operation inventory.

## Create, open, and replace

Open functions require existing Vaults and Lockboxes and never create them.
Use open or create only when creation is acceptable and replacement functions
only after an explicit destructive decision. `commit` persists pending
Lockbox changes; `free` releases the handle and content key held by the current process without
deleting the archive.

Lua can open the archive with its Lockbox password, a profile key, or a
credential resolved from the Vault. A profile signing key becomes an owner key
only when assigned to a Lockbox.

## Secrets, failures, and ownership

A vault passphrase, Lockbox password, and 32-byte content key are different
secrets. Lua strings are immutable and cannot be reliably erased, so avoid
copies and keep secret input lifetimes short. Memory passed to a secret
callback is cleared after return and must not escape the callback.

Release every Lockbox, Vault, key, secret, and agent handle on success and on
all later error paths. Native failures are raised as Lua errors with structured
details rather than exposed as public global error state.

## Optional session agent

Normal Lockbox opens keep their state in this process and never consult the
agent. Use `AgentSession` when Lockbox keys need to be shared across processes
or remain available after the process that opened the Lockbox exits. Closing
an entry forgets the cached content key but does not delete the Lockbox or
remove credentials stored in the Vault.

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

The installed `revault_api.lua` is the exact version LuaDoc reference.
Missing or placeholder class/method documentation is a binding defect.
