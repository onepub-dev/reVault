use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use install::{install_systemd, print_status, uninstall_systemd};
use lockbox_key_server::{install, server, server_log, store};
use lockbox_publish_protocol::{ServerStatus, TopologyRoute, TopologyServer};
use server::{bench_http, bench_http_flow, bench_http_receive, run_server};
use server_log::log_server_event;
use store::{PublishStore, ServerConfig, SmtpTlsMode};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            log_server_event(format!("command failed: {err}"));
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("run") | None => {
            let config = config_from_args(args.collect())?;
            let bind = config.bind_addr.clone();
            let store = Arc::new(PublishStore::open(config)?);
            store.start_topology_background();
            run_server(&bind, store)?;
        }
        Some("install") => install_systemd(args.any(|arg| arg == "--force-config"))?,
        Some("uninstall") => uninstall_systemd(args.any(|arg| arg == "--purge-data"))?,
        Some("status") => print_status()?,
        Some("resync-peer") => {
            let (mut config_args, peer_url) = split_peer_url_args(args.collect())?;
            let config = config_from_args(std::mem::take(&mut config_args))?;
            let store = PublishStore::open(config)?;
            let sent = store.resync_peer(&peer_url)?;
            println!("resynced_live_publishes={sent}");
        }
        Some("bench-store") => {
            let config = config_from_args(args.collect())?;
            require_dev_command(&config, "bench-store")?;
            store::bench_store(config)?;
        }
        Some("bench-http") => {
            let config = config_from_args(args.collect())?;
            require_dev_command(&config, "bench-http")?;
            bench_http(config)?;
        }
        Some("bench-http-receive") => {
            let config = config_from_args(args.collect())?;
            require_dev_command(&config, "bench-http-receive")?;
            bench_http_receive(config)?;
        }
        Some("bench-http-flow") => {
            let config = config_from_args(args.collect())?;
            require_dev_command(&config, "bench-http-flow")?;
            bench_http_flow(config)?;
        }
        Some("--help") | Some("-h") | Some("help") => print_help(args.any(|arg| arg == "--dev")),
        Some(other) => {
            return Err(format!("unknown command `{other}`").into());
        }
    }
    Ok(())
}

fn require_dev_command(
    config: &ServerConfig,
    command: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if !config.developer_mode {
        return Err(format!("command `{command}` requires --dev").into());
    }
    Ok(())
}

fn print_help(dev: bool) {
    println!("Usage:");
    println!("  lockbox_key_server run [--config PATH] [--bind ADDR] [--dev]");
    println!("  lockbox_key_server install [--force-config]");
    println!("  lockbox_key_server uninstall [--purge-data]");
    println!("  lockbox_key_server status");
    println!("  lockbox_key_server resync-peer [--config PATH] --peer-url URL");
    if dev {
        println!("  lockbox_key_server bench-store [--dev options]");
        println!("  lockbox_key_server bench-http [--dev options]");
        println!("  lockbox_key_server bench-http-receive [--dev options]");
        println!("  lockbox_key_server bench-http-flow [--dev options]");
    }
    println!();
    println!("Options:");
    println!("  --config PATH         Read server config file");
    println!("  --bind ADDR          Bind address for the HTTP server");
    println!("  --dev                Enable developer/test command-line overrides");
    println!("  --peer-url URL        Peer /v1/replicate URL for resync-peer");
    if !dev {
        println!("  --help --dev          Show developer/test overrides");
        return;
    }
    println!();
    println!("Developer/test options:");
    println!("  --state-dir PATH      Directory used for persisted publish store state");
    println!("  --server-id N          Server routing id, 0..35 (0..9, a..z), default 0");
    println!("  --cluster-id ID        Public topology cluster id");
    println!("  --public-url URL       External URL for this server's publish API");
    println!("  --topology-version N   Public topology version");
    println!("  --topology-token TOKEN  Shared token for topology heartbeat");
    println!(
        "  --topology-stale-after-ms N  Ignore peers that have not checked in within this age"
    );
    println!("  --topology-heartbeat-interval-ms N  Interval between topology heartbeat posts");
    println!("  --topology-server ID=URL[,STATUS]  Add server to public topology");
    println!("  --route OWNER=PRIMARY[,FAILOVER...]  Add owner routing rule");
    println!("  --replication-token TOKEN  Shared peer replication token");
    println!("  --replication-peer-url URL  Peer /v1/replicate URL");
    println!("  --origin-epoch N      Local replication epoch");
    println!("  --promoted-owner N    Serve replicated published payloads for owner id N");
    println!("  --requests N           Benchmark request count");
    println!("  --payload-bytes N      Benchmark payload size");
    println!("  --concurrency N        Benchmark concurrency");
    println!(
        "  --preload-published-payloads N     Benchmark published payloads to create before timing"
    );
    println!("  --compact-min-bytes N  Segment size before background compaction");
    println!("  --rate-limit-per-minute N  Per-IP request rate, 0 disables");
    println!("  --rate-limit-burst N       Per-IP burst capacity");
    println!("  --verification-ttl-seconds N       Email verification link lifetime");
    println!("  --default-receive-ttl-seconds N    Receive lifetime after email verification");
    println!("  --max-receive-ttl-seconds N        Maximum requested receive lifetime");
    println!("  --smtp-host HOST      SMTP server hostname");
    println!("  --smtp-port N         SMTP server port, default 587");
    println!("  --smtp-username USER  SMTP username");
    println!("  --smtp-password PASS  SMTP password or app password");
    println!("  --smtp-from EMAIL     Sender address");
    println!("  --smtp-tls MODE       starttls, tls, or none");
    println!("  --smtp-timeout-seconds N");
    println!("  --verification-email-subject TEXT");
    println!("  --verification-email-template TEXT");
    println!("  --verification-email-rate-limit-per-hour N");
    println!("  --verification-email-ip-rate-limit-per-hour N");
}

fn split_peer_url_args(
    args: Vec<String>,
) -> Result<(Vec<String>, String), Box<dyn std::error::Error>> {
    let mut out = Vec::new();
    let mut peer_url = None;
    let mut index = 0usize;
    while index < args.len() {
        if args[index] == "--peer-url" {
            index += 1;
            peer_url = Some(
                args.get(index)
                    .ok_or("missing value for --peer-url")?
                    .to_string(),
            );
        } else {
            out.push(args[index].clone());
        }
        index += 1;
    }
    Ok((out, peer_url.ok_or("missing --peer-url")?))
}

fn config_from_args(args: Vec<String>) -> Result<ServerConfig, Box<dyn std::error::Error>> {
    let mut config = ServerConfig::default();
    let dev_options = args.iter().any(|arg| arg == "--dev");
    let mut index = 0;
    while index < args.len() {
        let option = args[index].as_str();
        if dev_only_option(option) && !dev_options {
            return Err(format!(
                "option `{option}` requires --dev; put server configuration in --config PATH"
            )
            .into());
        }
        match option {
            "--bind" => {
                index += 1;
                config.bind_addr = args
                    .get(index)
                    .ok_or("missing value for --bind")?
                    .to_string();
            }
            "--state-dir" => {
                index += 1;
                config.state_dir = PathBuf::from(args.get(index).ok_or("missing value")?);
            }
            "--config" => {
                index += 1;
                apply_config_file(&mut config, args.get(index).ok_or("missing value")?)?;
            }
            "--dev" => config.developer_mode = true,
            "--server-id" => {
                index += 1;
                config.server_id =
                    parse_server_id(args.get(index).ok_or("missing value for --server-id")?)?;
            }
            "--cluster-id" => {
                index += 1;
                config.cluster_id = args
                    .get(index)
                    .ok_or("missing value for --cluster-id")?
                    .to_string();
            }
            "--public-url" => {
                index += 1;
                config.public_url = Some(
                    args.get(index)
                        .ok_or("missing value for --public-url")?
                        .to_string(),
                );
            }
            "--topology-version" => {
                index += 1;
                config.topology_version = args
                    .get(index)
                    .ok_or("missing value for --topology-version")?
                    .parse()?;
            }
            "--topology-token" => {
                index += 1;
                config.topology_token = Some(
                    args.get(index)
                        .ok_or("missing value for --topology-token")?
                        .to_string(),
                );
            }
            "--topology-server" => {
                index += 1;
                config.topology_servers.push(parse_topology_server(
                    args.get(index).ok_or("missing value")?,
                )?);
            }
            "--topology-stale-after-ms" => {
                index += 1;
                config.topology_stale_after_ms = args
                    .get(index)
                    .ok_or("missing value for --topology-stale-after-ms")?
                    .parse()?;
            }
            "--topology-heartbeat-interval-ms" => {
                index += 1;
                config.topology_heartbeat_interval_ms = args
                    .get(index)
                    .ok_or("missing value for --topology-heartbeat-interval-ms")?
                    .parse()?;
            }
            "--route" => {
                index += 1;
                config.topology_routes.push(parse_topology_route(
                    args.get(index).ok_or("missing value")?,
                )?);
            }
            "--replication-token" => {
                index += 1;
                config.replication_token = Some(
                    args.get(index)
                        .ok_or("missing value for --replication-token")?
                        .to_string(),
                );
            }
            "--replication-peer-url" => {
                index += 1;
                config.replication_peer_urls.push(
                    args.get(index)
                        .ok_or("missing value for --replication-peer-url")?
                        .to_string(),
                );
            }
            "--origin-epoch" => {
                index += 1;
                config.origin_epoch = args
                    .get(index)
                    .ok_or("missing value for --origin-epoch")?
                    .parse()?;
            }
            "--promoted-owner" => {
                index += 1;
                config
                    .promoted_owner_ids
                    .push(args.get(index).ok_or("missing value")?.parse()?);
            }
            "--requests" => {
                index += 1;
                config.benchmark_requests = args
                    .get(index)
                    .ok_or("missing value for --requests")?
                    .parse()?;
            }
            "--payload-bytes" => {
                index += 1;
                config.benchmark_payload_bytes = args
                    .get(index)
                    .ok_or("missing value for --payload-bytes")?
                    .parse()?;
            }
            "--concurrency" => {
                index += 1;
                config.benchmark_concurrency = args
                    .get(index)
                    .ok_or("missing value for --concurrency")?
                    .parse()?;
            }
            "--preload-published-payloads" => {
                index += 1;
                config.benchmark_preload_published_payloads = args
                    .get(index)
                    .ok_or("missing value for --preload-published-payloads")?
                    .parse()?;
            }
            "--compact-min-bytes" => {
                index += 1;
                config.compact_min_bytes = args
                    .get(index)
                    .ok_or("missing value for --compact-min-bytes")?
                    .parse()?;
            }
            "--rate-limit-per-minute" => {
                index += 1;
                config.rate_limit_per_minute = args
                    .get(index)
                    .ok_or("missing value for --rate-limit-per-minute")?
                    .parse()?;
            }
            "--rate-limit-burst" => {
                index += 1;
                config.rate_limit_burst = args
                    .get(index)
                    .ok_or("missing value for --rate-limit-burst")?
                    .parse()?;
            }
            "--verification-ttl-seconds" => {
                index += 1;
                config.verification_ttl = Duration::from_secs(
                    args.get(index)
                        .ok_or("missing value for --verification-ttl-seconds")?
                        .parse()?,
                );
            }
            "--default-receive-ttl-seconds" => {
                index += 1;
                config.default_receive_ttl = Duration::from_secs(
                    args.get(index)
                        .ok_or("missing value for --default-receive-ttl-seconds")?
                        .parse()?,
                );
            }
            "--max-receive-ttl-seconds" => {
                index += 1;
                config.max_receive_ttl = Duration::from_secs(
                    args.get(index)
                        .ok_or("missing value for --max-receive-ttl-seconds")?
                        .parse()?,
                );
            }
            "--smtp-host" => {
                index += 1;
                config.smtp_host = Some(
                    args.get(index)
                        .ok_or("missing value for --smtp-host")?
                        .to_string(),
                );
            }
            "--smtp-port" => {
                index += 1;
                config.smtp_port = args
                    .get(index)
                    .ok_or("missing value for --smtp-port")?
                    .parse()?;
            }
            "--smtp-username" => {
                index += 1;
                config.smtp_username = Some(
                    args.get(index)
                        .ok_or("missing value for --smtp-username")?
                        .to_string(),
                );
            }
            "--smtp-password" => {
                index += 1;
                config.smtp_password = Some(
                    args.get(index)
                        .ok_or("missing value for --smtp-password")?
                        .to_string(),
                );
            }
            "--smtp-from" => {
                index += 1;
                config.smtp_from = Some(
                    args.get(index)
                        .ok_or("missing value for --smtp-from")?
                        .to_string(),
                );
            }
            "--smtp-tls" => {
                index += 1;
                config.smtp_tls =
                    parse_smtp_tls_mode(args.get(index).ok_or("missing value for --smtp-tls")?)?;
            }
            "--smtp-timeout-seconds" => {
                index += 1;
                config.smtp_timeout = Duration::from_secs(
                    args.get(index)
                        .ok_or("missing value for --smtp-timeout-seconds")?
                        .parse()?,
                );
            }
            "--verification-email-subject" => {
                index += 1;
                config.verification_email_subject = args
                    .get(index)
                    .ok_or("missing value for --verification-email-subject")?
                    .to_string();
            }
            "--verification-email-template" => {
                index += 1;
                config.verification_email_template = args
                    .get(index)
                    .ok_or("missing value for --verification-email-template")?
                    .to_string();
            }
            "--verification-email-rate-limit-per-hour" => {
                index += 1;
                config.verification_email_rate_limit_per_hour = args
                    .get(index)
                    .ok_or("missing value for --verification-email-rate-limit-per-hour")?
                    .parse()?;
            }
            "--verification-email-ip-rate-limit-per-hour" => {
                index += 1;
                config.verification_email_ip_rate_limit_per_hour = args
                    .get(index)
                    .ok_or("missing value for --verification-email-ip-rate-limit-per-hour")?
                    .parse()?;
            }
            "--help" | "-h" => {
                print_help(dev_options);
                std::process::exit(0);
            }
            other => return Err(format!("unknown option `{other}`").into()),
        }
        index += 1;
    }
    Ok(config)
}

fn dev_only_option(option: &str) -> bool {
    matches!(
        option,
        "--state-dir"
            | "--server-id"
            | "--cluster-id"
            | "--public-url"
            | "--topology-version"
            | "--topology-token"
            | "--topology-server"
            | "--topology-stale-after-ms"
            | "--topology-heartbeat-interval-ms"
            | "--route"
            | "--replication-token"
            | "--replication-peer-url"
            | "--origin-epoch"
            | "--promoted-owner"
            | "--requests"
            | "--payload-bytes"
            | "--concurrency"
            | "--preload-published-payloads"
            | "--compact-min-bytes"
            | "--rate-limit-per-minute"
            | "--rate-limit-burst"
            | "--verification-ttl-seconds"
            | "--default-receive-ttl-seconds"
            | "--max-receive-ttl-seconds"
            | "--smtp-host"
            | "--smtp-port"
            | "--smtp-username"
            | "--smtp-password"
            | "--smtp-from"
            | "--smtp-tls"
            | "--smtp-timeout-seconds"
            | "--verification-email-subject"
            | "--verification-email-template"
            | "--verification-email-rate-limit-per-hour"
            | "--verification-email-ip-rate-limit-per-hour"
    )
}

#[derive(Default)]
struct TopologyServerTable {
    id: Option<u8>,
    url: Option<String>,
    status: Option<ServerStatus>,
}

#[derive(Default)]
struct TopologyRouteTable {
    owner: Option<u8>,
    primary: Option<u8>,
    failover: Vec<u8>,
}

enum ConfigTable {
    TopologyServer(TopologyServerTable),
    Route(TopologyRouteTable),
}

fn apply_config_file(
    config: &mut ServerConfig,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut table = None;
    for (line_no, raw_line) in fs::read_to_string(path)?.lines().enumerate() {
        let line = raw_line
            .split_once('#')
            .map(|(value, _)| value)
            .unwrap_or(raw_line)
            .trim();
        if line.is_empty() {
            continue;
        }
        if let Some(name) = parse_array_table_header(line) {
            flush_config_table(config, table.take())
                .map_err(|err| format!("{path}:{}: {err}", line_no + 1))?;
            table = Some(match name {
                "topology_server" => ConfigTable::TopologyServer(TopologyServerTable::default()),
                "route" => ConfigTable::Route(TopologyRouteTable::default()),
                other => {
                    return Err(
                        format!("{path}:{}: unknown config table `{other}`", line_no + 1).into(),
                    )
                }
            });
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| format!("{path}:{}: expected key = value", line_no + 1))?;
        let key = key.trim();
        let value = value.trim();
        if let Some(current) = table.as_mut() {
            apply_config_table_value(current, key, value)
        } else {
            apply_config_value(config, key, parse_config_value(value)?)
        }
        .map_err(|err| format!("{path}:{}: {err}", line_no + 1))?;
    }
    flush_config_table(config, table).map_err(|err| format!("{path}: {err}"))?;
    Ok(())
}

fn parse_array_table_header(line: &str) -> Option<&str> {
    line.strip_prefix("[[")?.strip_suffix("]]").map(str::trim)
}

fn parse_config_value(value: &str) -> Result<String, Box<dyn std::error::Error>> {
    let value = value.trim();
    if let Some(value) = value.strip_prefix('"') {
        let value = value
            .strip_suffix('"')
            .ok_or("unterminated quoted config value")?;
        return Ok(value.replace("\\n", "\n").replace("\\\"", "\""));
    }
    Ok(value.to_string())
}

fn apply_config_value(
    config: &mut ServerConfig,
    key: &str,
    value: String,
) -> Result<(), Box<dyn std::error::Error>> {
    match key {
        "bind_addr" => config.bind_addr = value,
        "state_dir" => config.state_dir = PathBuf::from(value),
        "server_id" => config.server_id = parse_server_id(&value)?,
        "cluster_id" => config.cluster_id = value,
        "public_url" => {
            config.public_url = if value.is_empty() { None } else { Some(value) };
        }
        "topology_version" => config.topology_version = value.parse()?,
        "topology_token" => {
            config.topology_token = if value.is_empty() { None } else { Some(value) };
        }
        "topology_stale_after_ms" => config.topology_stale_after_ms = value.parse()?,
        "topology_heartbeat_interval_ms" => {
            config.topology_heartbeat_interval_ms = value.parse()?;
        }
        "replication_token" => {
            config.replication_token = if value.is_empty() { None } else { Some(value) };
        }
        "replication_peer_url" => config.replication_peer_urls.push(value),
        "origin_epoch" => config.origin_epoch = value.parse()?,
        "promoted_owner" => config.promoted_owner_ids.push(parse_server_id(&value)?),
        "compact_min_bytes" => config.compact_min_bytes = value.parse()?,
        "rate_limit_per_minute" => config.rate_limit_per_minute = value.parse()?,
        "rate_limit_burst" => config.rate_limit_burst = value.parse()?,
        "verification_ttl_seconds" => {
            config.verification_ttl = Duration::from_secs(value.parse()?);
        }
        "default_receive_ttl_seconds" => {
            config.default_receive_ttl = Duration::from_secs(value.parse()?);
        }
        "max_receive_ttl_seconds" => {
            config.max_receive_ttl = Duration::from_secs(value.parse()?);
        }
        "smtp_host" => {
            config.smtp_host = if value.is_empty() { None } else { Some(value) };
        }
        "smtp_port" => config.smtp_port = value.parse()?,
        "smtp_username" => {
            config.smtp_username = if value.is_empty() { None } else { Some(value) };
        }
        "smtp_password" => {
            config.smtp_password = if value.is_empty() { None } else { Some(value) };
        }
        "smtp_from" => {
            config.smtp_from = if value.is_empty() { None } else { Some(value) };
        }
        "smtp_tls" => config.smtp_tls = parse_smtp_tls_mode(&value)?,
        "smtp_timeout_seconds" => {
            config.smtp_timeout = Duration::from_secs(value.parse()?);
        }
        "verification_email_subject" => config.verification_email_subject = value,
        "verification_email_template" => config.verification_email_template = value,
        "verification_email_rate_limit_per_hour" => {
            config.verification_email_rate_limit_per_hour = value.parse()?;
        }
        "verification_email_ip_rate_limit_per_hour" => {
            config.verification_email_ip_rate_limit_per_hour = value.parse()?;
        }
        "max_payload_bytes" => config.max_payload_bytes = value.parse()?,
        "default_ttl_seconds" | "max_ttl_seconds" => {
            return Err(format!(
                "`{key}` was replaced by default_receive_ttl_seconds and max_receive_ttl_seconds"
            )
            .into());
        }
        "shard_count" => config.shard_count = value.parse()?,
        "max_receives_per_publish" => config.max_receives_per_publish = value.parse()?,
        "index_cache_entries" => config.index_cache_entries = value.parse()?,
        "developer_mode" => config.developer_mode = value.parse()?,
        other => return Err(format!("unknown config key `{other}`").into()),
    }
    Ok(())
}

fn apply_config_table_value(
    table: &mut ConfigTable,
    key: &str,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    match table {
        ConfigTable::TopologyServer(server) => match key {
            "id" => server.id = Some(parse_server_id(&parse_config_value(value)?)?),
            "url" => server.url = Some(parse_config_value(value)?),
            "status" => server.status = Some(parse_server_status(&parse_config_value(value)?)?),
            other => return Err(format!("unknown topology_server key `{other}`").into()),
        },
        ConfigTable::Route(route) => match key {
            "owner" => route.owner = Some(parse_server_id(&parse_config_value(value)?)?),
            "primary" => route.primary = Some(parse_server_id(&parse_config_value(value)?)?),
            "failover" => route.failover = parse_server_id_array(value)?,
            other => return Err(format!("unknown route key `{other}`").into()),
        },
    }
    Ok(())
}

fn flush_config_table(
    config: &mut ServerConfig,
    table: Option<ConfigTable>,
) -> Result<(), Box<dyn std::error::Error>> {
    match table {
        Some(ConfigTable::TopologyServer(server)) => {
            let id = server.id.ok_or("topology_server.id is required")?;
            let url = server.url.ok_or("topology_server.url is required")?;
            config.topology_servers.push(TopologyServer {
                id,
                url,
                status: server.status.unwrap_or(ServerStatus::Active),
                last_seen_ms: None,
            });
        }
        Some(ConfigTable::Route(route)) => {
            config.topology_routes.push(TopologyRoute {
                owner_id: route.owner.ok_or("route.owner is required")?,
                primary_id: route.primary.ok_or("route.primary is required")?,
                failover_ids: route.failover,
            });
        }
        None => {}
    }
    Ok(())
}

fn parse_topology_server(value: &str) -> Result<TopologyServer, Box<dyn std::error::Error>> {
    let (id, rest) = value
        .split_once('=')
        .ok_or("topology server must be ID=URL[,STATUS]")?;
    let id = parse_server_id(id)?;
    let mut parts = rest.splitn(2, ',');
    let url = parts.next().unwrap_or_default().to_string();
    let status = parse_server_status(parts.next().unwrap_or("active"))?;
    Ok(TopologyServer {
        id,
        url,
        status,
        last_seen_ms: None,
    })
}

fn parse_server_status(value: &str) -> Result<ServerStatus, Box<dyn std::error::Error>> {
    Ok(match value {
        "active" => ServerStatus::Active,
        "standby" => ServerStatus::Standby,
        "promoted" => ServerStatus::Promoted,
        "disabled" => ServerStatus::Disabled,
        other => return Err(format!("unknown server status `{other}`").into()),
    })
}

fn parse_smtp_tls_mode(value: &str) -> Result<SmtpTlsMode, Box<dyn std::error::Error>> {
    Ok(match value {
        "starttls" => SmtpTlsMode::StartTls,
        "tls" | "ssl" => SmtpTlsMode::Tls,
        "none" => SmtpTlsMode::None,
        other => return Err(format!("unknown smtp_tls mode `{other}`").into()),
    })
}

fn parse_server_id_array(value: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let value = value.trim();
    let value = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .ok_or("expected server id array")?
        .trim();
    if value.is_empty() {
        return Ok(Vec::new());
    }
    value
        .split(',')
        .map(|part| parse_server_id(&parse_config_value(part.trim())?))
        .collect()
}

fn parse_topology_route(value: &str) -> Result<TopologyRoute, Box<dyn std::error::Error>> {
    let (owner_id, rest) = value
        .split_once('=')
        .ok_or("route must be OWNER=PRIMARY[,FAILOVER...]")?;
    let mut ids = Vec::new();
    for part in rest.split(',') {
        ids.push(parse_server_id(part)?);
    }
    if ids.is_empty() {
        return Err("route must include a primary server id".into());
    }
    let Some((primary_id, failover_ids)) = ids.split_first() else {
        return Err("route must include a primary server id".into());
    };
    Ok(TopologyRoute {
        owner_id: parse_server_id(owner_id)?,
        primary_id: *primary_id,
        failover_ids: failover_ids.to_vec(),
    })
}

fn parse_server_id(value: &str) -> Result<u8, Box<dyn std::error::Error>> {
    if value.len() == 1 {
        let id = match value.as_bytes().first().copied() {
            Some(byte @ b'0'..=b'9') => byte - b'0',
            Some(byte @ b'a'..=b'z') => byte - b'a' + 10,
            _ => {
                let id = value.parse::<u8>()?;
                if id >= 36 {
                    return Err(format!("server id must be 0..35: {id}").into());
                }
                return Ok(id);
            }
        };
        if id >= 36 {
            return Err(format!("server id must be 0..35: {id}").into());
        }
        return Ok(id);
    }
    let id = value.parse()?;
    if id >= 36 {
        return Err(format!("server id must be 0..35: {id}").into());
    }
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::{apply_config_file, config_from_args, require_dev_command};
    use lockbox_key_server::store::{ServerConfig, SmtpTlsMode};
    use lockbox_publish_protocol::ServerStatus;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    static TEST_FILE_COUNTER: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn config_cli_rejects_hidden_server_options_without_dev() {
        let err = config_from_args(vec![
            "--state-dir".to_string(),
            "/tmp/lockbox-key-server-test".to_string(),
        ])
        .unwrap_err()
        .to_string();
        assert!(err.contains("requires --dev"));
        assert!(err.contains("--config PATH"));
    }

    #[test]
    fn config_cli_allows_hidden_server_options_with_dev() {
        let config = config_from_args(vec![
            "--dev".to_string(),
            "--state-dir".to_string(),
            "/tmp/lockbox-key-server-test".to_string(),
            "--server-id".to_string(),
            "a".to_string(),
        ])
        .unwrap();
        assert!(config.developer_mode);
        assert_eq!(
            config.state_dir,
            PathBuf::from("/tmp/lockbox-key-server-test")
        );
        assert_eq!(config.server_id, 10);
    }

    #[test]
    fn benchmark_commands_require_dev_mode() {
        let config = config_from_args(Vec::new()).unwrap();
        let err = require_dev_command(&config, "bench-store")
            .unwrap_err()
            .to_string();
        assert_eq!(err, "command `bench-store` requires --dev");

        let config = config_from_args(vec!["--dev".to_string()]).unwrap();
        require_dev_command(&config, "bench-store").unwrap();
    }

    #[test]
    fn config_file_accepts_topology_arrays_of_tables() {
        let path = temp_config_path("topology-arrays");
        fs::write(
            &path,
            r#"
bind_addr = "127.0.0.1:8099"
cluster_id = "production"
max_receives_per_publish = 3

[[topology_server]]
id = 0
url = "https://keypublish0.example.com/v1/publish"
status = "active"

[[topology_server]]
id = 1
url = "https://keypublish1.example.com/v1/publish"
status = "standby"

[[route]]
owner = 0
primary = 0
failover = [1]

[[route]]
owner = 1
primary = 1
failover = [0]
"#,
        )
        .unwrap();
        let mut config = ServerConfig::default();

        apply_config_file(&mut config, path.to_str().unwrap()).unwrap();

        assert_eq!(config.bind_addr, "127.0.0.1:8099");
        assert_eq!(config.cluster_id, "production");
        assert_eq!(config.max_receives_per_publish, 3);
        assert_eq!(config.topology_servers.len(), 2);
        assert_eq!(config.topology_servers[0].id, 0);
        assert_eq!(
            config.topology_servers[0].url,
            "https://keypublish0.example.com/v1/publish"
        );
        assert_eq!(config.topology_servers[0].status, ServerStatus::Active);
        assert_eq!(config.topology_servers[1].id, 1);
        assert_eq!(config.topology_servers[1].status, ServerStatus::Standby);
        assert_eq!(config.topology_routes.len(), 2);
        assert_eq!(config.topology_routes[0].owner_id, 0);
        assert_eq!(config.topology_routes[0].primary_id, 0);
        assert_eq!(config.topology_routes[0].failover_ids, vec![1]);
        assert_eq!(config.topology_routes[1].owner_id, 1);
        assert_eq!(config.topology_routes[1].primary_id, 1);
        assert_eq!(config.topology_routes[1].failover_ids, vec![0]);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn config_file_accepts_email_ttl_and_smtp_settings() {
        let path = temp_config_path("email-smtp");
        fs::write(
            &path,
            r#"
verification_ttl_seconds = 1800
default_receive_ttl_seconds = 7200
max_receive_ttl_seconds = 7200
smtp_host = "smtp.gmail.com"
smtp_port = 587
smtp_username = "publisher@example.com"
smtp_password = "app-password"
smtp_from = "publisher@example.com"
smtp_tls = "starttls"
smtp_timeout_seconds = 300
verification_email_subject = "Verify {publish_code}"
verification_email_template = "Line one\n{verification_url}"
"#,
        )
        .unwrap();
        let mut config = ServerConfig::default();

        apply_config_file(&mut config, path.to_str().unwrap()).unwrap();

        assert_eq!(config.verification_ttl, Duration::from_secs(1800));
        assert_eq!(config.default_receive_ttl, Duration::from_secs(7200));
        assert_eq!(config.max_receive_ttl, Duration::from_secs(7200));
        assert_eq!(config.smtp_host.as_deref(), Some("smtp.gmail.com"));
        assert_eq!(config.smtp_port, 587);
        assert_eq!(
            config.smtp_username.as_deref(),
            Some("publisher@example.com")
        );
        assert_eq!(config.smtp_password.as_deref(), Some("app-password"));
        assert_eq!(config.smtp_from.as_deref(), Some("publisher@example.com"));
        assert_eq!(config.smtp_tls, SmtpTlsMode::StartTls);
        assert_eq!(config.smtp_timeout, Duration::from_secs(300));
        assert_eq!(config.verification_email_subject, "Verify {publish_code}");
        assert_eq!(
            config.verification_email_template,
            "Line one\n{verification_url}"
        );
        let _ = fs::remove_file(path);
    }

    #[test]
    fn config_file_rejects_incomplete_topology_tables() {
        let path = temp_config_path("incomplete-topology-array");
        fs::write(
            &path,
            r#"
[[topology_server]]
id = 0
"#,
        )
        .unwrap();
        let mut config = ServerConfig::default();

        let err = apply_config_file(&mut config, path.to_str().unwrap())
            .unwrap_err()
            .to_string();

        assert!(err.contains("topology_server.url is required"));
        let _ = fs::remove_file(path);
    }

    fn temp_config_path(name: &str) -> PathBuf {
        let counter = TEST_FILE_COUNTER.fetch_add(1, Ordering::SeqCst);
        std::env::temp_dir().join(format!(
            "lockbox-key-server-{name}-{}-{counter}.toml",
            std::process::id()
        ))
    }
}
