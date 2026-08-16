package com.onepub.revault;

import java.nio.file.Path;
import java.util.Objects;

/**
 * An opened persistent reVault Vault.
 *
 * <p>Use {@link #open()} when the vault passphrase is in platform storage, or
 * {@link #open(byte[])} when the caller has acquired the vault passphrase. The
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
public final class Vault implements AutoCloseable {
  private final Revault runtime;
  private final Revault.VaultDirectoryHandle directory;
  private final Revault.LocalVaultHandle local;
  private final AgentSession agent;

  private Vault(Revault runtime, Revault.VaultDirectoryHandle directory) {
    this.runtime = Objects.requireNonNull(runtime);
    this.directory = Objects.requireNonNull(directory);
    this.local = runtime.openLocalVault();
    this.agent = new AgentSession(runtime);
  }

  /** Opens the default Vault using the passphrase held in platform storage. */
  public static Vault open() {
    var runtime = Revault.load();
    return new Vault(runtime, runtime.openOrCreateDefaultVaultDirectory(runtime.getPlatformPassword()));
  }

  /** Opens the default Vault with an explicitly supplied vault passphrase. */
  public static Vault open(byte[] vaultPassphrase) {
    var runtime = Revault.load();
    return new Vault(runtime, runtime.openOrCreateDefaultVaultDirectory(vaultPassphrase));
  }

  /** Opens an existing Vault directory with its vault passphrase. */
  public static Vault open(Path root, byte[] vaultPassphrase) {
    var runtime = Revault.load();
    return new Vault(runtime, runtime.openVaultDirectory(root.toString(), vaultPassphrase));
  }

  /** Returns controls for the optional session-agent process. */
  public AgentSession agentSession() { return agent; }

  /** Opens a password-protected lockbox and retains its key until close. */
  public Revault.Lockbox openLockboxWithPassword(Path path, byte[] lockboxPassword) {
    return local.openWithPassword(path.toString(), lockboxPassword);
  }

  /** Creates a password-protected lockbox at {@code path}. */
  public Revault.Lockbox createLockboxWithPassword(Path path, byte[] lockboxPassword) {
    return local.createWithPassword(path.toString(), lockboxPassword);
  }

  /** Forgets every temporary key cached by this Vault's agent session. */
  public void closeAllAgentEntries() { agent.closeAll(); }

  /** Closes lockbox/local-vault resources and the persistent Vault handle. */
  @Override public void close() {
    local.close();
    directory.close();
  }
}
