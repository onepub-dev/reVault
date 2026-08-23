package com.onepub.revault;

/** Worker scheduling policy accepted by lockbox operations. */
public enum WorkerPolicy { AUTO("auto"), SINGLE("single");
  private final String wire;
  WorkerPolicy(String wire) { this.wire = wire; }
  String wire() { return wire; }
}
