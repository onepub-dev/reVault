package com.onepub.revault;

/** Layout and utilization details for one encrypted page in a lockbox archive. */
public final class PageInspection {
  private final com.onepub.revault.internal.PageInspection view;
  private final long offset;
  private final long pageId;
  private final long sequence;
  private final long pageSize;
  private final long encryptedBodyLen;
  private final long unusedBytes;
  private final long objectCount;
  private final java.util.List<PageObject> objects;

  /** Creates an application-owned PageInspection value. */
  public PageInspection(long offset, long pageId, long sequence, long pageSize, long encryptedBodyLen, long unusedBytes, long objectCount, java.util.List<PageObject> objects) {
    this.view = null;
    this.offset = offset;
    this.pageId = pageId;
    this.sequence = sequence;
    this.pageSize = pageSize;
    this.encryptedBodyLen = encryptedBodyLen;
    this.unusedBytes = unusedBytes;
    this.objectCount = objectCount;
    this.objects = objects;
  }

  PageInspection(com.onepub.revault.internal.PageInspection view) {
    this.view = java.util.Objects.requireNonNull(view);
    this.offset = 0;
    this.pageId = 0;
    this.sequence = 0;
    this.pageSize = 0;
    this.encryptedBodyLen = 0;
    this.unusedBytes = 0;
    this.objectCount = 0;
    this.objects = null;
  }

  /** Byte offset at which the page begins in the archive. */
  public long offset() {
    return view == null ? offset : view.offset();
  }

  /** Identifier stored in the page header. */
  public long pageId() {
    return view == null ? pageId : view.pageId();
  }

  /** Commit sequence that wrote this page. */
  public long sequence() {
    return view == null ? sequence : view.sequence();
  }

  /** Total encoded page size in bytes. */
  public long pageSize() {
    return view == null ? pageSize : view.pageSize();
  }

  /** Encrypted body length in bytes. */
  public long encryptedBodyLen() {
    return view == null ? encryptedBodyLen : view.encryptedBodyLen();
  }

  /** Unused capacity remaining in the page. */
  public long unusedBytes() {
    return view == null ? unusedBytes : view.unusedBytes();
  }

  /** Number of logical objects stored in the page. */
  public long objectCount() {
    return view == null ? objectCount : view.objectCount();
  }

  /** Logical objects discovered in the page. */
  public java.util.List<PageObject> objects() {
    if (view == null) return objects;
    var result = new java.util.ArrayList<PageObject>(view.objectsLength());
    for (int index = 0; index < view.objectsLength(); index++) result.add(new PageObject(view.objects(index)));
    return java.util.List.copyOf(result);
  }

}
