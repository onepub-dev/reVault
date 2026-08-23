package com.onepub.revault;

/** A local human-readable label attached to one lockbox access slot. */
public final class AccessSlotLabel {
  private final com.onepub.revault.internal.AccessSlotLabel view;
  private final byte[] lockboxId;
  private final long slotId;
  private final String name;
  private final long updatedAtUnixMs;

  /**
   * Creates an application-owned access-slot label.
   *
   * @param lockboxId lockbox whose access slot is labelled
   * @param slotId stable identifier of the labelled access slot
   * @param name local human-readable label
   * @param updatedAtUnixMs last label update time in Unix milliseconds
   */
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

  /**
   * Returns the lockbox whose access slot is labelled.
   *
   * @return a defensive copy of the lockbox identifier
   */
  public byte[] lockboxId() {
    if (view == null) return lockboxId.clone();
    var result = new byte[view.lockboxIdLength()];
    for (int index = 0; index < result.length; index++) result[index] = (byte) view.lockboxId(index);
    return result;
  }

  /**
   * Returns the stable identifier of the labelled access slot.
   *
   * @return the access-slot identifier
   */
  public long slotId() {
    return view == null ? slotId : view.slotId();
  }

  /**
   * Returns the local human-readable label for the access slot.
   *
   * @return the label
   */
  public String name() {
    return view == null ? name : view.name();
  }

  /**
   * Returns the last label update time.
   *
   * @return the update time in Unix milliseconds
   */
  public long updatedAtUnixMs() {
    return view == null ? updatedAtUnixMs : view.updatedAtUnixMs();
  }

}
