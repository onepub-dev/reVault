use clap::ArgMatches;
use revault_lockbox_api::Error;

use super::context::CliResult;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OutputFormat {
    Table,
    Tsv,
    Json,
}

impl OutputFormat {
    pub(crate) fn parse(value: Option<&str>) -> CliResult<Self> {
        match value.unwrap_or("table") {
            "table" => Ok(Self::Table),
            "tsv" => Ok(Self::Tsv),
            "json" => Ok(Self::Json),
            value => Err(Error::InvalidInput(format!(
                "unsupported output format: {value}; expected table, tsv, or json"
            ))
            .into()),
        }
    }
}

pub(crate) fn output_format_from_matches(matches: &ArgMatches) -> CliResult<OutputFormat> {
    OutputFormat::parse(matches.get_one::<String>("format").map(String::as_str))
}

pub(crate) fn print_records(
    headers: &[&str],
    rows: Vec<Vec<String>>,
    format: OutputFormat,
) -> CliResult<()> {
    if rows.is_empty() {
        println!("empty");
        return Ok(());
    }
    match format {
        OutputFormat::Table => print_table(headers, &rows),
        OutputFormat::Tsv => print_tsv(&rows),
        OutputFormat::Json => print_json(headers, &rows)?,
    }
    Ok(())
}

fn print_table(headers: &[&str], rows: &[Vec<String>]) {
    let mut widths = headers
        .iter()
        .map(|header| header.len())
        .collect::<Vec<_>>();
    for row in rows {
        for (index, value) in row.iter().enumerate() {
            if let Some(width) = widths.get_mut(index) {
                *width = (*width).max(value.len());
            }
        }
    }
    print_table_row(
        &headers
            .iter()
            .map(|value| (*value).to_string())
            .collect::<Vec<_>>(),
        &widths,
    );
    for row in rows {
        print_table_row(row, &widths);
    }
}

fn print_table_row(row: &[String], widths: &[usize]) {
    for (index, value) in row.iter().enumerate() {
        if index > 0 {
            print!("  ");
        }
        let width = widths.get(index).copied().unwrap_or(value.len());
        print!("{value:<width$}");
    }
    println!();
}

fn print_tsv(rows: &[Vec<String>]) {
    for row in rows {
        println!("{}", row.join("\t"));
    }
}

fn print_json(headers: &[&str], rows: &[Vec<String>]) -> CliResult<()> {
    for row in rows {
        if row.len() != headers.len() {
            return Err(Error::InvalidInput("output row/header width mismatch".to_string()).into());
        }
        let fields = headers
            .iter()
            .zip(row.iter())
            .map(|(name, value)| format!("{}:{}", json_string(name), json_string(value)))
            .collect::<Vec<_>>();
        println!("{{{}}}", fields.join(","));
    }
    Ok(())
}

pub(crate) fn json_string(value: &str) -> String {
    serde_json::to_string(value).expect("serializing a string to JSON cannot fail")
}

/// Formats a byte count for terminal output using decimal SI units.
pub(crate) fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 7] = ["B", "KB", "MB", "GB", "TB", "PB", "EB"];
    if bytes < 1_000 {
        return format!("{bytes}B");
    }
    let mut value = bytes as f64;
    let mut unit = 0usize;
    while value >= 1_000.0 && unit + 1 < UNITS.len() {
        value /= 1_000.0;
        unit += 1;
    }
    format!("{value:.3}{}", UNITS[unit])
}

#[cfg(test)]
mod tests {
    use super::human_size;

    #[test]
    fn human_sizes_use_compact_decimal_units() {
        assert_eq!(human_size(999), "999B");
        assert_eq!(human_size(1_000), "1.000KB");
        assert_eq!(human_size(19_265_189_184), "19.265GB");
    }
}
