namespace Revault;

/// Persistent encrypted local store for profiles, private keys, contacts,
/// signing keys, and remembered lockbox metadata.
///
/// A Vault is returned by the explicit Revault open/create/replace methods;
/// opening an existing Vault never creates or replaces it.
public sealed class Vault : Revault.VaultStore
{
    internal Vault(Revault owner, IntPtr handle) : base(owner, handle) { }
}
