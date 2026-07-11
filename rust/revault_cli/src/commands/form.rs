use std::{
    fs,
    io::{Read, Write},
};

use clap::ArgMatches;
use revault_lockbox_api::{
    Error, FormDefinition, FormFieldDefinition, FormFieldKind, FormValue, LockboxPath, SecretString,
};

use super::context::{
    cli_error, default_vault, open_existing, open_or_create, require_arg, Access, CliResult,
};
use super::output::{output_format_from_matches, print_records, OutputFormat};
use super::{default_lockbox_for_command, optional_lockbox_positionals, positional_values};
use crate::secret_prompt::prompt_secret;

pub(crate) fn run_matches(matches: &ArgMatches, access: &Access) -> CliResult<()> {
    let (command, sub) = matches
        .subcommand()
        .ok_or_else(|| Error::InvalidInput("missing form command".to_string()))?;
    match command {
        "define" => define_matches(sub, access),
        "definitions" => definitions_with_format(
            &optional_lockbox_positionals(positional_values(sub, "args"), 0)?,
            access,
            output_format_from_matches(sub)?,
        ),
        "use" => use_vault_definition(
            &[
                required_value(sub, "form"),
                sub.get_one::<String>("lockbox")
                    .cloned()
                    .map(Ok)
                    .unwrap_or_else(default_lockbox_for_command)?,
            ],
            access,
        ),
        "capture" => capture_definition(
            &optional_lockbox_positionals(positional_values(sub, "args"), 1)?,
            access,
        ),
        "add" => add_matches(sub, access),
        "edit" => edit_matches(sub, access),
        "set" => set_matches(sub, access),
        "get" => get_matches(sub, access),
        "show" => inspect(
            &optional_lockbox_positionals(positional_values(sub, "args"), 1)?,
            access,
        ),
        "list" => list_with_format(
            &optional_lockbox_positionals(positional_values(sub, "args"), 0)?,
            access,
            output_format_from_matches(sub)?,
        ),
        "remove" => remove(
            &optional_lockbox_positionals(positional_values(sub, "args"), 1)?,
            access,
        ),
        _ => Err(Error::InvalidInput(format!("unknown form command: {command}")).into()),
    }
}

fn required_value(matches: &ArgMatches, name: &str) -> String {
    matches
        .get_one::<String>(name)
        .unwrap_or_else(|| panic!("clap did not provide required argument {name}"))
        .clone()
}

fn optional_value<'a>(matches: &'a ArgMatches, name: &str) -> Option<&'a str> {
    matches.get_one::<String>(name).map(String::as_str)
}

fn string_values(matches: &ArgMatches, name: &str) -> Vec<String> {
    matches
        .get_many::<String>(name)
        .map(|values| values.cloned().collect())
        .unwrap_or_default()
}

fn define_matches(matches: &ArgMatches, access: &Access) -> CliResult<()> {
    let args = optional_lockbox_positionals(positional_values(matches, "args"), 0)?;
    let lockbox_path = require_arg(&args, 0, "lockbox")?;
    let alias = args.get(1).cloned();
    let name = optional_value(matches, "name")
        .map(str::to_string)
        .or_else(|| alias.clone())
        .ok_or_else(|| {
            Error::InvalidInput("form define requires an alias or --name".to_string())
        })?;
    let alias = alias.unwrap_or_else(|| default_form_alias(&name));
    let description = optional_value(matches, "description").unwrap_or_default();
    let type_id = optional_value(matches, "definition-id")
        .map(revault_lockbox_api::FormTypeId::new)
        .transpose()?;
    let fields = string_values(matches, "field")
        .iter()
        .map(|field| parse_field_spec(field))
        .collect::<CliResult<Vec<_>>>()?;
    define_options(
        lockbox_path,
        alias,
        name,
        description.to_string(),
        type_id,
        fields,
        access,
    )
}

fn define_options(
    lockbox_path: &str,
    alias: String,
    name: String,
    description: String,
    type_id: Option<revault_lockbox_api::FormTypeId>,
    fields: Vec<FormFieldDefinition>,
    access: &Access,
) -> CliResult<()> {
    let mut lb = open_or_create(lockbox_path, access)?;
    let definition = if let Some(type_id) = type_id {
        lb.define_form_with_type_id_and_description(type_id, &alias, &name, &description, fields)?
    } else {
        lb.define_form_with_description(&alias, &name, &description, fields)?
    };
    lb.commit()?;
    print_form_definition_saved(&definition);
    Ok(())
}

pub(crate) fn print_form_definition_saved(definition: &FormDefinition) {
    println!("Form definition saved.");
    println!("  alias: {}", definition.alias);
    println!("  definition_id: {}", definition.type_id);
    println!("  revision: {}", definition.revision);
    println!("  name: {}", definition.name);
    if !definition.description.is_empty() {
        println!("  description: {}", definition.description);
    }
    println!("  fields: {}", definition.fields.len());
}

pub(crate) fn default_form_alias(name: &str) -> String {
    let mut alias = String::new();
    let mut previous_separator = false;
    for ch in name.trim().chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            alias.push(ch.to_ascii_lowercase());
            previous_separator = false;
        } else if (ch == '-' || ch.is_whitespace()) && !alias.is_empty() && !previous_separator {
            alias.push('-');
            previous_separator = true;
        }
        if alias.len() >= 128 {
            break;
        }
    }
    while alias.ends_with('-') {
        alias.pop();
    }
    if alias.is_empty() {
        alias.push_str("form");
    }
    if !alias
        .chars()
        .next()
        .is_some_and(|ch| ch == '_' || ch.is_ascii_alphabetic())
    {
        alias.insert_str(0, "form-");
        alias.truncate(128);
        while alias.ends_with('-') {
            alias.pop();
        }
    }
    alias
}

fn use_vault_definition(args: &[String], access: &Access) -> CliResult<()> {
    let form_name = require_arg(args, 0, "form name")?;
    let lockbox_path = require_arg(args, 1, "lockbox")?;
    let definition = default_vault()?.resolve_form_definition(form_name)?;
    let mut lb = open_or_create(lockbox_path, access)?;
    let definition = lb.import_form_definition(definition)?;
    lb.commit()?;
    print_form_definition_saved(&definition);
    Ok(())
}

fn capture_definition(args: &[String], access: &Access) -> CliResult<()> {
    let lockbox_path = require_arg(args, 0, "lockbox")?;
    let form_name = require_arg(args, 1, "form name")?;
    let new_name = args.get(2).map(String::as_str);
    let lb = open_existing(lockbox_path, access)?;
    let mut definition = lb.resolve_form_definition(form_name)?;
    let vault = default_vault()?;
    if let Some(new_name) = new_name {
        definition.alias = new_name.to_string();
        definition.name = new_name.to_string();
    } else if let Ok(existing) = vault.resolve_form_definition(&definition.alias) {
        if existing.type_id != definition.type_id {
            return Err(Error::AlreadyExists(format!(
                "vault form definition alias {}; pass a new form name",
                definition.alias
            ))
            .into());
        }
    }
    let definition = vault.import_form_definition(definition)?;
    print_form_definition_saved(&definition);
    Ok(())
}

fn definitions_with_format(
    args: &[String],
    access: &Access,
    format: OutputFormat,
) -> CliResult<()> {
    let lockbox_path = require_arg(args, 0, "lockbox")?;
    let lb = open_existing(lockbox_path, access)?;
    let rows = lb
        .list_form_definitions()?
        .into_iter()
        .map(|definition| {
            vec![
                definition.alias,
                definition.type_id.to_string(),
                definition.revision.to_string(),
                definition.name,
                definition.description,
                definition.fields.len().to_string(),
            ]
        })
        .collect::<Vec<_>>();
    print_records(
        &[
            "alias",
            "definition_id",
            "revision",
            "name",
            "description",
            "fields",
        ],
        rows,
        format,
    )?;
    Ok(())
}

fn add_matches(matches: &ArgMatches, access: &Access) -> CliResult<()> {
    let args = optional_lockbox_positionals(positional_values(matches, "args"), 1)?;
    let lockbox_path = require_arg(&args, 0, "lockbox")?;
    let path = form_record_path(require_arg(&args, 1, "form path")?)?;
    let form_type = required_value(matches, "type");
    let name = optional_value(matches, "name")
        .map(str::to_string)
        .unwrap_or_else(|| default_form_name(path.as_str()));
    let assignments = string_values(matches, "set")
        .iter()
        .map(|assignment| parse_field_assignment(assignment))
        .collect::<CliResult<Vec<_>>>()?;
    let interactive = matches.get_flag("interactive");
    let mut lb = open_or_create(lockbox_path, access)?;
    lb.create_parent_dirs_for(&path)?;
    let record = lb.create_form_record(&path, &form_type, &name)?;
    let definition = lb.resolve_form_definition(record.type_id.as_str())?;
    apply_normal_field_assignments(&mut lb, &path, &definition, assignments)?;
    if interactive {
        fill_missing_fields_interactively(&mut lb, &path, &definition)?;
    }
    lb.commit()?;
    println!("{}\t{}\t{}", record.path, record.name, record.type_id);
    Ok(())
}

fn edit_matches(matches: &ArgMatches, access: &Access) -> CliResult<()> {
    let args = optional_lockbox_positionals(positional_values(matches, "args"), 1)?;
    let lockbox_path = require_arg(&args, 0, "lockbox")?;
    let path = form_record_path(require_arg(&args, 1, "form path")?)?;
    let assignments = string_values(matches, "set")
        .iter()
        .map(|assignment| parse_field_assignment(assignment))
        .collect::<CliResult<Vec<_>>>()?;
    let interactive = matches.get_flag("interactive");
    if assignments.is_empty() && !interactive {
        return Err(
            Error::InvalidInput("form edit requires --set or --interactive".to_string()).into(),
        );
    }
    let mut lb = open_existing(lockbox_path, access)?;
    let record = lb
        .get_form_record(&path)?
        .ok_or_else(|| Error::NotFound(format!("form record {path}")))?;
    let definition = lb.resolve_form_definition(record.type_id.as_str())?;
    apply_normal_field_assignments(&mut lb, &path, &definition, assignments)?;
    if interactive {
        edit_fields_interactively(&mut lb, &path, &definition)?;
    }
    lb.commit()?;
    Ok(())
}

fn apply_normal_field_assignments(
    lb: &mut revault_lockbox_api::Lockbox,
    path: &LockboxPath,
    definition: &FormDefinition,
    assignments: Vec<(String, String)>,
) -> CliResult<()> {
    for (field_id, value) in assignments {
        let field = definition
            .fields
            .iter()
            .find(|field| field.id == field_id)
            .ok_or_else(|| Error::InvalidInput(format!("unknown form field: {field_id}")))?;
        if field.kind.is_secret() {
            return Err(Error::InvalidInput(format!(
                "field {field_id} is secret; use --interactive or form set --secret --stdin"
            ))
            .into());
        }
        lb.set_form_field_normal(path, &field_id, &value)?;
    }
    Ok(())
}

fn set_matches(matches: &ArgMatches, access: &Access) -> CliResult<()> {
    let args = optional_lockbox_positionals(positional_values(matches, "args"), 2)?;
    let lockbox_path = require_arg(&args, 0, "lockbox")?;
    let path = form_record_path(require_arg(&args, 1, "form path")?)?;
    let field_id = require_arg(&args, 2, "field id")?;
    let source = match (
        matches.get_flag("interactive"),
        matches.get_flag("stdin"),
        optional_value(matches, "explicit-value"),
        optional_value(matches, "file"),
        optional_value(matches, "from-env"),
        args.get(3),
    ) {
        (true, false, None, None, None, None) => FieldValueSource::Interactive,
        (false, true, None, None, None, None) => FieldValueSource::Stdin,
        (false, false, Some(value), None, None, None) => FieldValueSource::Value(value.to_string()),
        (false, false, None, Some(value), None, None) => FieldValueSource::File(value.to_string()),
        (false, false, None, None, Some(value), None) => {
            FieldValueSource::FromEnv(value.to_string())
        }
        (false, false, None, None, None, Some(value)) => FieldValueSource::Literal(value.clone()),
        _ => {
            return Err(Error::InvalidInput(
                "form set accepts exactly one value source".to_string(),
            )
            .into())
        }
    };
    let mut lb = open_existing(lockbox_path, access)?;
    if matches.get_flag("secret") {
        let value = read_secret_value(source)?;
        lb.set_form_field_secret(&path, field_id, &value)?;
    } else {
        let value = read_normal_value(source)?;
        lb.set_form_field_normal(&path, field_id, &value)?;
    }
    lb.commit()?;
    println!("{}\t{}\tupdated", path, field_id);
    Ok(())
}

fn remove(args: &[String], access: &Access) -> CliResult<()> {
    let lockbox_path = require_arg(args, 0, "lockbox")?;
    let path = form_record_path(require_arg(args, 1, "form path")?)?;
    let mut lb = open_existing(lockbox_path, access)?;
    lb.delete_form_record(&path)?;
    lb.commit()?;
    Ok(())
}

fn get_matches(matches: &ArgMatches, access: &Access) -> CliResult<()> {
    let args = optional_lockbox_positionals(positional_values(matches, "args"), 2)?;
    let request = FormGetRequest {
        lockbox_path: require_arg(&args, 0, "lockbox")?.to_string(),
        path: require_arg(&args, 1, "form path")?.to_string(),
        field_id: require_arg(&args, 2, "field id")?.to_string(),
        reveal_secret: matches.get_flag("secret"),
        output: optional_value(matches, "output").map(str::to_string),
        overwrite: matches.get_flag("overwrite"),
    };
    let path = form_record_path(&request.path)?;
    let lb = open_existing(&request.lockbox_path, access)?;
    let value = lb
        .get_form_field(&path, &request.field_id)?
        .ok_or_else(|| Error::NotFound(format!("form field {}", request.field_id)))?;
    match value.value {
        FormValue::Normal(value) => request.write_value(value.as_bytes())?,
        FormValue::Secret(value) if request.reveal_secret => {
            value.with_str(|value| request.write_value(value.as_bytes()))??;
        }
        FormValue::Secret(_) => {
            return Err(cli_error("field is secret; pass --secret to print it"));
        }
    }
    Ok(())
}

struct FormGetRequest {
    lockbox_path: String,
    path: String,
    field_id: String,
    reveal_secret: bool,
    output: Option<String>,
    overwrite: bool,
}

impl FormGetRequest {
    fn write_value(&self, bytes: &[u8]) -> CliResult<()> {
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

fn inspect(args: &[String], access: &Access) -> CliResult<()> {
    let lockbox_path = require_arg(args, 0, "lockbox")?;
    let path = form_record_path(require_arg(args, 1, "form path")?)?;
    let lb = open_existing(lockbox_path, access)?;
    let record = lb
        .get_form_record(&path)?
        .ok_or_else(|| Error::NotFound(format!("form record {path}")))?;
    let definition = lb.resolve_form_definition(record.type_id.as_str())?;
    println!("path\t{}", record.path);
    println!("name\t{}", record.name);
    println!("alias\t{}", record.definition_alias);
    println!("definition_id\t{}", record.type_id);
    println!("revision\t{}", record.definition_revision);
    for field in &definition.fields {
        let value = record
            .values
            .iter()
            .find(|value| value.field_id == field.id);
        let display = match value.map(|value| &value.value) {
            Some(FormValue::Normal(value)) => value.clone(),
            Some(FormValue::Secret(_)) => "<secret>".to_string(),
            None => String::new(),
        };
        println!("field\t{}\t{}\t{}", field.id, field.label, display);
    }
    for value in &record.values {
        if definition
            .fields
            .iter()
            .all(|field| field.id != value.field_id)
        {
            println!(
                "unknown-field\t{}\t{}",
                value.field_id, value.captured_label
            );
        }
    }
    Ok(())
}

fn list_with_format(args: &[String], access: &Access, format: OutputFormat) -> CliResult<()> {
    let lockbox_path = require_arg(args, 0, "lockbox")?;
    let pattern = args.get(1).map(String::as_str);
    let lb = open_existing(lockbox_path, access)?;
    let rows = lb
        .list_form_records()?
        .into_iter()
        .filter(|record| {
            pattern.is_none_or(|pattern| form_path_matches(pattern, record.path.as_str()))
        })
        .map(|record| {
            vec![
                record.path.to_string(),
                record.name,
                record.definition_alias,
                record.type_id.to_string(),
                record.definition_revision.to_string(),
            ]
        })
        .collect::<Vec<_>>();
    print_records(
        &["path", "name", "alias", "definition_id", "revision"],
        rows,
        format,
    )?;
    Ok(())
}

enum FieldValueSource {
    Interactive,
    Literal(String),
    Value(String),
    File(String),
    Stdin,
    FromEnv(String),
}

pub(crate) fn parse_field_spec(spec: &str) -> CliResult<FormFieldDefinition> {
    let mut parts = spec.splitn(4, ':');
    let id = parts.next().unwrap_or_default().to_string();
    let kind = match parts.next() {
        Some(kind) => parse_field_kind(kind)?,
        None => FormFieldKind::Text,
    };
    let required = matches!(parts.next(), Some("required"));
    let label = parts
        .next()
        .filter(|label| !label.is_empty())
        .unwrap_or(&id)
        .to_string();
    Ok(FormFieldDefinition {
        id,
        label,
        kind,
        required,
    })
}

fn parse_field_assignment(spec: &str) -> CliResult<(String, String)> {
    let Some((field, value)) = spec.split_once('=') else {
        return Err(
            Error::InvalidInput("form field assignment must be FIELD=VALUE".to_string()).into(),
        );
    };
    if field.is_empty() {
        return Err(Error::InvalidInput("form field id cannot be empty".to_string()).into());
    }
    Ok((field.to_string(), value.to_string()))
}

fn default_form_name(path: &str) -> String {
    path.trim_end_matches('/')
        .rsplit('/')
        .find(|part| !part.is_empty())
        .unwrap_or("form")
        .to_string()
}

fn fill_missing_fields_interactively(
    lb: &mut revault_lockbox_api::Lockbox,
    path: &LockboxPath,
    definition: &revault_lockbox_api::FormDefinition,
) -> CliResult<()> {
    let record = lb
        .get_form_record(path)?
        .ok_or_else(|| Error::NotFound(format!("form record {path}")))?;
    for field in &definition.fields {
        if record.values.iter().any(|value| value.field_id == field.id) {
            continue;
        }
        if field.kind.is_secret() {
            let value = prompt_secret(&format!("{}: ", field.label))?;
            if !field.required && value.is_empty() {
                continue;
            }
            lb.set_form_field_secret(path, &field.id, &value)?;
        } else {
            let value = prompt_normal_field(&field.label)?;
            if !field.required && value.is_empty() {
                continue;
            }
            lb.set_form_field_normal(path, &field.id, &value)?;
        }
    }
    Ok(())
}

fn edit_fields_interactively(
    lb: &mut revault_lockbox_api::Lockbox,
    path: &LockboxPath,
    definition: &revault_lockbox_api::FormDefinition,
) -> CliResult<()> {
    let record = lb
        .get_form_record(path)?
        .ok_or_else(|| Error::NotFound(format!("form record {path}")))?;
    for field in &definition.fields {
        let existing = record
            .values
            .iter()
            .find(|value| value.field_id == field.id)
            .map(|value| &value.value);
        if field.kind.is_secret() {
            let value = prompt_secret(&format!("{}: ", field.label))?;
            if value.is_empty() && (existing.is_some() || !field.required) {
                continue;
            }
            lb.set_form_field_secret(path, &field.id, &value)?;
        } else {
            let existing_normal = match existing {
                Some(FormValue::Normal(value)) => Some(value.as_str()),
                _ => None,
            };
            let value = prompt_normal_field_with_default(&field.label, existing_normal)?;
            if value.is_empty() && (existing.is_some() || !field.required) {
                continue;
            }
            lb.set_form_field_normal(path, &field.id, &value)?;
        }
    }
    Ok(())
}

fn prompt_normal_field(label: &str) -> CliResult<String> {
    prompt_normal_field_with_default(label, None)
}

fn prompt_normal_field_with_default(label: &str, default: Option<&str>) -> CliResult<String> {
    if let Some(default) = default {
        print!("{label} [{default}]: ");
    } else {
        print!("{label}: ");
    }
    std::io::stdout().flush()?;
    let mut value = String::new();
    std::io::stdin().read_line(&mut value)?;
    Ok(trim_trailing_newline(value))
}

fn parse_field_kind(value: &str) -> CliResult<FormFieldKind> {
    match value {
        "text" => Ok(FormFieldKind::Text),
        "secret" | "password" => Ok(FormFieldKind::Secret),
        "url" => Ok(FormFieldKind::Url),
        "email" => Ok(FormFieldKind::Email),
        "date" => Ok(FormFieldKind::Date),
        "month" => Ok(FormFieldKind::Month),
        "notes" => Ok(FormFieldKind::Notes),
        "number" => Ok(FormFieldKind::Number),
        _ => Err(Error::InvalidInput(format!("unsupported form field kind: {value}")).into()),
    }
}

fn read_normal_value(source: FieldValueSource) -> CliResult<String> {
    Ok(match source {
        FieldValueSource::Interactive => prompt_normal_field("Value")?,
        FieldValueSource::Literal(value) => value,
        FieldValueSource::Value(value) => value,
        FieldValueSource::File(path) => trim_trailing_newline(String::from_utf8(fs::read(path)?)?),
        FieldValueSource::Stdin => {
            let mut value = String::new();
            std::io::stdin().lock().read_to_string(&mut value)?;
            trim_trailing_newline(value)
        }
        FieldValueSource::FromEnv(name) => std::env::var(name)?,
    })
}

fn read_secret_value(source: FieldValueSource) -> CliResult<SecretString> {
    match &source {
        FieldValueSource::Literal(_) | FieldValueSource::Value(_) => {
            return Err(Error::InvalidInput(
                "secret form fields require --stdin, --file, --interactive, or --from-env"
                    .to_string(),
            )
            .into());
        }
        FieldValueSource::Interactive => return Ok(prompt_secret("Secret value: ")?),
        FieldValueSource::FromEnv(name) => {
            return SecretString::try_from_env(name)?
                .ok_or_else(|| Error::InvalidInput(format!("{name} is not set")).into());
        }
        FieldValueSource::File(_) | FieldValueSource::Stdin => {}
    }
    let value = read_normal_value(source)?;
    Ok(SecretString::try_from_bytes(value.into_bytes())?)
}

fn trim_trailing_newline(mut value: String) -> String {
    if value.ends_with('\n') {
        value.pop();
        if value.ends_with('\r') {
            value.pop();
        }
    }
    value
}

fn form_record_path(value: &str) -> CliResult<LockboxPath> {
    let value = if value.starts_with('/') {
        value.to_string()
    } else {
        format!("/{value}")
    };
    Ok(LockboxPath::new(value)?)
}

fn form_path_matches(pattern: &str, path: &str) -> bool {
    if pattern.contains('*') || pattern.contains('?') {
        let pattern = pattern.trim_start_matches('/');
        let path = path.trim_start_matches('/');
        return glob_matches(pattern, path);
    }
    let pattern = if pattern.starts_with('/') {
        pattern.to_string()
    } else {
        format!("/{pattern}")
    };
    path == pattern
        || path
            .strip_prefix(&pattern)
            .is_some_and(|rest| rest.starts_with('/'))
}

fn glob_matches(pattern: &str, text: &str) -> bool {
    let pattern = pattern.as_bytes();
    let text = text.as_bytes();
    let mut dp = vec![false; text.len() + 1];
    dp[0] = true;
    for &p in pattern {
        let mut next = vec![false; text.len() + 1];
        match p {
            b'*' => {
                next[0] = dp[0];
                for index in 0..text.len() {
                    next[index + 1] = dp[index + 1] || next[index] || dp[index];
                }
            }
            b'?' => {
                next[1..(text.len() + 1)].copy_from_slice(&dp[..text.len()]);
            }
            byte => {
                for index in 0..text.len() {
                    next[index + 1] = dp[index] && text[index] == byte;
                }
            }
        }
        dp = next;
    }
    dp[text.len()]
}
