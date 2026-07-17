# reVault for Lua

reVault is an encrypted archive and local-vault library for files, credentials,
keys, and typed records. The LuaJIT package uses FFI and includes the matching
native runtime. See the
[reVault documentation](https://github.com/onepub-dev/reVault/tree/main/docs).

```shell
luarocks install revault_api 0.2.0-1
```

```lua
local revault = require('revault_api')
local vault = revault.Vault.new()
local box = vault:lockbox_create(string.rep('\0', 32))
box:add_file('/hello.txt', 'hello\n', false)
box:set_variable('owner', 'alice')
box:set_secret_variable('token', 'secret')
box:with_secret_variable('token', function(token, length)
  -- Consume token[0..length-1] only inside this callback.
end)
box:commit()
box:free()
```

The callback receives temporary FFI memory that is cleared after it returns.
Lua strings are immutable, so avoid putting secrets in retained strings.
