package com.onepub.revault;

/** A local human-readable label attached to one lockbox access slot. */
public final class AccessSlotLabel {
  private final com.onepub.revault.internal.AccessSlotLabel view;
  private final byte[] lockboxId;
  private final long slotId;
  private final String name;
  private final long updatedAtUnixMs;

  /** Creates an application-owned AccessSlotLabel value. */
  public AccessSlotLabel(byte[] lockboxId, long slotId, String name, long updatedAtUnixMs) {
    this.view = null;
    this.lockboxId = lockboxId;
    this.slotId = slotId;
    this.name = name;
    this.updatedAtUnixMs = updatedAtUnixMs;
  }

  AccessSlotLabel(com.onepub.revault.internal.AccessSlotLabel view) {
    this.view = java.util.Objects.requireNonNull(view);
    this.lockboxId = null;
    this.slotId = 0;
    this.name = null;
    this.updatedAtUnixMs = 0;
  }

  /** Lockbox whose access slot is labelled. */
  public byte[] lockboxId() {
    if (view == null) return lockboxId.clone();
    var result = new byte[view.lockboxIdLength()];
    for (int index = 0; index < result.length; index++) result[index] = (byte) view.lockboxId(index);
    return result;
  }

  /** Stable identifier of the labelled access slot. */
  public long slotId() {
    return view == null ? slotId : view.slotId();
  }

  /** Local human-readable label for the access slot. */
  public String name() {
    return view == null ? name : view.name();
  }

  /** Last label update time in Unix milliseconds. */
  public long updatedAtUnixMs() {
    return view == null ? updatedAtUnixMs : view.updatedAtUnixMs();
  }

}
