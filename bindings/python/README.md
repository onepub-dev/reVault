# reVault for Python

reVault is an encrypted archive and local-vault library for files, credentials,
keys, and typed records. Platform wheels include the native library and expose
typed reVault domain values while keeping the native transport private. See the
[reVault manual](https://docs.revault.onepub.dev/).

```shell
python -m pip install revault-api
```

The [complete method-example index](https://github.com/onepub-dev/reVault/blob/main/bindings/API_EXAMPLES.md)
is maintained in the source repository.

```python
from revault_api import Revault, SecretString, Vault

runtime = Revault.load()                 # loading does not open a Vault
signing = runtime.generate_profile_signing_key_pair()
public_signing_key = signing.public_key()
with runtime.lockbox_create(bytes(32)) as box:  # process-local Lockbox
    box.set_owner_signing_key(signing)  # Profile becomes this Lockbox's owner
    box.add_file("/hello.txt", b"hello\n", False)
    box.set_variable("owner", "alice")
    box.set_secret_variable("token", bytearray(b"secret"))
    size = box.with_secret_variable("token", lambda token: len(token))
    box.commit()
public_signing_key.close()
signing.close()

with SecretString("vault passphrase") as vault_passphrase:
    persistent = Vault.open_or_create("/tmp/revault-vault", vault_passphrase)
    persistent.close()
```

`Revault.load(native_library_path=...)` opens an application-owned carrier.
Otherwise a non-empty inherited `REVAULT_LIBRARY` is used before the packaged
carrier. A bare library name delegates to the operating-system search path.

The value passed to a secret callback is a temporary `bytearray`; it is
cleared after the callback. Do not convert it to a retained `str` or `bytes`.
`SecretString` and `SecretBytes` own mutable buffers and wipe them on
`close()`. Use `AgentSession` explicitly when delegating selected content keys;
ordinary Lockbox operations never contact the agent.
