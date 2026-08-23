package com.onepub.revault;

/** Decoded-page cache strategy accepted by the native runtime. */
public enum CacheMode { BYTES("bytes"), PAGES("pages");
  private final String wire;
  CacheMode(String wire) { this.wire = wire; }
  String wire() { return wire; }
}
