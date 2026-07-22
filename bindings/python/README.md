# reVault for Python

reVault is an encrypted archive and local-vault library for files, credentials,
keys, and typed records. Platform wheels include the native library and expose
typed reVault domain values while keeping the native transport private. See the
[reVault documentation](https://github.com/onepub-dev/reVault/tree/main/docs).

```shell
python -m pip install revault-api==0.2.0
```

```python
from revault_api import Vault

vault = Vault()
with vault.lockbox_create(bytes(32)) as box:  # load a real key securely
    box.add_file("/hello.txt", b"hello\n", False)
    box.set_variable("owner", "alice")
    box.set_secret_variable("token", bytearray(b"secret"))
    size = box.with_secret_variable("token", lambda token: len(token))
    box.commit()
```

The value passed to a secret callback is a temporary `bytearray`; it is
cleared after the callback. Do not convert it to a retained `str` or `bytes`.
