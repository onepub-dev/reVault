package com.onepub.revault;

/** Header, owner-signature, and key-slot information read from a lockbox file without opening its contents. */
public final class FileInspection {
  private final com.onepub.revault.internal.FileInspection view;
  private final byte[] lockboxId;
  private final boolean headerReadable;
  private final long keyDirectoryGeneration;
  private final long keyDirectoryCopyCount;
  private final boolean ownerSigned;
  private final java.util.List<KeySlot> keySlots;

  /** Creates an application-owned FileInspection value. */
  public FileInspection(byte[] lockboxId, boolean headerReadable, long keyDirectoryGeneration, long keyDirectoryCopyCount, boolean ownerSigned, java.util.List<KeySlot> keySlots) {
    this.view = null;
    this.lockboxId = lockboxId;
    this.headerReadable = headerReadable;
    this.keyDirectoryGeneration = keyDirectoryGeneration;
    this.keyDirectoryCopyCount = keyDirectoryCopyCount;
    this.ownerSigned = ownerSigned;
    this.keySlots = keySlots;
  }

  FileInspection(com.onepub.revault.internal.FileInspection view) {
    this.view = java.util.Objects.requireNonNull(view);
    this.lockboxId = null;
    this.headerReadable = false;
    this.keyDirectoryGeneration = 0;
    this.keyDirectoryCopyCount = 0;
    this.ownerSigned = false;
    this.keySlots = null;
  }

  /** Stable binary identifier read from the lockbox header. */
  public byte[] lockboxId() {
    if (view == null) return lockboxId.clone();
    var result = new byte[view.lockboxIdLength()];
    for (int index = 0; index < result.length; index++) result[index] = (byte) view.lockboxId(index);
    return result;
  }

  /** Whether the archive header passed structural validation. */
  public boolean headerReadable() {
    return view == null ? headerReadable : view.headerReadable();
  }

  /** Latest readable access-key directory generation. */
  public long keyDirectoryGeneration() {
    return view == null ? keyDirectoryGeneration : view.keyDirectoryGeneration();
  }

  /** Number of readable redundant key-directory copies. */
  public long keyDirectoryCopyCount() {
    return view == null ? keyDirectoryCopyCount : view.keyDirectoryCopyCount();
  }

  /** Whether commits require an owner signature. */
  public boolean ownerSigned() {
    return view == null ? ownerSigned : view.ownerSigned();
  }

  /** Password and contact access methods found in the header. */
  public java.util.List<KeySlot> keySlots() {
    if (view == null) return keySlots;
    var result = new java.util.ArrayList<KeySlot>(view.keySlotsLength());
    for (int index = 0; index < view.keySlotsLength(); index++) result.add(new KeySlot(view.keySlots(index)));
    return java.util.List.copyOf(result);
  }

}
