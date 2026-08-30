# reVault for JavaScript

reVault is an encrypted archive and local-vault library for files, credentials,
keys, and typed records. The Node package selects a platform-native carrier and
returns documented reVault domain values while keeping its binary transport private. See the
[reVault manual](https://docs.revault.onepub.dev/).

```shell
npm install @onepub-dev/revault-api
```

`Revault.load(nativeLibraryPath)` selects an application-owned carrier.
Otherwise a non-empty inherited `REVAULT_LIBRARY` is used before the matching
npm native-carrier package. A bare library name delegates to the
operating-system search path. Native selection is process-wide and must happen
before the first native operation.

The [complete method-example index](https://github.com/onepub-dev/reVault/blob/main/bindings/API_EXAMPLES.md)
is maintained in the source repository.

```js
import { Revault, SecretString, Vault } from '@onepub-dev/revault-api';

const runtime = await Revault.load(); // loading does not open a Vault
const signing = runtime.generateProfileSigningKeyPair();
const publicSigningKey = signing.publicKey();
const box = runtime.lockboxCreate(Buffer.alloc(32)); // process-local Lockbox
box.setOwnerSigningKey(signing); // the Profile now occupies this Lockbox's owner role
box.addFile('/hello.txt', Buffer.from('hello\n'), false);
box.setVariable('owner', 'alice');
box.setSecretVariable('token', Buffer.from('secret'));
box.withSecretVariable('token', token => token.length);
box.commit();
box.free();
publicSigningKey.dispose(); signing.dispose();

const vaultPassphrase = new SecretString('Vault passphrase');
const persistent = Vault.openOrCreate('/tmp/revault-vault', vaultPassphrase);
persistent.close(); vaultPassphrase.close();
```

Secret callback buffers are cleared after use. `AgentSession` is explicit;
ordinary Lockbox operations never contact the agent. The hosted WebAssembly package
has the same API; the standalone browser module cannot provide OS vault or
Session Agent facilities.
