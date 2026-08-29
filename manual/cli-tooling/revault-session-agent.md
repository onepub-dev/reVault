---
description: How the local Session Agent works and what it protects.
---

# reVault Session Agent

The reVault Session Agent is a local, per-user process. It keeps the content keys of open Lockboxes in memory so that a sequence of commands does not require your Vault passphrase each time.

The first command that needs the agent starts it automatically. It communicates only through a user-scoped local socket on Unix-like systems or a named pipe on Windows.

## What it does

The agent:

* caches content keys for open Lockboxes;
* expires inactive entries after their time to live;
* extends an entry's expiry when it is used;
* clears cached keys when the machine starts to suspend;
* can prevent sleep while a secret operation is active; and
* can terminate an active secret operation if the machine nevertheless suspends.

The default Lockbox session duration is 15 minutes. You can choose another duration when opening a Lockbox:

```bash
lbx secrets.lbox open --duration 1h
```

Use `lbx session` to see open sessions, `lbx session close-all` to clear them, or `lbx session stop` to clear them and stop the agent.

## Agent and Auto Open

The agent does **not** normally retain your Vault passphrase. [Auto Open](session-management.md#auto-open) is the separate feature that stores that passphrase in your operating system's secure credential store.

This distinction matters. Closing a Lockbox clears its cached content key, but Auto Open may still allow a later command to unlock the Vault and open the Lockbox again.

## Suspend protection

The following configuration values default to `true`:

```toml
agent.prevent_sleep = true
agent.terminate_on_suspend = true
```

`prevent_sleep` asks the operating system to remain awake while a sensitive operation is active. `terminate_on_suspend` stops registered reVault operations if suspension cannot be prevented. Cached content keys are cleared on a suspend request in either case.

Environment variables can override these values:

```text
LOCKBOX_AGENT_PREVENT_SLEEP
LOCKBOX_AGENT_TERMINATE_ON_SUSPEND
```

See [Configuration file](../configuration-file.md) for file locations and precedence.

## Diagnostics and logging

Start with:

```bash
lbx doctor
lbx session
```

Set `LOCKBOX_SESSION_AGENT_LOG` to a file path when you need an explicit agent log. Without it, the agent uses platform logging with a local file fallback.

The agent is a convenience and exposure-reduction feature, not a sandbox. A process already running as your desktop user may be able to communicate with it. Protect the desktop session itself, and consider disabling Auto Open on particularly sensitive systems.
