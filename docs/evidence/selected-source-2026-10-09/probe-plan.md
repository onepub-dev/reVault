# Fixed selected-source aggregate functional probe

After affected correctness and strict Clippy pass, freeze release unit-test binary,
exact source diff/new source snapshots, compiler/package identities and runner.
Run exactly16 fresh processes, mode0..15, one each, no retries or extra samples.
Run only typed_form_selected_source_aggregate_probe --ignored --test-threads=1
--nocapture with unique absolute REVAULT_FORM_IMAGE paths and REVAULT_FORM_MODE.
No overlapping owned builds/tests. Preserve failures and partial outputs. Record
inherited CPU affinity and RLIMIT_MEMLOCK; do not change either policy.

Fixture: existing FileStore13,631,586-byte logical form snapshot, including exact
1MiB definition name/description/label, record name/captured label and eight1MiB
secret fields. One caller Arc<SecretString> supplies the synthetic initial values;
normal metadata ordinary. Drop writer and every caller object before reopening
writer for selected-source replacement of every form text. Clone-trap wrapper
rejects any underlying storage.clone; read guards reject ordinary allocating
reads over live/retired/attempted form/variable pages. This does not prove arbitrary
read_at_into caller buffer provenance; reviewed renderer uses guarded buffers.

New source layouts preserve logical bytes, UUID/context and increment text
revision, stage replacements from admitted old selected extents and retire exact
old extents through existing journal. Capture after_selected_source VmLck/VmRSS/
VmHWM immediately after writer returns, before explicit guarded retired-zero audit.
Drop writer, reopen an independent FileStore reader handle in the same fresh
probe process (not a separate verifier process), verify every normal text and all eight
secret values one at a time, then stream all definition/record events and validate
all delivered bytes. Record after_read and after_salvage endpoints. Existing setup
endpoints remain; arenas may be retained and reused across phases.

Per-mode retention: full output, unique marker SELECTED_FORM_AGGREGATE with mode
and all phase metrics, process exit status, archive hash/size and logical/stored
payload byte counts. Require exactlyone marker and successful test result percase;
reportall failures. Hash binary and runner before/after, retain stability result.
No timing ratios, elapsed-time acceptance, incremental peak, arbitrary concurrency,
whole-object getter capacity, released compatibility or whole-format acceptance.
VmHWM is whole-process peak including setup; VmRSS/VmLck are phase endpoints.
Report source replacement archive growth candidly; no aging/compaction claim.
Mode bits: protected1,signed2,compression4,padding8. Form secure pages remain
uncompressed even with compressionbit enabled.
