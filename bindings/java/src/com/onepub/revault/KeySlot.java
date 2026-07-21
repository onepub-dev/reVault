package com.onepub.revault;

/** One password or contact credential that can unlock a lockbox content key. */
public final class KeySlot {
  private final com.onepub.revault.internal.KeySlot view;
  private final long id;
  private final String protection;
  private final String algorithm;

  /** Creates an application-owned KeySlot value. */
  public KeySlot(long id, String protection, String algorithm) {
    this.view = null;
    this.id = id;
    this.protection = protection;
    this.algorithm = algorithm;
  }

  KeySlot(com.onepub.revault.internal.KeySlot view) {
    this.view = java.util.Objects.requireNonNull(view);
    this.id = 0;
    this.protection = null;
    this.algorithm = null;
  }

  /** Stable slot identifier used when removing this access method. */
  public long id() {
    return view == null ? id : view.id();
  }

  /** Access method, such as password or contact key. */
  public String protection() {
    return view == null ? protection : view.protection();
  }

  /** Cryptographic algorithm protecting the content key. */
  public String algorithm() {
    return view == null ? algorithm : view.algorithm();
  }

}
