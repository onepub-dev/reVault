package com.onepub.revault;

/** The version, size, checksum, and creation time of an exported local-vault backup. */
public final class VaultBackupManifest {
  private final com.onepub.revault.internal.VaultBackupManifest view;
  private final long formatVersion;
  private final long createdAtUnixMs;
  private final String vaultFileName;
  private final long vaultSize;
  private final String vaultSha256;

  /** Creates an application-owned VaultBackupManifest value. */
  public VaultBackupManifest(long formatVersion, long createdAtUnixMs, String vaultFileName, long vaultSize, String vaultSha256) {
    this.view = null;
    this.formatVersion = formatVersion;
    this.createdAtUnixMs = createdAtUnixMs;
    this.vaultFileName = vaultFileName;
    this.vaultSize = vaultSize;
    this.vaultSha256 = vaultSha256;
  }

  VaultBackupManifest(com.onepub.revault.internal.VaultBackupManifest view) {
    this.view = java.util.Objects.requireNonNull(view);
    this.formatVersion = 0;
    this.createdAtUnixMs = 0;
    this.vaultFileName = null;
    this.vaultSize = 0;
    this.vaultSha256 = null;
  }

  /** Backup container format version. */
  public long formatVersion() {
    return view == null ? formatVersion : view.formatVersion() & 0xffffffffL;
  }

  /** Backup creation time in Unix milliseconds. */
  public long createdAtUnixMs() {
    return view == null ? createdAtUnixMs : view.createdAtUnixMs();
  }

  /** Metadata-vault filename stored in the backup. */
  public String vaultFileName() {
    return view == null ? vaultFileName : view.vaultFileName();
  }

  /** Encrypted vault payload size in bytes. */
  public long vaultSize() {
    return view == null ? vaultSize : view.vaultSize();
  }

  /** Lowercase SHA-256 digest of the encrypted vault payload. */
  public String vaultSha256() {
    return view == null ? vaultSha256 : view.vaultSha256();
  }

}
