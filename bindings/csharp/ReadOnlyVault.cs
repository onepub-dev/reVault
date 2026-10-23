namespace Revault;

/// Read-only Vault view that can retrieve stored credentials. Close it promptly
/// to release its shared archive lock so a waiting writer may proceed.
public sealed class ReadOnlyVault : Revault.ReadOnlyVaultStore
{
    internal ReadOnlyVault(Revault owner, IntPtr handle) : base(owner, handle) { }
}
