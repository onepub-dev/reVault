package com.onepub.revault;

import java.nio.file.Path;
import java.util.Objects;

/**
 * A metadata-only view of a persistent Vault.
 *
 * <p>This facade can discover profiles, contacts, forms, and known Lockbox
 * paths without loading private signing material. It owns a native handle and
 * should be used with try-with-resources.
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
