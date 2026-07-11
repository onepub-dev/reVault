# Lockbox Core Implementation Guide

This guide records implementation rules for `lockbox_core`. The normative
on-disk format is documented in [ARCHIVE_FORMAT.md](ARCHIVE_FORMAT.md).

## API Boundary

Public APIs should expose object-level operations such as files, directories,
symlinks, variables, forms, contacts, and recovery. They should not expose page
management, raw object bytes, or internal allocation details as normal caller
contracts.

Language bindings should preserve the same boundary. Bindings can wrap
`lockbox_core`, but they should not reimplement archive serialization.

The endian interop tests are part of the release gate. They verify that every
on-disk numeric field is encoded with an explicit byte order and does not depend
on native-endian struct layout.

## Page Cache Boundary

Normal TOC, file, variable, form, key-directory, recovery-output, and extraction
reads and writes should go through the page cache. The cache owns page decoding,
clear-text checksum validation, encrypted page authentication, body
decompression, object decoding, body encoding, compression, encryption, and
dirty page flushing.

Direct raw page decoding is reserved for:

- fixed-header bootstrap reads
- recovery scans of damaged archives
- format tests that deliberately validate low-level decoding

Higher-level code should not bypass the page cache for normal page reads or
writes. That rule keeps checksum, AEAD, compression, and object decoding
behavior consistent across features.

## Page Cache Responsibilities

The page cache should provide a single implementation path for:

- validating fixed page headers and header checksums
- validating clear-text key-directory pages
- authenticating encrypted page bodies
- decoding and caching page-body object streams
- encoding dirty pages before commit
- applying page-body compression and encryption
- tracking dirty pages until the commit root is published

The fixed header is the bootstrap exception because it is a fixed-location
structure rather than a page-body object. Key-directory pages should still use
the page read/decode path once their physical offsets are known.

The cache may retain decoded compression frames for read-heavy workloads. This
is an implementation optimization and does not change the archive format.

## Compression

Native zstd libraries may be used to produce or decode zstd frames, but native
zstd is an implementation detail. Archives must still store zstd frames with the
standard zstd algorithm id and the frames must remain readable as ordinary zstd
frames.

Compression should be opportunistic. If zstd output is not smaller than the
input, store the record uncompressed with the `none` algorithm id.

## Cache Modes

The cache should support different memory profiles without changing behavior:

- `Interactive`: keep recently used metadata and data pages decoded.
- `BulkImport`: keep hot metadata pages and flush file-data pages promptly.
- `Recovery`: keep minimal state and tolerate corrupt pages.
- `CacheLimit::Auto`: choose a conservative memory budget from available system
  memory.

Disabling or limiting the cache must not change archive correctness. It should
only affect performance.

## Commit And Compaction

Writes use copy-on-write page replacement. New or changed objects are written to
new pages, a new commit root is written, and the fixed header is updated last.

The normal commit flow should be:

1. Load the current roots through the page cache.
2. Apply the logical change to decoded file, symlink, variable, form, TOC, or
   key-directory state.
3. Write new file-data and metadata objects for changed content.
4. Rewrite touched BTree leaves and ancestors so the new roots reference the
   changed objects and unchanged subtrees by their existing page/object
   references.
5. Write any new key-directory primary and mirror copies, plus free-space index
   objects.
6. Write a new commit root with the next commit sequence and the previous
   commit-root reference.
7. Write commit-auth signatures for the new commit root.
8. Validate that the new root, commit-auth object, and any new key-directory
   references can be read back and authenticated.
9. Flush the newly written pages and update the fixed header last.

Updating a file should produce a new TOC root that references the touched TOC
leaf and changed ancestors. Unchanged TOC pages may remain referenced by the new
root and by previous commit roots until unreachable history is reclaimed.

Do not overwrite pages reachable from the current root while publishing a normal
logical update. Allocate from the free-space index only when the range is not
reachable from the current graph or any retained historical graph that must
remain valid. If reachability is uncertain, append new pages rather than
reusing old ranges.

If a write fails before the fixed header is updated, the old fixed header should
continue to select the old current view. If the fixed-header write is torn or
points at incomplete data, header checksums, page checksums, AEAD tags, and
commit-auth verification should cause readers to reject the partial commit and
fall back only through explicit recovery logic.

After the fixed header is updated, the implementation should flush the file and
directory metadata using the platform's durability primitives. The exact fsync
sequence is platform-specific, but the ordering requirement is not: publish data
and authenticated roots before publishing the fixed header that selects them.

Compaction is a logical rewrite. It builds a replacement archive from the
current reachable object graph, validates that replacement, and then atomically
replaces the original file. It is not in-place page defragmentation.

Zeroing freed pages or free regions can reduce recoverability from ordinary
storage, but it is not a cryptographic erasure guarantee on SSDs, filesystems
with snapshots, remote sync providers, or backup media.

## Variables And Forms

Variable and form updates should operate on the decoded current tree rather than
raw page scans. Current namespaces store only current entries. Deleted or
replaced secret values must not remain reachable from the current commit root.

A variable update should:

1. Load the current variable BTree through the page cache.
2. Decode the target leaf.
3. Apply the set or delete operation to the current entry set.
4. Rewrite the touched leaf and ancestors with copy-on-write pages.
5. Redact old pages that contained replaced or deleted secret values.
6. Publish a commit root that points at the new variable root.

Forms follow the same current-tree and redaction model for replaced records and
secret fields.

Form secret values should be materialized through `SecretString`; normal form
values can be materialized as ordinary strings. Updating a form definition
should create a new revision. Existing records keep the definition revision they
captured until they are explicitly updated.

## Key Directory Maintenance

Removing a password or contact should be treated as a confidentiality-changing
maintenance operation, not just a key-directory metadata edit.

The maintenance flow should:

1. Create a new content key or new wrapping generation.
2. Rewrite current reachable file, symlink, variable, form, TOC, and metadata
   pages under the new generation.
3. Write a new key directory without the removed slot.
4. Publish a commit root that references only the rewritten current graph and
   the new key-directory primary and mirror.
5. Redact old reachable copies where possible.

Old backups, filesystem snapshots, remote copies, and storage-level remnants may
still contain earlier encrypted pages or key directories. User-facing tooling
should describe this as key removal from the current lockbox, not guaranteed
erasure from every historical copy.

When writing a new key-directory generation, write exactly two copies: copy
index `0` for the primary and copy index `1` for the mirror. Initial creation
writes generation `1`; each later key-directory change increments the
generation by one.

The mirror allocator should prefer an existing free range or append position
that is at least 1 MiB away from the primary copy. It should not create an
artificial zero-filled gap only to force that distance, because that would
increase the minimum `.lbox` size. If no naturally separated range exists, write
the mirror using normal allocation.

If the fixed-header key-directory pointer is damaged, recovery can look for
valid clear-text key-directory copies by validating page checksums, grouping
decoded key-directory payloads by lockbox UUID, and selecting the highest
generation that successfully unwraps the content key.

## Recovery

Recovery scans may read raw pages after the fixed header because the fixed
header, TOC, free-space index, or key-directory pointer may be damaged. Once a
valid page is found, normal page validation rules still apply: clear-text pages
need valid checksums and encrypted pages need valid AEAD authentication.

Recovery does not require a valid fixed header, TOC, or free-space index. It may
authenticate and decrypt intact pages independently, locate valid commit roots,
rebuild a best-effort current view from the highest valid commit root, salvage
file objects whose metadata can still be associated with paths, and report
intact, corrupt, and lost counts.

If the fixed-header commit-root pointer is corrupt or stale, recovery can choose
the highest valid commit root by sequence as the root for the recovered view.

Recovered output should contain only complete, path-bearing entries. Recovery
should report partial or orphaned data rather than inventing placeholder names
or writing shortened files.

If metadata for a file survives but one or more referenced file-data records are
missing or corrupt, the file should be counted as partial and should not be
written as a shortened file. If file-data records survive but no valid TOC/index
metadata can associate them with a lockbox path and ordering, recovery should
report corruption or omit the orphaned content; it should not invent unnamed
placeholder files.

Recovery is not an undelete guarantee. Once freed pages or free regions have
been overwritten, the old objects are no longer recoverable.
