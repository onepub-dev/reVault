package com.onepub.revault;

/** A named recipient public key stored in the local vault address book. */
public final class Contact {
  private final com.onepub.revault.internal.Contact view;
  private final String name;
  private final byte[] key;

  /** Creates an application-owned Contact value. */
  public Contact(String name, byte[] key) {
    this.view = null;
    this.name = name;
    this.key = key;
  }

  Contact(com.onepub.revault.internal.Contact view) {
    this.view = java.util.Objects.requireNonNull(view);
    this.name = null;
    this.key = null;
  }

  /** Local address-book name of the contact. */
  public String name() {
    return view == null ? name : view.name();
  }

  /** Serialized contact public key used to grant lockbox access. */
  public byte[] key() {
    if (view == null) return key.clone();
    var result = new byte[view.keyLength()];
    for (int index = 0; index < result.length; index++) result[index] = (byte) view.key(index);
    return result;
  }

}
