# Shared-control image components

Date: 2026-09-27. Based on `2ab5aa91`. The commit containing this evidence adds
an isolated publication/credential/private-envelope image. It is not a complete
archive writer: typed records, payload placement, allocation and journal integration
are still absent. There is no public API or release-format change.

## Implemented

The experimental `RV4SHR01` publication profile authenticates a distinct prefix
using the existing owner-signature or symmetric-MAC code. Existing `RV4PUB02`
candidate bytes and independent cryptographic vectors remain unchanged. The caller
must select the expected profile; old candidate open does not accept the new one.
Changing magic/version and recomputing a checksum does not preserve a signature
or MAC, including when root addresses would otherwise be valid in both profiles.

Each 64 KiB control region reserves these roles:

| Offset within region | Bytes | Role |
| --- | --- | --- |
| 0 | 8,192 | Publication/owner proof |
| 8,192 | 4,096 | Preparation stub, reserved but not implemented |
| 12,288 | 4,096 | Public wrapped-key directory |
| 16,384 | 49,152 | Private metadata envelope |

Root validation rejects role crossing, control overlap, wrong inline offsets,
length overflow and copies sharing a physical failure region. External roots
remain possible at aligned offsets in separate regions. Allocation-map roots
cannot use an inline private/public slot. Root reads verify selected stored-byte
digests and propagate I/O errors. This validates placement, not graph ownership.

The private `RV4CAT01` envelope uses a distinct HKDF key domain and existing
ChaCha20-Poly1305. It has a 44-byte header, a 12-byte private length/codec prefix,
and an optional 16-byte tag. Private padding is zero plaintext inside the AEAD.
Output allocation is fixed at 49,152 bytes; decoded content and Zstd window are
bounded at 65,536 bytes. It honours archive compression options, including no
compression. Overlarge input or encoded content explicitly requires authenticated
overflow, which is not yet implemented. Buffers use the existing capacity-wiping
buffer. Typed catalogue parsing is not supplied by this byte envelope.

Actual in-memory persisted bytes connect password bootstrap, publication
authentication and private-envelope decryption. Reopening with either entire
64 KiB control region zeroed still recovers the directory and private bytes from
the surviving authenticated copy. These are component fixtures because no public
CLI writes this experimental profile. Fixture construction is not a crash-safe
creation or mutation API.

## Validation

From `rust/`, before formatting:

```bash
cargo test -p revault_lockbox_api --release --features external-source --lib file_format
cargo clippy -p revault_lockbox_api --all-targets --features external-source,native-block-layout -- -D warnings
```

[158 format tests pass](format-tests.log), five probes remain explicitly ignored;
[strict Clippy passes](clippy.log). Six added tests cover role placement,
cross-profile authority, whole-region loss, end-to-end component bootstrap/decode,
16 protection/compression/padding combinations, wrong keys/context, malformed
lengths/padding/codec and explicit overflow refusal. Existing publication crash,
independent-vector and bootstrap rekey tests remain green. No new process-death
or power-loss claim is made for the shared-control profile.

The cost model now budgets 72 bytes rather than 60 for worst-case envelope
overhead and follows the archive's compression policy. Earlier frozen cost-model
results remain historical observations; this change does not relabel them as
measurements of a complete shared-control archive.

## Remaining protocol work

Implement typed catalogue/ownership together with a 4 KiB preparation stub and
bounded authenticated overflow retaining the existing reservation capacity. Wire
staged writes, durable publication and retirement into that protocol, then test
actual partial writes, power loss, process death and recovery. Public access
mutation/overflow, full object semantics, migration and the complete performance
matrix remain required.

The abstract ordering model also omits sealed-length and allocation-map encoding.
Its final truncation step is not implementable merely by shrinking storage after
publishing inline roots: the selected sealed length and pending/free accounting
must agree. A safe implementation may retain erased temporary ranges as explicitly
owned free space until source-preserving compaction, or prove a separate shrink
protocol. Do not infer permission to truncate from the model's reachability check.
The proposed compacted size is not a bound on update-time disk usage.
