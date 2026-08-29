---
description: How to setup a key sharing server.
---

# Key sharing service

reVault provides a public key sharing server to make it simple to share keys between parties.&#x20;

* Party A 'publishes' their public key to a key server via the reVault cli 'lbx vault identity publish' command.
* The key server emails the user requesting that they validate ownership of the email address by clicking the included email verification link.
* Party A provides Party B with a unique publish code.
* Party B 'receives' the public key from the key server by running 'lbx vault contact receive'.
* Party B contacts Party A via an alternate channel and request the fingerprint.
* Party B enters the fingerprint resulting in a new contact being added to their vault.

The reVault key server is able to validate the ownership of a key to an email address.

The key server is designed to be only one of the two channel communications reVault requires establish trust in a public key.&#x20;

The public key is passed via the reVault key server but the keys' fingerprint must be passed via an independant channel.

### Locating a reVault Key server

reVault runs a public key server as a service to the community. The reVault key server is able to run fault tolerant pairs but the public key server currently runs a single monitored instance.

## Organisational Key Server

An organisation can also run an internal Key Sharing Server(s) internally, however you need a license to operate the key server.&#x20;

If you are running an internal key server you can consider increase the ttl of published keys to be several months so that users don't have to keep re-publishing their keys. But if you do this you need to have policies and procedures around retiring keys when a staff member leaves.

## HELP required

To make the reVault key server more stable we are looking for a community minded organisation to run a secondary (tertiary...) public key server. The key server is light weight consuming < 100MB of memory and < 100MB of disk and can handle some 20,000 requests per second on a 4 core server.&#x20;

If you are interested then email me at bsutton \[@] onepub.dev

The server is a single rust executable with no other dependencies.

## Installing

Installing the reVault Key server.

The reVault key server is a single self installing executable that runs on Linux (Windows/MacOS is possible but the installer currently only supports Linux).

The server needs to have port 443 open as all requests are sent over https.

## Install the reVault key server&#x20;

The reVault Key Server is published as a rust crate.&#x20;

To install the key server run:

```
cargo install lockbox_key_server
```

You may need to add cargo to your PATH:

```
export PATH="$HOME/.cargo/bin:$PATH"
```

You will need to add the export to somewhere like \~/.bashrc so it survives reboots.

You can run the key server directly on the cli:

```
lockbox_key_server run
```

To install the key server so it runs as a systemd daemon and restarts on boot you need to run:

```
sudo lockbox_key_server install 
```

Now check that the server is running:

```
sudo systemctl status lockbox-key-server.service
sudo lockbox_key_server status
```

### Useful paths

```
Service unit: /etc/systemd/system/lockbox-share-server.service
- Config: /etc/lockbox/share-server.toml
- Log file: /var/log/lockbox-key-server/server.log
- Database dir: /var/lib/lockbox-key-server
```

### Database

The key server runs a simple in process binary database. The db is used to cache long running requests sessions as well as persistence if the key server restarts.

On startup the key server does a basic check to ensure the db isn't corrupt. If it is, it simply deletes the database.  The consequence of the delete is that any outstanding publish commands will need to be re-issued by the user. Given the standard TTL for a publish is 15 min this shouldn't affect many users.

## Advance configuration options

### Replication

You can configure up to 32 reVault Key servers which are able to operate in as a ring of replicated fail over pairs.

Each server in the ring sends replication data to the next server in the ring and broadcasts that server is its replica.  If there are only two servers in the ring then they essentially replicate both ways. &#x20;

State information (open publish requests) are shared between pairs.&#x20;

### 1) Core model

* Replication is peer-to-peer using POST /v1/replicate with servers set up in a ring - each server replicating its state to the next server in the ring.
* When a user publishes a key we return a share code with the first two digits being the locator code (a-z0-9). Each digit represents a server index - so a max of 32 servers. The first position is the primary server and the second the secondary server.
* The publishing users gives the share code and email address to the receiving user.&#x20;
* The receiving server replicates the publish request to the next server in the ring, signing the request with the replication-token (HMAC) .
* Inter server replication is over https.
* Runtime events replicated: put\_share, fetch\_count, tombstone (delete/expiry).
* The Outbound replication worker sends to replication-peer-url&#x20;
* Incoming replication is only accepted if the HMAC signature verifies, otherwise ReplicationUnauthorized is returned to the calling server.&#x20;

Currently the key servers are located at:

https:/keyshare0.revault.onepub.dev

### 2) Required configuration for two-node standby style

Create one config per node and keep both with same:

* cluster\_id
* topology\_version
* same replication\_token
* matching public topology entries (topology\_server, route)
* each node points to the other via replication\_peer\_url

Example node A (/etc/lockbox/key-server-a.toml):

bind\_addr = "0.0.0.0:8089" state\_dir = "/var/lib/lockbox-key-server-a" server\_id = 0 cluster\_id = "prod-cluster" topology\_version = 1 public\_url = "https://ks-a.example.com/v1/share" topology\_server = "0=https://ks-a.example.com/v1/share,active" topology\_server = "1=https://ks-b.example.com/v1/share,standby" route = "0=0,1" route = "1=1,0" replication\_token = "REPLACE\_WITH\_STRONG\_TOKEN" replication\_peer\_url = "https://ks-b.example.com/v1/replicate" origin\_epoch = 1700000000000

Example node B (/etc/lockbox/key-server-b.toml):

bind\_addr = "0.0.0.0:8089" state\_dir = "/var/lib/lockbox-key-server-b" server\_id = 1 cluster\_id = "prod-cluster" topology\_version = 1 public\_url = "https://ks-b.example.com/v1/share" topology\_server = "0=https://ks-a.example.com/v1/share,standby" topology\_server = "1=https://ks-b.example.com/v1/share,active" route = "0=0,1" route = "1=1,0" replication\_token = "REPLACE\_WITH\_STRONG\_TOKEN" replication\_peer\_url = "https://ks-a.example.com/v1/replicate" origin\_epoch = 1700000000000

Notes:

* route = OWNER=PRIMARY\[,FAILOVER...] means the client will try PRIMARY first then failovers.
* For standby recovery/failover, promotion is controlled by runtime config (--promoted-owner) and topology updates.

### 3) Start servers

./target/release/lockbox\_key\_server run --config /etc/lockbox/key-server-a.toml ./target/release/lockbox\_key\_server run --config /etc/lockbox/key-server-b.toml

If using service install flow, remember service always runs with /etc/lockbox/key-server.toml unless you edit unit file or use run manually.

### 4) Verify replication is wired

* POST /v1/share receives creates/fetches on each server.
* GET /v1/topology exposes topology metadata from each server.
* GET /v1/status exposes replication\_pending and replication\_last\_sequence (binary protocol payload).

### 5) Manual backfill / resync command

Use this if one node was offline and missed events:

./target/release/lockbox\_key\_server resync-peer\
\--config /etc/lockbox/key-server-a.toml\
\--peer-url https://ks-b.example.com/v1/replicate

resync-peer streams current live shares to the peer URL as replication events.

### 6) Failover/standby promotion flow (operator action)

1. On the standby node, add promoted owner IDs (or restart with config override), e.g. --promoted-owner 0 to serve owner 0 shares if primary is down.
2. Update topology status/routes so clients can fail over correctly.
3. Restart/reload servers so new config takes effect.
4. When old primary returns, run resync-peer from promoted/healthy node to it, then reassign promotion only after it has caught up.

### 7) CLI equivalent (no config file)

Key replication flags from command line:

* \--replication-token TOKEN
* \--replication-peer-url URL
* \--origin-epoch N
* \--promoted-owner N
* \--topology-server ID=URL\[,STATUS]
* \--route OWNER=PRIMARY\[,FAILOVER...]
* \--peer-url URL (only for resync-peer command)

\--help on the binary prints all supported switches.

### Command line Options

## lockbox\_key\_server command line switches

Useful one-off flags:

```bash
lockbox_key_server run --bind 0.0.0.0:8089 --state-dir /tmp/lockbox-key-server
lockbox_key_server install --force-config
lockbox_key_server uninstall --purge-data
lockbox_key_server resync-peer --peer-url https://peer.example/v1/replicate
```

### Command forms

* `lockbox_key_server`\
  Equivalent to `lockbox_key_server run`.
* `lockbox_key_server run [options]`
* `lockbox_key_server install [--force-config]`
* `lockbox_key_server uninstall [--purge-data]`
* `lockbox_key_server status`
* `lockbox_key_server resync-peer --peer-url URL [options]`
* `lockbox_key_server bench-store [options]`
* `lockbox_key_server bench-http [options]`
* `lockbox_key_server bench-http-fetch [options]`
* `lockbox_key_server bench-http-flow [options]`
* `lockbox_key_server help`
* `lockbox_key_server --help` / `-h`

### Global flags

* `--help`, `-h`\
  Print help and exit immediately.

### `run` options (also shared by commands that call `config_from_args`)

All options below are parsed for `run`, `bench-*`, `resync-peer` (indirectly), and are applied from `ServerConfig`.

#### Core server configuration

| Switch             | Type    | Default                       | Description                                                                            |
| ------------------ | ------- | ----------------------------- | -------------------------------------------------------------------------------------- |
| `--config PATH`    | string  | —                             | Load options from file first (`key = value`, `#` comments supported). Can be repeated. |
| `--bind ADDR`      | string  | `127.0.0.1:8089`              | Bind address for the HTTP server (e.g. `0.0.0.0:8089`).                                |
| `--state-dir PATH` | path    | `/var/lib/lockbox-key-server` | Directory used for persisted share store state.                                        |
| `--developer`      | flag    | false                         | Enables developer mode and switches state dir to a temp directory.                     |
| `--server-id N`    | integer | `0`                           | Routing server id. Must be 0..9.                                                       |
| `--cluster-id ID`  | string  | `"default"`                   | Public topology cluster id.                                                            |
| `--public-url URL` | string  | derived from `--bind`         | Public `/v1/share` base URL for this server.                                           |

#### Topology

| Switch                                | Type    | Default | Description                                                                               |
| ------------------------------------- | ------- | ------- | ----------------------------------------------------------------------------------------- |
| `--topology-version N`                | integer | `1`     | Public topology version.                                                                  |
| `--topology-server ID=URL[,STATUS]`   | string  | none    | Add topology entry. `STATUS` is `active` (default), `standby`, `promoted`, or `disabled`. |
| `--route OWNER=PRIMARY[,FAILOVER...]` | string  | none    | Add owner routing rule. At least one primary server id is required.                       |
| `--promoted-owner N`                  | integer | none    | Add promoted owner id. Can be repeated.                                                   |

#### Replication

| Switch                       | Type    | Default              | Description                                       |
| ---------------------------- | ------- | -------------------- | ------------------------------------------------- |
| `--replication-token TOKEN`  | string  | none                 | Shared replication token.                         |
| `--replication-peer-url URL` | string  | none                 | Allowed peer replication URLs. Can be repeated.   |
| `--origin-epoch N`           | integer | current epoch millis | Origin epoch for replication conflict resolution. |

#### Benchmarking

| Switch               | Type    | Default | Description                                |
| -------------------- | ------- | ------- | ------------------------------------------ |
| `--requests N`       | integer | `50000` | Number of requests for benchmark commands. |
| `--payload-bytes N`  | integer | `512`   | Payload size for benchmarking.             |
| `--concurrency N`    | integer | `0`     | Concurrency for benchmarking.              |
| `--preload-shares N` | integer | `0`     | Live shares to create before timing.       |

#### Storage and limits

| Switch                                          | Type    | Default    | Description                                             |
| ----------------------------------------------- | ------- | ---------- | ------------------------------------------------------- |
| `--compact-min-bytes N`                         | integer | `67108864` | Bytes in storage before background compaction runs.     |
| `--rate-limit-per-minute N`                     | integer | `120`      | Per-IP request limit. `0` disables rate limiting.       |
| `--rate-limit-burst N`                          | integer | `40`       | Per-IP rate limit burst capacity.                       |
| `--verification-email-command PATH`             | path    | none       | Invoked as `<command> <email> <url>`.                   |
| `--verification-email-rate-limit-per-hour N`    | integer | `5`        | Per-email verification email rate limit (per hour).     |
| `--verification-email-ip-rate-limit-per-hour N` | integer | `30`       | Per-source-IP verification email rate limit (per hour). |

### `install`, `uninstall`, `status`

#### `install [--force-config]`

* `--force-config`\
  Re-write `/etc/lockbox/key-server.toml` during install even when it already exists.

#### `uninstall [--purge-data]`

* `--purge-data`\
  Remove persisted data/cache/config paths on uninstall:
  * `/var/lib/lockbox-key-server`
  * `/var/cache/lockbox-key-server`
  * `/var/log/lockbox-key-server`
  * `/etc/lockbox/key-server.toml`

#### `status`

* No switches. Prints unit/config/state/log status.

### `resync-peer`

* `--peer-url URL`\
  Required. Target peer `/v1/replicate` endpoint.
* Other options: any `run` config option above (except `--peer-url`) may be passed and are parsed as part of configuration.

### Notes

* Unrecognized options cause an error.
* `--topology-server` and `--route` can be provided multiple times.
* `--replication-peer-url` and `--promoted-owner` can be provided multiple times.
* The parser is not using `clap`; flags are manually processed.



### Uninstall

```
sudo lockbox_key_server uninstall 
or
sudo lockbox_key_server uninstall --purge-data
```



