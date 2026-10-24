package com.onepub.revault;

/** Public reciprocal exchange status. Verification is a local trust decision; no management capability or private key is returned. */
public final class ContactExchangeStatus {
  private final com.onepub.revault.internal.ContactExchangeStatus view;
  private final String id;
  private final String relay;
  private final String invitation;
  private final String inviterEmail;
  private final String recipientEmail;
  private final String fingerprint;
  private final long expiresMs;
  private final boolean accepted;
  private final boolean verified;
  private final String contactName;
  private final boolean relayPending;

  /** Creates an application-owned ContactExchangeStatus value. */
  public ContactExchangeStatus(String id, String relay, String invitation, String inviterEmail, String recipientEmail, String fingerprint, long expiresMs, boolean accepted, boolean verified, String contactName, boolean relayPending) {
    this.view = null;
    this.id = id;
    this.relay = relay;
    this.invitation = invitation;
    this.inviterEmail = inviterEmail;
    this.recipientEmail = recipientEmail;
    this.fingerprint = fingerprint;
    this.expiresMs = expiresMs;
    this.accepted = accepted;
    this.verified = verified;
    this.contactName = contactName;
    this.relayPending = relayPending;
  }

  ContactExchangeStatus(com.onepub.revault.internal.ContactExchangeStatus view) {
    this.view = java.util.Objects.requireNonNull(view);
    this.id = null;
    this.relay = null;
    this.invitation = null;
    this.inviterEmail = null;
    this.recipientEmail = null;
    this.fingerprint = null;
    this.expiresMs = 0;
    this.accepted = false;
    this.verified = false;
    this.contactName = null;
    this.relayPending = false;
  }

  /** Random identifier of the saved exchange. */
  public String id() {
    return view == null ? id : view.id();
  }

  /** Pinned HTTPS relay origin. */
  public String relay() {
    return view == null ? relay : view.relay();
  }

  /** Invitation URL to share only with the intended recipient. */
  public String invitation() {
    return view == null ? invitation : view.invitation();
  }

  /** Signed claimed inviter email, not independently verified by the relay. */
  public String inviterEmail() {
    return view == null ? inviterEmail : view.inviterEmail();
  }

  /** Signed claimed intended recipient email. */
  public String recipientEmail() {
    return view == null ? recipientEmail : view.recipientEmail();
  }

  /** Full shared fingerprint to compare independently; empty until both contributions are available. */
  public String fingerprint() {
    return view == null ? fingerprint : view.fingerprint();
  }

  /** Signed invitation expiry in Unix milliseconds. */
  public long expiresMs() {
    return view == null ? expiresMs : view.expiresMs();
  }

  /** Whether both signed public key bundles are locally pinned. */
  public boolean accepted() {
    return view == null ? accepted : view.accepted();
  }

  /** Whether this Vault independently verified and saved the contact. */
  public boolean verified() {
    return view == null ? verified : view.verified();
  }

  /** Local saved contact name, or empty before verification. */
  public String contactName() {
    return view == null ? contactName : view.contactName();
  }

  /** Whether the relay operation needs retrying; locally saved state is retained. */
  public boolean relayPending() {
    return view == null ? relayPending : view.relayPending();
  }

}
