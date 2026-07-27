package com.onepub.revault;

/** Whether a lockbox is owner-signed and, when available, the signing-key fingerprint. */
public final class OwnerInspection {
  private final com.onepub.revault.internal.OwnerInspection view;
  private final boolean signed;
  private final String fingerprint;
  private final boolean hasFingerprint;

  /** Creates an application-owned OwnerInspection value. */
  public OwnerInspection(boolean signed, String fingerprint, boolean hasFingerprint) {
    this.view = null;
    this.signed = signed;
    this.fingerprint = fingerprint;
    this.hasFingerprint = hasFingerprint;
  }

  OwnerInspection(com.onepub.revault.internal.OwnerInspection view) {
    this.view = java.util.Objects.requireNonNull(view);
    this.signed = false;
    this.fingerprint = null;
    this.hasFingerprint = false;
  }

  /** Whether the lockbox requires owner-signed commits. */
  public boolean signed() {
    return view == null ? signed : view.signed();
  }

  /** Owner signing-key fingerprint when one is configured. */
  public String fingerprint() {
    return view == null ? fingerprint : view.fingerprint();
  }

  /** Whether an owner fingerprint is available. */
  public boolean hasFingerprint() {
    return view == null ? hasFingerprint : view.hasFingerprint();
  }

}
