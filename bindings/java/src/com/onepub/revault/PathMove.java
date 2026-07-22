package com.onepub.revault;

/** A source and destination pair used to rename a variable or form record atomically. */
public final class PathMove {
  private final com.onepub.revault.internal.PathMove view;
  private final String source;
  private final String destination;

  /** Creates an application-owned PathMove value. */
  public PathMove(String source, String destination) {
    this.view = null;
    this.source = source;
    this.destination = destination;
  }

  PathMove(com.onepub.revault.internal.PathMove view) {
    this.view = java.util.Objects.requireNonNull(view);
    this.source = null;
    this.destination = null;
  }

  /** Existing variable name or form-record path to rename. */
  public String source() {
    return view == null ? source : view.source();
  }

  /** New variable name or form-record path. */
  public String destination() {
    return view == null ? destination : view.destination();
  }

}
