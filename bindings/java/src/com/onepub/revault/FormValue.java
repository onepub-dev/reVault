package com.onepub.revault;

/** The current value and sensitivity metadata for one field in a stored form record. */
public final class FormValue {
  private final com.onepub.revault.internal.FormValue view;
  private final String fieldId;
  private final String label;
  private final String kind;
  private final String value;
  private final boolean secret;

  /** Creates an application-owned FormValue value. */
  public FormValue(String fieldId, String label, String kind, String value, boolean secret) {
    this.view = null;
    this.fieldId = fieldId;
    this.label = label;
    this.kind = kind;
    this.value = value;
    this.secret = secret;
  }

  FormValue(com.onepub.revault.internal.FormValue view) {
    this.view = java.util.Objects.requireNonNull(view);
    this.fieldId = null;
    this.label = null;
    this.kind = null;
    this.value = null;
    this.secret = false;
  }

  /** Identifier of the form field that owns this value. */
  public String fieldId() {
    return view == null ? fieldId : view.fieldId();
  }

  /** Display label captured from the form revision. */
  public String label() {
    return view == null ? label : view.label();
  }

  /** Field kind captured from the form revision. */
  public String kind() {
    return view == null ? kind : view.kind();
  }

  /** Plain value, or an empty string when the field is secret. */
  public String value() {
    return view == null ? value : view.value();
  }

  /** Whether the value must be read through a scoped secret callback. */
  public boolean secret() {
    return view == null ? secret : view.secret();
  }

}
