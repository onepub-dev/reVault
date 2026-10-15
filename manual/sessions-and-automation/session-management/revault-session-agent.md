---
description: How the local Session Agent works and what it protects.
---

# Session Agent

The Session Agent is a local process for one user. It keeps the content keys of open Lockboxes in memory so that a sequence of commands does not require your Vault passphrase each time.

The first command that needs the agent starts it automatically. It communicates only through a user-scoped local socket on Unix-like systems or a named pipe on Windows.

## What it does

The agent:

* caches content keys for open Lockboxes;
* expires inactive entries after their time to live;
* extends an entry's expiry when it is used;
* clears cached keys when its active watcher receives a suspend notification;
* can prevent sleep while a secret operation is active; and
* can terminate an active secret operation if the machine nevertheless suspends.

By default, a cached Lockbox key expires after 15 minutes. You can choose another duration when opening a Lockbox:

```bash
lbx secrets.lbox open --duration 1h
```

Use `lbx session` to see open sessions, `lbx session close-all` to clear them, or `lbx session stop` to clear them and stop the agent.

## Agent and Auto Open

The agent does **not** normally retain your Vault passphrase. [Auto Open](./#auto-open) is the separate feature that stores that passphrase in the platform credential store.

This distinction matters. Closing a Lockbox clears its cached content key, but Auto Open may still allow a later command to unlock the Vault and open the Lockbox again.

## Suspend protection

The following configuration values default to `true`:

```toml
agent.prevent_sleep = true
agent.terminate_on_suspend = true
```

Both settings default to `true`. `prevent_sleep` requests an operating-system sleep inhibitor during sensitive work. `terminate_on_suspend` requires a ready local suspend watcher before the operation accesses secrets; the process terminates itself when notified, without asking the agent to signal a numeric PID. If requested protection is unavailable, the operation fails with an actionable error.

The agent clears cached keys when its active watcher receives a suspend notification. `lockbox session` and `lockbox doctor` report whether that watcher is active. An unavailable watcher does not provide suspend cleanup; cache expiry and explicit session closure remain available. Notifications and inhibition depend on operating-system facilities and cannot guarantee protection against forced suspend or notifications that the system does not deliver.

For deliberately unprotected headless automation without a system bus or supported power service, explicitly disable both policies:

```bash
export LOCKBOX_AGENT_PREVENT_SLEEP=false
export LOCKBOX_AGENT_TERMINATE_ON_SUSPEND=false
```

This opt-out removes suspend inhibition and local suspend termination; it does not make cached keys safe across sleep when monitoring is unavailable. Prefer short sessions and close them with `lockbox session stop` after use. These settings are read when the agent starts: stop an existing agent before changing them, then start the next session with the intended environment.

Environment variables can override these values:

```
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
