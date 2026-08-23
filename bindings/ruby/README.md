# reVault for Ruby

reVault is an encrypted archive and local-vault library for files, credentials,
keys, and typed records. Platform gems use Fiddle and contain the matching
native runtime. See the
[reVault documentation](https://github.com/onepub-dev/reVault/tree/main/docs).

```shell
gem install revault_api -v 0.2.0
```

The complete method-example index is in [`../API_EXAMPLES.md`](../API_EXAMPLES.md).

```ruby
runtime = Revault.load                 # only loads the native runtime
vault = runtime
box = vault.lockbox_create("\0" * 32)
box.add_file('/hello.txt', "hello\n", false)
box.set_variable('owner', 'alice')
box.set_secret_variable('token', 'secret')
box.with_secret_variable('token') do |token|
  # Consume the temporary mutable String only inside this block.
end
box.commit
box.free

persistent = Revault::Vault.open_or_create('/tmp/revault-vault', Revault::SecretString.new('vault passphrase'))
persistent.close
```

The temporary secret String is overwritten after the block. Ruby strings are
not reliably zeroizable once copied, so do not retain or duplicate it.
`AgentSession` is explicit and caches selected content keys temporarily;
ordinary Lockbox operations never contact the agent.
