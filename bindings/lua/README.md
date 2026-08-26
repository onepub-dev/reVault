# reVault for Lua

reVault is an encrypted archive and local-vault library for files, credentials,
keys, and typed records. The LuaJIT package uses FFI and includes the matching
native runtime. See the
[reVault manual](https://docs.revault.onepub.dev/).

```shell
luarocks install revault_api
```

The [complete method-example index](https://github.com/onepub-dev/reVault/blob/main/bindings/API_EXAMPLES.md)
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

local persistent = revault.Vault.open_or_create('/tmp/revault-vault', 'vault passphrase')
persistent:close()
```

Pass a carrier path to `revault.Revault.load(native_library_path)` for an
application-owned installation. Otherwise a non-empty inherited
`REVAULT_LIBRARY` is used before the installed rock carrier. A bare library
name delegates to the operating-system search path.

The callback receives temporary FFI memory that is cleared after it returns.
Lua strings are immutable, so avoid putting secrets in retained strings.
Use `AgentSession` explicitly for delegated content keys; ordinary Lockbox
operations never contact the agent.
