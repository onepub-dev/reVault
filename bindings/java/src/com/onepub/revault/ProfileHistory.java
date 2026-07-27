package com.onepub.revault;

/** The active generation and rotation history for a named vault profile. */
public final class ProfileHistory {
  private final com.onepub.revault.internal.ProfileHistory view;
  private final String name;
  private final long activeGeneration;
  private final java.util.List<ProfileGeneration> generations;

  /** Creates an application-owned ProfileHistory value. */
  public ProfileHistory(String name, long activeGeneration, java.util.List<ProfileGeneration> generations) {
    this.view = null;
    this.name = name;
    this.activeGeneration = activeGeneration;
    this.generations = generations;
  }

  ProfileHistory(com.onepub.revault.internal.ProfileHistory view) {
    this.view = java.util.Objects.requireNonNull(view);
    this.name = null;
    this.activeGeneration = 0;
    this.generations = null;
  }

  /** Vault profile name whose generations are listed. */
  public String name() {
    return view == null ? name : view.name();
  }

  /** Generation number currently used for new access grants. */
  public long activeGeneration() {
    return view == null ? activeGeneration : view.activeGeneration() & 0xffffffffL;
  }

  /** Active and retired contact-key generations. */
  public java.util.List<ProfileGeneration> generations() {
    if (view == null) return generations;
    var result = new java.util.ArrayList<ProfileGeneration>(view.generationsLength());
    for (int index = 0; index < view.generationsLength(); index++) result.add(new ProfileGeneration(view.generations(index)));
    return java.util.List.copyOf(result);
  }

}
