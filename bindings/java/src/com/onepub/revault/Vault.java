package com.onepub.revault;

import java.nio.file.Path;
import java.util.Objects;

/**
 * An opened persistent reVault Vault.
 *
 * <p>Use {@link #open()} when the Vault passphrase is in platform storage, or
 * {@link #open(byte[])} when the caller has acquired the Vault passphrase. The
 * byte array remains caller-owned and should be wiped after this call returns.
 * The returned object owns the native vault directory and must be closed.
 *
 * <pre>{@code
 * try (var vault = Vault.open(vaultPassphrase);
 *      var box = vault.openLockboxWithPassword(Path.of("team.lbox"), lockboxPassword)) {
 *   System.out.println(box.list("/", true));
 * }
 * }</pre>
 */
public final class Vault extends Revault.VaultHandle {
  private final Revault runtime;
  private final Revault.LockboxSessionHandle local;
  private final AgentSession agent;

  private Vault(Revault runtime, Revault.VaultHandle directory) {
    runtime.super(directory.detach());
    this.runtime = Objects.requireNonNull(runtime);
    Objects.requireNonNull(directory);
    this.local = runtime.openLockboxSession();
    this.agent = new AgentSession(runtime);
  }

  /** Opens the default Vault using the passphrase held in platform storage. */
  public static Vault open() {
    var runtime = Revault.load();
    var passphrase = runtime.getPlatformPassword();
    try { return new Vault(runtime, runtime.openVault(runtime.defaultVaultRoot(), passphrase)); }
    finally { java.util.Arrays.fill(passphrase, (byte) 0); }
  }

  /** Opens the default Vault with an explicitly supplied Vault passphrase. */
  public static Vault open(byte[] vaultPassphrase) {
    var runtime = Revault.load();
    return new Vault(runtime, runtime.openVault(runtime.defaultVaultRoot(), vaultPassphrase));
  }

  /** Opens an existing Vault directory with its Vault passphrase. */
  public static Vault open(Path root, byte[] vaultPassphrase) {
    var runtime = Revault.load();
    return new Vault(runtime, runtime.openVault(root.toString(), vaultPassphrase));
  }

  /** Opens an existing Vault or creates it when absent. */
  public static Vault openOrCreate(Path root, byte[] vaultPassphrase) {
    var runtime = Revault.load();
    return new Vault(runtime, runtime.openOrCreateVault(root.toString(), vaultPassphrase));
  }

  /** Creates a fresh Vault, replacing persistent data at {@code root}. */
  public static Vault create(Path root, byte[] vaultPassphrase) {
    var runtime = Revault.load();
    return new Vault(runtime, runtime.replaceVault(root.toString(), vaultPassphrase));
  }

  /** Explicit destructive alias for {@link #create(Path, byte[])}. */
  public static Vault replace(Path root, byte[] vaultPassphrase) {
    return create(root, vaultPassphrase);
  }

  /** Opens the metadata-only view for an existing Vault. */
  public static ReadOnlyVault openReadOnly(Path root, byte[] vaultPassphrase) {
    var runtime = Revault.load();
    return new ReadOnlyVault(runtime, runtime.openReadOnlyVault(root.toString(), vaultPassphrase));
  }

  /** Opens the metadata-only view for the platform-default Vault. */
  public static ReadOnlyVault openReadOnly(byte[] vaultPassphrase) {
    var runtime = Revault.load();
    return new ReadOnlyVault(runtime, runtime.openDefaultReadOnlyVault(vaultPassphrase));
  }

  /** Returns controls for the optional Session Agent process. */
  public AgentSession agentSession() { return agent; }

  /** Opens a password-protected lockbox and retains its key until close. */
  public Revault.Lockbox openLockboxWithPassword(Path path, byte[] lockboxPassword) {
    return local.openWithPassword(path.toString(), lockboxPassword);
  }

  /** Creates a password-protected lockbox at {@code path}. */
  public Revault.Lockbox createLockboxWithPassword(Path path, byte[] lockboxPassword) {
    return local.createWithPassword(path.toString(), lockboxPassword);
  }

  /** Forgets every temporary key cached by this Vault's Session Agent. */
  public void closeAllAgentEntries() { agent.closeAll(); }

  /** Closes lockbox/local-vault resources and the persistent Vault handle. */
  @Override public void close() {
    local.close();
    super.close();
  }
}
