use super::context::{open_existing, open_for_reading, Access, CliResult};
use super::output::OutputFormat;
use clap::ArgMatches;
use revault_lockbox_api::{Error, Lockbox, LockboxPath, VariableName, WritableLockboxState};

#[derive(Clone, Copy)]
pub(super) enum Target {
    Variable,
    Record,
    Definition,
    Field,
}

pub(super) fn vault_run(matches: &ArgMatches, field_target: bool) -> CliResult<()> {
    let (operation, args) = matches
        .subcommand()
        .ok_or_else(|| Error::InvalidInput("missing description operation".into()))?;
    let reference = args.get_one::<String>("target").expect("required target");
    let field = args
        .try_get_one::<String>("field-id")
        .ok()
        .flatten()
        .map(String::as_str)
        .unwrap_or_default();
    let text = if operation == "set" {
        Some(super::variables::read_description(args)?)
    } else {
        None
    };
    let vault = super::context::default_vault()?;
    if operation == "get" {
        let description = if field_target {
            vault.form_field_description(reference, field)?
        } else {
            Some(vault.resolve_form_definition(reference)?.description)
        };
        let description = description.ok_or_else(|| Error::NotFound("description".into()))?;
        use std::io::Write;
        std::io::stdout().write_all(description.as_bytes())?;
    } else {
        match (field_target, text) {
            (true, Some(text)) => {
                vault.set_form_field_description(reference, field, &text)?;
            }
            (true, None) => {
                vault.clear_form_field_description(reference, field)?;
            }
            (false, Some(text)) => {
                vault.set_form_definition_description(reference, &text)?;
            }
            (false, None) => {
                vault.clear_form_definition_description(reference)?;
            }
        }
        println!("Description updated: {reference}");
    }
    Ok(())
}

pub(super) fn run(matches: &ArgMatches, access: &Access, target: Target) -> CliResult<()> {
    let (operation, args) = matches
        .subcommand()
        .ok_or_else(|| Error::InvalidInput("missing description operation".into()))?;
    let reference = args.get_one::<String>("target").expect("required target");
    let field = args
        .try_get_one::<String>("field-id")
        .ok()
        .flatten()
        .map(String::as_str);
    let path = super::default_lockbox_for_command()?;
    match operation {
        "get" => {
            let lockbox = open_for_reading(&path, access)?;
            let text = get(&lockbox, target, reference, field)?
                .ok_or_else(|| Error::NotFound("description".into()))?;
            use std::io::Write;
            std::io::stdout().write_all(text.as_bytes())?;
        }
        "set" | "clear" => {
            // Read interactive input before taking the writable archive lock.
            let text = if operation == "set" {
                Some(super::variables::read_description(args)?)
            } else {
                None
            };
            let mut lockbox = open_existing(&path, access)?;
            change(&mut lockbox, target, reference, field, text.as_deref())?;
            lockbox.commit()?;
            println!(
                "Description {}: {reference}",
                if text.is_some() { "set" } else { "cleared" }
            );
        }
        _ => return Err(Error::InvalidInput("unknown description operation".into()).into()),
    }
    Ok(())
}

fn get<State>(
    lockbox: &Lockbox<State>,
    target: Target,
    reference: &str,
    field: Option<&str>,
) -> CliResult<Option<String>> {
    Ok(match target {
        Target::Variable => lockbox.variable_description(&VariableName::new(reference)?)?,
        Target::Record => lockbox.form_record_description(&LockboxPath::new(reference)?)?,
        Target::Definition => Some(lockbox.resolve_form_definition(reference)?.description),
        Target::Field => {
            lockbox.form_field_description(reference, field.expect("required field"))?
        }
    })
}

fn change<State: WritableLockboxState>(
    lockbox: &mut Lockbox<State>,
    target: Target,
    reference: &str,
    field: Option<&str>,
    text: Option<&str>,
) -> CliResult<()> {
    match (target, text) {
        (Target::Variable, Some(text)) => {
            lockbox.set_variable_description(&VariableName::new(reference)?, text)?
        }
        (Target::Variable, None) => {
            lockbox.clear_variable_description(&VariableName::new(reference)?)?
        }
        (Target::Record, Some(text)) => {
            lockbox.set_form_record_description(&LockboxPath::new(reference)?, text)?
        }
        (Target::Record, None) => {
            lockbox.clear_form_record_description(&LockboxPath::new(reference)?)?
        }
        (Target::Definition, Some(text)) => {
            lockbox.set_form_definition_description(reference, text)?;
        }
        (Target::Definition, None) => {
            lockbox.clear_form_definition_description(reference)?;
        }
        (Target::Field, Some(text)) => {
            lockbox.set_form_field_description(reference, field.expect("required field"), text)?;
        }
        (Target::Field, None) => {
            lockbox.clear_form_field_description(reference, field.expect("required field"))?;
        }
    }
    Ok(())
}

pub(super) fn description_cell(text: &str, format: OutputFormat) -> String {
    match format {
        OutputFormat::Json => text.to_string(),
        OutputFormat::Tsv => text
            .replace('\\', "\\\\")
            .replace('\t', "\\t")
            .replace('\r', "\\r")
            .replace('\n', "\\n"),
        OutputFormat::Table => {
            let first = text.lines().next().unwrap_or_default();
            let mut preview = first
                .chars()
                .take(100)
                .map(|c| if c == '\t' { ' ' } else { c })
                .collect::<String>();
            if first.chars().count() > 100 || text.lines().count() > 1 {
                preview.push('…');
            }
            preview
        }
    }
}

pub(super) fn print_description(label: &str, text: &str) {
    if !text.is_empty() {
        println!("{label}:");
        for line in text.split('\n') {
            println!("  {line}");
        }
    }
}
