# Encrypted and signed public reVault versus compression-only ZIP

This is a fresh descriptive public CLI screen of the current branch, using the
same two pinned one-file 1 GiB sources as the completed plaintext comparison.
It does not reuse prior timing samples. ZIP supplies compression and CRC, not
reVault's encryption, owner authentication or transaction/recovery semantics.

## Fixed scope and commands

Use the preserved current CLI binary `08bf819f90c3b77a7e69f0639c7bd5f73dde275b6e13f0d443a3285fe98d00f2`,
built from `765e4ac2`; `d635f33b` adds only evidence/docs/tools. Verify runtime
source equality and retain the original build provenance. No runtime edit or
new build is needed. Info-ZIP zip 3.0 and unzip 6.00 are independently hashed.

Each source has its own new isolated synthetic Vault and Session Agent. Outside
timing, `lockbox vault init` creates the normal default profile/key pair with a
fresh random in-memory passphrase, supplied only in the child environment.
All init stdout/stderr is discarded because it can contain recovery keys.
No credential values, recovery output, environment dumps or Vault files enter
evidence. No existing user Vault or agent is accessed. Stop owned agents in
bounded cleanup. Private test Vaults live only in the owned fixture directory and are removed
after the owned agent stops; only allowlisted metadata and exit status remain.

Native write is the sum of these complete public subprocesses:

```text
lockbox ARCHIVE create --encryption chacha20-poly1305 --signing owner --compression zstd --compression-level 3
lockbox ARCHIVE open
lockbox ARCHIVE add SOURCE --to /file-000000.bin
```

The stored basename is taken from the pinned source manifest, not assumed.
Native read is a separate `lockbox ARCHIVE extract --to FRESH_DIRECTORY`.
The owned agent is already running and the synthetic Vault passphrase is
available noninteractively. Vault initialization is excluded, but normal
per-command Vault opens/password derivation, owner-key loading and fresh
explicit archive open/credential discovery incurred by create/open/add remain inside
the complete-command timing. Read follows the successful write and public
doctor/list verification, with the normal populated session state; it includes
archive open, owner-authenticated metadata/payload verification, decryption and
extraction performed by the public endpoint. This is not a cold-session or
separate password-unlock benchmark. No claim excludes all credential costs.
Doctor/list outside timing must confirm encryption, owner signing, Zstd level3,
default padding and the exact single stored file before accepting each sample.

ZIP write is `zip -6 -j -q ARCHIVE SOURCE`; read is
`unzip -q ARCHIVE -d FRESH_DIRECTORY`. Record actual ZIP compression method,
including random input; do not assume Stored fallback. Clear ZIP environment
options through a complete isolated child environment.

## Frozen measurements and gates

Two sources × four rounds × two tools × write/read = 32 accepted operations.
Round0 is excluded preparation (8 operations); rounds1–3 give 24 measured
operations, three per case/tool/operation. Alternate tool order by round/case.
Use affinity CPU0–4 and normal Auto write workers; one-file native ExtractMany
does not activate the >=256-file extraction pool. Candidate readers are absent.

GNU time records full subprocess elapsed/user/system/peak RSS. Sum native
create/open/add time and CPU, take the maximum subprocess peak RSS rather than sum.
Session-agent RSS is excluded and must be stated. Report medians/full ranges,
ratio of medians, maximum RSS, archive bytes and observed codecs. Three repeats
are descriptive, not a confidence interval or performance qualification.
Report separate `sync -d` output-drain time and CLI+drain totals; this is not a
claim of equivalent crash durability. Hash every accepted archive before its planned removal. Hash source/tool/runner/
analyzer/protocol/build identities before and after the batch.

Fresh archives/outputs every sample, source-cold before write and archive-cold
before read using the existing bounded zero-resident admission. Independent
byte comparison and exact one-file/type checks occur outside timing. Host
sources remain 0664; native ordinary import stores0600 and default extraction
restores0600, while ZIP preserves0664. Verify the respective modes under
umask0022; do not claim permission equivalence or original native preservation.

Reuse established 24 GiB aggregate owned-artifact budget with headroom and
reserve output/archive capacity before each sample. Include previous plaintext
contexts and new protected contexts. Work deadline30minutes with60seconds
cleanup reserve and bounded kill grace, individual command cap600seconds.
Preserve any failure and stop; no outcome-dependent retries or extra samples.
Before freeze, one tiny untimed protected/ZIP smoke verifies exact syntax,
default Vault credential flow, doctor/list parsing, bytes and respective modes.
No tests/builds run concurrently. Parent audits the completed report and sends
the explicitly requested email to bsutton@onepub.dev.

## Untimed preparation correction

The first tiny smoke created the protected archive but its first add refused
with exit10 (closed Lockbox). No setup/freeze/timed sample followed. Public
`open` explicitly loads a matching local Vault profile and caches the content
key and owner signer for later commands. The corrected complete write workflow
therefore times create + open + add; it does not hide the required opening cost.
A fresh owned smoke context is used; prior failure/status evidence remains.
