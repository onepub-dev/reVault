package com.onepub.revault;

/** A lockbox key currently held by the local session agent, identified by lockbox and path. */
public final class AgentEntry {
  private final com.onepub.revault.internal.AgentEntry view;
  private final String id;
  private final String path;

  /** Creates an application-owned AgentEntry value. */
  public AgentEntry(String id, String path) {
    this.view = null;
    this.id = id;
    this.path = path;
  }

  AgentEntry(com.onepub.revault.internal.AgentEntry view) {
    this.view = java.util.Objects.requireNonNull(view);
    this.id = null;
    this.path = null;
  }

  /** Stable lockbox identifier for the cached key. */
  public String id() {
    return view == null ? id : view.id();
  }

  /** Host path associated with the cached lockbox key. */
  public String path() {
    return view == null ? path : view.path();
  }

}
