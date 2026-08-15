# Public Language API Rules

These rules apply to language facades under `bindings/`. The C ABI is a shared
transport and capability contract; it is not the public API design template.

The Dart 0.3 facade is the current terminology and workflow prototype. Do not
mechanically copy it to the other languages until its API review is complete.

## Domain terminology

- `Revault` is the native-runtime loader or package entry point. Loading it does
  not open a vault or lockbox.
- `Vault` is the persistent encrypted local store for profiles, private keys,
  contacts, signing keys, and remembered lockbox metadata. Do not expose this
  domain object as `VaultDirectory`; a directory is a storage detail.
- A signing key belongs to a Vault `Profile`. Use `ProfileSigningKeyPair` and
  `ProfileSigningPublicKey` for the identity. Use `owner` only for the role a
  Profile's signing key occupies after assignment to a Lockbox, such as
  `Lockbox.setOwnerSigningKey` and `OwnerInspection`. A Contact may carry the
  public verification key for a Profile but cannot sign.
- `Lockbox` is a portable encrypted `.lbox` archive.
- `AgentSession` controls the single optional session-agent process and its
  temporary content-key cache. Do not call it `LocalVault`, and do not describe
  each cached lockbox as a separate session.
- Use `open` and `close` for lockbox availability, matching the CLI. Explain
  that the agent caches a content key rather than retaining an open file handle.
- Distinguish a vault passphrase, a lockbox password, and a lockbox content key
  in every name and comment. Never call all three `password` or `key` when the
  distinction is known.

## Workflow and lifetime rules

- Normal library operations are process-local. `Lockbox.open` retains its key
  in the returned object until `close` and must not implicitly start, contact,
  or populate the session agent.
- Agent use is explicit. It exists for short-lived CLI commands, cooperating
  processes, and time-limited delegation of selected lockbox content keys.
- `Lockbox.close` releases the current process's handle and key.
  `AgentSession.closeLockbox` forgets the agent's independent cached key, and
  `AgentSession.closeAll` forgets every cached lockbox key. Neither operation
  deletes persistent credentials or lockbox files.
- Opening an existing resource never creates or replaces it. Give open,
  open-or-create, create, and replace distinct APIs with prominent destructive
  documentation.
- A common workflow must have one obvious, executable path in package-level
  documentation. Byte-oriented and raw-key operations must not obscure
  file-, vault-, and password-oriented workflows.

## Credential security

- A platform-stored vault passphrase provides unattended same-user access on
  platforms that do not enforce user presence per retrieval. It therefore
  permits opening every lockbox for which that vault contains a credential.
- Do not present agent TTL, `close`, or `closeAll` as an authentication boundary
  while an applicable persistent credential remains available without user
  interaction. They still provide memory hygiene and operational cache control.
- Remembered lockbox passwords are encrypted records inside the Vault, not
  independent operating-system credentials. Never persist a raw content key
  permanently; agent content-key entries are temporary.
- Credential-free lockbox opening may use the platform-stored Vault passphrase
  to open the Vault and resolve a lockbox credential there. It must never
  implicitly consult the session agent.
- Design credential acquisition so a future platform provider can require
  user-mediated biometric or equivalent authentication for each import. Expose
  and feature-detect user-presence guarantees; never infer them merely from the
  existence of an OS credential store.
- Secret byte arrays are wiped as soon as their required lifetime ends. Public
  docs state who owns each secret and when callers must wipe or close it.
- Use an owning secret type for passphrases, passwords, and binary keys when the
  language permits it. In Dart these are `SecretString` and `SecretBytes`;
  callers close them to wipe their buffers. Document when an immutable source
  string remains outside the binding's control.

## Language-native facade rules

- Translate native failures at the FFI boundary. Exception-oriented languages
  throw typed exceptions containing stable structured details; Go returns an
  error and Rust returns `Result`. Public `lastError` state is allowed only in
  the low-level C API.
- Model closed sets with enums or sealed types, including key formats, workload
  profiles, worker policies, cache modes, and agent activity kinds. Do not
  accept undocumented strings for closed native enums.
- Use named arguments or configuration objects for adjacent primitive values.
  Boolean arguments must be named at the call site where the language permits.
- Use the language's normal resource-lifetime pattern. Document whether close
  commits, discards pending changes, forgets an agent entry, or only releases
  memory. Repeated close should be safe when that matches language convention.
- Keep native handles and generated FlatBuffer/protobuf types private. Public
  values are stable language-native domain objects.

## Names and documentation

- Do not use `value`, `data`, `bytes`, `key`, or `id` when a precise parameter
  name is known. Prefer `exportedPrivateKey`, `lockboxArchive`, `contentKey`,
  `lockboxId`, `vaultPassphrase`, and `lockboxPassword`.
- Every public operation documents the meaning and security role of each
  argument; accepted format, encoding, and size; where non-obvious inputs come
  from; side effects; ownership; cleanup; and expected failures.
- Never say “canonical value,” “private record,” or “supported format” without
  defining the representation and linking the operations that produce and
  consume it. Enumerate automatically detected formats.
- Link paired operations: import to export, decode to encode, create to open,
  and open to the source of its credential.
- Every public class, constructor, and operation has a worked example showing
  its normal context. A one-line restatement of the signature is not an
  example. Generated domain-model documentation follows the same rule.
- Describe contracts directly. Labels such as “advanced”, “low-level”,
  “simple”, or “convenient” do not explain provenance, lifetime, side effects,
  or intended use and must not substitute for those facts.
- Include a 0.2.x-to-0.3 migration example for every breaking Dart rename.

## Verification and rollout

- Treat native symbol conformance and public-facade quality as separate tests.
  Test idiomatic errors, enums, nullability, resource lifetime, secret cleanup,
  and complete common workflows at the facade level.
- Generated files are regenerated from their source and are never hand-edited
  unless the repository explicitly identifies them as maintained bindings.
- Keep the non-Dart facades unchanged while the Dart 0.3 prototype is under
  review. Once accepted, adapt its behavior to each language's conventions
  rather than reproducing Dart signatures mechanically.
