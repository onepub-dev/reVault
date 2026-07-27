package com.onepub.revault;

/** Availability and configuration of the operating-system credential store used for the vault password. */
public final class PlatformStatus {
  private final com.onepub.revault.internal.PlatformStatus view;
  private final boolean supported;
  private final boolean disabled;
  private final String scope;
  private final String backend;
  private final String item;

  /** Creates an application-owned PlatformStatus value. */
  public PlatformStatus(boolean supported, boolean disabled, String scope, String backend, String item) {
    this.view = null;
    this.supported = supported;
    this.disabled = disabled;
    this.scope = scope;
    this.backend = backend;
    this.item = item;
  }

  PlatformStatus(com.onepub.revault.internal.PlatformStatus view) {
    this.view = java.util.Objects.requireNonNull(view);
    this.supported = false;
    this.disabled = false;
    this.scope = null;
    this.backend = null;
    this.item = null;
  }

  /** Whether a usable operating-system credential store exists. */
  public boolean supported() {
    return view == null ? supported : view.supported();
  }

  /** Whether the user disabled credential-store integration. */
  public boolean disabled() {
    return view == null ? disabled : view.disabled();
  }

  /** Application-specific scope used to isolate the stored password. */
  public String scope() {
    return view == null ? scope : view.scope();
  }

  /** Operating-system credential-store backend in use. */
  public String backend() {
    return view == null ? backend : view.backend();
  }

  /** Credential item name used by the backend. */
  public String item() {
    return view == null ? item : view.item();
  }

}
