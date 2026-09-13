---
description: "Read immutable remote Lockboxes through bounded external byte sources."
---

# External byte sources

Enable the `external-source` Cargo feature on `revault_lockbox_api` to read a Lockbox from an immutable external source. The API works in native Rust and Rust compiled to browser WASM. It uses the existing archive format, parsers and integrity checks; it does not require migration or a format version increase.

`ReadAtSource` supplies bytes at absolute offsets. A native application can implement synchronous remote reads. A browser application can use `SparseSource`, fetch missing blocks asynchronously, and retry through `ExternalReader`. HTTP transport belongs to the application.

## Supported archives

| Archive | Opening behavior |
| --- | --- |
| Unencrypted, unsigned | Fetch metadata first, then the file pages needed by each operation. |
| Unencrypted, signed | Opening authenticates all current content using the existing signed-content digest. Expect to fetch the entire live content. |
| Encrypted, with a known content key | Use `ExternalReader::with_content_key`; normal encryption and signature verification remain in force. |
| Password or contact discovery | No external-source discovery API is provided. Obtain the content key separately. |
| Pending transaction recovery | Read-only opening cannot perform required writes. Recover using a writable local copy and publish a new immutable revision. |

Compression is supported through the existing decoders. Reading part of a compressed frame may require downloading and decoding the whole frame. A strong HTTP ETag identifies a revision; it does not authenticate an unsigned archive or a publisher.

## Read and retry

Construct `SparseSource` with the archive length, an immutable revision identifier, a block size and a cache budget. For example, use 64 KiB blocks and a 16 MiB payload cache. Share it through `Arc`, then construct an `ExternalReader` with `ExternalReaderOptions`.

Call `reader.try_read(|archive| archive.get_file(&path))`. The closure receives a borrowed `Lockbox<ReadOnly>` and can also inspect variables, forms, symlinks and other readable metadata.

1. On `Ok(value)`, consume the completed result.
2. On `ExternalReadError::Source(SourceError::MissingRange { offset, length })`, fetch that exact range from the same revision. Validate the response, supply it with `source.supply(revision, offset, bytes)`, then repeat the operation.
3. On any other source error or `RetryLimitExceeded`, stop. Source failures are terminal except for `MissingRange`. Use a new source and reader when switching revisions.
4. `ExternalReadError::Archive` reports normal archive errors, such as a missing logical path, invalid key, or corrupt content. These are not instructions to fetch another range.

Keep the closure repeatable. It may execute before a later source read fails: defer file writes, UI publication and other external side effects until `try_read` returns `Ok`. A borrowed Lockbox or streaming file handle cannot escape the closure. For large outputs, read and return bounded logical file slices using `open_file`, `Seek`, and `Read` inside the closure.

The reader checks source failures even when an internal fallback or the closure itself returns success. Once a source read fails, further physical reads in that attempt fail, and the reader discards parsed state before retrying. This prevents incomplete fallback metadata from being reused. Callers do not need the experiment's separate `clear_missing`/`missing` flag protocol.

Each reader requires mutable access for an operation, serializing its retries. Independent readers can share one `SparseSource`; their operation-failure state is separate. Applications still need to deduplicate concurrent network fetches if they want to avoid duplicate requests.

## Resource limits and cancellation

`SparseSource` stores only supplied blocks. It evicts the least recently used blocks to stay within its payload budget. Its metadata is bounded by the block count. A supplied block must be aligned and complete; the last block can be shorter. Identical repeated blocks are accepted. A different revision or conflicting cached bytes invalidates the entire source.

The transport must validate the revision on **every refetch after eviction**. The cache does not retain an unbounded hash history of evicted blocks. Use a fresh source for a new revision.

`ExternalReaderOptions` provides:

- `max_read_bytes`: maximum physical read or requested missing range, checked before backend allocation; defaults to 64 MiB. Keep the sparse block size within this limit.
- `max_missing_attempts`: maximum consecutive incomplete attempts; defaults to 4096. Exhaustion terminates the reader instead of looping indefinitely.
- `max_source_bytes_per_operation`: cumulative physical read bytes across an operation and its retries, including cache hits; defaults to 256 MiB. This bounds repeated read work even when the network supplies each block only once.
- `lockbox`: existing decoded-page cache and workload controls, separate from the sparse payload cache.

Budget network buffers, returned file bytes and decoded caches separately. An operation's working set must fit the sparse cache, or eviction may cause repeated downloads until the retry limit is reached. Source failures discard parsed state, so retries can repeat parsing and decoding work. Read smaller logical file slices or raise the explicit budget when appropriate. This is a bounded retry interface, not a resumable asynchronous decoder.

`reader.cancel()` releases that reader's parsed state and content key. `source.cancel()` invalidates every reader sharing the source and clears its blocks. Cached operations check cancellation and known source invalidation too. The transport owns cancellation of in-flight HTTP requests; cancellation cannot interrupt a synchronous callback that has not returned.

## HTTP transport contract

For the browser adapter:

1. Obtain the exact representation length and a strong ETag, or an equivalent immutable object-version token. Enforce an application archive-size limit before creating the source.
2. Request `Range: bytes=start-end` with `If-Match` for the original ETag. Require HTTP 206, the exact `Content-Range` including total length, and the unchanged ETag.
3. Reject content encoding that changes the byte representation. Require the exact response-body length and cap it before buffering. Reject HTTP 200 full-download fallback, HTTP 412, missing validators and truncated responses.
4. Supply the validated block under the original revision identifier. Never relabel bytes from a new revision as the old one.
5. Bound requests, retries, timeouts and in-flight memory. Keep published archive revisions immutable and configure CORS to expose the required headers when origins differ.

The Rust core does not implement or certify an HTTP client. The original browser experiment in `onepub-doc`, commit `82123f2`, records transport and performance results for its prototype API and older archives. Those measurements are not benchmarks of this supported API. Its adapter must be updated to the checked `ExternalReader` interface before reuse.

## Bindings and release coverage

This is an opt-in Rust embedding API, also usable from a custom WASM wrapper. Existing C ABI and generated Java, Dart, JavaScript and other language facades are unchanged. Their standard file-backed APIs continue to work. A cross-language external-source callback protocol would require a separate ABI design; the shipped JavaScript/WASM facade does not expose this feature automatically.

Release CI runs the external-source regression suite and checks compilation for portable targets with and without the feature. Tests cover incremental file and metadata reads across compression, signing and encryption choices; byte equality; secret values; injected open and lazy-read failures; swallowed errors; short and invalid reads; source changes; cache eviction; retry limits; cancellation; concurrent readers; corruption; and read-only enforcement.
