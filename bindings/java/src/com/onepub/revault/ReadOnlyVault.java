package com.onepub.revault;

import java.nio.file.Path;
import java.util.Objects;

/**
 * A read-only view of a persistent Vault, including stored credentials.
 *
 * <p>It owns a shared-lock native handle and should be used with
 * try-with-resources so waiting writers can proceed when reads finish.
 */
public final class ReadOnlyVault extends Revault.ReadOnlyVaultHandle {
  private final Revault runtime;

  ReadOnlyVault(Revault runtime, Revault.ReadOnlyVaultHandle handle) {
    runtime.super(handle.detach());
    this.runtime = Objects.requireNonNull(runtime);
    Objects.requireNonNull(handle);
  }

  /** Opens an existing Vault metadata view. */
  public static ReadOnlyVault open(Path root, byte[] vaultPassphrase) {
    var runtime = Revault.load();
    return new ReadOnlyVault(runtime, runtime.openReadOnlyVault(root.toString(), vaultPassphrase));
  }

  /** Opens the platform-default Vault metadata view. */
  public static ReadOnlyVault open(byte[] vaultPassphrase) {
    var runtime = Revault.load();
    return new ReadOnlyVault(runtime, runtime.openDefaultReadOnlyVault(vaultPassphrase));
  }
}
