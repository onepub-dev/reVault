use clap::{Arg, ArgMatches};
use revault_lockbox_api::ExtractPolicy;

pub(super) fn args() -> [Arg; 3] {
    [
        Arg::new("max-file-bytes").long("max-file-bytes").value_name("SIZE")
            .value_parser(parse_bytes)
            .help("Maximum bytes per file (default 1G). Integer bytes or K/M/G, KiB/MiB/GiB; M = 1024^2 bytes. Zero permits only empty files."),
        Arg::new("max-total-bytes").long("max-total-bytes").value_name("SIZE")
            .value_parser(parse_bytes)
            .help("Maximum total file bytes (default 10G). Same units as --max-file-bytes; zero permits no file bytes."),
        Arg::new("max-entries").long("max-entries").value_name("COUNT")
            .value_parser(parse_entries)
            .help("Maximum selected files, directories and symlinks (default 100000). Non-negative integer; zero allows no entries."),
    ]
}

pub(super) fn policy(matches: &ArgMatches) -> ExtractPolicy {
    let mut policy = ExtractPolicy {
        overwrite: matches.get_flag("overwrite"),
        restore_symlinks: matches.get_flag("restore-symlinks"),
        restore_permissions: !matches.get_flag("no-restore-permissions"),
        ..ExtractPolicy::default()
    };
    if let Some(value) = matches.get_one::<u64>("max-file-bytes") {
        policy.max_file_bytes = *value;
    }
    if let Some(value) = matches.get_one::<u64>("max-total-bytes") {
        policy.max_total_bytes = *value;
    }
    if let Some(value) = matches.get_one::<usize>("max-entries") {
        policy.max_files = *value;
    }
    policy
}

fn parse_bytes(input: &str) -> Result<u64, String> {
    let end = input
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(input.len());
    let (number, suffix) = input.split_at(end);
    let number = number
        .parse::<u64>()
        .map_err(|_| "expected a non-negative integer size".to_owned())?;
    let multiplier = match suffix.to_ascii_lowercase().as_str() {
        "" | "b" => 1,
        "k" | "kib" => 1024,
        "m" | "mib" => 1024 * 1024,
        "g" | "gib" => 1024 * 1024 * 1024,
        _ => return Err("use integer bytes or K/M/G, KiB/MiB/GiB (binary units)".to_owned()),
    };
    number
        .checked_mul(multiplier)
        .ok_or_else(|| "size exceeds the maximum supported byte count".to_owned())
}

fn parse_entries(input: &str) -> Result<usize, String> {
    if input.is_empty() || !input.bytes().all(|b| b.is_ascii_digit()) {
        return Err("expected a non-negative integer entry count".to_owned());
    }
    input
        .parse()
        .map_err(|_| "entry count exceeds the platform limit".to_owned())
}
