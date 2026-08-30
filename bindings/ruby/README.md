# reVault for Ruby

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

Platform gems contain the matching reVault engine and provide Lockbox, Vault,
key, and session agent classes through Fiddle.

## Installation and native runtime

```shell
gem install revault_api
```

RubyGems publishes a platform gem for each supported operating system and
architecture. Each gem contains the matching reVault library and a Ruby shim;
the package loads both through Fiddle. Install the gem matching the Ruby
process architecture.

`Revault.load(native_library_path)` can load a native library supplied by the
application. Otherwise, reVault checks `REVAULT_LIBRARY` and then the library
installed by the gem. A bare name uses the operating system search path.
Selection is shared by the process and occurs before the Ruby API loads.

The [complete method example index](https://github.com/onepub-dev/reVault/blob/main/bindings/API_EXAMPLES.md)
is maintained in the source repository.

```ruby
runtime = Revault.load                 # only loads the native runtime
signing = runtime.generate_profile_signing_key_pair
public_signing_key = signing.public_key
box = runtime.lockbox_create("\0" * 32)
box.set_owner_signing_key(signing) # Profile becomes this Lockbox's owner
box.add_file('/hello.txt', "hello\n", false)
box.set_variable('owner', 'alice')
box.set_secret_variable('token', 'secret')
box.with_secret_variable('token') do |token|
  # Consume the temporary mutable String only inside this block.
end
box.commit
box.free
public_signing_key.free; signing.free

persistent = Revault::Vault.open_or_create('/tmp/revault-vault', Revault::SecretString.new('Vault passphrase'))
persistent.close
```

The temporary secret String is overwritten after the block. Ruby strings are
not reliably zeroizable once copied, so do not retain or duplicate it.
`AgentSession` is explicit and caches selected content keys temporarily;
ordinary Lockbox operations never contact the agent.

## Core API concepts

- `Revault` loads the runtime.
- `Vault` owns persistent local state.
- `Lockbox` owns an open archive.
- `AgentSession` caches selected content keys.

Free or close each object independently.

## API documentation and support

The gem source documents the public classes and methods for its release. The
[method examples](https://github.com/onepub-dev/reVault/blob/main/bindings/API_EXAMPLES.md)
and [Ruby conformance program](https://github.com/onepub-dev/reVault/blob/main/bindings/e2e/ruby/conformance.rb)
cover the common operation inventory.

## Create, open, and replace

Open methods require existing Vaults and Lockboxes and never create missing
state. Use open or create only when creation is acceptable and replacement
methods only after an explicit destructive choice. `commit` persists pending
Lockbox changes; `close` or `free` releases the handle and content key held by the current process
without deleting the archive.

Open that archive with its Lockbox password, a profile key, or a credential
resolved from the Vault. A profile signing key becomes a Lockbox owner key only
after explicit assignment.

## Secrets, failures, and ownership

A vault passphrase, Lockbox password, and 32-byte content key are different
secrets. Prefer mutable owned strings with minimal lifetime and overwrite them
after use. Do not copy secret values received by a callback into retained Ruby
objects. Close objects that own native resources in `ensure` blocks.

Native failures raise typed exceptions with structured details. Public code
should not depend on transport handles or mutable global error state.

## Optional session agent

Ordinary Lockbox operations never start or consult the agent. Use
`AgentSession` when Lockbox keys need to be shared across processes or remain
available after the process that opened the Lockbox exits. Closing an entry
forgets the cached key, not the Lockbox file or a credential stored in the
Vault.

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

Missing or placeholder YARD class/method documentation is a binding defect.
