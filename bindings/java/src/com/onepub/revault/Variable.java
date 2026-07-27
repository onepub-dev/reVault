package com.onepub.revault;

/** The name and sensitivity classification of a variable stored in a lockbox. */
public final class Variable {
  private final com.onepub.revault.internal.Variable view;
  private final String name;
  private final String sensitivity;

  /** Creates an application-owned Variable value. */
  public Variable(String name, String sensitivity) {
    this.view = null;
    this.name = name;
    this.sensitivity = sensitivity;
  }

  Variable(com.onepub.revault.internal.Variable view) {
    this.view = java.util.Objects.requireNonNull(view);
    this.name = null;
    this.sensitivity = null;
  }

  /** Name used to address the variable in the lockbox. */
  public String name() {
    return view == null ? name : view.name();
  }

  /** Whether the value is ordinary text or a protected secret. */
  public String sensitivity() {
    return view == null ? sensitivity : view.sensitivity();
  }

}
