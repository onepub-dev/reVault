# reVault for JavaScript

reVault is an encrypted archive and local-vault library for files, credentials,
keys, and typed records. The Node package selects a platform-native carrier and
returns documented reVault domain values while keeping its binary transport private. See the
[reVault documentation](https://github.com/onepub-dev/reVault/tree/main/docs).

```shell
npm install @onepub-dev/revault-api@0.2.0
```

```js
import { Vault } from '@onepub-dev/revault-api';

const vault = new Vault();
const box = vault.lockboxCreate(Buffer.alloc(32)); // load a real key securely
box.addFile('/hello.txt', Buffer.from('hello\n'), false);
box.setVariable('owner', 'alice');
box.setSecretVariable('token', Buffer.from('secret'));
box.withSecretVariable('token', token => token.length);
box.commit();
box.free();
```

Secret callback buffers are cleared after use. The hosted WebAssembly package
has the same API; the standalone browser module cannot provide OS vault or
session-agent facilities.
