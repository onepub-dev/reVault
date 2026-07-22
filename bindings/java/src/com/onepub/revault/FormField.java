package com.onepub.revault;

/** One named input in a reusable form definition, including its display label and sensitivity kind. */
public final class FormField {
  private final com.onepub.revault.internal.FormField view;
  private final String id;
  private final String label;
  private final String kind;
  private final boolean required;

  /** Creates an application-owned FormField value. */
  public FormField(String id, String label, String kind, boolean required) {
    this.view = null;
    this.id = id;
    this.label = label;
    this.kind = kind;
    this.required = required;
  }

  FormField(com.onepub.revault.internal.FormField view) {
    this.view = java.util.Objects.requireNonNull(view);
    this.id = null;
    this.label = null;
    this.kind = null;
    this.required = false;
  }

  /** Stable field identifier used when reading and writing records. */
  public String id() {
    return view == null ? id : view.id();
  }

  /** Human-readable label presented to a person entering data. */
  public String label() {
    return view == null ? label : view.label();
  }

  /** Field kind that determines validation and secret handling. */
  public String kind() {
    return view == null ? kind : view.kind();
  }

  /** Whether a record must provide a value for this field. */
  public boolean required() {
    return view == null ? required : view.required();
  }

}
