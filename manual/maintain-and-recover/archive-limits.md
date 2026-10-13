---
description: "Format-4 size, path, metadata and extraction limits, and what they mean in practice."
---

# Archive format and limits

This page describes the default format-4 archive used by the `0.5.x` compatibility line. An encoding limit, a writer limit and an extraction safety policy can impose different restrictions. Meeting one does not override the others. See [Versions and compatibility](../develop-with-revault/compatibility.md) before exchanging archives between releases.

Sizes below use binary units: 1 KiB = 1,024 bytes, 1 MiB = 1,048,576 bytes and 1 GiB = 1,073,741,824 bytes. Text limits count UTF-8 **bytes**, not characters.

## Archive and file capacity

File lengths and archive offsets use unsigned 64-bit fields. Files can span many pages and compression frames; an 8 MiB page limit is not an 8 MiB file limit. The encoding has no ZIP-style 4 GiB boundary.

The integer field width does not establish a supported maximum archive size. Filesystem limits, free disk space, addressable memory, metadata, the chosen API and operation policies still apply. This release does not claim that every archive up to the 64-bit encoding ceiling can be created or read.

There is no published, qualified maximum total file count. The goals of terabyte-scale files and millions of entries are design goals, not a statement that those scales have passed release qualification. The same distinction applies to the number of variables and forms. A low-memory result for one workload is not a guarantee for every archive.

For large files, use streaming or bounded-range APIs. An API such as `get_file` that returns the entire file needs enough addressable memory for that result, regardless of the archive's chunking. Browser and remote readers have [additional budgets](../develop-with-revault/external-sources.md#resource-limits-and-cancellation).

## Paths and stored metadata

| Item | Current limit or restriction |
| --- | --- |
| Full canonical archive path | 4,096 UTF-8 bytes, including separators and the leading `/`. |
| One path component | 255 UTF-8 bytes. |
| Path depth | 64 components, including the file name. |
| Variable name | 256 bytes for the canonical name, including its leading `/`; identifier syntax also applies. |
| Access-key slot name | 255 UTF-8 bytes. |
| Normal form values and form strings encoded with a 16-bit length | 65,535 UTF-8 bytes per string; field-specific validation and the complete-record limit can be lower. Secret form values use a different length encoding but still share the record limit below. |
| Variable or form metadata leaf | The current writer allows 124 KiB of encoded leaf contents. A complete variable entry, form definition or form record must fit, including names, labels, other fields and encoding overhead. |
| Stored permission bits | Unix-style `000`–`777`; special permission bits are rejected. |

A value passing the 1 MiB variable/form text validator is **not** a promise that it can be committed. The encoded metadata leaf has the smaller limit above. In particular, the usable value length of a single variable is less than 124 KiB after its name and framing are included. Store larger text as a file.

Paths use `/` on every platform. Empty components, `.` and `..`, backslashes, colons and prohibited control characters are rejected. The destination filesystem can impose additional naming and path limits during extraction.

The access-key directory must also fit its 128 KiB metadata page, including page and object framing. The number of recipients depends on their slot encoding and names; there is no unlimited-recipient guarantee.

## Practical metadata scaling

The current reader loads the complete file table of contents when opening an archive. It retains a path map and tree structures, including copies of leaf entries. Resident memory therefore grows with the number of entries, path lengths, and chunk and segment descriptors. Paged storage on disk does not imply constant-memory metadata access.

Variables are loaded lazily, but their first access decodes the whole variable tree. There is no qualified maximum metadata entry count or universal 100 MB memory guarantee. Workloads with many tiny files, long paths or many chunks per file need their own qualification. The per-record limits above constrain individual records; they do not bound total resident metadata. Further capacity and memory qualification is tracked in [issue #330](https://github.com/onepub-dev/reVault/issues/330).

## Pages and compression

| Unit | Maximum |
| --- | --- |
| Physical metadata page | 128 KiB. |
| Physical file-data page | 8 MiB. |
| Decoded contents of one file compression frame | 4 MiB. |

These limits apply to individual storage units. Larger files are split across units. Reading a small range inside a compressed frame can require decoding the whole frame. Compression is recorded in the archive; a reader does not guess whether a frame is compressed.

## Extraction safety limits

The current CLI uses these defaults when extracting to the filesystem:

| Policy | Default |
| --- | --- |
| One extracted file | 1 GiB. |
| Total file bytes in one extraction | 10 GiB. |
| Selected entries in one extraction | 100,000, including directory and symlink records. |

These are operation limits, not archive capacity limits. The current CLI does not expose options to raise these three numeric limits. Applications using the Rust API can supply an explicit `ExtractPolicy`; binding APIs that expose an extraction policy can supply their corresponding limits. Raising them requires enough memory, disk space and an appropriate destination filesystem. Proposed CLI overrides are tracked separately in [issue #331](https://github.com/onepub-dev/reVault/issues/331).

Stored permissions are restored by the CLI by default where supported. Symlink restoration requires `--restore-symlinks`, and replacing existing destinations requires `--overwrite`. See [File operations](../get-started/cli-tooling/file-operations.md). Ordinary CLI add and mirror import currently store file mode `0600`, so restoring that stored mode does not necessarily reproduce the source host's mode.

## Large deletion and replacement transactions

One redaction transaction is limited to 65,536 physical ranges, 262,144 object IDs and 2^50 bytes (1 PiB) of redaction work. These are transaction limits, not a supported archive-size rating. Large bulk changes may need smaller batches and separate commits.

Deletion and redaction affect the current archive file. They cannot erase another copy, a backup or a filesystem snapshot. See [Transactions and recovery](../develop-with-revault/transactions.md) and [Reclaim Lockbox space](compaction.md).

The [format-4 specification](https://github.com/onepub-dev/reVault/blob/7cb57d5667c6308b0ee91793f195b84db5378800/rust/revault_lockbox_api/ARCHIVE_FORMAT.md) describes the detailed encodings. Experimental archive layouts are outside this page's scope.
