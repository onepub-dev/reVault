package com.onepub.revault;

/** The name and sensitivity classification of a variable stored in a lockbox. */
public final class Variable {
  private final com.onepub.revault.internal.Variable view;
  private final String name;
  private final String sensitivity;
  private final String description;
  private final boolean hasDescription;
  /** Creates a value without an item description. */
  public Variable(String name, String sensitivity) { this(name, sensitivity, "", false); }

  /** Creates an application-owned Variable value. */
  public Variable(String name, String sensitivity, String description, boolean hasDescription) {
    this.view = null;
    this.name = name;
    this.sensitivity = sensitivity;
    this.description = description;
    this.hasDescription = hasDescription;
  }

  Variable(com.onepub.revault.internal.Variable view) {
    this.view = java.util.Objects.requireNonNull(view);
    this.name = null;
    this.sensitivity = null;
    this.description = null;
    this.hasDescription = false;
  }

  /** Name used to address the variable in the lockbox. */
  public String name() {
    return view == null ? name : view.name();
  }

  /** Whether the value is ordinary text or a protected secret. */
  public String sensitivity() {
    return view == null ? sensitivity : view.sensitivity();
  }

  /** Multiline description stored with this variable. */
  public String description() {
    return view == null ? description : view.description();
  }

  /** Whether the variable description is present, including empty text. */
  public boolean hasDescription() {
    return view == null ? hasDescription : view.hasDescription();
  }

}
