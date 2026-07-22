package com.onepub.revault;

/** Structured category, version, guidance, and artifact context for the most recent native failure. */
public final class ErrorDetails {
  private final com.onepub.revault.internal.ErrorDetails view;
  private final String category;
  private final String artifactKind;
  private final long foundVersion;
  private final long supportedVersion;
  private final String message;
  private final String guidance;

  /** Creates an application-owned ErrorDetails value. */
  public ErrorDetails(String category, String artifactKind, long foundVersion, long supportedVersion, String message, String guidance) {
    this.view = null;
    this.category = category;
    this.artifactKind = artifactKind;
    this.foundVersion = foundVersion;
    this.supportedVersion = supportedVersion;
    this.message = message;
    this.guidance = guidance;
  }

  ErrorDetails(com.onepub.revault.internal.ErrorDetails view) {
    this.view = java.util.Objects.requireNonNull(view);
    this.category = null;
    this.artifactKind = null;
    this.foundVersion = 0;
    this.supportedVersion = 0;
    this.message = null;
    this.guidance = null;
  }

  /** Stable error category suitable for programmatic handling. */
  public String category() {
    return view == null ? category : view.category();
  }

  /** Kind of archive or vault artifact involved in the failure. */
  public String artifactKind() {
    return view == null ? artifactKind : view.artifactKind();
  }

  /** Format version read from the failing artifact. */
  public long foundVersion() {
    return view == null ? foundVersion : view.foundVersion() & 0xffffffffL;
  }

  /** Newest format version supported by this library. */
  public long supportedVersion() {
    return view == null ? supportedVersion : view.supportedVersion() & 0xffffffffL;
  }

  /** Human-readable explanation of the failure. */
  public String message() {
    return view == null ? message : view.message();
  }

  /** Suggested corrective action for the caller or user. */
  public String guidance() {
    return view == null ? guidance : view.guidance();
  }

}
