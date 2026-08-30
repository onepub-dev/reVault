# Session Agent

The Session Agent temporarily caches the content keys of open Lockboxes. It is
a local process for one user that avoids repeated prompts while reducing the
exposure of decrypted keys on disk.

## What it does

- Stores temporary cache entries for Lockbox content keys.
- May also cache a Vault passphrase and an owner signing key after the normal
  Vault flow has obtained them. These entries are typed, scoped to one Vault
  and Profile, and are never included in the session listing.
- Returns cached keys to subsequent commands.
- Evicts entries automatically by TTL or on inactivity.
- Clears all cached entries when the machine is suspending.
- Provides diagnostics for running sessions and explicit lock operations.

## Design

The feature is implemented in `revault_vault_api`:

- `revault_vault_api::get` / `put` / `forget` / `forget_all` / `list` / `stop`
  call into a platform client module.
- Client calls are made through local IPC transport:
  - Unix: Unix-domain socket.
  - Windows: named pipe.
- The agent process can run in-process (same binary) when started with
  `__agent`.
- CLI startup code dispatches `__agent` and `__agent_security_check` internally
  before normal subcommand parsing.

## Transport + protocol

Requests share a compact binary frame with:

- Header: 9 bytes (`LBX1` magic + message type + u32 payload length LE).
- Maximum message size: 128 KiB.

### Cache operations

Cache messages:

- `get` → `cache-miss` / `cache-key` / protocol errors.
- `put` → store a key with optional path and TTL.
- `forget` / `forget-all` → remove one or all cached entries.
- `stop` → clear all cached entries and terminate agent.
- `list` → list cached lockboxes for diagnostics.
- `info` → report the agent protocol and implementation versions.

The key payload stores:

- lockbox id
- key length + key bytes
- optional path string for diagnostics
- TTL in seconds

TTL defaults to 15 minutes when omitted. Expiry is absolute from the `put`
operation: a `get` hit copies the key but does not extend the agent entry.
The receiving process owns that copy and may continue using its open Lockbox
handle after the agent entry expires, until it closes or finalizes the handle.

### Control path (sleep behavior)

The same transport also supports control messages to track command activity:

- register secret activity (`pid`, `kind`) and returns a token
- unregister activity (`pid`, `token`)

`SecretActivityKind` values currently include:

- `open`
- `open`
- `variable`
- `form`
- `recovery`
- `vault`

The control path is used by high-level commands that perform secret
operations to keep the machine awake for sensitive work and optionally terminate
those processes if suspend is requested.

## Lifecycle

- On first cache operation, client code ensures an agent is running.
- If an agent from an incompatible CLI installation is already running, the
  client stops it, clears its cached secrets, and starts the agent from the
  current CLI automatically.
- If absent, the client starts the current binary with `__agent` and waits briefly
  for the endpoint to become available.
- Server loop runs until:
  - an explicit `stop` request arrives, or
  - 10 minutes of inactivity are observed with no cached secrets and no active
    secret operations.

Installing a new CLI does not replace an agent process that is already
running. The compatibility check above handles this automatically on the next
agent operation. Users do not normally need to restart the agent manually.

## TTL and inactivity behavior

- Default TTL: 15 minutes.
- TTL is validated as positive.
- Inactive cache entries are pruned on accept loop and when servicing requests.
- Cache hits do not extend expiry; TTL is absolute from the cache `put`.
- `lbx session close-all` clears all cached entries from the CLI side.
- `lbx <LOCKBOX> close` clears one cached Lockbox key from the CLI side.

## Platform notes

- Unix
  - Explicit `LOCKBOX_SESSION_AGENT_DIR` takes precedence.
  - `XDG_RUNTIME_DIR` is used only when owned by the effective user. On Linux,
    reVault otherwise tries `/run/user/<effective-uid>` before a uid-scoped
    temporary directory. This prevents a sudo-inherited root environment from
    selecting root's agent after privileges are released.
  - Socket directory defaults to:
    - `LOCKBOX_SESSION_AGENT_DIR` (if set), else
    - `${XDG_RUNTIME_DIR}/lockbox`, else
    - a temporary per-user fallback in `std::env::temp_dir()`.
  - Socket is created as `agent.sock`.
  - Parent directory permissions are set to `0700`.
- Windows
  - Named pipe is `\\.\pipe\lockbox-agent-<scope>`.
  - Scope includes user and, when `LOCKBOX_SESSION_AGENT_DIR` is set, a hash of
    that value to avoid cross-profile collisions.
  - Pipe ACL is owner-only.

## Sleep and security behavior

Configuration (defaults are true):

- `agent.prevent_sleep` / `agent.suspend_inhibit` (config file)
- `agent.terminate_on_suspend` (config file)
- `LOCKBOX_AGENT_PREVENT_SLEEP` (environment override)
- `LOCKBOX_AGENT_TERMINATE_ON_SUSPEND` (environment override)

Configuration source order:

- `LOCKBOX_AGENT_CONFIG` if set.
- else `LOCKBOX_CONFIG`.
- else platform default:
  - macOS: `~/Library/Application Support/reVault/config.toml`
  - Windows: `%APPDATA%\reVault\config.toml` or `%LOCALAPPDATA%`
  - Linux/Unix: `$XDG_CONFIG_HOME/lockbox/config.toml` or `~/.config/lockbox/config.toml`

Behavior:

- If prevent-sleep is enabled and there is at least one active secret activity,
  the agent acquires a platform-specific sleep inhibitor.
- On suspend request, cached keys are always cleared.
- If terminate-on-suspend is enabled, registered active secret processes are
  terminated; otherwise they are kept in memory but no longer protected by a sleep
  inhibitor.

## Logging

`LOCKBOX_SESSION_AGENT_LOG` can point to a file path for explicit agent logging.
Without it, platform logging is used with a file fallback:

- Unix: platform logs (syslog) with fallback under local state cache.
- Windows: Event Log source `reVault Agent`.

## CLI surface

The user-facing session controls live under `lbx session` and each Lockbox:

- `lbx session` — list open Lockboxes.
- `lbx <LOCKBOX> close` — clear one cached Lockbox key.
- `lbx session close-all` — close every open Lockbox.
- `lbx session stop` — stop the Session Agent.

## Shell completion

`lockbox completion generate --shell <bash|zsh|fish|powershell|elvish>` writes
the clap dynamic-completion registration script to stdout. `completion install`
and `completion uninstall` use per-user standard completion directories; pass
`--path` for an explicit location. PowerShell has no equivalent auto-loaded
completion directory, so install adds a marked, idempotent block to the user's
PowerShell profile and uninstall removes only that managed block.

Completion reads only public command metadata, an explicitly supplied Vault
passphrase, or a Vault passphrase already cached by the Session Agent. It never
prompts, opens the owner signing key, or requests signing material. If the
Vault or Session Agent is unavailable, it returns static clap suggestions.

Auto Open is controlled through `lbx session auto-open`.

`lbx doctor` includes Session Agent diagnostics and can help when Auto Open
or transport behavior looks wrong.

## Security notes

- Secrets are stored in process memory and never intentionally written
  to disk by the agent cache.
- The transport is local-only and process-user scoped (`agent` process profile
  checks are used on Windows).
- Control requests are plain binary frames; cache requests use secure frame
  encoding to reduce secret lifetime in transit.
- The protocol intentionally returns explicit errors for malformed frames, invalid
  message sizes, and unsupported message types.
