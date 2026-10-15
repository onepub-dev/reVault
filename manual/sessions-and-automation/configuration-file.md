# Configuration file

Most people can use reVault without a configuration file. The current file controls Session Agent suspend protection.

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

reVault looks for configuration in this order:

1. the path in `LOCKBOX_AGENT_CONFIG`;
2. the path in `LOCKBOX_CONFIG`;
3. the platform default.

| Platform | Default path |
| --- | --- |
| macOS | `~/Library/Application Support/reVault/config.toml` |
| Windows | `%APPDATA%\reVault\config.toml`, falling back to `%LOCALAPPDATA%` |
| Linux and other Unix systems | `$XDG_CONFIG_HOME/lockbox/config.toml`, or `~/.config/lockbox/config.toml` |

Environment variables may override the two agent values:

```text
LOCKBOX_AGENT_PREVENT_SLEEP
LOCKBOX_AGENT_TERMINATE_ON_SUSPEND
```

Use `true` or `false`. Run `lbx doctor` after changing security-related configuration to confirm the platform features reVault can use.
