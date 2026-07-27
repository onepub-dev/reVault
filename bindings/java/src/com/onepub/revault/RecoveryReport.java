package com.onepub.revault;

/** The files and metadata recovered, or found damaged, while inspecting or salvaging a lockbox. */
public final class RecoveryReport {
  private final com.onepub.revault.internal.RecoveryReport view;
  private final java.util.List<LockboxEntry> intactFiles;
  private final long intactFileCount;
  private final long partialFiles;
  private final long corruptRecords;
  private final boolean tocRecovered;
  private final boolean variablesRecovered;
  private final long variableCount;
  private final boolean formsRecovered;
  private final long formDefinitionCount;
  private final long formRecordCount;

  /** Creates an application-owned RecoveryReport value. */
  public RecoveryReport(java.util.List<LockboxEntry> intactFiles, long intactFileCount, long partialFiles, long corruptRecords, boolean tocRecovered, boolean variablesRecovered, long variableCount, boolean formsRecovered, long formDefinitionCount, long formRecordCount) {
    this.view = null;
    this.intactFiles = intactFiles;
    this.intactFileCount = intactFileCount;
    this.partialFiles = partialFiles;
    this.corruptRecords = corruptRecords;
    this.tocRecovered = tocRecovered;
    this.variablesRecovered = variablesRecovered;
    this.variableCount = variableCount;
    this.formsRecovered = formsRecovered;
    this.formDefinitionCount = formDefinitionCount;
    this.formRecordCount = formRecordCount;
  }

  RecoveryReport(com.onepub.revault.internal.RecoveryReport view) {
    this.view = java.util.Objects.requireNonNull(view);
    this.intactFiles = null;
    this.intactFileCount = 0;
    this.partialFiles = 0;
    this.corruptRecords = 0;
    this.tocRecovered = false;
    this.variablesRecovered = false;
    this.variableCount = 0;
    this.formsRecovered = false;
    this.formDefinitionCount = 0;
    this.formRecordCount = 0;
  }

  /** Files whose complete contents remain recoverable. */
  public java.util.List<LockboxEntry> intactFiles() {
    if (view == null) return intactFiles;
    var result = new java.util.ArrayList<LockboxEntry>(view.intactFilesLength());
    for (int index = 0; index < view.intactFilesLength(); index++) result.add(new LockboxEntry(view.intactFiles(index)));
    return java.util.List.copyOf(result);
  }

  /** Number of completely recoverable files. */
  public long intactFileCount() {
    return view == null ? intactFileCount : view.intactFileCount();
  }

  /** Number of files for which only some content is recoverable. */
  public long partialFiles() {
    return view == null ? partialFiles : view.partialFiles();
  }

  /** Number of encrypted records that failed validation. */
  public long corruptRecords() {
    return view == null ? corruptRecords : view.corruptRecords();
  }

  /** Whether a usable table of contents was recovered. */
  public boolean tocRecovered() {
    return view == null ? tocRecovered : view.tocRecovered();
  }

  /** Whether variable metadata was recovered. */
  public boolean variablesRecovered() {
    return view == null ? variablesRecovered : view.variablesRecovered();
  }

  /** Number of recovered variables. */
  public long variableCount() {
    return view == null ? variableCount : view.variableCount();
  }

  /** Whether form definitions and records were recovered. */
  public boolean formsRecovered() {
    return view == null ? formsRecovered : view.formsRecovered();
  }

  /** Number of recovered form definitions. */
  public long formDefinitionCount() {
    return view == null ? formDefinitionCount : view.formDefinitionCount();
  }

  /** Number of recovered form records. */
  public long formRecordCount() {
    return view == null ? formRecordCount : view.formRecordCount();
  }

}
