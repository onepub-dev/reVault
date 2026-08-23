namespace Revault;

/// Read-only metadata view for profile, contact, form, and known-lockbox
/// discovery. It never loads private signing material.
public sealed class ReadOnlyVault : Revault.ReadOnlyVaultStore
{
    internal ReadOnlyVault(Revault owner, IntPtr handle) : base(owner, handle) { }
}
