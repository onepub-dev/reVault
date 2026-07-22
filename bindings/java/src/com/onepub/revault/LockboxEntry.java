package com.onepub.revault;

/** Metadata for one file, directory, or symbolic link stored at a lockbox path. */
public final class LockboxEntry {
  private final com.onepub.revault.internal.LockboxEntry view;
  private final String path;
  private final LockboxEntryKind kind;
  private final long length;
  private final long permissions;

  /** Creates an application-owned LockboxEntry value. */
  public LockboxEntry(String path, LockboxEntryKind kind, long length, long permissions) {
    this.view = null;
    this.path = path;
    this.kind = kind;
    this.length = length;
    this.permissions = permissions;
  }

  LockboxEntry(com.onepub.revault.internal.LockboxEntry view) {
    this.view = java.util.Objects.requireNonNull(view);
    this.path = null;
    this.kind = null;
    this.length = 0;
    this.permissions = 0;
  }

  /** Absolute lockbox path of the stored entry. */
  public String path() {
    return view == null ? path : view.path();
  }

  /** Filesystem kind: file, directory, or symbolic link. */
  public LockboxEntryKind kind() {
    return view == null ? kind : LockboxEntryKind.values()[view.kind()];
  }

  /** Logical file length in bytes; zero for directories. */
  public long length() {
    return view == null ? length : view.length();
  }

  /** Portable Unix permission bits stored with the entry. */
  public long permissions() {
    return view == null ? permissions : view.permissions() & 0xffffffffL;
  }

}
