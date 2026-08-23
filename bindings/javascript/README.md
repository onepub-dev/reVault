# reVault for JavaScript

reVault is an encrypted archive and local-vault library for files, credentials,
keys, and typed records. The Node package selects a platform-native carrier and
returns documented reVault domain values while keeping its binary transport private. See the
[reVault documentation](https://github.com/onepub-dev/reVault/tree/main/docs).

```shell
npm install @onepub-dev/revault-api@0.2.0
```

The complete method-example index is in [`../API_EXAMPLES.md`](../API_EXAMPLES.md).

```js
import { Revault, SecretString, Vault } from '@onepub-dev/revault-api';

const runtime = await Revault.load(); // loading does not open a Vault
const box = runtime.lockboxCreate(Buffer.alloc(32)); // process-local Lockbox
box.addFile('/hello.txt', Buffer.from('hello\n'), false);
box.setVariable('owner', 'alice');
box.setSecretVariable('token', Buffer.from('secret'));
box.withSecretVariable('token', token => token.length);
box.commit();
box.free();

const vaultPassphrase = new SecretString('vault passphrase');
const persistent = Vault.openOrCreate('/tmp/revault-vault', vaultPassphrase);
persistent.close(); vaultPassphrase.close();
```

Secret callback buffers are cleared after use. `AgentSession` is explicit;
ordinary Lockbox operations never contact the agent. The hosted WebAssembly package
has the same API; the standalone browser module cannot provide OS vault or
session-agent facilities.
