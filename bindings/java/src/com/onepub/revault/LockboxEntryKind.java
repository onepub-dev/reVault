package com.onepub.revault;

/** Identifies the filesystem object stored at a lockbox path. */
public enum LockboxEntryKind {
  /** No recognized kind was reported. */ UNSPECIFIED,
  /** A regular file. */ FILE,
  /** A symbolic link. */ SYMLINK,
  /** A directory. */ DIRECTORY
}
