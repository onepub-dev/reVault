use std::{
    fs,
    io::{Read, Write},
};

use clap::ArgMatches;
use revault_lockbox_api::{
    Error, SecretString, VariableName, VariableNamePattern, VariableSensitivity, VariableValueRef,
};

use super::context::{open_existing, open_or_create, require_arg, Access, CliResult};
use super::output::{json_string, output_format_from_matches, print_records};
use super::{optional_lockbox_positionals, positional_values};
use crate::secret_prompt::prompt_secret;

pub(crate) fn run_matches(matches: &ArgMatches, access: &Access) -> CliResult<()> {
    let (subcommand, sub) = matches
        .subcommand()
        .ok_or_else(|| Error::InvalidInput("missing variable command".to_string()))?;
    match subcommand {
        "set" => {
            let args = optional_lockbox_positionals(positional_values(sub, "args"), 1)?;
            let request = VariableSetRequest::from_matches(sub, &args[1..])?;
            set_variable_request(&args[0], request, access)?;
        }
        "get" => {
            let args = optional_lockbox_positionals(positional_values(sub, "args"), 1)?;
            let request = VariableGetRequest::from_matches(sub, &args[1..])?;
            get_variable_request(&args[0], request, access)?;
        }
        "list" | "ls" => {
            let args = optional_lockbox_positionals(positional_values(sub, "args"), 0)?;
            let pattern = match args[1..].as_ref() {
                [] => None,
                [pattern] => Some(VariableNamePattern::new(pattern)?),
                _ => {
                    return Err(Error::InvalidInput(
                        "variable list accepts at most one path or glob pattern".to_string(),
                    )
                    .into());
                }
            };
            let lb = open_existing(&args[0], access)?;
            let mut rows = Vec::new();
            for (name, sensitivity) in lb.list_variables()? {
                if pattern
                    .as_ref()
                    .is_some_and(|pattern| !name.matches_pattern(pattern))
                {
                    continue;
                }
                rows.push(vec![
                    name.to_string(),
                    sensitivity_name(sensitivity).to_string(),
                ]);
            }
            print_records(
                &["name", "sensitivity"],
                rows,
                output_format_from_matches(sub)?,
            )?;
        }
        "export" => {
            let args = optional_lockbox_positionals(positional_values(sub, "args"), 0)?;
            let request = VariableExportRequest::from_matches(sub, &args[1..])?;
            let lb = open_existing(&args[0], access)?;
            lb.visit_variables(|name, value| match value {
                VariableValueRef::Normal(value) => {
                    if let Some(name) = request.export_name(name) {
                        println!("{}", request.format.format_assignment(&name, value));
                    }
                    Ok(())
                }
                VariableValueRef::Secret(_) => Ok(()),
            })?;
        }
        "remove" => {
            let args = optional_lockbox_positionals(positional_values(sub, "args"), 1)?;
            let name = VariableName::new(require_arg(&args, 1, "name")?)?;
            let mut lb = open_existing(&args[0], access)?;
            lb.delete_variable(&name)?;
            lb.commit()?;
        }
        "move" | "mv" => {
            let args = optional_lockbox_positionals(positional_values(sub, "args"), 2)?;
            move_variables(
                &args[0],
                require_arg(&args, 1, "source path or glob")?,
                require_arg(&args, 2, "destination path")?,
                access,
            )?;
        }
        _ => {
            return Err(
                Error::InvalidInput(format!("unknown variable command: {subcommand}")).into(),
            )
        }
    }
    Ok(())
}

fn move_variables(
    lockbox_path: &str,
    source_pattern: &str,
    destination: &str,
    access: &Access,
) -> CliResult<()> {
    let pattern = VariableNamePattern::new(source_pattern)?;
    let destination = VariableName::new(destination)?;
    let mut lb = open_existing(lockbox_path, access)?;
    let moves = lb
        .list_variables()?
        .into_iter()
        .filter(|(name, _)| name.matches_pattern(&pattern))
        .map(|(name, _)| {
            let target = moved_path(source_pattern, name.as_str(), destination.as_str())?;
            Ok((name, VariableName::new(target)?))
        })
        .collect::<CliResult<Vec<_>>>()?;
    if moves.is_empty() {
        return Err(Error::NotFound(format!("no variables match {source_pattern}")).into());
    }
    lb.move_variables(&moves)?;
    lb.commit()?;
    for (source, destination) in moves {
        println!("{source}\t{destination}\tmoved");
    }
    Ok(())
}

fn moved_path(pattern: &str, source: &str, destination: &str) -> CliResult<String> {
    let canonical_pattern = pattern.trim_start_matches('/');
    let components = canonical_pattern.split('/').collect::<Vec<_>>();
    let wildcard = components
        .iter()
        .position(|component| component.contains('*') || component.contains('?'));
    let anchor_components = match wildcard {
        Some(index) => &components[..index],
        None => &components[..components.len().saturating_sub(1)],
    };
    let anchor = if anchor_components.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", anchor_components.join("/"))
    };
    let relative = source
        .strip_prefix(&anchor)
        .unwrap_or(source)
        .trim_start_matches('/');
    Ok(format!(
        "{}/{}",
        destination.trim_end_matches('/'),
        relative
    ))
}

fn set_variable_request(
    lockbox_path: &str,
    request: VariableSetRequest,
    access: &Access,
) -> CliResult<()> {
    let mut lb = open_or_create(lockbox_path, access)?;
    let existing = lb.variable_sensitivity(&request.name)?;
    let effective_sensitivity = existing.unwrap_or(if request.secret {
        VariableSensitivity::Secret
    } else {
        VariableSensitivity::Normal
    });

    if let Some(existing) = existing {
        if request.secret && existing == VariableSensitivity::Normal {
            return Err(Error::InvalidOperation(
                "variable is not secret; delete and recreate it".to_string(),
            )
            .into());
        }
        if !request.secret
            && existing == VariableSensitivity::Secret
            && request.positional.is_some()
        {
            return Err(Error::InvalidInput(
                "secret variables require an explicit value source".to_string(),
            )
            .into());
        }
    }

    match effective_sensitivity {
        VariableSensitivity::Normal => {
            let value = request.read_normal_value()?;
            lb.set_variable(&request.name, &value)?;
        }
        VariableSensitivity::Secret => {
            if request.positional.is_some() {
                return Err(Error::InvalidInput(
                    "secret variables cannot use positional values".to_string(),
                )
                .into());
            }
            let value = request.read_secret_value()?;
            lb.set_secret_variable(&request.name, &value)?;
        }
    }
    lb.commit()?;
    println!("Variable set: {}", request.name.as_str());
    Ok(())
}

fn get_variable_request(
    lockbox_path: &str,
    request: VariableGetRequest,
    access: &Access,
) -> CliResult<()> {
    let name = VariableName::new(&request.name)?;
    let lb = open_existing(lockbox_path, access)?;
    if request.secret {
        if let Some(write_result) = lb.with_secret_variable(&name, |value| {
            value.with_bytes(|value| request.write_value_bytes(value))
        })? {
            write_result??;
        } else {
            return Err(Error::NotFound(format!("variable {name}")).into());
        }
    } else if let Some(value) = lb.get_variable(&name)? {
        request.write_value(&value)?;
    } else {
        return Err(Error::NotFound(format!("variable {name}")).into());
    }
    Ok(())
}

struct VariableGetRequest {
    secret: bool,
    name: String,
    output: Option<String>,
    overwrite: bool,
}

impl VariableGetRequest {
    fn from_matches(matches: &ArgMatches, args: &[String]) -> CliResult<Self> {
        let name = match args {
            [name] => name.clone(),
            [] => return Err(Error::InvalidInput("missing variable name".to_string()).into()),
            _ => {
                return Err(Error::InvalidInput(
                    "variable get accepts exactly one variable name".to_string(),
                )
                .into())
            }
        };
        let output = matches.get_one::<String>("output").cloned();
        let overwrite = matches.get_flag("overwrite");
        if overwrite && output.is_none() {
            return Err(Error::InvalidInput("--overwrite requires --output".to_string()).into());
        }
        Ok(Self {
            secret: matches.get_flag("secret"),
            name,
            output,
            overwrite,
        })
    }

    fn write_value(&self, value: &str) -> CliResult<()> {
        self.write_value_bytes(value.as_bytes())
    }

    fn write_value_bytes(&self, bytes: &[u8]) -> CliResult<()> {
        if let Some(path) = &self.output {
            write_output_file(path, bytes, self.overwrite)?;
        } else {
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            stdout.write_all(bytes)?;
            stdout.write_all(b"\n")?;
        }
        Ok(())
    }
}

fn write_output_file(path: &str, bytes: &[u8], overwrite: bool) -> CliResult<()> {
    let mut options = fs::OpenOptions::new();
    options.write(true);
    if overwrite {
        options.create(true).truncate(true);
    } else {
        options.create_new(true);
    }
    configure_private_output_file(&mut options);
    let mut file = options.open(path).map_err(|err| {
        if err.kind() == std::io::ErrorKind::AlreadyExists {
            Error::AlreadyExists(path.to_string())
        } else {
            Error::Io(format!("create {path}: {err}"))
        }
    })?;
    set_private_output_permissions(&file)?;
    file.write_all(bytes)?;
    Ok(())
}

#[cfg(unix)]
fn configure_private_output_file(options: &mut fs::OpenOptions) {
    use std::os::unix::fs::OpenOptionsExt;

    options.mode(0o600);
}

#[cfg(not(unix))]
fn configure_private_output_file(_options: &mut fs::OpenOptions) {}

#[cfg(unix)]
fn set_private_output_permissions(file: &fs::File) -> CliResult<()> {
    use std::os::unix::fs::PermissionsExt;

    file.set_permissions(fs::Permissions::from_mode(0o600))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_private_output_permissions(_file: &fs::File) -> CliResult<()> {
    Ok(())
}

struct VariableSetRequest {
    name: VariableName,
    secret: bool,
    positional: Option<String>,
    source: Option<ValueSource>,
}

impl VariableSetRequest {
    fn from_matches(matches: &ArgMatches, args: &[String]) -> CliResult<Self> {
        let (name, positional) = match args {
            [assignment] => match assignment.split_once('=') {
                Some((name, value)) => {
                    if name.is_empty() {
                        return Err(Error::InvalidInput(
                            "missing variable name before '='".to_string(),
                        )
                        .into());
                    }
                    (VariableName::new(name)?, Some(value.to_string()))
                }
                None => (VariableName::new(assignment)?, None),
            },
            [name, value] => (VariableName::new(name)?, Some(value.clone())),
            [] => return Err(Error::InvalidInput("missing variable name".to_string()).into()),
            _ => {
                return Err(Error::InvalidInput(
                    "variable set accepts at most one positional value".to_string(),
                )
                .into())
            }
        };
        let mut source = None;
        if matches.get_flag("interactive") {
            set_source(&mut source, ValueSource::Interactive)?;
        }
        if matches.get_flag("stdin") {
            set_source(&mut source, ValueSource::Stdin)?;
        }
        if let Some(value) = matches.get_one::<String>("value") {
            set_source(&mut source, ValueSource::Value(value.clone()))?;
        }
        if let Some(value) = matches.get_one::<String>("file") {
            set_source(&mut source, ValueSource::File(value.clone()))?;
        }
        if let Some(value) = matches.get_one::<String>("from-env") {
            set_source(&mut source, ValueSource::FromEnv(value.clone()))?;
        }
        if source.is_some() == positional.is_some() {
            return Err(Error::InvalidInput(
                "variable set requires exactly one value source".to_string(),
            )
            .into());
        }
        Ok(Self {
            name,
            secret: matches.get_flag("secret"),
            positional,
            source,
        })
    }

    fn read_normal_value(&self) -> CliResult<String> {
        if let Some(value) = &self.positional {
            return Ok(value.clone());
        }
        match self
            .source
            .as_ref()
            .ok_or_else(|| Error::InvalidInput("missing value source".to_string()))?
        {
            ValueSource::Interactive => prompt_secret("Value: ")?
                .with_str(str::to_string)
                .map_err(Box::<dyn std::error::Error>::from),
            ValueSource::Value(value) => Ok(value.clone()),
            ValueSource::File(path) => Ok(String::from_utf8(fs::read(path)?)?),
            ValueSource::Stdin => {
                let mut bytes = Vec::new();
                std::io::stdin().lock().read_to_end(&mut bytes)?;
                Ok(String::from_utf8(bytes)?)
            }
            ValueSource::FromEnv(name) => Ok(std::env::var(name)?),
        }
    }

    fn read_secret_value(&self) -> CliResult<SecretString> {
        match self
            .source
            .as_ref()
            .ok_or_else(|| Error::InvalidInput("missing value source".to_string()))?
        {
            ValueSource::Interactive => Ok(prompt_secret("Secret value: ")?),
            ValueSource::Value(value) => {
                let _ = value;
                Err(Error::InvalidInput(
                    "--value is not accepted for secret variable values; use --stdin, --file, --interactive, or --from-env"
                        .to_string(),
                )
                .into())
            }
            ValueSource::File(path) => read_secret_file(path),
            ValueSource::Stdin => read_secret_stdin(),
            ValueSource::FromEnv(name) => SecretString::try_from_env(name)?
                .ok_or_else(|| Error::InvalidInput(format!("{name} is not set")).into()),
        }
    }
}

fn read_secret_file(path: &str) -> CliResult<SecretString> {
    let mut file = fs::File::open(path)?;
    read_secret_from(&mut file)
}

fn read_secret_stdin() -> CliResult<SecretString> {
    let mut stdin = std::io::stdin().lock();
    read_secret_from(&mut stdin)
}

fn read_secret_from(input: &mut impl Read) -> CliResult<SecretString> {
    let mut secret = SecretString::new();
    let mut buffer = [0u8; 8192];
    loop {
        let read = input.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        secret.try_extend_from_slice(&buffer[..read])?;
        buffer[..read].fill(0);
        std::hint::black_box(&mut buffer[..read]);
    }
    Ok(secret)
}

enum ValueSource {
    Interactive,
    Value(String),
    File(String),
    Stdin,
    FromEnv(String),
}

enum VariableExportFormat {
    Posix,
    PowerShell,
    Cmd,
    Json,
}

struct VariableExportRequest {
    pattern: Option<VariableNamePattern>,
    format: VariableExportFormat,
}

impl VariableExportRequest {
    fn from_matches(matches: &ArgMatches, args: &[String]) -> CliResult<Self> {
        let pattern = match args {
            [] => None,
            [pattern] => Some(VariableNamePattern::new(pattern)?),
            _ => {
                return Err(Error::InvalidInput(
                    "variable export accepts at most one path or glob pattern".to_string(),
                )
                .into())
            }
        };
        let format = match matches.get_one::<String>("format").map(String::as_str) {
            Some(value) => VariableExportFormat::parse_value(Some(value))?,
            None => VariableExportFormat::Posix,
        };
        Ok(Self { pattern, format })
    }

    fn export_name(&self, name: &VariableName) -> Option<String> {
        if self
            .pattern
            .as_ref()
            .is_some_and(|pattern| !name.matches_pattern(pattern))
        {
            return None;
        }
        export_all_name(name.as_str())
    }
}

fn export_all_name(name: &str) -> Option<String> {
    let name = name.strip_prefix('/').unwrap_or(name);
    if name.is_empty() {
        return None;
    }
    Some(name.replace('/', "_"))
}

impl VariableExportFormat {
    fn parse_value(value: Option<&str>) -> CliResult<Self> {
        match value {
            Some("posix") => Ok(Self::Posix),
            Some("powershell") => Ok(Self::PowerShell),
            Some("cmd") => Ok(Self::Cmd),
            Some("json") => Ok(Self::Json),
            Some(value) => Err(Error::InvalidInput(format!(
                "unsupported variable export format: {value}"
            ))
            .into()),
            None => Err(Error::InvalidInput("missing --format argument".to_string()).into()),
        }
    }

    fn format_assignment(&self, name: &str, value: &str) -> String {
        match self {
            Self::Posix => format!("{name}={}", posix_quote(value)),
            Self::PowerShell => format!("$env:{name} = {}", powershell_quote(value)),
            Self::Cmd => format!("set \"{name}={}\"", cmd_quote_value(value)),
            Self::Json => format!(
                "{{\"name\":{},\"value\":{}}}",
                json_string(name),
                json_string(value)
            ),
        }
    }
}

fn set_source(target: &mut Option<ValueSource>, source: ValueSource) -> CliResult<()> {
    if target.is_some() {
        return Err(Error::InvalidInput(
            "variable set accepts exactly one value source".to_string(),
        )
        .into());
    }
    *target = Some(source);
    Ok(())
}

fn sensitivity_name(sensitivity: VariableSensitivity) -> &'static str {
    match sensitivity {
        VariableSensitivity::Normal => "normal",
        VariableSensitivity::Secret => "secret",
    }
}

fn posix_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn powershell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn cmd_quote_value(value: &str) -> String {
    value.replace('"', "\"\"")
}

#[cfg(test)]
mod move_tests {
    use super::moved_path;

    #[test]
    fn root_glob_preserves_variable_names() {
        assert_eq!(moved_path("/*", "/TMP", "/dev").unwrap(), "/dev/TMP");
        assert_eq!(moved_path("/*", "/MODE", "/dev").unwrap(), "/dev/MODE");
    }

    #[test]
    fn nested_glob_preserves_paths_below_its_fixed_prefix() {
        assert_eq!(
            moved_path("/production/**", "/production/api/TOKEN", "/archive").unwrap(),
            "/archive/api/TOKEN"
        );
    }
}
