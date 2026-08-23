package com.onepub.revault;

/** I/O workload policy used to tune lockbox page access. */
public enum WorkloadProfile { INTERACTIVE("interactive"), BULK_IMPORT("bulk-import");
  private final String wire;
  WorkloadProfile(String wire) { this.wire = wire; }
  String wire() { return wire; }
}
