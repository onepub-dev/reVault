use super::context::CliMessage;
use revault_lockbox_api::Error;
use std::env;
use std::fmt::Write as _;
use std::io::{self, IsTerminal, Write};

const RED_BOLD: &str = "\x1b[1;31m";
const YELLOW_BOLD: &str = "\x1b[1;33m";
const CYAN: &str = "\x1b[36m";
const RESET: &str = "\x1b[0m";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub(crate) enum ExitCode {
    General = 1,
    Usage = 2,
    LockboxClosed = 10,
    AuthenticationFailed = 11,
    NotFound = 12,
    VaultUnavailable = 13,
    UnsupportedFormat = 14,
    CorruptData = 15,
}

impl ExitCode {
    pub(crate) const fn as_i32(self) -> i32 {
        self as i32
    }
}

pub(crate) fn exit_code(err: &(dyn std::error::Error + 'static)) -> i32 {
    if let Some(error) = err.downcast_ref::<clap::Error>() {
        return error.exit_code();
    }
    if let Some(message) = err.downcast_ref::<CliMessage>() {
        return message.exit_code().as_i32();
    }
    if let Some(error) = err.downcast_ref::<Error>() {
        return api_error_exit_code(error).as_i32();
    }
    ExitCode::General.as_i32()
}

fn api_error_exit_code(error: &Error) -> ExitCode {
    match error {
        Error::InvalidInput(_) | Error::InvalidPath(_) | Error::InvalidKeyMaterial(_) => {
            ExitCode::Usage
        }
        Error::InvalidKey => ExitCode::AuthenticationFailed,
        Error::NotFound(_) => ExitCode::NotFound,
        Error::VaultUnavailable(_) => ExitCode::VaultUnavailable,
        Error::UnsupportedFormatVersion { .. } => ExitCode::UnsupportedFormat,
        Error::CorruptHeader
        | Error::CorruptRecord
        | Error::CorruptVaultRecord(_)
        | Error::Truncated => ExitCode::CorruptData,
        Error::AlreadyExists(_)
        | Error::InvalidOperation(_)
        | Error::LockUnavailable(_)
        | Error::Configuration(_)
        | Error::UnsupportedHostPath(_)
        | Error::Io(_)
        | Error::SecurityLimitExceeded(_) => ExitCode::General,
    }
}

pub(crate) fn print_error(err: &(dyn std::error::Error + 'static)) -> io::Result<()> {
    let colour = io::stderr().is_terminal()
        && env::var_os("NO_COLOR").is_none()
        && env::var("TERM").map_or(true, |term| term != "dumb");
    let rendered = render_error(err, colour);
    let mut stderr = io::stderr().lock();
    stderr.write_all(rendered.as_bytes())
}

fn render_error(err: &(dyn std::error::Error + 'static), colour: bool) -> String {
    if let Some(message) = err.downcast_ref::<CliMessage>() {
        return render_cli_message(message, colour);
    }
    if let Some(error) = err.downcast_ref::<Error>() {
        return render_api_error(error, colour);
    }

    let mut out = String::new();
    heading(&mut out, "Error", RED_BOLD, colour);
    write_indented(&mut out, &err.to_string());
    out
}

fn render_cli_message(message: &CliMessage, colour: bool) -> String {
    let mut out = String::new();
    heading(&mut out, "Error", RED_BOLD, colour);
    write_indented(&mut out, message.summary());
    for (label, value) in message.details() {
        out.push('\n');
        heading(&mut out, label, YELLOW_BOLD, colour);
        write_indented(&mut out, value);
    }
    if let Some(next_step) = message.next_step() {
        out.push('\n');
        heading(&mut out, "Next step", CYAN, colour);
        write_indented(&mut out, next_step);
    }
    out
}

fn render_api_error(error: &Error, colour: bool) -> String {
    let (summary, detail, next_step) = match error {
        Error::CorruptHeader => (
            "Lockbox header is damaged".to_string(),
            "The file header could not be read or authenticated.".to_string(),
            Some(error.guidance()),
        ),
        Error::UnsupportedFormatVersion {
            artifact,
            found,
            supported,
        } => (
            format!("Unsupported {} format", artifact.as_str()),
            format!("Found version {found}; this reVault build supports version {supported}."),
            Some(error.guidance()),
        ),
        Error::CorruptRecord => (
            "Lockbox data is damaged".to_string(),
            "A page or record failed validation.".to_string(),
            Some(error.guidance()),
        ),
        Error::CorruptVaultRecord(detail) => (
            "Vault data is damaged".to_string(),
            detail.clone(),
            Some(error.guidance()),
        ),
        Error::InvalidInput(detail) => ("Invalid input".to_string(), detail.clone(), None),
        Error::InvalidPath(detail) => (
            "Invalid lockbox path".to_string(),
            detail.clone(),
            Some(error.guidance()),
        ),
        Error::NotFound(detail) => (
            "Lockbox entry not found".to_string(),
            detail.clone(),
            Some(error.guidance()),
        ),
        Error::AlreadyExists(detail) => (
            "Lockbox entry already exists".to_string(),
            detail.clone(),
            Some("Choose another path, or explicitly replace the existing entry."),
        ),
        Error::InvalidKey => (
            "Unable to open the lockbox".to_string(),
            "The password or key was not accepted, or the lockbox failed authentication."
                .to_string(),
            Some(error.guidance()),
        ),
        Error::InvalidKeyMaterial(detail) => (
            "Invalid key".to_string(),
            detail.clone(),
            Some(error.guidance()),
        ),
        Error::InvalidOperation(detail) => (
            "Operation is not available".to_string(),
            detail.clone(),
            None,
        ),
        Error::VaultUnavailable(detail) => (
            "Vault is unavailable".to_string(),
            detail.clone(),
            Some("Run `lbx doctor` to check the vault and session configuration."),
        ),
        Error::LockUnavailable(detail) => (
            "Lockbox is in use".to_string(),
            detail.clone(),
            Some(error.guidance()),
        ),
        Error::Configuration(detail) => (
            "Configuration problem".to_string(),
            detail.clone(),
            Some(error.guidance()),
        ),
        Error::UnsupportedHostPath(detail) => (
            "Unsupported file type".to_string(),
            detail.clone(),
            Some(error.guidance()),
        ),
        Error::Io(detail) => (
            "Could not access a file or platform service".to_string(),
            detail.clone(),
            Some(error.guidance()),
        ),
        Error::SecurityLimitExceeded(detail) => (
            "Safety limit exceeded".to_string(),
            detail.clone(),
            Some(error.guidance()),
        ),
        Error::Truncated => (
            "Lockbox file is incomplete".to_string(),
            "The file ended before a complete lockbox structure could be read.".to_string(),
            Some(error.guidance()),
        ),
    };

    let mut out = String::new();
    heading(&mut out, "Error", RED_BOLD, colour);
    write_indented(&mut out, &summary);
    if !detail.is_empty() {
        out.push('\n');
        heading(&mut out, "Details", YELLOW_BOLD, colour);
        write_indented(&mut out, &detail);
    }
    if let Some(next_step) = next_step {
        out.push('\n');
        heading(&mut out, "Next step", CYAN, colour);
        write_indented(&mut out, next_step);
    }
    out
}

fn heading(out: &mut String, label: &str, style: &str, colour: bool) {
    if colour {
        let _ = writeln!(out, "{style}{label}:{RESET}");
    } else {
        let _ = writeln!(out, "{label}:");
    }
}

fn write_indented(out: &mut String, value: &str) {
    for line in value.lines() {
        let _ = writeln!(out, "  {line}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use revault_lockbox_api::ArtifactKind;

    #[test]
    fn structured_error_is_readable_without_colour() {
        let message = CliMessage {
            exit_code: ExitCode::LockboxClosed,
            summary: "Lockbox is closed".to_string(),
            details: vec![
                ("Lockbox".to_string(), "/tmp/secrets.lbox".to_string()),
                (
                    "Auto-open".to_string(),
                    "Your local vault uses format version 1; this reVault build uses version 2."
                        .to_string(),
                ),
            ],
            next_step: Some(
                "Migrate the vault, then retry:\n  lbx migrate vault --replace".to_string(),
            ),
        };

        assert_eq!(
            render_error(&message, false),
            "Error:\n  Lockbox is closed\n\nLockbox:\n  /tmp/secrets.lbox\n\nAuto-open:\n  Your local vault uses format version 1; this reVault build uses version 2.\n\nNext step:\n  Migrate the vault, then retry:\n    lbx migrate vault --replace\n"
        );
    }

    #[test]
    fn api_errors_have_labels_and_recovery_guidance() {
        let error = Error::UnsupportedFormatVersion {
            artifact: ArtifactKind::Vault,
            found: 1,
            supported: 2,
        };
        let rendered = render_error(&error, false);
        assert!(rendered.contains("Error:\n  Unsupported vault format"));
        assert!(rendered.contains("Details:\n  Found version 1"));
        assert!(rendered.contains("Next step:\n  Run `lockbox migrate vault"));
    }

    #[test]
    fn colour_is_only_added_when_requested() {
        let error = Error::InvalidInput("bad value".to_string());
        assert!(!render_error(&error, false).contains("\x1b["));
        assert!(render_error(&error, true).contains(RED_BOLD));
        assert!(render_error(&error, true).contains(YELLOW_BOLD));
    }

    #[test]
    fn low_level_prefixes_are_replaced_with_plain_language() {
        let error = Error::Io("open /tmp/secrets.lbox: permission denied".to_string());
        let rendered = render_error(&error, false);
        assert!(rendered.contains("Could not access a file or platform service"));
        assert!(!rendered.contains("io error:"));
        assert!(rendered.contains("Next step:"));
    }

    #[test]
    fn errors_have_stable_exit_codes() {
        assert_eq!(exit_code(&Error::InvalidKey), 11);
        assert_eq!(exit_code(&Error::NotFound("/missing".to_string())), 12);
        assert_eq!(
            exit_code(&Error::UnsupportedFormatVersion {
                artifact: ArtifactKind::Lockbox,
                found: 2,
                supported: 1,
            }),
            14
        );
        assert_eq!(exit_code(&Error::CorruptRecord), 15);
    }
}
