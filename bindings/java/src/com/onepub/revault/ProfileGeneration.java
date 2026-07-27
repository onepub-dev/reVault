package com.onepub.revault;

/** One active or retired generation of the contact keys belonging to a named vault profile. */
public final class ProfileGeneration {
  private final com.onepub.revault.internal.ProfileGeneration view;
  private final long index;
  private final String status;
  private final byte[] contactFingerprint;
  private final long createdAtUnixMs;
  private final long retiredAtUnixMs;
  private final boolean hasRetiredAt;

  /** Creates an application-owned ProfileGeneration value. */
  public ProfileGeneration(long index, String status, byte[] contactFingerprint, long createdAtUnixMs, long retiredAtUnixMs, boolean hasRetiredAt) {
    this.view = null;
    this.index = index;
    this.status = status;
    this.contactFingerprint = contactFingerprint;
    this.createdAtUnixMs = createdAtUnixMs;
    this.retiredAtUnixMs = retiredAtUnixMs;
    this.hasRetiredAt = hasRetiredAt;
  }

  ProfileGeneration(com.onepub.revault.internal.ProfileGeneration view) {
    this.view = java.util.Objects.requireNonNull(view);
    this.index = 0;
    this.status = null;
    this.contactFingerprint = null;
    this.createdAtUnixMs = 0;
    this.retiredAtUnixMs = 0;
    this.hasRetiredAt = false;
  }

  /** Generation number used to address this key version. */
  public long index() {
    return view == null ? index : view.index() & 0xffffffffL;
  }

  /** Lifecycle state, such as active or retired. */
  public String status() {
    return view == null ? status : view.status();
  }

  /** Fingerprint of this generation's contact public key. */
  public byte[] contactFingerprint() {
    if (view == null) return contactFingerprint.clone();
    var result = new byte[view.contactFingerprintLength()];
    for (int index = 0; index < result.length; index++) result[index] = (byte) view.contactFingerprint(index);
    return result;
  }

  /** Creation time in Unix milliseconds. */
  public long createdAtUnixMs() {
    return view == null ? createdAtUnixMs : view.createdAtUnixMs();
  }

  /** Retirement time in Unix milliseconds when retired. */
  public long retiredAtUnixMs() {
    return view == null ? retiredAtUnixMs : view.retiredAtUnixMs();
  }

  /** Whether a retirement time is present. */
  public boolean hasRetiredAt() {
    return view == null ? hasRetiredAt : view.hasRetiredAt();
  }

}
