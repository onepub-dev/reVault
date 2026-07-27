package com.onepub.revault;

/** A named structured record stored at a lockbox path and tied to a form-definition revision. */
public final class FormRecord {
  private final com.onepub.revault.internal.FormRecord view;
  private final String path;
  private final String name;
  private final String typeId;
  private final String definitionAlias;
  private final long definitionRevision;
  private final java.util.List<FormValue> values;

  /** Creates an application-owned FormRecord value. */
  public FormRecord(String path, String name, String typeId, String definitionAlias, long definitionRevision, java.util.List<FormValue> values) {
    this.view = null;
    this.path = path;
    this.name = name;
    this.typeId = typeId;
    this.definitionAlias = definitionAlias;
    this.definitionRevision = definitionRevision;
    this.values = values;
  }

  FormRecord(com.onepub.revault.internal.FormRecord view) {
    this.view = java.util.Objects.requireNonNull(view);
    this.path = null;
    this.name = null;
    this.typeId = null;
    this.definitionAlias = null;
    this.definitionRevision = 0;
    this.values = null;
  }

  /** Absolute lockbox path that identifies the record. */
  public String path() {
    return view == null ? path : view.path();
  }

  /** Human-readable name assigned to this record. */
  public String name() {
    return view == null ? name : view.name();
  }

  /** Stable identifier of the record's form type. */
  public String typeId() {
    return view == null ? typeId : view.typeId();
  }

  /** Alias of the form definition used by the record. */
  public String definitionAlias() {
    return view == null ? definitionAlias : view.definitionAlias();
  }

  /** Exact form revision against which the record was created. */
  public long definitionRevision() {
    return view == null ? definitionRevision : view.definitionRevision() & 0xffffffffL;
  }

  /** Ordered non-secret field metadata and values. */
  public java.util.List<FormValue> values() {
    if (view == null) return values;
    var result = new java.util.ArrayList<FormValue>(view.valuesLength());
    for (int index = 0; index < view.valuesLength(); index++) result.add(new FormValue(view.values(index)));
    return java.util.List.copyOf(result);
  }

}
