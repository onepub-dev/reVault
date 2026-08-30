# reVault for Ruby

reVault is an encrypted archive and local-vault library for files, credentials,
keys, and typed records. Platform gems use Fiddle and contain the matching
native runtime. See the
[reVault manual](https://docs.revault.onepub.dev/).

```shell
gem install revault_api
```

`Revault.load(native_library_path)` selects an application-owned carrier.
Otherwise a non-empty inherited `REVAULT_LIBRARY` is used before the matching
platform gem carrier. A bare library name delegates to the operating-system
search path. Selection is process-wide and occurs before the Ruby shim loads.

The [complete method-example index](https://github.com/onepub-dev/reVault/blob/main/bindings/API_EXAMPLES.md)
is maintained in the source repository.

```ruby
runtime = Revault.load                 # only loads the native runtime
signing = runtime.generate_profile_signing_key_pair
public_signing_key = signing.public_key
vault = runtime
box = vault.lockbox_create("\0" * 32)
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
