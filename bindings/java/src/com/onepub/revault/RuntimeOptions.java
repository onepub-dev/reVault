package com.onepub.revault;

/** The workload and worker policies currently applied to an open lockbox. */
public final class RuntimeOptions {
  private final com.onepub.revault.internal.RuntimeOptions view;
  private final String workloadProfile;
  private final String workerPolicy;

  /** Creates an application-owned RuntimeOptions value. */
  public RuntimeOptions(String workloadProfile, String workerPolicy) {
    this.view = null;
    this.workloadProfile = workloadProfile;
    this.workerPolicy = workerPolicy;
  }

  RuntimeOptions(com.onepub.revault.internal.RuntimeOptions view) {
    this.view = java.util.Objects.requireNonNull(view);
    this.workloadProfile = null;
    this.workerPolicy = null;
  }

  /** I/O workload policy used to tune page access. */
  public String workloadProfile() {
    return view == null ? workloadProfile : view.workloadProfile();
  }

  /** Worker scheduling policy and effective parallelism. */
  public String workerPolicy() {
    return view == null ? workerPolicy : view.workerPolicy();
  }

}
