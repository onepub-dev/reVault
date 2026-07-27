package com.onepub.revault;

/** A logical or physical byte range emitted while walking the contents of a lockbox. */
public final class StreamChunk {
  private final com.onepub.revault.internal.StreamChunk view;
  private final String path;
  private final long fileOffset;
  private final long length;
  private final long physicalOffset;
  private final boolean sparse;
  private final byte[] data;

  /** Creates an application-owned StreamChunk value. */
  public StreamChunk(String path, long fileOffset, long length, long physicalOffset, boolean sparse, byte[] data) {
    this.view = null;
    this.path = path;
    this.fileOffset = fileOffset;
    this.length = length;
    this.physicalOffset = physicalOffset;
    this.sparse = sparse;
    this.data = data;
  }

  StreamChunk(com.onepub.revault.internal.StreamChunk view) {
    this.view = java.util.Objects.requireNonNull(view);
    this.path = null;
    this.fileOffset = 0;
    this.length = 0;
    this.physicalOffset = 0;
    this.sparse = false;
    this.data = null;
  }

  /** Lockbox file path to which this byte range belongs. */
  public String path() {
    return view == null ? path : view.path();
  }

  /** Logical byte offset within the file. */
  public long fileOffset() {
    return view == null ? fileOffset : view.fileOffset();
  }

  /** Logical range length in bytes. */
  public long length() {
    return view == null ? length : view.length();
  }

  /** Archive byte offset, when physical streaming is requested. */
  public long physicalOffset() {
    return view == null ? physicalOffset : view.physicalOffset();
  }

  /** Whether the range represents a sparse zero-filled extent. */
  public boolean sparse() {
    return view == null ? sparse : view.sparse();
  }

  /** File bytes for a populated logical range. */
  public byte[] data() {
    if (view == null) return data.clone();
    var result = new byte[view.dataLength()];
    for (int index = 0; index < result.length; index++) result[index] = (byte) view.data(index);
    return result;
  }

}
