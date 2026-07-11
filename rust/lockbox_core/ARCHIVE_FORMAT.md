# Lockbox Archive Format

This document records the intended pre-1.0 production format of reVault
Lockboxes. All on-disk numeric fields are little-endian unless stated otherwise.

On-disk structures are field encodings, not native memory layouts. Native-endian
conversions, memory transmutation, and raw struct layout are not part of the
format. Every numeric field uses the explicit byte order stated by this
document.

The physical unit of the lockbox is a variable-size page rounded to a 1 KiB
quantum and selected from a small set of page classes. Metadata pages may grow
up to 128 KiB. File-data pages may grow up to 8 MiB. Higher level structures
such as TOC nodes, file chunks, variables, forms, key directories, free-space
indexes, commit roots, and commit-auth records are encoded as objects inside
pages.

At a high level, a lockbox file is shaped like this:

```text
secrets.lbox
|-- fixed header
|   |-- latest commit-root page offset
|   |-- latest key-directory page offset
|   |-- latest commit-auth page offset
|   `-- public lockbox UUID
|
|-- clear-text key-directory pages
|   |-- password slot
|   |   |-- Argon2id salt and parameters
|   |   `-- encrypted content key
|   `-- contact slot
|       |-- ML-KEM-768 encapsulation ciphertext
|       `-- encrypted content key
|
`-- encrypted page area
    |-- commit root
    |   |-- TOC root reference
    |   |-- variable root reference
    |   |-- form root reference
    |   |-- free-space root reference
    |   `-- key-directory mirror references
    |
    |-- commit auth
    |   `-- owner signatures for the published commit root
    |
    |-- TOC pages
    |   `-- current file and symlink entries
    |
    |-- variable pages
    |   `-- current variable entries
    |
    |-- form pages
    |   `-- current form definitions and form records
    |
    |-- file-data pages
    |   `-- encrypted file chunks and symlink metadata
    |
    `-- free-space index pages
        `-- reusable physical ranges
```

The ordering of pages can change over time because current roots point at the
latest committed object graph. Not all encrypted page types need to be present
except for the commit root.

## Common Encodings

Unless a field table says otherwise, integer fields are little-endian fixed
width fields.

Some record layouts use a 16-bit unsigned little-endian length-prefixed UTF-8
string:

```text
offset  size  field
0       2     UTF-8 byte length as unsigned little-endian integer
2       n     UTF-8 bytes
```

Some record layouts use a 32-bit unsigned little-endian length-prefixed byte
string:

```text
offset  size  field
0       4     byte length as unsigned little-endian integer
4       n     bytes
```

`varint` is an unsigned 64-bit LEB128-style integer. Each byte contributes
seven payload bits. The high bit is set when another byte follows. Encodings
longer than 10 bytes are invalid.

`page-tree children` means:

```text
offset  size  field
0       4     child count
4       n     child entries
```

Each child entry is:

```text
offset  size  field
0       n     first key as 16-bit unsigned little-endian length-prefixed UTF-8 string
n       8     child page offset
```

## Cryptography, Checksums, Signing, And Compression

The archive has four distinct protection layers:

- public checksums for fixed-location and clear-text structures
- authenticated page encryption for private archive content
- key-directory wrapping for the random lockbox content key
- owner signatures over the published commit root

Checksums are corruption detectors, not secret authenticators. Encrypted pages
and wrapped keys use AEAD authentication. Commit authenticity is provided by the
commit-auth signatures.

### Public Checksums

All archive SHA-256 checksums use this domain-separated checksum function:

```text
checksum(data) =
  SHA-256(
    "lockbox-v1-public-checksum/sha256" ||
    u64_le(len(data)) ||
    data
  )
```

This checksum is used for:

- fixed header bytes `0..64`
- page header bytes `0..64`
- clear-text page bodies, stored as `checksum(body) || body`
- compression-frame digests over the stored compressed frame bytes
- commit-root digests stored in commit-auth payloads
- commit-auth payload digests used for commit-auth chaining

Public checksums detect torn writes, malformed records, and accidental
corruption. They do not make public bytes confidential and do not replace AEAD
authentication for encrypted content.

### Content Key And Page Encryption

A password-protected or contact-protected lockbox has a random 32-byte content
key. The content key itself is stored only after wrapping in key-directory
slots. Direct content-key lockboxes use supplied high-entropy content-key bytes
and do not need a key-directory slot.

Encrypted page bodies use ChaCha20-Poly1305. The 32-byte page AEAD key is
derived from the content key as:

```text
page_key =
  SHA-256(
    "lockbox-v1-content-key/chacha20poly1305" ||
    u64_le(len(content_key)) ||
    content_key
  )
```

Each encrypted page write stores a fresh 96-bit nonce in the page header. The
nonce must be unique for the content key. The stored encrypted body is:

```text
ChaCha20-Poly1305 ciphertext || 16-byte tag
```

Page AEAD associated data binds the ciphertext to the public page identity:

- domain string `LBX1PAGE`
- page header version
- lockbox UUID
- page id
- commit sequence
- public flags
- stored body length

Clear-text pages are limited to public page classes, currently the key
directory. They set the clear-text page flag, use a zero nonce, and store
`checksum(body) || body`.

### Key Directory Wrapping

The key directory is public metadata. It stores key slots that can unwrap the
lockbox content key, but it must not store contact identities, file metadata,
variable names, form data, or private content.

Password slots use:

- 16-byte random salt
- Argon2id version `0x13`
- memory cost 64 MiB
- 3 iterations
- parallelism 1
- 32-byte output key
- ChaCha20-Poly1305 wrapping of the content key with a zero nonce

Contact slots use hybrid public-key wrapping:

- X25519 ephemeral-static Diffie-Hellman
- ML-KEM-768 encapsulation
- HKDF-SHA-256 with salt `"lockbox-v1-hybrid-contact-wrap"`
- HKDF info `"x25519-mlkem768-chacha20poly1305"`
- ChaCha20-Poly1305 wrapping of the content key with a zero nonce

The zero nonce is valid for key wrapping because each wrapping key is unique to
one password slot salt or one freshly generated contact encapsulation. The nonce
must not be reused with the same wrapping key for different plaintexts.

### Commit Signing

Commit authentication is stored as an encrypted page object. Its signed message
is:

```text
"lockbox-v1-commit-auth" || signed_commit_auth_fields
```

The signed fields include the lockbox UUID, commit sequence, commit-root page
offset, commit-root checksum, previous commit-auth offset and checksum, commit
flags, and owner public-key headers.

Current owner commits must carry both signatures:

- Ed25519
- ML-DSA-65

A commit is treated as owner-signed only when the commit-root checksum matches
the encoded commit root and both signatures verify over the signed message.

### Compression

Lockbox uses zstd for both metadata page-body compression and file compression
frames when compression is selected. Page sizes and compression-frame expansion
limits are separate limits: an 8 MiB file-data page is a physical container,
while the 4 MiB limit applies to the decompressed payload of each individual
file compression frame stored in those containers.

Page body compression records use these algorithm ids inside the encrypted page
body:

```text
0  none
1  zstd
```

File compression-frame records use these algorithm ids:

```text
0  none
1  zstd
```

Algorithm `0` stores raw uncompressed bytes. Algorithm `1` stores a standard
zstd frame. Page-body decompression is bounded by the maximum page logical size.
File compression-frame decompression is bounded by 4 MiB per frame, regardless
of the physical file-data page size, and declared zstd content sizes must match
the stored expected length when present.

## Fixed Header

The fixed header is 96 bytes and is the only mutable fixed-location structure in
the file.

```text
offset  size  field
0       8     magic: "LBX1HDR\0"
8       2     version: 1
10      2     header flags
12      4     header length as 32-bit unsigned little-endian integer: 96
16      8     latest commit-root page offset, or 0
24      8     latest commit sequence
32      8     latest public key-directory offset, or 0
40      16    public lockbox UUID
56      8     latest commit-auth page offset, or 0
64      32    checksum over bytes 0..64
```

The lockbox UUID is public metadata. It identifies a lockbox even if the file is
renamed or moved. It must not be derived from file names, content, contacts, or
passwords.

The header checksum is the domain-separated public checksum. It detects torn or
malformed header updates. It is not a security boundary. Security decisions must
be based on authenticated pages and authenticated commit roots.

The key-directory pointer remains in the fixed header because users need the key
directory before the content key is opened. The key directory stores only
open metadata and must not contain private file metadata.

The commit-auth pointer is public bootstrap metadata. The pointed-to object is
encrypted and authenticated as a normal page object, and it signs the commit
root referenced by the same header.

## Commit Model And Update Atomicity

The fixed header selects the current view by naming the latest commit-root page,
latest commit sequence, latest key-directory page, and latest commit-auth page.
The current logical archive is the object graph reachable from that commit root.

Logical updates use copy-on-write at the page/object graph level. A new current
view is published by writing replacement file-data, metadata, TOC, variable,
form, free-space, key-directory, commit-auth, and commit-root objects as needed,
then updating the fixed header to point at the new commit root. The fixed header
is the only fixed-location structure that is expected to change in place during
normal commits.

Pages and objects reachable from the current commit root must not be overwritten
to publish a different current view. They can become reusable only after they
are no longer reachable from the current graph or any retained historical graph
that the archive still treats as valid. Confidentiality-changing operations may
redact or rewrite old copies; after that, affected historical roots are not a
format-level retention guarantee.

A commit is current only when the fixed header checksum is valid, the referenced
commit root can be authenticated and decoded, and the referenced commit-auth
object signs that commit root. Partially written pages, commit roots, commit-auth
objects, or key-directory copies that are not selected by a valid fixed header
do not define the current view.

Commit sequence numbers order commit roots that are otherwise valid. They do
not override the current view selected by a valid fixed header.

## Pages

Every page has a variable physical size rounded to a 1 KiB quantum. Metadata
pages may grow up to 128 KiB. File-data pages may grow up to 8 MiB. The
physical page size is the smallest page class that fits the encoded page body,
so typical pages are at least 95% full unless the page is extremely small. Page
headers expose the stored body length so an intact page can be authenticated
without requiring trailing padding.

```text
offset  size  field
0       8     magic: "LBX1PAG\0"
8       2     page header version: 1
10      2     public flags
12      4     header length as 32-bit unsigned little-endian integer
16      8     page id
24      8     commit sequence that wrote this page
32      12    AEAD nonce, or zero for clear-text pages
44      4     stored body length as 32-bit unsigned little-endian integer
48      16    reserved header extension
64      32    checksum over bytes 0..64
H       m     stored body
H+m     p     zero padding to the physical page size
```

The nonce is generated per page write and must be unique for the content key. It
must not be derived only from record kind, object kind, or commit sequence.

Most page bodies are encrypted with ChaCha20-Poly1305. Encrypted pages store
the AEAD nonce in the page header and the ciphertext plus authentication tag in
the stored body. The AEAD tag authenticates the page body and the associated
data listed below.

Some page classes are clear-text pages. Clear-text pages set public flag
`0x0001`, store a zero nonce, and store `checksum(body) || body` as the stored
body. The key directory is currently a clear-text page class because the keys
must be read before the content key is available for decryption. For clear-text
pages, object headers and payloads are public and must not contain metadata that
must remain private to avoid leakage.

For encrypted pages, only the page header is public. Object kinds, object
lengths, logical paths, symlink targets, variable names,
permissions, compression selection, and file contents are inside the encrypted
body.

Page AEAD associated data includes:

- domain string `LBX1PAGE`
- page header version
- lockbox UUID
- page id
- commit sequence
- public flags
- stored body length

## Page Body

The decoded page body is an object container. For encrypted pages this is the
decrypted AEAD plaintext. For clear-text pages this is the checksummed body
after the checksum prefix has been validated and removed.

```text
offset  size  field
0       1     page body version: 1
1       1     page-body compression mode: 0 = none, 1 = compression record
2       2     reserved
4       8     uncompressed object-stream length as 64-bit unsigned little-endian integer
12      4     reserved
16      n     object stream, or a compression record when mode is 1
```

When the page-body compression mode is `1`, bytes from offset `16` contain:

```text
offset  size  field
0       8     uncompressed object-stream length as 64-bit unsigned little-endian integer
8       1     compression algorithm: 0 = none, 1 = zstd
9       8     stored compressed or uncompressed length as 64-bit unsigned little-endian integer
17      n     stored bytes
```

Valid page-body compression modes are:

- metadata pages may use mode `0` or mode `1`
- file-data pages use mode `0` because file bytes are already compressed at the
  compression-frame boundary

## Compression-Framed File Extents

The production file-data layout is page-packed framed extents, not one file
object per physical page. A file-data page is a variable-size physical
container capped at 8 MiB. That 8 MiB cap limits the encrypted page body, not
the decompressed size of any one compression frame. Its encrypted body may
contain:

- many complete small files
- many chunks from one or more files
- one segment of a large compression frame
- a mix of complete chunks and segments, as long as the page body fits

File bytes are grouped into independent bounded compression frames. A
compression frame can hold one large-file extent or many small files. The
uncompressed compression-frame payload is the concatenation of file slices and
is capped independently at 4 MiB per frame. The compressed frame bytes may be
split into one or more page segments and those segments may be packed into
8 MiB file-data pages with other extents. File-data pages do not apply a second
page-body compression layer. Compression frames whose declared decompressed
length exceeds the format safety limit, currently 4 MiB, are invalid.

The compression-frame manifest is a schema-specific binary structure, not a generic
compressed blob. Its payload is:

```text
offset  size  field
0       4     magic: "LBFM"
4       1     manifest version: 1
5       n     compression-frame id as varint
...     n     compression algorithm as varint
...     n     uncompressed compression-frame length as varint
...     n     compressed compression-frame length as varint
...     32    checksum over the compressed frame bytes
...     n     slice count as varint
...     n     slice records
```

Each manifest slice record is:

```text
offset  size  field
0       n     path byte length as varint
...     n     path UTF-8 bytes
...     n     permissions as varint
...     n     total file length as varint, or 0 when unknown
...     n     file offset as varint
...     n     compression-frame offset as varint
...     n     slice length as varint
```

Each file-data segment stores compact compression-frame identity metadata
followed by the segment's compressed bytes. The first segment in a compression
frame also stores the compression-frame manifest. The manifest records its
compression algorithm using the same file compression-frame algorithm ids. The
manifest therefore travels with the compression-frame data once as a
self-contained slice index, without repeating paths on every segment. TOC chunk
entries contain the file index: logical file offset, slice length,
compression-frame offset, compression-frame length, compression algorithm,
compression-frame id, compression-frame digest, and ordered physical page
segments needed to reassemble the compressed compression-frame. Each physical
segment reference contains the page offset, physical page length, encrypted
object id, stored compression-frame offset, and segment length.

Each file-data segment object payload is:

```text
offset  size  field
0       4     magic: "LBCS"
4       1     segment version: 1
5       n     compression-frame id as varint
...     n     compression algorithm as varint
...     n     uncompressed compression-frame length as varint
...     n     compressed compression-frame length as varint
...     32    checksum over the compressed frame bytes
...     n     uncompressed manifest length as varint, or 0 when absent
...     n     manifest compression algorithm as varint
...     n     stored manifest length as varint
...     n     stored manifest bytes
...     n     segment offset in compressed frame as varint
...     n     segment byte length as varint
...     n     segment bytes
```

This layout supports random access at compression-frame granularity. TOC chunk
entries identify the physical page segments needed for a logical file range, and
each referenced page segment is independently authenticated before its bytes can
contribute to a reconstructed compression frame.

A complete compression frame has exactly one manifest-bearing segment, complete
segment coverage, and a matching compression-frame digest. A slice record may
store `0` for the total file length when the final length was not known when the
compression frame was created; the TOC is authoritative when available.

Paths remain private because both TOC entries and segment metadata are inside
encrypted page bodies. They are exposed only after the content key is available.

## Objects

The object stream contains typed objects. Object headers are private on
encrypted pages because they are part of the encrypted page body. Object headers
are public on clear-text pages, so clear-text page classes must be limited to
public metadata.

Object stream:

```text
offset  size  field
0       4     object count
4       n     object entries
```

Each object entry is:

```text
offset  size  field
0       1     object kind
1       1     object header version
2       2     object flags
4       8     object id
12      8     object payload length as 64-bit unsigned little-endian integer
20      n     object payload
```

Object ids are stable references used by TOC entries and indexes. A logical file
may reference one or more file-data objects. Multiple small logical files may be
packed into one compression frame, and that compression frame may be split across
multiple file-data objects.

Symlinks are current TOC entries that reference symlink metadata objects. The current
TOC stores the symlink node kind plus the metadata page offset, object length,
and object id. It does not store the symlink target. The target is stored only
inside the referenced symlink object, and many symlink objects may be packed into
one metadata page.

Symlink object payload:

```text
offset  size  field
0       n     symlink path as 16-bit unsigned little-endian length-prefixed UTF-8 string
n       m     symlink target path as 16-bit unsigned little-endian length-prefixed UTF-8 string
```

Object kinds:

```text
1       commit root
2       TOC leaf node
3       TOC internal node
4       file data
5       reserved legacy packed file data; not emitted by the current format
6       symlink
7       reserved legacy variable set; not emitted by the current format
8       reserved legacy variable delete; not emitted by the current format
9       key directory
10      free-space index leaf
11      free-space index internal
12      reserved legacy delete marker; not emitted by the current format
13      variable leaf node
14      variable internal node
15      form leaf node
16      form internal node
17      commit auth
```

## Commit Root

The fixed header points to the latest commit-root page offset. The
commit root is an encrypted object inside that page.

The commit root payload contains:

```text
offset  size  field
0       1     commit-root payload version: 1
1       7     reserved
8       8     commit sequence
16      8     TOC root object reference
24      8     variable root object reference, or zero
32      8     form root object reference, or zero
40      8     free-space index root object reference
48      8     primary key-directory offset, or zero
56      8     key-directory mirror offset, or zero
64      8     key-directory generation
72      8     previous commit-root reference, or zero
80      8     commit flags
```

A valid current view is rooted at the commit-root page referenced by the fixed
header. Commit sequence numbers order commit roots that are otherwise valid.

## Commit Authentication

Commit authentication is an encrypted `commit auth` object referenced by the
fixed header. Its payload starts with magic `LBX1AUTH`, version `1`, and signed
fields:

```text
offset  size  field
0       8     magic: "LBX1AUTH"
8       1     commit-auth version: 1
9       7     reserved
16      16    lockbox UUID
32      8     commit sequence
40      8     commit-root page offset
48      32    checksum of the encoded commit root
80      8     previous commit-auth offset, or 0
88      32    previous commit-auth checksum, or zeroes
120     8     flags
128     4     owner public-key header count
132     n     owner public-key headers
...     4     signature count
...     n     signatures
```

Each owner public-key header is:

```text
offset  size  field
0       2     signature algorithm
2       4     public-key byte length as 32-bit unsigned little-endian integer
6       n     public-key bytes
```

Each signature entry repeats the algorithm and public key and then stores the
signature bytes:

```text
offset  size  field
0       2     signature algorithm
2       4     public-key byte length as 32-bit unsigned little-endian integer
6       n     public-key bytes
6+n     4     signature byte length as 32-bit unsigned little-endian integer
10+n    m     signature bytes
```

Signature algorithm ids:

```text
1  Ed25519
2  ML-DSA-65
```

The signature count must equal the owner public-key header count, and each
signature entry must repeat the matching algorithm and public key. Current owner
commits carry both Ed25519 and ML-DSA-65 signatures. A signed owner inspection
is valid only when the commit-root digest matches and both signatures verify.

Rollback attacks on a standalone copied file cannot be fully prevented without an
external freshness anchor. Lockbox detects internal corruption; it cannot prove
that an attacker has not replaced the entire file with an older valid copy.

An external freshness anchor is state outside the lockbox that records the
latest known version, generation, hash, or signed timestamp. Examples include a
server-side object generation number, transparency log entry, signed TOC,
append-only audit log, or application database row. Lockbox can reject stale
internal metadata within one file by choosing the highest authenticated
generation; only an external anchor can detect replacement of the whole file
with an older but internally valid lockbox.

## Table Of Contents

The TOC is a current-entry BTree. Tombstones are not stored in the current TOC.
Deletes remove entries from the current TOC. Deleted files and symlinks are
absent from the current TOC after their metadata has been redacted.

TOC leaf and internal object payloads start with:

```text
offset  size  field
0       1     TOC node version: 1
1       1     node kind: 1 = leaf, 2 = internal
```

TOC internal payloads store `page-tree children` from offset `2`. The child key
is the first logical path in the child subtree.

TOC leaf payloads store this structure from offset `2`:

```text
offset  size  field
0       n     entry count as varint
...     n     frame descriptor count as varint
...     n     frame descriptors
...     n     TOC entries
```

Each frame descriptor is:

```text
offset  size  field
0       n     compression-frame id as varint
...     n     compression algorithm as varint
...     n     uncompressed compression-frame length as varint
...     n     compressed compression-frame length as varint
...     32    compression-frame digest
...     n     segment count as varint
...     n     segment records
```

Each frame descriptor segment record is:

```text
offset  size  field
0       n     page offset as varint
...     n     physical page length as varint
...     n     encrypted object id as varint
...     n     segment offset in compressed frame as varint
...     n     segment byte length as varint
```

Each TOC entry is:

```text
offset  size  field
0       1     flags
1       n     front-coded logical path
...     n     logical length as varint
...     n     metadata object page offset as varint, or 0
...     n     metadata object length as varint, or 0
...     n     metadata object id as varint, or 0
...     n     permissions as varint, only when custom-permissions flag is set
...     n     chunk count as varint
...     n     chunk records
```

TOC entry flags:

```text
0x01  deleted
0x02  symlink
0x04  custom permissions
```

Every 128th path is a restart path encoded as:

```text
offset  size  field
0       n     path byte length as varint
...     n     path UTF-8 bytes
```

Other paths are front-coded against the previous path:

```text
offset  size  field
0       n     shared prefix byte length as varint
...     n     suffix byte length as varint
...     n     suffix UTF-8 bytes
```

Each TOC chunk record is:

```text
offset  size  field
0       n     stored path byte length as varint, or 0 to reuse entry path
...     n     stored path UTF-8 bytes when length is nonzero
...     n     logical file offset as varint
...     n     logical chunk length as varint
...     n     compression-frame offset as varint
...     n     frame descriptor index as varint
```

Decode rules are intentionally strict:

- leaf entries must be strictly sorted by logical path
- duplicate leaf paths are treated as corrupt
- internal children must be strictly sorted by separator path
- duplicate separators are treated as corrupt
- child references must resolve to valid TOC objects
- every stored path must pass logical path validation
- missing or corrupt child objects make the TOC corrupt

Previous commit roots may reference older TOC pages. The current TOC is the TOC
reachable from the latest valid commit root.

## Variables

Variables are encrypted metadata, not files. They must not appear in
file listings, file recovery listings, visualizations, or unauthenticated public
metadata.

The committed variable namespace stores only current entries, like the TOC. It
must contain only the current value for each variable name. Old variable values
are secret material and must not remain decryptable in old historical pages
after variable replacement, variable deletion, or contact changes are committed.

The variable structure is commit-root referenced. The commit root points to a
variable root object, or zero when no variables exist. Variable entries are
packed into encrypted variable leaf pages so many small variables share a page.
Variable internal pages contain only sorted routing names and child page
offsets; variable values exist only in variable leaves.

Variable leaf and internal object payloads start with:

```text
offset  size  field
0       1     variable node version: 1
1       1     node kind: 1 = leaf, 2 = internal
```

Variable internal payloads store `page-tree children` from offset `2`. The
child key is the first internal variable name in the child subtree.

Variable leaf payloads store this structure from offset `2`:

```text
offset  size  field
0       4     entry count
4       n     variable entries
```

Each variable entry is:

```text
offset  size  field
0       n     stored variable name as 16-bit unsigned little-endian length-prefixed UTF-8 string
n       1     sensitivity tag: 0 = normal, 1 = secret
n+1     4     value byte length as 32-bit unsigned little-endian integer
n+5     m     UTF-8 value bytes
```

Stored variable names are sorted. Current normal variables use the `.plain/`
prefix and current secret variables use the `.secret/` prefix. The sensitivity
tag must match the stored-name prefix.

The archive format uses a root-referenced variable BTree containing only current
entries.

The format contains no variable-history records; only current variable BTree
entries are valid.

## Forms

Forms are encrypted structured metadata, not files. They are stored under a
commit-root referenced form root object, or zero when no forms exist.

The form tree stores two classes of current entries:

- form definitions keyed by form type id and revision
- form records keyed by lockbox path

Form leaf and internal object payloads start with:

```text
offset  size  field
0       1     form node version: 1
1       1     node kind: 1 = leaf, 2 = internal
```

Form internal payloads store `page-tree children` from offset `2`. The child key
is the first form tree key in the child subtree.

Form leaf payloads store this structure from offset `2`:

```text
offset  size  field
0       4     entry count
4       n     form entries
```

Each form entry is:

```text
offset  size  field
0       n     form tree key as 16-bit unsigned little-endian length-prefixed UTF-8 string
n       1     entry kind: 1 = definition, 2 = record
n+1     m     definition or record payload
```

Definition keys are `d/<form-type-id>/<revision>`, where revision is encoded as
10 ASCII decimal digits. Record keys are `r<lockbox-path>`.

Form definition payload:

```text
offset  size  field
0       n     form type id as 16-bit unsigned little-endian length-prefixed UTF-8 string
n       m     alias as 16-bit unsigned little-endian length-prefixed UTF-8 string
...     4     revision
...     n     display name as 16-bit unsigned little-endian length-prefixed UTF-8 string
...     2     optional description marker: 0xffff when a description follows
...     n     description as 16-bit unsigned little-endian length-prefixed UTF-8 string
...     4     field count
...     n     field definitions
```

Form definitions written before descriptions omit the marker and description; in
that case the field count follows the display name directly.

Each field definition is:

```text
offset  size  field
0       n     field id as 16-bit unsigned little-endian length-prefixed UTF-8 string
n       m     label as 16-bit unsigned little-endian length-prefixed UTF-8 string
...     1     field kind
...     1     required flag: 0 = false, 1 = true
```

Each field definition contains a field id, label, kind, and required flag.
Supported field kinds are text, secret, URL, email, date, month, notes, and
number. Date values are stored as `YYYY-MM-DD`. Month values are year-month
values stored as `YYYY-MM`.

Field kind ids:

```text
1  text
2  secret
3  URL
4  email
5  date
6  month
7  notes
8  number
```

Form record payload:

```text
offset  size  field
0       n     record lockbox path as 16-bit unsigned little-endian length-prefixed UTF-8 string
n       m     record name as 16-bit unsigned little-endian length-prefixed UTF-8 string
...     n     form type id as 16-bit unsigned little-endian length-prefixed UTF-8 string
...     n     captured definition alias, 16-bit unsigned little-endian length-prefixed UTF-8 string
...     4     captured definition revision
...     4     value count
...     n     field values
```

Each form field value is:

```text
offset  size  field
0       n     field id as 16-bit unsigned little-endian length-prefixed UTF-8 string
n       m     captured field label as 16-bit unsigned little-endian length-prefixed UTF-8 string
...     1     field kind
...     1     value tag: 0 = normal string, 1 = secret bytes
...     n     value payload
```

Normal value payloads are UTF-8 strings with a 16-bit unsigned little-endian
byte length. Secret value payloads are byte strings with a 32-bit unsigned
little-endian byte length.

The value tag must match the field kind sensitivity. Secret field values use tag
`1`; non-secret field values use tag `0`.

Old form pages must not remain reachable after records or secret fields are
replaced.

## Free-Space Index

Reusable physical pages and reusable free regions are tracked by a transactional
free-space index committed with the same commit root as the TOC.

The index is maintained in two logical orders:

- by offset/page id, for coalescing adjacent free ranges
- by size, for best-fit allocation

The free-space index has no effect on logical archive contents. It is
reconstructable from valid pages and the reachable objects referenced by the
latest valid TOC and commit root.

The root object may be either a `free index leaf` or a `free index internal`
object. Leaf payloads contain sorted non-overlapping `(offset, length)` free
ranges. Internal payloads contain sorted child references:

```text
offset  size  field
0       1     free-index version: 1
1       1     node kind: 0 = leaf, 1 = internal
2       2     reserved
4       4     entry count
8       n     leaf ranges or internal children
```

Leaf entries are `(free_offset, free_length)`. Internal entries are
`(first_free_offset, child_page_offset)`. Children must be strictly sorted by
`first_free_offset`. Free-index pages are append-only during commit so the
published index never lists the page that stores the index itself.

## Key Directory

The key directory is public open metadata referenced by the fixed header and
mirrored in the commit root. It stores only slot ids, slot kinds,
password salts, contact wrapping ciphertexts, ephemeral contact wrapping
material, and encrypted content-key bytes.
It must not store paths, file names, variable names, or file
contents.

The open boundary is deliberately narrow:

```text
before open                         after successful unwrap
-------------                         -----------------------
fixed header
    |
    v
clear-text key directory
    |-- password slot ------ password ------.
    |                                      |
    |-- contact slot ----- contact key ---+--> content key
                                           |
                                           v
                                  encrypted pages become readable
                                  |-- commit root
                                  |-- TOC
                                  |-- variables
                                  |-- forms
                                  `-- file data
```

It also must not store contact identities. Contact names, email addresses,
local vault aliases, public contact keys, and stable public-key fingerprints
would let a holder of one lockbox correlate membership with another lockbox or
another user's vault. The format stores only the slot material needed to attempt
open. ML-KEM-768 encapsulation data must be freshly generated per slot creation
so the same contact key does not produce a reusable cross-lockbox identifier.

The key directory is intentionally readable before the content key is available,
so it is stored as a clear-text metadata page. Its wrapped content-key values
are authenticated by their wrapping algorithms. The page format owns integrity
for the clear-text page by adding and validating the page body checksum. The
key-directory payload itself is length-limited and contains only enough
structure to parse the key slots.

Every key-directory payload has this public header inside the key-directory
page object:

```text
offset  size  field
0       8     magic: "LBX1KEY\0"
8       2     key-directory version: 1
10      2     flags
12      4     header length as 32-bit unsigned little-endian integer: 64
16      8     total key-directory length as 64-bit unsigned little-endian integer
24      8     key-directory generation
32      16    lockbox UUID
48      4     copy index
52      12    reserved
64      n     key-slot payload
```

The key-slot payload starts with a 32-bit unsigned little-endian slot count
followed by that many slot records.

Password slot record:

```text
offset  size  field
0       1     slot kind: 1
1       8     slot id
9       4     salt length as 32-bit unsigned little-endian integer
13      n     Argon2id salt
13+n    4     encrypted content-key length as 32-bit unsigned little-endian integer
17+n    m     ChaCha20-Poly1305 wrapped content key
```

Hybrid contact slot record:

```text
offset      size  field
0           1     slot kind: 2
1           8     slot id
9           4     X25519 ephemeral public-key length as 32-bit unsigned little-endian integer
13          n     X25519 ephemeral public key
13+n        4     ML-KEM-768 ciphertext length as 32-bit unsigned little-endian integer
17+n        m     ML-KEM-768 encapsulation ciphertext
17+n+m      4     encrypted content-key length as 32-bit unsigned little-endian integer
21+n+m      k     ChaCha20-Poly1305 wrapped content key
```

Every key-directory generation has two copies: a primary copy referenced by the
fixed header and commit root, plus one mirror copy referenced by the commit
root. Both copies contain the same generation, lockbox UUID, key slots, and
wrapped content-key material. The copy index is `0` for the primary copy and
`1` for the mirror copy.

Key-directory generation increments by one each time key slots or key wrapping
change. Initial creation writes generation `1`; the next key-directory change
writes generation `2`; the next writes generation `3`, and so on. Ordinary
file, symlink, variable, form, or TOC commits keep referencing the existing
key-directory generation.

The two copies are independent pages at arbitrary page offsets. They are not a
single disk segment in the archive format, and they must not be interpreted as
an adjacent multi-page record. The format does not require an artificial gap
between the copies and does not require increasing the minimum lockbox size to
force separation.

The fixed header pointer is not the only valid key-directory reference. A valid
key-directory mirror contains the lockbox UUID and wrapped content-key material
needed to authenticate encrypted pages.

Removing a password or contact is not just a metadata delete. Because old
history may contain old key directories or data pages, current reachable content
must be rewritten under the new key directory before the old key slot can be
treated as gone for confidentiality purposes.
