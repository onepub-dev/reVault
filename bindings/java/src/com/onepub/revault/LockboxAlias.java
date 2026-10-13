package com.onepub.revault;

/** A local Vault alias mapping a normalized name to a stable lockbox identity; resolution does not open the target. */
public final class LockboxAlias {
  private final com.onepub.revault.internal.LockboxAlias view;
  private final String name;
  private final byte[] lockboxId;

  /** Creates an application-owned LockboxAlias value. */
  public LockboxAlias(String name, byte[] lockboxId) {
    this.view = null;
    this.name = name;
    this.lockboxId = lockboxId;
  }

  LockboxAlias(com.onepub.revault.internal.LockboxAlias view) {
    this.view = java.util.Objects.requireNonNull(view);
    this.name = null;
    this.lockboxId = null;
  }

  /** Case-sensitive normalized NFC alias name. */
  public String name() {
    return view == null ? name : view.name();
  }

  /** Stable binary target identity; verify it when opening the resolved path. */
  public byte[] lockboxId() {
    if (view == null) return lockboxId.clone();
    var result = new byte[view.lockboxIdLength()];
    for (int index = 0; index < result.length; index++) result[index] = (byte) view.lockboxId(index);
    return result;
  }

}
