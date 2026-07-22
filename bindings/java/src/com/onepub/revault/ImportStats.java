package com.onepub.revault;

/** Time spent reading host files and preparing encrypted pages during the latest import work. */
public final class ImportStats {
  private final com.onepub.revault.internal.ImportStats view;
  private final String hostStatNanos;
  private final String hostReadNanos;
  private final String framePrepareNanos;
  private final String pageWriteNanos;

  /** Creates an application-owned ImportStats value. */
  public ImportStats(String hostStatNanos, String hostReadNanos, String framePrepareNanos, String pageWriteNanos) {
    this.view = null;
    this.hostStatNanos = hostStatNanos;
    this.hostReadNanos = hostReadNanos;
    this.framePrepareNanos = framePrepareNanos;
    this.pageWriteNanos = pageWriteNanos;
  }

  ImportStats(com.onepub.revault.internal.ImportStats view) {
    this.view = java.util.Objects.requireNonNull(view);
    this.hostStatNanos = null;
    this.hostReadNanos = null;
    this.framePrepareNanos = null;
    this.pageWriteNanos = null;
  }

  /** Nanoseconds spent reading host filesystem metadata, as decimal text. */
  public String hostStatNanos() {
    return view == null ? hostStatNanos : view.hostStatNanos();
  }

  /** Nanoseconds spent reading host file content, as decimal text. */
  public String hostReadNanos() {
    return view == null ? hostReadNanos : view.hostReadNanos();
  }

  /** Nanoseconds spent preparing encrypted records, as decimal text. */
  public String framePrepareNanos() {
    return view == null ? framePrepareNanos : view.framePrepareNanos();
  }

  /** Nanoseconds spent writing encrypted pages, as decimal text. */
  public String pageWriteNanos() {
    return view == null ? pageWriteNanos : view.pageWriteNanos();
  }

}
