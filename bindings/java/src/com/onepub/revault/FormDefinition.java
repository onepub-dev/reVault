package com.onepub.revault;

/** A versioned form schema used to validate and label structured records in a lockbox. */
public final class FormDefinition {
  private final com.onepub.revault.internal.FormDefinition view;
  private final String typeId;
  private final String alias;
  private final long revision;
  private final String name;
  private final String description;
  private final java.util.List<FormField> fields;

  /** Creates an application-owned FormDefinition value. */
  public FormDefinition(String typeId, String alias, long revision, String name, String description, java.util.List<FormField> fields) {
    this.view = null;
    this.typeId = typeId;
    this.alias = alias;
    this.revision = revision;
    this.name = name;
    this.description = description;
    this.fields = fields;
  }

  FormDefinition(com.onepub.revault.internal.FormDefinition view) {
    this.view = java.util.Objects.requireNonNull(view);
    this.typeId = null;
    this.alias = null;
    this.revision = 0;
    this.name = null;
    this.description = null;
    this.fields = null;
  }

  /** Stable identifier shared by every revision of this form type. */
  public String typeId() {
    return view == null ? typeId : view.typeId();
  }

  /** Short name used to resolve the current form revision. */
  public String alias() {
    return view == null ? alias : view.alias();
  }

  /** Monotonically increasing revision number. */
  public long revision() {
    return view == null ? revision : view.revision() & 0xffffffffL;
  }

  /** Human-readable name shown for this form. */
  public String name() {
    return view == null ? name : view.name();
  }

  /** Explanation shown to people completing the form. */
  public String description() {
    return view == null ? description : view.description();
  }

  /** Ordered inputs accepted by this form revision. */
  public java.util.List<FormField> fields() {
    if (view == null) return fields;
    var result = new java.util.ArrayList<FormField>(view.fieldsLength());
    for (int index = 0; index < view.fieldsLength(); index++) result.add(new FormField(view.fields(index)));
    return java.util.List.copyOf(result);
  }

}
