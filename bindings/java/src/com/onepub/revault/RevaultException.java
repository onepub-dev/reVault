package com.onepub.revault;

/** A native operation failure with stable structured diagnostics. */
public final class RevaultException extends RuntimeException {
  private static final long serialVersionUID = 1L;
  private final transient ErrorDetails details;

  RevaultException(String message, ErrorDetails details) {
    super(message);
    this.details = details;
  }

  /** Structured native error details, when the runtime supplied them. */
  public ErrorDetails details() { return details; }
}
