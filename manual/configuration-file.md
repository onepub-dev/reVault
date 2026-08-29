# Configuration file

Most people can use reVault without a configuration file. The current file controls Session Agent suspend protection.

```toml
agent.prevent_sleep = true
agent.terminate_on_suspend = true
```

Both settings default to `true`. `prevent_sleep` asks the operating system to remain awake during sensitive work. `terminate_on_suspend` stops registered reVault operations if suspension proceeds. Cached content keys are cleared on a suspend request regardless.

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
