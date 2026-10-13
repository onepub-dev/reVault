package com.onepub.revault;

/** The host capabilities used to protect cached secrets across suspend and sleep. */
public final class SleepSupport {
  private final com.onepub.revault.internal.SleepSupport view;
  private final boolean suspendNotifications;
  private final boolean sleepInhibition;
  private final boolean supported;
  private final boolean cacheClearingActive;

  /** Creates an application-owned SleepSupport value. */
  public SleepSupport(boolean suspendNotifications, boolean sleepInhibition, boolean supported, boolean cacheClearingActive) {
    this.view = null;
    this.suspendNotifications = suspendNotifications;
    this.sleepInhibition = sleepInhibition;
    this.supported = supported;
    this.cacheClearingActive = cacheClearingActive;
  }

  SleepSupport(com.onepub.revault.internal.SleepSupport view) {
    this.view = java.util.Objects.requireNonNull(view);
    this.suspendNotifications = false;
    this.sleepInhibition = false;
    this.supported = false;
    this.cacheClearingActive = false;
  }

  /** Whether suspend notification support is compiled for this host. */
  public boolean suspendNotifications() {
    return view == null ? suspendNotifications : view.suspendNotifications();
  }

  /** Whether sleep inhibition support is compiled for this host. */
  public boolean sleepInhibition() {
    return view == null ? sleepInhibition : view.sleepInhibition();
  }

  /** Whether this build includes the host integration; this is not runtime watcher readiness. */
  public boolean supported() {
    return view == null ? supported : view.supported();
  }

  /** Whether the current compatible Session Agent reports an active suspend cache watcher; false when absent or unavailable. Notification delivery is platform-dependent. */
  public boolean cacheClearingActive() {
    return view == null ? cacheClearingActive : view.cacheClearingActive();
  }

}
