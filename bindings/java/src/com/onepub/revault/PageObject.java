package com.onepub.revault;

/** One logical object recorded inside an inspected encrypted lockbox page. */
public final class PageObject {
  private final com.onepub.revault.internal.PageObject view;
  private final long id;
  private final String kind;
  private final long payloadLen;

  /** Creates an application-owned PageObject value. */
  public PageObject(long id, String kind, long payloadLen) {
    this.view = null;
    this.id = id;
    this.kind = kind;
    this.payloadLen = payloadLen;
  }

  PageObject(com.onepub.revault.internal.PageObject view) {
    this.view = java.util.Objects.requireNonNull(view);
    this.id = 0;
    this.kind = null;
    this.payloadLen = 0;
  }

  /** Object identifier recorded in the encrypted page. */
  public long id() {
    return view == null ? id : view.id();
  }

  /** Kind of logical object stored in the page. */
  public String kind() {
    return view == null ? kind : view.kind();
  }

  /** Encrypted object payload length in bytes. */
  public long payloadLen() {
    return view == null ? payloadLen : view.payloadLen();
  }

}
