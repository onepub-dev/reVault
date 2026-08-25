namespace Revault;

/// Decoded-page cache strategy accepted by the native runtime.
public enum CacheMode
{
    /// Keep decoded pages in a byte-bounded cache.
    Bytes,
    /// Keep decoded pages in a page-count-bounded cache.
    Pages
}
/// I/O workload policy accepted by lockbox creation/opening.
public enum WorkloadProfile
{
    /// Favor low latency for interactive operations.
    Interactive,
    /// Favor throughput for bulk imports.
    BulkImport
}
/// Worker scheduling policy accepted by lockbox creation/opening.
public enum WorkerPolicy
{
    /// Let the native runtime select the worker count.
    Auto,
    /// Use one worker.
    Single
}
/// Temporary operation retained by the optional session agent.
public enum AgentActivityKind
{
    /// A lockbox operation.
    Lockbox,
    /// A form operation.
    Form,
    /// A key operation.
    Key
}

/// Typed native failure. The message is diagnostic text from the current
/// thread; details are available through the low-level diagnostic API.
public sealed class RevaultException : Exception
{
    /// Creates an exception with the native diagnostic message.
    public RevaultException(string message) : base(message) { }
}

/// Controls the single optional session-agent process and temporary cache.
public sealed class AgentSession
{
    private readonly Revault owner;
    internal AgentSession(Revault owner) => this.owner = owner;
    /// Starts the optional session agent.
    public void Start() => owner.StartAgent();
    /// Stops the optional session agent.
    public void Close() => owner.StopAgent();
    /// Reports whether the session agent is running.
    public bool IsRunning => owner.AgentIsRunning;
    /// Forgets all secrets retained by the session agent.
    public void CloseAll() => owner.ForgetAllAgentSecrets();
    /// Forgets one lockbox key retained by the session agent.
    public void CloseLockbox(byte[] lockboxId) => owner.ForgetAgentKey(lockboxId);
    /// Caches a profile signing identity for the requested session TTL.
    public void CacheProfileSigningKey(string vaultId, string profile,
        Revault.ProfileSigningKeyPair key, ulong ttlSeconds) =>
        owner.CacheProfileSigningKey(vaultId, profile, key, ttlSeconds);
    /// Returns a profile signing identity cached by the session agent.
    public Revault.ProfileSigningKeyPair ProfileSigningKey(string vaultId, string profile) =>
        owner.ProfileSigningKey(vaultId, profile);
    /// Forgets one cached profile signing identity.
    public void ForgetProfileSigningKey(string vaultId, string profile) =>
        owner.ForgetProfileSigningKey(vaultId, profile);
}
