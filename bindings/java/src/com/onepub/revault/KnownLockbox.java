package com.onepub.revault;

/** A lockbox identifier and host path remembered by the local vault for later discovery. */
public final class KnownLockbox {
  private final com.onepub.revault.internal.KnownLockbox view;
  private final byte[] lockboxId;
  private final String path;
  private final long lastSeenUnixMs;

  /** Creates an application-owned KnownLockbox value. */
  public KnownLockbox(byte[] lockboxId, String path, long lastSeenUnixMs) {
    this.view = null;
    this.lockboxId = lockboxId;
    this.path = path;
    this.lastSeenUnixMs = lastSeenUnixMs;
  }

  KnownLockbox(com.onepub.revault.internal.KnownLockbox view) {
    this.view = java.util.Objects.requireNonNull(view);
    this.lockboxId = null;
    this.path = null;
    this.lastSeenUnixMs = 0;
  }

  /** Stable binary identifier of the remembered lockbox. */
  public byte[] lockboxId() {
    if (view == null) return lockboxId.clone();
    var result = new byte[view.lockboxIdLength()];
    for (int index = 0; index < result.length; index++) result[index] = (byte) view.lockboxId(index);
    return result;
  }

  /** Last known host filesystem path of the lockbox. */
  public String path() {
    return view == null ? path : view.path();
  }

  /** Most recent observation time in Unix milliseconds. */
  public long lastSeenUnixMs() {
    return view == null ? lastSeenUnixMs : view.lastSeenUnixMs();
  }

}
