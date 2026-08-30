package com.onepub.revault;

/** Controls the optional process-wide Session Agent and its temporary cache. */
public final class AgentSession {
  /** Kinds of short-lived activity that may retain an agent cache entry. */
  public enum ActivityKind { lockbox, form, key }

  private final Revault runtime;

  AgentSession(Revault runtime) { this.runtime = runtime; }

  /** Starts the agent process if it is not already running. */
  public void start() { runtime.startAgent(); }

  /** Stops the agent process after forgetting its temporary entries. */
  public void stop() { runtime.stopAgent(); }

  /** Returns whether the agent process is currently running. */
  public boolean isRunning() { return runtime.agentIsRunning(); }

  /** Forgets every temporary key held by the agent. */
  public void closeAll() { runtime.forgetAllAgentSecrets(); }

  /** Caches a profile signing identity for the requested session TTL. */
  public void cacheProfileSigningKey(String vaultId, String profile,
      Revault.ProfileSigningKeyPair key, long ttlSeconds) {
    runtime.cacheProfileSigningKey(vaultId, profile, key, ttlSeconds);
  }

  /** Returns a profile signing identity cached by the Session Agent. */
  public Revault.ProfileSigningKeyPair profileSigningKey(String vaultId, String profile) {
    return runtime.profileSigningKey(vaultId, profile);
  }

  /** Forgets one cached profile signing identity. */
  public void forgetProfileSigningKey(String vaultId, String profile) {
    runtime.forgetProfileSigningKey(vaultId, profile);
  }

  /** Begins a scoped activity and releases it with try-with-resources. */
  public Revault.AgentActivity beginActivity(ActivityKind kind) {
    return runtime.beginAgentActivity(kind.name());
  }

  /** Verifies that this process can communicate with the agent. */
  public void verifyTransport() { runtime.verifyAgentTransport(); }
}
