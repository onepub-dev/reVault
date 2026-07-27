package com.onepub.revault;

/** Current capacity, occupancy, hit, and miss counters for an open lockbox cache. */
public final class CacheStats {
  private final com.onepub.revault.internal.CacheStats view;
  private final long limitBytes;
  private final long usedBytes;
  private final long entries;
  private final long hits;
  private final long misses;

  /** Creates an application-owned CacheStats value. */
  public CacheStats(long limitBytes, long usedBytes, long entries, long hits, long misses) {
    this.view = null;
    this.limitBytes = limitBytes;
    this.usedBytes = usedBytes;
    this.entries = entries;
    this.hits = hits;
    this.misses = misses;
  }

  CacheStats(com.onepub.revault.internal.CacheStats view) {
    this.view = java.util.Objects.requireNonNull(view);
    this.limitBytes = 0;
    this.usedBytes = 0;
    this.entries = 0;
    this.hits = 0;
    this.misses = 0;
  }

  /** Maximum decoded-page memory permitted for the cache. */
  public long limitBytes() {
    return view == null ? limitBytes : view.limitBytes();
  }

  /** Decoded-page memory currently held by the cache. */
  public long usedBytes() {
    return view == null ? usedBytes : view.usedBytes();
  }

  /** Number of decoded pages currently cached. */
  public long entries() {
    return view == null ? entries : view.entries();
  }

  /** Reads served by an already decoded page. */
  public long hits() {
    return view == null ? hits : view.hits();
  }

  /** Reads that required decoding another page. */
  public long misses() {
    return view == null ? misses : view.misses();
  }

}
