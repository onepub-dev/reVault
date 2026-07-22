package com.onepub.revault;

/** The host capabilities used to protect cached secrets across suspend and sleep. */
public final class SleepSupport {
  private final com.onepub.revault.internal.SleepSupport view;
  private final boolean suspendNotifications;
  private final boolean sleepInhibition;
  private final boolean supported;

  /** Creates an application-owned SleepSupport value. */
  public SleepSupport(boolean suspendNotifications, boolean sleepInhibition, boolean supported) {
    this.view = null;
    this.suspendNotifications = suspendNotifications;
    this.sleepInhibition = sleepInhibition;
    this.supported = supported;
  }

  SleepSupport(com.onepub.revault.internal.SleepSupport view) {
    this.view = java.util.Objects.requireNonNull(view);
    this.suspendNotifications = false;
    this.sleepInhibition = false;
    this.supported = false;
  }

  /** Whether the host reports impending system suspend. */
  public boolean suspendNotifications() {
    return view == null ? suspendNotifications : view.suspendNotifications();
  }

  /** Whether the agent can delay sleep while handling secrets. */
  public boolean sleepInhibition() {
    return view == null ? sleepInhibition : view.sleepInhibition();
  }

  /** Whether the host supplies enough integration for safe caching. */
  public boolean supported() {
    return view == null ? supported : view.supported();
  }

}
