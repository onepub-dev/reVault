use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use clap::{error::ErrorKind, Arg, ArgAction, ArgMatches, Command};
use install::{
    install_systemd, print_status, start_systemd, stop_systemd, uninstall_systemd, CONFIG_PATH,
};
use revault_key_server::{install, server, server_log, store};
use revault_publish_protocol::{ServerStatus, TopologyRoute, TopologyServer};
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
    let mut args: Vec<String> = env::args().skip(1).collect();
    let wants_help = args.iter().any(|arg| arg == "--help" || arg == "-h");
    let wants_version = args.iter().any(|arg| arg == "--version" || arg == "-V");
    let root_help = args.first().is_some_and(|arg| arg == "help")
        || (args.first().is_some_and(|arg| arg.starts_with('-')) && wants_help);
    if root_help {
        key_server_command(true).print_help()?;
        println!();
        return Ok(());
    }
    if args.is_empty() || (!wants_version && args.first().is_some_and(|arg| arg.starts_with('-'))) {
        args.insert(0, "run".to_string());
    }
    let mut argv = vec!["revault_key_server".to_string()];
    argv.extend(args.iter().cloned());
    let matches = match key_server_command(args.iter().any(|arg| arg == "--dev"))
        .try_get_matches_from(argv)
    {
        Ok(matches) => matches,
        Err(err) if err.kind() == ErrorKind::DisplayHelp => {
            err.print()?;
            return Ok(());
        }
        Err(err) => return Err(err.into()),
    };
    match matches.subcommand() {
        Some(("run", _)) => {
            let config = config_from_args(args[1..].to_vec())?;
            let bind = config.bind_addr.clone();
            println!("reVault key server starting");
            println!("  bind address: {bind}");
            println!("  state directory: {}", config.state_dir.display());
            println!(
                "  public URL: {}",
                config.public_url.as_deref().unwrap_or("not configured")
            );
            let store = Arc::new(PublishStore::open(config)?);
            store.start_topology_background();
            run_server(&bind, store)?;
        }
        Some(("install", command)) => install_systemd(command.get_flag("force-config"))?,
        Some(("uninstall", command)) => uninstall_systemd(command.get_flag("purge-data"))?,
        Some(("start", _)) => start_systemd()?,
        Some(("stop", _)) => stop_systemd()?,
        Some(("doctor", _)) => print_doctor()?,
        Some(("resync-peer", _)) => {
            let (mut config_args, peer_url) = split_peer_url_args(args[1..].to_vec())?;
            let config = config_from_args(std::mem::take(&mut config_args))?;
            let store = PublishStore::open(config)?;
            let sent = store.resync_peer(&peer_url)?;
            println!("resynced_live_publishes={sent}");
        }
        Some(("bench-store", _)) => {
            let config = config_from_args(args[1..].to_vec())?;
            require_dev_command(&config, "bench-store")?;
            store::bench_store(config)?;
        }
        Some(("bench-http", _)) => {
            let config = config_from_args(args[1..].to_vec())?;
            require_dev_command(&config, "bench-http")?;
            bench_http(config)?;
        }
        Some(("bench-http-receive", _)) => {
            let config = config_from_args(args[1..].to_vec())?;
            require_dev_command(&config, "bench-http-receive")?;
            bench_http_receive(config)?;
        }
        Some(("bench-http-flow", _)) => {
            let config = config_from_args(args[1..].to_vec())?;
            require_dev_command(&config, "bench-http-flow")?;
            bench_http_flow(config)?;
        }
        _ => {
            key_server_command(false).print_help()?;
            println!();
        }
    }
    Ok(())
}

fn key_server_command(dev_help: bool) -> Command {
    Command::new("revault_key_server")
        .version(env!("CARGO_PKG_VERSION"))
        .about("High-throughput reVault key rendezvous server")
        .subcommand(add_config_args(
            Command::new("run").about("Run the key server"),
            dev_help,
        ))
        .subcommand(
            Command::new("install")
                .about("Install the system service")
                .arg(
                    Arg::new("force-config")
                        .long("force-config")
                        .action(ArgAction::SetTrue)
                        .help("Rewrite the default config during install"),
                ),
        )
        .subcommand(
            Command::new("uninstall")
                .about("Uninstall the system service")
                .arg(
                    Arg::new("purge-data")
                        .long("purge-data")
                        .action(ArgAction::SetTrue)
                        .help("Remove persisted service data"),
                ),
        )
        .subcommand(Command::new("start").about("Start the system service"))
        .subcommand(Command::new("stop").about("Stop the system service"))
        .subcommand(Command::new("doctor").about("Check the installed service and configuration"))
        .subcommand(add_config_args(
            Command::new("resync-peer")
                .about("Resync live publishes to a peer")
                .arg(
                    Arg::new("peer-url")
                        .long("peer-url")
                        .value_name("URL")
                        .num_args(1)
                        .required(true)
                        .help("Peer /v1/replicate URL"),
                ),
            dev_help,
        ))
        .subcommand(add_config_args(
            Command::new("bench-store")
                .about("Benchmark the store")
                .hide(!dev_help),
            dev_help,
        ))
        .subcommand(add_config_args(
            Command::new("bench-http")
                .about("Benchmark publish over HTTP")
                .hide(!dev_help),
            dev_help,
        ))
        .subcommand(add_config_args(
            Command::new("bench-http-receive")
                .about("Benchmark receive over HTTP")
                .hide(!dev_help),
            dev_help,
        ))
        .subcommand(add_config_args(
            Command::new("bench-http-flow")
                .about("Benchmark publish/receive flow over HTTP")
                .hide(!dev_help),
            dev_help,
        ))
}

fn print_doctor() -> Result<(), Box<dyn std::error::Error>> {
    print_status()?;

    let config_path = PathBuf::from(CONFIG_PATH);
    if !config_path.exists() {
        println!();
        println!("Configuration details unavailable: the configuration file is missing.");
        println!("Next step: run `sudo lockbox_key_server install` to create it.");
        return Ok(());
    }

    let config = config_from_args(vec!["--config".to_string(), CONFIG_PATH.to_string()])?;
    let smtp_configured = config.smtp_host.is_some()
        && config.smtp_username.is_some()
        && config.smtp_password.is_some();

    println!();
    println!("Application configuration");
    println!("  Configuration is valid: YES");
    println!("  Bind address: {}", config.bind_addr);
    println!(
        "  Public URL: {}",
        config.public_url.as_deref().unwrap_or("not configured")
    );
    println!("  State directory: {}", config.state_dir.display());
    println!(
        "  State directory present: {}",
        doctor_yes_no(config.state_dir.exists())
    );
    println!("  Topology servers: {}", config.topology_servers.len());
    println!("  Topology routes: {}", config.topology_routes.len());
    println!("  SMTP complete: {}", doctor_yes_no(smtp_configured));

    if config.public_url.is_none() {
        println!(
            "  Warning: public URL is not configured; external clients cannot route reliably."
        );
    }
    if !smtp_configured {
        println!("  Warning: SMTP is incomplete; email verification will not work.");
    }
    if !config.state_dir.exists() {
        println!("  Warning: the state directory will be created when the service starts.");
    }
    Ok(())
}

fn doctor_yes_no(value: bool) -> &'static str {
    if value {
        "YES"
    } else {
        "NO"
    }
}

fn config_command(dev_help: bool) -> Command {
    add_config_args(Command::new("config"), dev_help)
}

fn add_config_args(command: Command, dev_help: bool) -> Command {
    command
        .arg(config_arg("config", "config", "PATH", false, dev_help))
        .arg(config_arg("bind", "bind", "ADDR", false, dev_help))
        .arg(
            Arg::new("dev")
                .long("dev")
                .action(ArgAction::SetTrue)
                .help("Enable developer/test command-line overrides"),
        )
        .arg(config_arg("state-dir", "state-dir", "PATH", true, dev_help))
        .arg(config_arg("server-id", "server-id", "N", true, dev_help))
        .arg(config_arg("cluster-id", "cluster-id", "ID", true, dev_help))
        .arg(config_arg(
            "public-url",
            "public-url",
            "URL",
            true,
            dev_help,
        ))
        .arg(config_arg(
            "topology-version",
            "topology-version",
            "N",
            true,
            dev_help,
        ))
        .arg(config_arg(
            "topology-token",
            "topology-token",
            "TOKEN",
            true,
            dev_help,
        ))
        .arg(repeated_config_arg(
            "topology-server",
            "topology-server",
            "ID=URL[,STATUS]",
            true,
            dev_help,
        ))
        .arg(config_arg(
            "topology-stale-after-ms",
            "topology-stale-after-ms",
            "N",
            true,
            dev_help,
        ))
        .arg(config_arg(
            "topology-heartbeat-interval-ms",
            "topology-heartbeat-interval-ms",
            "N",
            true,
            dev_help,
        ))
        .arg(repeated_config_arg(
            "route",
            "route",
            "OWNER=PRIMARY[,FAILOVER...]",
            true,
            dev_help,
        ))
        .arg(config_arg(
            "replication-token",
            "replication-token",
            "TOKEN",
            true,
            dev_help,
        ))
        .arg(repeated_config_arg(
            "replication-peer-url",
            "replication-peer-url",
            "URL",
            true,
            dev_help,
        ))
        .arg(config_arg(
            "origin-epoch",
            "origin-epoch",
            "N",
            true,
            dev_help,
        ))
        .arg(repeated_config_arg(
            "promoted-owner",
            "promoted-owner",
            "N",
            true,
            dev_help,
        ))
        .arg(config_arg("requests", "requests", "N", true, dev_help))
        .arg(config_arg(
            "payload-bytes",
            "payload-bytes",
            "N",
            true,
            dev_help,
        ))
        .arg(config_arg(
            "concurrency",
            "concurrency",
            "N",
            true,
            dev_help,
        ))
        .arg(config_arg(
            "preload-published-payloads",
            "preload-published-payloads",
            "N",
            true,
            dev_help,
        ))
        .arg(config_arg(
            "compact-min-bytes",
            "compact-min-bytes",
            "N",
            true,
            dev_help,
        ))
        .arg(config_arg(
            "rate-limit-per-minute",
            "rate-limit-per-minute",
            "N",
            true,
            dev_help,
        ))
        .arg(config_arg(
            "rate-limit-burst",
            "rate-limit-burst",
            "N",
            true,
            dev_help,
        ))
        .arg(config_arg(
            "verification-ttl-seconds",
            "verification-ttl-seconds",
            "N",
            true,
            dev_help,
        ))
        .arg(config_arg(
            "default-receive-ttl-seconds",
            "default-receive-ttl-seconds",
            "N",
            true,
            dev_help,
        ))
        .arg(config_arg(
            "max-receive-ttl-seconds",
            "max-receive-ttl-seconds",
            "N",
            true,
            dev_help,
        ))
        .arg(config_arg("smtp-host", "smtp-host", "HOST", true, dev_help))
        .arg(config_arg("smtp-port", "smtp-port", "N", true, dev_help))
        .arg(config_arg(
            "smtp-username",
            "smtp-username",
            "USER",
            true,
            dev_help,
        ))
        .arg(config_arg(
            "smtp-password",
            "smtp-password",
            "PASS",
            true,
            dev_help,
        ))
        .arg(config_arg(
            "smtp-from",
            "smtp-from",
            "EMAIL",
            true,
            dev_help,
        ))
        .arg(config_arg("smtp-tls", "smtp-tls", "MODE", true, dev_help))
        .arg(config_arg(
            "smtp-timeout-seconds",
            "smtp-timeout-seconds",
            "N",
            true,
            dev_help,
        ))
        .arg(config_arg(
            "verification-email-subject",
            "verification-email-subject",
            "TEXT",
            true,
            dev_help,
        ))
        .arg(config_arg(
            "verification-email-template",
            "verification-email-template",
            "TEXT",
            true,
            dev_help,
        ))
        .arg(config_arg(
            "verification-email-rate-limit-per-hour",
            "verification-email-rate-limit-per-hour",
            "N",
            true,
            dev_help,
        ))
        .arg(config_arg(
            "verification-email-ip-rate-limit-per-hour",
            "verification-email-ip-rate-limit-per-hour",
            "N",
            true,
            dev_help,
        ))
}

fn config_arg(
    id: &'static str,
    long: &'static str,
    value_name: &'static str,
    dev_only: bool,
    dev_help: bool,
) -> Arg {
    let mut arg = Arg::new(id).long(long).value_name(value_name).num_args(1);
    if dev_only && !dev_help {
        arg = arg.hide(true);
    }
    arg
}

fn repeated_config_arg(
    id: &'static str,
    long: &'static str,
    value_name: &'static str,
    dev_only: bool,
    dev_help: bool,
) -> Arg {
    config_arg(id, long, value_name, dev_only, dev_help).action(ArgAction::Append)
}

fn validate_config_args_with_clap(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut argv = vec!["config".to_string()];
    argv.extend(args.iter().cloned());
    let matches =
        config_command(args.iter().any(|arg| arg == "--dev")).try_get_matches_from(argv)?;
    validate_config_matches(&matches)
}

fn validate_config_matches(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    if matches.get_flag("dev") {
        return Ok(());
    }
    for id in DEV_OPTION_IDS {
        if matches.contains_id(id) {
            return Err(format!(
                "option `--{id}` requires --dev; put server configuration in --config PATH"
            )
            .into());
        }
    }
    Ok(())
}

const DEV_OPTION_IDS: &[&str] = &[
    "state-dir",
    "server-id",
    "cluster-id",
    "public-url",
    "topology-version",
    "topology-token",
    "topology-server",
    "topology-stale-after-ms",
    "topology-heartbeat-interval-ms",
    "route",
    "replication-token",
    "replication-peer-url",
    "origin-epoch",
    "promoted-owner",
    "requests",
    "payload-bytes",
    "concurrency",
    "preload-published-payloads",
    "compact-min-bytes",
    "rate-limit-per-minute",
    "rate-limit-burst",
    "verification-ttl-seconds",
    "default-receive-ttl-seconds",
    "max-receive-ttl-seconds",
    "smtp-host",
    "smtp-port",
    "smtp-username",
    "smtp-password",
    "smtp-from",
    "smtp-tls",
    "smtp-timeout-seconds",
    "verification-email-subject",
    "verification-email-template",
    "verification-email-rate-limit-per-hour",
    "verification-email-ip-rate-limit-per-hour",
];

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
    println!("  revault_key_server run [--config PATH] [--bind ADDR] [--dev]");
    println!("  revault_key_server install [--force-config]");
    println!("  revault_key_server uninstall [--purge-data]");
    println!("  revault_key_server start");
    println!("  revault_key_server stop");
    println!("  revault_key_server doctor");
    println!("  revault_key_server resync-peer [--config PATH] --peer-url URL");
    if dev {
        println!("  revault_key_server bench-store [--dev options]");
        println!("  revault_key_server bench-http [--dev options]");
        println!("  revault_key_server bench-http-receive [--dev options]");
        println!("  revault_key_server bench-http-flow [--dev options]");
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
    validate_config_args_with_clap(&args)?;
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

#[derive(Default, serde::Deserialize)]
struct TomlConfigFile {
    #[serde(default)]
    topology_server: Vec<TomlTopologyServerTable>,
    #[serde(default)]
    route: Vec<TomlTopologyRouteTable>,
    #[serde(flatten)]
    values: BTreeMap<String, toml::Value>,
}

#[derive(serde::Deserialize)]
struct TomlTopologyServerTable {
    id: Option<toml::Value>,
    url: Option<String>,
    status: Option<String>,
}

#[derive(serde::Deserialize)]
struct TomlTopologyRouteTable {
    owner: Option<toml::Value>,
    primary: Option<toml::Value>,
    #[serde(default)]
    failover: Vec<toml::Value>,
}

fn apply_config_file(
    config: &mut ServerConfig,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let text = fs::read_to_string(path)?;
    let parsed: TomlConfigFile = toml::from_str(&text).map_err(|err| format!("{path}: {err}"))?;
    for (key, value) in parsed.values {
        apply_toml_config_value(config, &key, value).map_err(|err| format!("{path}: {err}"))?;
    }
    for server in parsed.topology_server {
        let id = server.id.ok_or("topology_server.id is required")?;
        let url = server.url.ok_or("topology_server.url is required")?;
        config.topology_servers.push(TopologyServer {
            id: parse_server_id_from_toml(&id)
                .map_err(|err| format!("{path}: topology_server.id: {err}"))?,
            url,
            status: match server.status {
                Some(status) => parse_server_status(&status)
                    .map_err(|err| format!("{path}: topology_server.status: {err}"))?,
                None => ServerStatus::Active,
            },
            last_seen_ms: None,
        });
    }
    for route in parsed.route {
        let owner = route.owner.ok_or("route.owner is required")?;
        let primary = route.primary.ok_or("route.primary is required")?;
        let mut failover_ids = Vec::with_capacity(route.failover.len());
        for value in &route.failover {
            failover_ids.push(
                parse_server_id_from_toml(value)
                    .map_err(|err| format!("{path}: route.failover: {err}"))?,
            );
        }
        config.topology_routes.push(TopologyRoute {
            owner_id: parse_server_id_from_toml(&owner)
                .map_err(|err| format!("{path}: route.owner: {err}"))?,
            primary_id: parse_server_id_from_toml(&primary)
                .map_err(|err| format!("{path}: route.primary: {err}"))?,
            failover_ids,
        });
    }
    Ok(())
}

fn apply_toml_config_value(
    config: &mut ServerConfig,
    key: &str,
    value: toml::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    match key {
        "replication_peer_url" => {
            for value in toml_string_values(value)? {
                config.replication_peer_urls.push(value);
            }
            Ok(())
        }
        "promoted_owner" => {
            for value in toml_values(value) {
                config
                    .promoted_owner_ids
                    .push(parse_server_id_from_toml(&value)?);
            }
            Ok(())
        }
        other => apply_config_value(config, other, toml_scalar_to_string(&value)?),
    }
}

fn toml_values(value: toml::Value) -> Vec<toml::Value> {
    match value {
        toml::Value::Array(values) => values,
        value => vec![value],
    }
}

fn toml_string_values(value: toml::Value) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    toml_values(value)
        .iter()
        .map(toml_scalar_to_string)
        .collect()
}

fn toml_scalar_to_string(value: &toml::Value) -> Result<String, Box<dyn std::error::Error>> {
    Ok(match value {
        toml::Value::String(value) => value.clone(),
        toml::Value::Integer(value) => value.to_string(),
        toml::Value::Float(value) => value.to_string(),
        toml::Value::Boolean(value) => value.to_string(),
        toml::Value::Datetime(value) => value.to_string(),
        toml::Value::Array(_) | toml::Value::Table(_) => {
            return Err("expected scalar config value".into());
        }
    })
}

fn parse_server_id_from_toml(value: &toml::Value) -> Result<u8, Box<dyn std::error::Error>> {
    parse_server_id(&toml_scalar_to_string(value)?)
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
    use revault_key_server::store::{ServerConfig, SmtpTlsMode};
    use revault_publish_protocol::ServerStatus;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    static TEST_FILE_COUNTER: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn config_cli_rejects_hidden_server_options_without_dev() {
        let err = config_from_args(vec![
            "--state-dir".to_string(),
            "/tmp/revault-key-server-test".to_string(),
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
            "/tmp/revault-key-server-test".to_string(),
            "--server-id".to_string(),
            "a".to_string(),
        ])
        .unwrap();
        assert!(config.developer_mode);
        assert_eq!(
            config.state_dir,
            PathBuf::from("/tmp/revault-key-server-test")
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
            "revault-key-server-{name}-{}-{counter}.toml",
            std::process::id()
        ))
    }
}
