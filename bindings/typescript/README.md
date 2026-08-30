# reVault for TypeScript

reVault is an encrypted archive and local-vault library for files, credentials,
keys, and typed records. Type declarations ship with the JavaScript package;
this directory is its strict compile-time conformance consumer. See the
[reVault manual](https://docs.revault.onepub.dev/).

```shell
npm install @onepub-dev/revault-api
```

`Revault.load(nativeLibraryPath?: string)` selects an application-owned
carrier. Otherwise a non-empty inherited `REVAULT_LIBRARY` is used before the
matching npm native-carrier package. A bare library name delegates to the
operating-system search path.

The [complete method-example index](https://github.com/onepub-dev/reVault/blob/main/bindings/API_EXAMPLES.md)
is maintained in the source repository.

```ts
import { Revault, SecretString, Vault } from '@onepub-dev/revault-api';

const runtime = await Revault.load();
const signing = runtime.generateProfileSigningKeyPair();
const publicSigningKey = signing.publicKey();
const box = runtime.lockboxCreate(new Uint8Array(32));
box.setOwnerSigningKey(signing); // the Profile now occupies this Lockbox's owner role
box.setVariable('owner', 'alice');
box.setSecretVariable('token', new TextEncoder().encode('secret'));
const length: number | undefined = box.withSecretVariable(
  'token',
  token => token.length,
);
box.commit();
box.free();
publicSigningKey.dispose(); signing.dispose();

const passphrase = new SecretString('Vault passphrase');
const persistent = Vault.openOrCreate('/tmp/revault-vault', passphrase);
persistent.close(); passphrase.close();
```

The callback buffer is temporary and cleared after return. Strict TypeScript
consumers should keep `strict` enabled to preserve nullable-result checks.
`AgentSession` is explicit; process-local Lockbox opens never contact it.
