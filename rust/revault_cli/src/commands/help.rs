use clap::{Arg, ArgAction, Command, ValueHint};
use clap_complete::engine::ArgValueCompleter;

use super::completion;

const ABOUT: &str =
    "Create encrypted file archives, store secrets safely, and grant access with public keys.";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const VERBOSE_HELP_TEMPLATE: &str = "\
{about-with-newline}
{before-help}
{usage-heading} {usage}

{all-args}{after-help}\
";

pub(crate) fn command(verbose: bool) -> Command {
    let command = Command::new("lockbox")
        .about(ABOUT)
        .version(VERSION)
        .disable_help_subcommand(true)
        .arg_required_else_help(true)
        .subcommand_required(true)
        .subcommand_precedence_over_arg(true)
        .subcommand_help_heading("Available commands")
        .after_help(
            "Run \"lockbox [LOCKBOX] <command> --help\" for more information about a command.",
        )
        .next_help_heading("Global options")
        .arg(
            Arg::new("command-lockbox")
                .value_name("LOCKBOX")
                .required(false)
                .add(ArgValueCompleter::new(completion::lockbox_path_candidates))
                .help("Lockbox for the command. Defaults to the session default lockbox."),
        )
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .global(true)
                .action(ArgAction::SetTrue)
                .help("Show detailed command forms and advanced options."),
        )
        .arg(
            Arg::new("key")
                .long("key")
                .global(true)
                .value_name("RAW_CONTENT_KEY")
                .hide(!verbose)
                .help("Developer override: open with a raw content key supplied out of band."),
        )
        .subcommands([
            archive_command("create", "Create a new encrypted lockbox.")
                .override_usage("lockbox <LOCKBOX> create [OPTIONS]")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox vault init\n  lockbox secrets.lbox create\n  lockbox secrets.lbox create --password\n  lockbox secrets.lbox create --for alice",
                    "Context:\n  Use create when starting a new encrypted archive. By default it creates a lockbox for the vault's default profile. Use --password when you need a password-protected lockbox.",
                ))
                .arg(
                    Arg::new("password")
                        .long("password")
                        .conflicts_with("for")
                        .action(ArgAction::SetTrue)
                        .help("Create a password-protected lockbox."),
                )
                .arg(
                    Arg::new("for")
                        .long("for")
                        .conflicts_with("password")
                        .value_name("PROFILE_OR_CONTACT")
                        .help("Create the lockbox for one of your profiles or a saved contact.")
                        .add(ArgValueCompleter::new(completion::named_candidates)),
                )
                .arg(
                    Arg::new("description")
                        .long("description")
                        .value_name("TEXT")
                        .help("Store an encrypted description in the new lockbox."),
                ),
            archive_command("open", "Open the lockbox for later commands.")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox secrets.lbox open\n  lockbox secrets.lbox open --duration 30m\n  LOCKBOX_PASSWORD=secret lockbox secrets.lbox open\n  printf '%s\\n' \"$LOCKBOX_PASSWORD\" | lockbox secrets.lbox open --password-stdin",
                    "Context:\n  Opens the Lockbox for later commands. Close the Lockbox when you have finished working with it. On supported platforms, the Session Agent will automatically close the Lockbox after 30 minutes.",
                ))
                .arg(
                    Arg::new("duration")
                        .short('d')
                        .long("duration")
                        .value_name("DURATION")
                        .help("Keep the lockbox open for this session duration, such as 30s, 30m, 2h, or 1d."),
                )
                .arg(
                    Arg::new("password-env")
                        .long("password-env")
                        .value_name("NAME")
                        .conflicts_with_all(["password-file", "password-stdin"])
                        .help("Read the Lockbox password from this variable."),
                )
                .arg(
                    Arg::new("password-file")
                        .long("password-file")
                        .value_name("FILE")
                        .value_hint(ValueHint::FilePath)
                        .conflicts_with_all(["password-env", "password-stdin"])
                        .help("Read the Lockbox password from a file."),
                )
                .arg(
                    Arg::new("password-stdin")
                        .long("password-stdin")
                        .action(ArgAction::SetTrue)
                        .conflicts_with_all(["password-env", "password-file"])
                        .help("Read the Lockbox password from stdin."),
                ),
            archive_command("close", "Close the lockbox.")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox secrets.lbox close\n  lockbox close",
                    "Context:\n  Closes the given Lockbox or the default Lockbox if no argument is given. On supported platforms the Session Agent will automatically close the Lockbox after 30 minutes.",
                )),
            file_command("add", "Add a file or directory to a lockbox.")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox add ./notes.txt\n  lockbox add --recursive .\n  lockbox secrets.lbox add ./*.key\n  lockbox secrets.lbox add ./*.key --to keys/\n  lockbox secrets.lbox add ./notes.txt --to docs/readme.txt\n  lockbox secrets.lbox add ./notes.txt --to docs/readme.txt --overwrite\n  lockbox secrets.lbox add --recursive ./src ./assets README.md",
                    "Context:\n  Add imports one or more host files into the selected lockbox. Put the lockbox before the command, or omit it to use the session default. Every positional argument is a source; use --to for the logical destination. Relative logical destinations are rooted at the lockbox root. Existing files are protected unless --overwrite is explicit. Pass --recursive for a directory source. Use --jobs in verbose mode to tune large imports.",
                ))
                .arg(
                    Arg::new("recursive")
                        .short('r')
                        .long("recursive")
                        .action(ArgAction::SetTrue)
                        .help("Recursively import a directory source."),
                )
                .arg(
                    Arg::new("jobs")
                        .long("jobs")
                        .value_name("auto|1|N")
                        .hide(!verbose)
                        .help("Set import worker count."),
                )
                .arg(
                    Arg::new("to")
                        .long("to")
                        .value_name("LOCKBOX_PATH")
                        .add(ArgValueCompleter::new(completion::archive_directory_candidates))
                        .help("Logical destination. End with / when adding multiple sources."),
                )
                .arg(
                    Arg::new("overwrite")
                        .long("overwrite")
                        .action(ArgAction::SetTrue)
                        .help("Replace mapped files that already exist in the lockbox."),
                )
                .arg(
                    Arg::new("quiet")
                        .short('q')
                        .long("quiet")
                        .action(ArgAction::SetTrue)
                        .help("Suppress progress messages; result output is unchanged."),
                )
                .arg(filter_arg("include"))
                .arg(filter_arg("exclude"))
                .arg(
                    Arg::new("sources")
                        .value_name("SOURCE")
                        .num_args(1..)
                        .action(ArgAction::Append)
                        .required(true)
                        .value_hint(ValueHint::AnyPath)
                        .help("One or more host files or directories; directories require --recursive."),
                ),
            mirror_command(verbose),
            file_command("extract", "Extract a file, directory, or complete lockbox.")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox secrets.lbox extract /notes.txt ./notes.txt\n  lockbox secrets.lbox extract /docs ./docs\n  lockbox secrets.lbox extract /docs --to ./docs\n  lockbox secrets.lbox extract --to ./restore\n  lockbox secrets.lbox extract --to ./restore --overwrite",
                    "Context:\n  Extract copies encrypted content back to the host filesystem. Supply a stored file or directory and its exact host destination to extract one selection. Omit the stored path and use --to to restore the whole lockbox.",
                ))
                .arg(
                    Arg::new("to")
                        .long("to")
                        .value_name("DESTINATION")
                        .value_hint(ValueHint::AnyPath)
                        .help("Exact host destination for a selected path, or directory for the full lockbox."),
                )
                .arg(
                    Arg::new("overwrite")
                        .long("overwrite")
                        .action(ArgAction::SetTrue)
                        .help("Overwrite existing files."),
                )
                .arg(
                    Arg::new("restore-symlinks")
                        .long("restore-symlinks")
                        .action(ArgAction::SetTrue)
                        .help("Restore symlinks when extracting a directory."),
                )
                .arg(
                    Arg::new("restore-permissions")
                        .long("restore-permissions")
                        .action(ArgAction::SetTrue)
                        .help("Restore file permissions when extracting a directory."),
                )
                .arg(
                    Arg::new("args")
                        .value_name("PATH DESTINATION")
                        .num_args(0..=2)
                        .action(ArgAction::Append)
                        .help("Stored file/directory and exact host destination; omit both to extract the full lockbox with --to.")
                        .add(ArgValueCompleter::new(completion::archive_value_candidates)),
                ),
            file_command("cat", "Write stored files to stdout.")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox secrets.lbox cat /notes.txt\n  lockbox secrets.lbox cat /notes.txt > notes.txt",
                    "Context:\n  Cat streams one or more stored files to stdout. Use it for inspection, piping, or shell redirection when you do not want reVault to create host files directly.",
                ))
                .arg(
                    Arg::new("args")
                        .value_name("PATH")
                        .num_args(1..)
                        .action(ArgAction::Append)
                        .required(true)
                        .help("One or more stored paths in the selected lockbox.")
                        .add(ArgValueCompleter::new(completion::archive_value_candidates)),
                ),
            file_command("list", "List stored entries.")
                .visible_alias("ls")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox secrets.lbox list\n  lockbox secrets.lbox list /project\n  lockbox secrets.lbox list '/project/**/*.txt'\n  lockbox secrets.lbox list --recursive --format json",
                    "Context:\n  List shows files and inferred directories stored in a lockbox. The default view mirrors a normal directory listing; pass a glob pattern to match stored paths, or use --recursive when scripts or audits need full stored paths.",
                ))
                .arg(output_format_arg())
                .arg(
                    Arg::new("recursive")
                        .short('R')
                        .long("recursive")
                        .action(ArgAction::SetTrue)
                        .help("List entries below child directories."),
                )
                .arg(
                    Arg::new("args")
                        .value_name("PATH")
                        .num_args(0..=1)
                        .action(ArgAction::Append)
                        .help("Optional stored path or glob in the selected lockbox.")
                        .add(ArgValueCompleter::new(completion::archive_value_candidates)),
                ),
            file_command("remove", "Remove a stored entry.")
                .visible_aliases(["rm", "delete"])
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox secrets.lbox remove notes.txt\n  lockbox secrets.lbox remove package.json package-lock.json\n  lockbox secrets.lbox rm '*.json'\n  lockbox secrets.lbox rm '**/*.json'\n  lockbox secrets.lbox remove --recursive old/\n  lockbox secrets.lbox remove --force old.txt",
                    "Context:\n  Remove accepts one or more stored paths or archive glob patterns and validates the complete batch before committing. A quoted * matches within one lockbox directory; use ** for recursive matching. Removing a directory requires --recursive. Without --force, reVault asks once for confirmation before changing the archive.",
                ))
                .arg(
                    Arg::new("force")
                        .long("force")
                        .action(ArgAction::SetTrue)
                        .help("Remove without an interactive confirmation."),
                )
                .arg(
                    Arg::new("recursive")
                        .short('r')
                        .visible_short_alias('R')
                        .long("recursive")
                        .action(ArgAction::SetTrue)
                        .help("Remove selected directories and their contents."),
                )
                .arg(
                    Arg::new("args")
                        .value_name("PATH_OR_GLOB...")
                        .num_args(1..)
                        .action(ArgAction::Append)
                        .required(true)
                        .help("One or more stored paths or archive globs in the selected lockbox.")
                        .add(ArgValueCompleter::new(completion::archive_value_candidates)),
                ),
            file_command("move", "Move or rename a stored entry.")
                .visible_aliases(["mv", "rename"])
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox secrets.lbox move draft.txt final.txt\n  lockbox secrets.lbox mv old-dir archive/old-dir\n  lockbox secrets.lbox rename old.txt new.txt",
                    "Context:\n  Move changes the path stored inside the lockbox. It does not touch host filesystem paths; both arguments are lockbox paths. Use the short mv alias or the equally descriptive rename synonym.",
                ))
                .arg(
                    Arg::new("args")
                        .value_names(["FROM", "TO"])
                        .num_args(2)
                        .required(true)
                        .help("Stored source and destination paths in the selected lockbox.")
                        .add(ArgValueCompleter::new(completion::archive_value_candidates)),
                ),
            variables_command(verbose),
            description_command(verbose),
            form_command(verbose),
            session_command(verbose),
            completion_command(),
            access_command(verbose),
            archive_command("doctor", "Diagnose and maintain vaults and lockboxes.")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox doctor\n  lockbox secrets.lbox doctor\n  lockbox damaged.lbox doctor recover --dry-run\n  lockbox doctor migrate vault --replace\n  lockbox doctor migrate lockbox secrets.lbox --replace",
                    "Context:\n  Doctor is the maintenance namespace for Vault and Lockbox health. With no Lockbox path, it reports local configuration and runtime state. With a Lockbox path, it inspects public metadata and performs deeper checks when the Lockbox can be opened. Recover repairs or salvages damaged Lockboxes; migrate upgrades valid Vaults and Lockboxes between native format versions.",
                ))
                .subcommands([recovery_command(verbose), migration_command(verbose)]),
            vault_command(verbose),
            developer_command("visualize", "Print internal lockbox structure.")
                .visible_alias("visualise"),
            developer_command("keygen", "Generate raw keypair files.")
                .arg(required("private-key", "Private key output path."))
                .arg(required("public-key", "Public key output path.")),
            developer_command("open-key", "Open a lockbox using a vault private key.")
                .arg(
                    Arg::new("args")
                        .value_name("VAULT_KEY")
                        .num_args(0..=1)
                        .action(ArgAction::Append)
                        .help("Optional vault private key name."),
                ),
        ]);
    if verbose {
        apply_verbose_help_template(command)
    } else {
        command
    }
}

pub(crate) fn usage(verbose: bool) {
    eprintln!(
        "{ABOUT}

Version: {VERSION}

Usage: lockbox [LOCKBOX] <command> [arguments]

Global options:
    --verbose        Show detailed command forms and advanced options.
-h, --help           Print this usage information.
-V, --version        Print version information.

Available commands:

Archives
  create          Create a new encrypted lockbox.
  open            Open a lockbox for later commands.
  close           Close the lockbox.

Files
  add             Add a file or directory to a lockbox.
  mirror          Manage persistent host-directory mirror projects.
  extract         Extract files from a lockbox.
  cat             Write a stored file to stdout.
  list            List stored entries.
  remove          Remove stored entries (aliases: rm, delete).
  move            Move or rename a stored entry (aliases: mv, rename).

Data
  description     Get, set, or clear the encrypted lockbox description.
  variable        Store, retrieve, list, export, or remove variable values.
  form            Manage typed multi-field form records.

Session
  session         Manage the default Lockbox and keys cached by the Session Agent.

Completion
  completion      Generate or install dynamic shell completion.

Sharing
  access          Grant or revoke who can open a lockbox.

Diagnostics
  doctor          Diagnose and maintain vaults and lockboxes.

Vault
  vault           Manage profiles, contacts, and reusable forms."
    );

    if verbose {
        eprintln!(
            "
Advanced global options:
    --key <raw-content-key>    Developer override: open with a raw content key supplied out of band.

Advanced command options:
  lockbox [LOCKBOX] add --jobs auto|1|N <source>... [--to <lockbox-path>]

Developer and compatibility commands:
  keygen          Generate raw keypair files.
  open-key        Open a lockbox using a vault private key.
  visualize       Print internal lockbox structure.

Maintenance commands:
  doctor recover  Recover damaged Lockboxes.
  doctor migrate  Migrate Vaults and Lockboxes between native format versions.

Process variables:
  LOCKBOX_KEY=<raw-content-key> lockbox <command> ...
    LOCKBOX_PASSWORD=<password> lockbox <lockbox> open
  LOCKBOX_OPEN_DURATION=30m lockbox <lockbox> open
  LOCKBOX_VAULT_PASSWORD=<password> lockbox vault <command>
  LOCKBOX_PLATFORM_SECRET_STORE=auto|disabled lockbox vault <command>
  LOCKBOX_SESSION_AGENT_DIR=<dir> lockbox <command> ...
  LOCKBOX_VAULT_DIR=<dir> lockbox <command> ...

Raw content keys are for developer recovery and local testing. reVault does not
print or export them; normal commands should open through the vault session."
        );
    }

    eprintln!(
        "
Run \"lockbox <command> --help\" for more information about a command."
    );
}

fn archive_command(name: &'static str, about: &'static str) -> Command {
    base_command(name, about)
}

fn recovery_command(verbose: bool) -> Command {
    Command::new("recover")
        .about("Recover a lockbox using the safest applicable operation.")
        .after_help(verbose_help(
            verbose,
            "Examples:\n  lockbox damaged.lbox doctor recover\n  lockbox damaged.lbox doctor recover --output recovered.lbox\n  lockbox damaged.lbox doctor recover --dry-run --format table",
            "Context:\n  Recover first detects authenticated interrupted transaction cleanup and completes it in place. If no cleanup is pending, it scans the damaged lockbox and writes a new lockbox containing readable entries. By default the recovered file is written next to the original as <name>.recovered.lbox. Use --dry-run to inspect the operation without changing files.",
        ))
        .arg(
            Arg::new("output")
                .long("output")
                .short('o')
                .value_name("RECOVERED_LOCKBOX")
                .value_hint(ValueHint::AnyPath)
                .help("Write salvaged entries to this new lockbox."),
        )
        .arg(
            Arg::new("overwrite")
                .long("overwrite")
                .action(ArgAction::SetTrue)
                .help("Replace the salvage output file if it already exists."),
        )
        .arg(
            Arg::new("dry-run")
                .long("dry-run")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["output", "overwrite"])
                .help("Report the detected recovery operation without changing files."),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .action(ArgAction::SetTrue)
                .help("Suppress progress messages; recovery output is unchanged."),
        )
        .arg(output_format_arg())
}

fn file_command(name: &'static str, about: &'static str) -> Command {
    base_command(name, about)
}

fn mirror_command(verbose: bool) -> Command {
    file_command(
        "mirror",
        "Manage named host-to-lockbox directory mirrors.",
    )
    .override_usage(
        "lockbox [LOCKBOX] mirror <NAME> create --from <HOST_DIRECTORY> --to <LOCKBOX_DIRECTORY>\n       lockbox [LOCKBOX] mirror [NAME] <COMMAND>",
    )
    .subcommand_required(true)
    .arg_required_else_help(true)
    .subcommand_precedence_over_arg(true)
    .arg(
        Arg::new("project")
            .value_name("NAME")
            .required(false)
            .add(ArgValueCompleter::new(completion::mirror_project_candidates))
            .help("Mirror project name. Required before 'create'; otherwise omit when exactly one project exists."),
    )
    .subcommands([
        Command::new("create")
            .about("Create a mirror project without importing files.")
            .override_usage(
                "lockbox [LOCKBOX] mirror <NAME> create --from <HOST_DIRECTORY> --to <LOCKBOX_DIRECTORY>",
            )
            .arg(
                Arg::new("misplaced-project")
                    .value_name("MISPLACED_NAME")
                    .hide(true),
            )
            .arg(
                Arg::new("from")
                    .long("from")
                    .value_name("HOST_DIRECTORY")
                    .value_hint(ValueHint::DirPath),
            )
            .arg(
                Arg::new("to")
                    .long("to")
                    .value_name("LOCKBOX_DIRECTORY")
                    .add(ArgValueCompleter::new(completion::archive_directory_candidates)),
            )
            .arg(
                Arg::new("adopt")
                    .long("adopt")
                    .action(ArgAction::SetTrue)
                    .help("Allow the project to take ownership of an existing non-empty directory."),
            )
            .arg(
                Arg::new("strict")
                    .long("strict")
                    .action(ArgAction::SetTrue)
                    .help("Re-hash every selected source file before committing each update."),
            ),
        Command::new("projects")
            .about("List configured mirror projects.")
            .arg(output_format_arg()),
        Command::new("info")
            .about("Show one mirror project's configuration.")
            .arg(output_format_arg()),
        Command::new("status")
            .about("Calculate the complete update plan without changing the lockbox.")
            .arg(
                Arg::new("quiet")
                    .short('q')
                    .long("quiet")
                    .action(ArgAction::SetTrue)
                    .help("Suppress progress messages; status output is unchanged."),
            )
            .arg(output_format_arg()),
        Command::new("update")
            .about("Update the managed directory from its configured host source.")
            .arg(
                Arg::new("force")
                    .long("force")
                    .action(ArgAction::SetTrue)
                    .help("Apply without prompting after safety checks pass."),
            )
            .arg(
                Arg::new("allow-empty")
                    .long("allow-empty")
                    .action(ArgAction::SetTrue)
                    .help("Allow an empty selected source set to remove managed files."),
            )
            .arg(
                Arg::new("allow-large-delete")
                    .long("allow-large-delete")
                    .action(ArgAction::SetTrue)
                    .help("Allow removal of more than half the managed files."),
            )
            .arg(
                Arg::new("quiet")
                    .short('q')
                    .long("quiet")
                    .action(ArgAction::SetTrue)
                    .help("Suppress progress messages; result output is unchanged."),
            ),
        Command::new("configure")
            .about("Change persistent mirror behaviour.")
            .group(
                clap::ArgGroup::new("setting")
                    .required(true)
                    .multiple(true)
                    .args(["missing-files", "strict", "no-strict"]),
            )
            .arg(
                Arg::new("missing-files")
                    .long("missing-files")
                    .value_name("remove|retain")
                    .value_parser(["remove", "retain"])
                    .help(
                        "Choose whether updates remove or retain lockbox files absent from the selected host content.",
                    ),
            )
            .arg(
                Arg::new("strict")
                    .long("strict")
                    .action(ArgAction::SetTrue)
                    .conflicts_with("no-strict")
                    .help("Re-hash every selected source file before committing updates."),
            )
            .arg(
                Arg::new("no-strict")
                    .long("no-strict")
                    .action(ArgAction::SetTrue)
                    .help("Re-hash only source files whose metadata changes."),
            ),
        Command::new("rebind")
            .about("Bind the project to a moved or replaced host directory.")
            .arg(
                Arg::new("from")
                    .long("from")
                    .value_name("HOST_DIRECTORY")
                    .required(true)
                    .value_hint(ValueHint::DirPath),
            )
            .arg(
                Arg::new("force")
                    .long("force")
                    .action(ArgAction::SetTrue)
                    .help("Rebind without prompting."),
            ),
        Command::new("forget")
            .about("Remove project metadata while retaining its files.")
            .arg(
                Arg::new("force")
                    .long("force")
                    .action(ArgAction::SetTrue)
                    .help("Forget without prompting."),
            ),
        Command::new("destroy")
            .visible_alias("delete-project")
            .about("Delete the project and its complete managed directory.")
            .arg(
                Arg::new("force")
                    .long("force")
                    .action(ArgAction::SetTrue)
                    .help("Delete without prompting."),
            ),
        mirror_rule_command(),
        mirror_add_command(verbose),
        mirror_extract_command(),
        Command::new("cat")
            .about("Write a project file to stdout.")
            .arg(
                required("path", "Project-relative stored file path.")
                    .add(ArgValueCompleter::new(completion::mirror_entry_candidates)),
            ),
        Command::new("list")
            .visible_alias("ls")
            .about("List project entries.")
            .arg(output_format_arg())
            .arg(
                Arg::new("recursive")
                    .short('R')
                    .long("recursive")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                optional("path", "Optional project-relative path or glob.")
                    .add(ArgValueCompleter::new(completion::mirror_entry_candidates)),
            ),
        Command::new("remove")
            .visible_aliases(["rm", "delete"])
            .about("Remove project entries.")
            .arg(
                Arg::new("force")
                    .long("force")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("recursive")
                    .short('r')
                    .visible_short_alias('R')
                    .long("recursive")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("paths")
                    .value_name("PATH_OR_GLOB")
                    .num_args(1..)
                    .action(ArgAction::Append)
                    .required(true)
                    .add(ArgValueCompleter::new(completion::mirror_entry_candidates)),
            ),
        Command::new("move")
            .visible_aliases(["mv", "rename"])
            .about("Move or rename a project entry.")
            .arg(
                required("from", "Project-relative source path.")
                    .add(ArgValueCompleter::new(completion::mirror_entry_candidates)),
            )
            .arg(
                required("to", "Project-relative destination path.")
                    .add(ArgValueCompleter::new(completion::mirror_directory_candidates)),
            ),
    ])
    .after_help(verbose_help(
        verbose,
        "Examples:\n  lockbox store.lbox mirror project create --from ./project --to /projects/project\n  lockbox store.lbox mirror project status\n  lockbox store.lbox mirror project update\n  lockbox store.lbox mirror project rule add exclude '*.tmp'",
        "Context:\n  A mirror project exclusively owns one lockbox directory. Status is the only preview operation; update applies the freshly calculated plan.",
    ))
}

fn mirror_rule_command() -> Command {
    Command::new("rule")
        .visible_alias("rules")
        .about("List or change persistent mirror selection rules.")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommands([
            Command::new("list")
                .visible_alias("ls")
                .arg(
                    Arg::new("kind")
                        .value_name("include|exclude")
                        .value_parser(["include", "exclude"]),
                )
                .arg(output_format_arg()),
            Command::new("add")
                .arg(required_rule_kind())
                .arg(rule_patterns()),
            Command::new("remove")
                .visible_aliases(["rm", "delete"])
                .arg(required_rule_kind())
                .arg(
                    rule_patterns().add(ArgValueCompleter::new(completion::mirror_rule_candidates)),
                ),
            Command::new("clear").arg(
                Arg::new("kind")
                    .value_name("include|exclude|all")
                    .required(true)
                    .value_parser(["include", "exclude", "all"]),
            ),
        ])
}

fn required_rule_kind() -> Arg {
    Arg::new("kind")
        .value_name("include|exclude")
        .required(true)
        .value_parser(["include", "exclude"])
}

fn rule_patterns() -> Arg {
    Arg::new("patterns")
        .value_name("PATTERN")
        .num_args(1..)
        .action(ArgAction::Append)
        .required(true)
}

fn mirror_add_command(verbose: bool) -> Command {
    Command::new("add")
        .about("Add host files within the managed project directory.")
        .arg(
            Arg::new("recursive")
                .short('r')
                .long("recursive")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("jobs")
                .long("jobs")
                .value_name("auto|1|N")
                .hide(!verbose),
        )
        .arg(
            Arg::new("to")
                .long("to")
                .value_name("PROJECT_PATH")
                .add(ArgValueCompleter::new(
                    completion::mirror_directory_candidates,
                )),
        )
        .arg(
            Arg::new("overwrite")
                .long("overwrite")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .action(ArgAction::SetTrue),
        )
        .arg(filter_arg("include"))
        .arg(filter_arg("exclude"))
        .arg(
            Arg::new("sources")
                .value_name("SOURCE")
                .num_args(1..)
                .action(ArgAction::Append)
                .required(true)
                .value_hint(ValueHint::AnyPath),
        )
}

fn mirror_extract_command() -> Command {
    Command::new("extract")
        .about("Extract a project file, directory, or the complete project.")
        .after_help(
            "Examples:\n  lbx house.lbox mirror home extract notes.txt ./notes.txt\n  lbx house.lbox mirror home extract docs ./docs\n  lbx house.lbox mirror home extract docs --to ./docs\n  lbx house.lbox mirror home extract --to ./restore",
        )
        .arg(
            Arg::new("to")
                .long("to")
                .value_name("DESTINATION")
                .value_hint(ValueHint::AnyPath)
                .help("Exact host destination for a selected path, or directory for the complete project."),
        )
        .arg(
            Arg::new("overwrite")
                .long("overwrite")
                .action(ArgAction::SetTrue)
                .help("Overwrite existing files at the host destination."),
        )
        .arg(
            Arg::new("restore-symlinks")
                .long("restore-symlinks")
                .action(ArgAction::SetTrue)
                .help("Restore symlinks when extracting a directory."),
        )
        .arg(
            Arg::new("restore-permissions")
                .long("restore-permissions")
                .action(ArgAction::SetTrue)
                .help("Restore stored permissions when extracting a directory."),
        )
        .arg(
            Arg::new("args")
                .value_name("PATH DESTINATION")
                .num_args(0..=2)
                .action(ArgAction::Append)
                .help("Project-relative file/directory and exact host destination; omit both to extract the complete project with --to."),
        )
}

fn filter_arg(name: &'static str) -> Arg {
    Arg::new(name)
        .long(name)
        .value_name("PATTERN")
        .action(ArgAction::Append)
        .help(if name == "include" {
            "Include matching source-relative paths or globs. Repeat as needed."
        } else {
            "Exclude matching source-relative paths or globs. Repeat as needed."
        })
}

fn sharing_command(name: &'static str, about: &'static str) -> Command {
    base_command(name, about)
}

fn developer_command(name: &'static str, about: &'static str) -> Command {
    base_command(name, about).hide(true)
}

fn base_command(name: &'static str, about: &'static str) -> Command {
    Command::new(name)
        .about(about)
        .disable_help_subcommand(true)
}

fn variables_command(verbose: bool) -> Command {
    base_command(
        "variable",
        "Store, retrieve, list, export, or remove variable values.",
    )
    .visible_aliases(["var", "variables"])
    .after_help(verbose_help(
        verbose,
        "Examples:\n  lockbox secrets.lbox variable set APP_MODE production\n  lockbox secrets.lbox variable set APP_MODE=production\n  lockbox secrets.lbox variable get APP_MODE\n  lockbox secrets.lbox variable export",
        "Context:\n  Variables let you store name/value pairs securely in your lockbox. Names and matching are case-sensitive on every platform. For secrets, such as an API key, set the variable using the --secret flag to ensure an additional level of security is applied to those values.",
    ))
    .subcommand_required(true)
    .arg_required_else_help(true)
    .subcommand(
        Command::new("set")
            .about("Store a variable value.")
            .after_help(verbose_help(
                verbose,
                "Examples:\n  lockbox secrets.lbox variable set APP_MODE production\n  lockbox secrets.lbox variable set APP_MODE=production\n  lockbox secrets.lbox variable set --secret API_TOKEN --interactive\n  printf '%s' \"$TOKEN\" | lockbox secrets.lbox variable set --secret --stdin API_TOKEN",
                "Context:\n  Variables set writes one named value into a lockbox. Use --secret for values that should not be exported in bulk, such as tokens and passwords. Applying --secret to an existing normal variable upgrades it; making a secret variable normal still requires delete and recreate. Choose one value source: argument, prompt, stdin, file, or process environment. Secret values cannot use --value; use --stdin, --file, --interactive, or --from-env.",
            ))
            .arg(
                Arg::new("secret")
                    .short('s')
                    .long("secret")
                    .action(ArgAction::SetTrue)
                    .help("Store the value as secret."),
            )
            .arg(
                Arg::new("args")
                    .value_name("NAME[=VALUE] [VALUE]")
                    .num_args(1..=2)
                    .action(ArgAction::Append)
                    .required(true)
                    .add(ArgValueCompleter::new(completion::archive_value_candidates))
                    .help("Variable name and optional value in the selected lockbox."),
            )
            .arg(
                Arg::new("interactive")
                    .short('i')
                    .long("interactive")
                    .action(ArgAction::SetTrue)
                    .help("Prompt for the value."),
            )
            .arg(
                Arg::new("stdin")
                    .short('t')
                    .long("stdin")
                    .action(ArgAction::SetTrue)
                    .help("Read the value from stdin."),
            )
            .arg(
                Arg::new("value")
                    .short('v')
                    .long("value")
                    .value_name("VALUE")
                    .help("Read a normal value from this argument; not accepted with --secret."),
            )
            .arg(
                Arg::new("file")
                    .short('f')
                    .long("file")
                    .value_name("FILE")
                    .value_hint(ValueHint::FilePath)
                    .help("Read the value from a file."),
            )
            .arg(
                Arg::new("from-env")
                    .short('e')
                    .long("from-env")
                    .value_name("NAME")
                    .help("Read the value from a process variable."),
            ),
    )
    .subcommand(
        Command::new("get")
            .about("Print one stored variable value by name.")
            .after_help(verbose_help(
                verbose,
                "Examples:\n  lockbox secrets.lbox variable get APP_MODE\n  lockbox secrets.lbox variable get --secret API_TOKEN\n  lockbox secrets.lbox variable get --secret --output api-token.txt API_TOKEN",
                "Context:\n  Variables get reads one named value from a lockbox. Names are case-sensitive, independently of host environment-variable behavior. Secret values require --secret so accidental terminal output is an explicit user choice. Use --output when the exact bytes should go to a file.",
            ))
            .arg(
                Arg::new("secret")
                    .short('s')
                    .long("secret")
                    .action(ArgAction::SetTrue)
                    .help("Print a secret value."),
            )
            .arg(
                Arg::new("output")
                    .long("output")
                    .value_name("FILE")
                    .value_hint(ValueHint::AnyPath)
                    .help("Write the exact value bytes to a file instead of stdout."),
            )
            .arg(
                Arg::new("overwrite")
                    .long("overwrite")
                    .requires("output")
                    .action(ArgAction::SetTrue)
                    .help("Replace the output file if it already exists."),
            )
            .arg(
                Arg::new("args")
                    .value_name("NAME")
                    .num_args(1)
                    .required(true)
                    .add(ArgValueCompleter::new(completion::archive_value_candidates))
                    .help("Variable name in the selected lockbox."),
            ),
    )
    .subcommand(
        Command::new("list")
            .about("List variable values.")
            .visible_alias("ls")
            .after_help(verbose_help(
                verbose,
                "Examples:\n  lockbox secrets.lbox variable list\n  lockbox secrets.lbox variable list /production\n  lockbox secrets.lbox variable list '**/API_KEY'\n  lockbox secrets.lbox variable list --format json",
                "Context:\n  Variables list shows value names and whether each value is normal or secret. It does not print stored values. Paths and glob patterns are case-sensitive. Dot-prefixed variables are hidden unless --all is supplied. Pass a path such as /production to list that group, or a glob such as **/API_KEY to match names across groups.",
            ))
            .arg(output_format_arg())
            .arg(
                Arg::new("all")
                    .short('a')
                    .long("all")
                    .action(ArgAction::SetTrue)
                    .help("Include dot-prefixed hidden variables."),
            )
            .arg(
                Arg::new("args")
                    .value_name("PATTERN")
                    .num_args(0..=1)
                    .action(ArgAction::Append)
                    .add(ArgValueCompleter::new(completion::archive_value_candidates))
                    .help("Optional variable path or glob pattern."),
            ),
    )
    .subcommand(
        Command::new("export")
            .about("Print all non-secret variable values in an importable format.")
            .after_help(verbose_help(
                verbose,
                "Examples:\n  eval \"$(lockbox secrets.lbox variable export)\"\n  lockbox secrets.lbox variable export /production\n  lockbox secrets.lbox variable export '**/API_KEY'\n  lockbox secrets.lbox variable export --format posix > variables.sh\n  lockbox secrets.lbox variable export --format powershell | Invoke-Expression\n\nFormats:\n  posix       NAME='value' lines for sh, bash, and zsh. Default.\n  powershell  $env:NAME = 'value' lines for PowerShell.\n  cmd         set \"NAME=value\" lines for cmd.exe.\n  json        One JSON object per line with name and value fields.\n\n`variable export` writes to stdout. Use shell redirection to write it to a file.",
                "Context:\n  Variables export is intended for shell startup, CI setup, or scripting. It excludes secret and dot-prefixed hidden values. Use explicit variable get for hidden values or variable get --secret for secrets. The optional filter follows the same path or glob pattern rules as variable list. Grouped names are flattened with underscores for shell-safe output.",
            ))
            .arg(
                Arg::new("format")
                    .long("format")
                    .value_name("posix|powershell|cmd|json")
                    .default_value("posix")
                    .help("Output format."),
            )
            .arg(
                Arg::new("args")
                    .value_name("PATTERN")
                    .num_args(0..=1)
                    .action(ArgAction::Append)
                    .add(ArgValueCompleter::new(completion::archive_value_candidates))
                    .help("Optional variable path or glob pattern."),
            ),
    )
    .subcommand(
        Command::new("move")
            .visible_aliases(["mv", "rename"])
            .about("Move matching variables into another path.")
            .override_usage("lockbox [LOCKBOX] variable move [OPTIONS] <SOURCE> <DESTINATION>")
            .after_help(verbose_help(
                verbose,
                "Examples:\n  lockbox secrets.lbox variable move '/*' /dev\n  lockbox secrets.lbox variable mv '/production/*' /archive",
                "Context:\n  Move treats the destination as a variable group. Every match keeps its path relative to the non-glob source prefix. Existing destination variables are never overwritten. Quote glob patterns so the shell does not expand them.",
            ))
            .arg(
                Arg::new("args")
                    .value_names(["SOURCE", "DESTINATION"])
                    .num_args(2)
                    .required(true)
                    .action(ArgAction::Append)
                    .add(ArgValueCompleter::new(completion::archive_value_candidates))
                    .help("Source pattern and destination group in the selected lockbox."),
            ),
    )
    .subcommand(
        Command::new("remove")
            .visible_aliases(["rm", "delete"])
            .about("Remove variable values.")
            .after_help(verbose_help(
                verbose,
                "Examples:\n  lockbox secrets.lbox variable remove APP_MODE\n  lockbox secrets.lbox variable remove API_TOKEN",
                "Context:\n  Variables remove deletes one or more named values from a lockbox. It affects only lockbox records, not the current process environment.",
            ))
            .arg(
                Arg::new("args")
                    .value_name("NAME")
                    .num_args(1..)
                    .action(ArgAction::Append)
                    .required(true)
                    .add(ArgValueCompleter::new(completion::archive_value_candidates))
                    .help("One or more variable names in the selected lockbox."),
            ),
    )
}

fn description_command(verbose: bool) -> Command {
    base_command(
        "description",
        "Get, set, or clear the encrypted lockbox description.",
    )
    .after_help(verbose_help(
        verbose,
        "Examples:\n  lockbox secrets.lbox description get\n  lockbox secrets.lbox description set 'Deployment credentials for Project Atlas'\n  lockbox secrets.lbox description set --file purpose.txt\n  lockbox secrets.lbox description clear",
        "Context:\n  The description is encrypted inside the lockbox and cannot be read until the lockbox is opened. It accepts the same UTF-8 content and one-mebibyte limit as a normal variable value.",
    ))
    .subcommand_required(true)
    .arg_required_else_help(true)
    .subcommand(Command::new("get").about("Print the encrypted lockbox description."))
    .subcommand(
        Command::new("set")
            .about("Store or replace the encrypted lockbox description.")
            .arg(
                Arg::new("description")
                    .value_name("TEXT")
                    .num_args(0..=1)
                    .help("Description text."),
            )
            .arg(
                Arg::new("interactive")
                    .short('i')
                    .long("interactive")
                    .action(ArgAction::SetTrue)
                    .help("Prompt for the description."),
            )
            .arg(
                Arg::new("stdin")
                    .short('t')
                    .long("stdin")
                    .action(ArgAction::SetTrue)
                    .help("Read the description from stdin."),
            )
            .arg(
                Arg::new("file")
                    .short('f')
                    .long("file")
                    .value_name("FILE")
                    .value_hint(ValueHint::FilePath)
                    .help("Read the description from a UTF-8 file."),
            )
            .arg(
                Arg::new("from-env")
                    .short('e')
                    .long("from-env")
                    .value_name("NAME")
                    .help("Read the description from a process variable."),
            ),
    )
    .subcommand(Command::new("clear").about("Remove the encrypted lockbox description."))
}

fn form_command(verbose: bool) -> Command {
    base_command("form", "Manage typed multi-field form records.")
        .after_help(verbose_help(
            verbose,
            "Examples:\n  lockbox vault form define login --field username:text --field password:secret\n  lockbox secrets.lbox form use login\n  lockbox secrets.lbox form add /work/github --type login --name GitHub --set username=bsutton\n  lockbox secrets.lbox form show /work/github",
            "Context:\n  Forms store structured records inside a lockbox. Reusable definitions normally live in the vault and can be copied into a lockbox with form use. Definitions remain embedded in each lockbox so published lockboxes are self-describing.",
        ))
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("define")
                .about("Create or revise a form definition.")
                .override_usage(
                    "lockbox [LOCKBOX] form define [alias] --field <NAME[:KIND[:required[:LABEL]]]>...\n\nExample:\n  lockbox secrets.lbox form define login --field username:text --field password:secret",
                )
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox secrets.lbox form define login --field username:text --field password:secret\n  lockbox secrets.lbox form define --name Login --description \"Website sign-in\" --field username:text:required:User --field password:secret:required:Password\n  lockbox secrets.lbox form define login --name Login --description \"Website sign-in\" --field username:text:required:User --field password:secret:required:Password\n\nField form:\n  NAME[:KIND[:required[:LABEL]]]\n\nKinds:\n  text, secret, password, url, email, date, month, notes, number\n\nFormats:\n  date uses YYYY-MM-DD; month uses YYYY-MM",
                    "Context:\n  Define creates or revises a form definition. The alias is optional; when omitted, --name is required and an alias slug is derived from the display name. If the alias already resolves to exactly one definition, define appends a new revision. If an imported published lockbox has conflicting aliases, pass --definition-id to revise the intended definition explicitly.",
                ))
                .arg(
                    Arg::new("args")
                        .value_name("ALIAS")
                        .num_args(0..=1)
                        .action(ArgAction::Append)
                        .help("Optional form alias in the selected lockbox."),
                )
                .arg(
                    Arg::new("name")
                        .long("name")
                        .value_name("DISPLAY_NAME")
                        .help("Human display name for this form definition."),
                )
                .arg(
                    Arg::new("description")
                        .long("description")
                        .value_name("TEXT")
                        .help("Human description for this form definition."),
                )
                .arg(
                    Arg::new("definition-id")
                        .long("definition-id")
                        .alias("type-id")
                        .value_name("DEFINITION_ID")
                        .help("Revise or create this stable form definition id."),
                )
                .arg(
                    Arg::new("field")
                        .long("field")
                        .value_name("NAME[:KIND[:required[:LABEL]]]")
                        .action(ArgAction::Append)
                        .required(true)
                        .help("Add one field to the definition."),
                ),
        )
        .subcommand(
            Command::new("definitions")
                .about("List form definitions.")
                .arg(output_format_arg()),
        )
        .subcommand(
            Command::new("use")
                .about("Copy a vault form definition into a lockbox.")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox secrets.lbox form use login\n  lockbox form use login",
                    "Context:\n  Use copies a reusable definition from the vault into the lockbox. With a session default lockbox, the lockbox path can be omitted.",
                ))
                .arg(required("form", "Vault form alias or definition id.")),
        )
        .subcommand(
            Command::new("capture")
                .about("Copy a lockbox form definition into the vault.")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox secrets.lbox form capture login\n  lockbox secrets.lbox form capture login published-login\n  lockbox form capture login",
                    "Context:\n  Capture stores a lockbox definition in the vault so it can be reused. Pass a new form name when the vault already uses the same alias for a different definition.",
                ))
                .arg(
                    Arg::new("args")
                        .value_name("FORM NEW_NAME")
                        .num_args(1..=2)
                        .action(ArgAction::Append)
                        .required(true)
                        .help("Form name and optional vault name."),
                ),
        )
        .subcommand(
            Command::new("add")
                .about("Add a form record.")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox secrets.lbox form add /work/github --type login --name GitHub\n  lockbox secrets.lbox form add /work/github --type login --set username=bsutton --set site=https://github.com\n  lockbox secrets.lbox form add /work/github --type login --interactive",
                    "Context:\n  Add creates one form record in the lockbox. Use --set for non-secret values known up front. Use --interactive to prompt for remaining fields, including secret fields without echoing them.",
                ))
                .arg(
                    Arg::new("args")
                        .value_name("PATH")
                        .num_args(1)
                        .required(true)
                        .add(ArgValueCompleter::new(completion::archive_value_candidates))
                        .help("Form record path in the selected lockbox."),
                )
                .arg(
                    Arg::new("type")
                        .long("type")
                        .value_name("ALIAS_OR_DEFINITION_ID")
                        .required(true)
                        .add(ArgValueCompleter::new(completion::archive_value_candidates))
                        .help("Form definition alias or stable definition id."),
                )
                .arg(
                    Arg::new("name")
                        .long("name")
                        .value_name("RECORD_NAME")
                        .help("Display name for this record. Defaults to the last path component."),
                )
                .arg(
                    Arg::new("set")
                        .long("set")
                        .value_name("FIELD=VALUE")
                        .action(ArgAction::Append)
                        .help("Set one non-secret field while adding the form record."),
                )
                .arg(
                    Arg::new("interactive")
                        .long("interactive")
                        .short('i')
                        .action(ArgAction::SetTrue)
                        .help("Prompt for fields that were not supplied with --set."),
                ),
        )
        .subcommand(
            Command::new("edit")
                .about("Edit a form record.")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox secrets.lbox form edit /work/github --set username=bsutton\n  lockbox secrets.lbox form edit /work/github --interactive",
                    "Context:\n  Edit updates an existing form record. Use --interactive after a form definition revision to fill fields that exist in the latest definition but are missing from the stored record.",
                ))
                .arg(
                    Arg::new("args")
                        .value_name("PATH")
                        .num_args(1)
                        .required(true)
                        .add(ArgValueCompleter::new(completion::archive_value_candidates))
                        .help("Form record path in the selected lockbox."),
                )
                .arg(
                    Arg::new("set")
                        .long("set")
                        .value_name("FIELD=VALUE")
                        .action(ArgAction::Append)
                        .help("Set one non-secret field while editing the form record."),
                )
                .arg(
                    Arg::new("interactive")
                        .long("interactive")
                        .short('i')
                        .action(ArgAction::SetTrue)
                        .help("Prompt for fields missing from the current record."),
                ),
        )
        .subcommand(
            Command::new("set")
                .about("Set one form field value.")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox secrets.lbox form set /work/github username alice\n  printf '%s' \"$TOKEN\" | lockbox secrets.lbox form set --secret --stdin /work/github token",
                    "Context:\n  Form set updates one field. Applying --secret to a field currently defined as non-secret creates a new secret definition revision and upgrades existing values for that field across records of the same form type. Secret fields cannot be downgraded in place.",
                ))
                .arg(
                    Arg::new("args")
                        .value_names(["PATH", "FIELD", "VALUE"])
                        .num_args(2..=3)
                        .required(true)
                        .add(ArgValueCompleter::new(completion::archive_value_candidates))
                        .help("Form record path, field id, and optional value."),
                )
                .arg(
                    Arg::new("secret")
                        .long("secret")
                        .short('s')
                        .action(ArgAction::SetTrue)
                        .help("Set a secret field value."),
                )
                .arg(
                    Arg::new("explicit-value")
                        .long("value")
                        .short('v')
                        .value_name("VALUE")
                        .conflicts_with_all(["stdin", "file", "from-env", "interactive"])
                        .help("Set a literal non-secret field value."),
                )
                .arg(
                    Arg::new("stdin")
                        .long("stdin")
                        .short('t')
                        .action(ArgAction::SetTrue)
                        .conflicts_with_all(["explicit-value", "file", "from-env", "interactive"])
                        .help("Read the field value from stdin."),
                )
                .arg(
                    Arg::new("file")
                        .long("file")
                        .short('f')
                        .value_name("FILE")
                        .value_hint(ValueHint::FilePath)
                        .conflicts_with_all(["explicit-value", "stdin", "from-env", "interactive"])
                        .help("Read the field value from a file."),
                )
                .arg(
                    Arg::new("from-env")
                        .long("from-env")
                        .short('e')
                        .value_name("NAME")
                        .conflicts_with_all(["explicit-value", "stdin", "file", "interactive"])
                        .help("Read the field value from a variable."),
                )
                .arg(
                    Arg::new("interactive")
                        .long("interactive")
                        .short('i')
                        .action(ArgAction::SetTrue)
                        .conflicts_with_all(["explicit-value", "stdin", "file", "from-env"])
                        .help("Prompt for the field value."),
                ),
        )
        .subcommand(
            Command::new("get")
                .about("Print one form field value.")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox secrets.lbox form get /work/github username\n  lockbox secrets.lbox form get --secret /work/github password\n  lockbox secrets.lbox form get --secret --output password.txt /work/github password",
                    "Context:\n  Form get reads one field from a form record. Secret fields require --secret so accidental terminal output is an explicit user choice. Use --output when the exact bytes should go to a file.",
                ))
                .arg(
                    Arg::new("args")
                        .value_names(["PATH", "FIELD"])
                        .num_args(2)
                        .required(true)
                        .add(ArgValueCompleter::new(completion::archive_value_candidates))
                        .help("Form record path and field id."),
                )
                .arg(
                    Arg::new("secret")
                        .long("secret")
                        .short('s')
                        .action(ArgAction::SetTrue)
                        .help("Print a secret field value."),
                )
                .arg(
                    Arg::new("output")
                        .long("output")
                        .value_name("FILE")
                        .value_hint(ValueHint::AnyPath)
                        .help("Write the field value to this file."),
                )
                .arg(
                    Arg::new("overwrite")
                        .long("overwrite")
                        .requires("output")
                        .action(ArgAction::SetTrue)
                        .help("Replace the output file if it already exists."),
                ),
        )
        .subcommand(
            Command::new("show")
                .about("Show one form record.")
                .arg(
                    Arg::new("args")
                        .value_name("PATH")
                        .num_args(1)
                        .required(true)
                        .add(ArgValueCompleter::new(completion::archive_value_candidates))
                        .help("Form record path in the selected lockbox."),
                ),
        )
        .subcommand(
            Command::new("list")
                .visible_alias("ls")
                .about("List form records.")
                .arg(output_format_arg())
                .arg(
                    Arg::new("args")
                        .value_name("PATTERN")
                        .num_args(0..=1)
                        .action(ArgAction::Append)
                        .add(ArgValueCompleter::new(completion::archive_value_candidates))
                        .help("Optional form-record path or glob pattern."),
                ),
        )
        .subcommand(
            Command::new("move")
                .visible_aliases(["mv", "rename"])
                .about("Move matching form records into another path.")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox secrets.lbox form move '/work/*' /archive\n  lockbox secrets.lbox form mv '/dev/*' /production",
                    "Context:\n  Move treats the destination as a form-record directory. Every match keeps its path relative to the non-glob source prefix. Existing destination records are never overwritten. Quote glob patterns so the shell does not expand them.",
                ))
                .arg(
                    Arg::new("args")
                        .value_names(["SOURCE", "DESTINATION"])
                        .num_args(2)
                        .required(true)
                        .action(ArgAction::Append)
                        .add(ArgValueCompleter::new(completion::archive_value_candidates))
                        .help("Source pattern and destination directory."),
                ),
        )
        .subcommand(
            Command::new("remove")
                .visible_aliases(["rm", "delete"])
                .about("Remove form records.")
                .arg(
                    Arg::new("args")
                        .value_name("PATH")
                        .num_args(1..)
                        .action(ArgAction::Append)
                        .required(true)
                        .add(ArgValueCompleter::new(completion::archive_value_candidates))
                        .help("One or more form record paths in the selected lockbox."),
                ),
        )
}

fn session_command(verbose: bool) -> Command {
    base_command("session", "Manage the default Lockbox and keys cached by the Session Agent.")
        .disable_help_subcommand(true)
        .arg_required_else_help(false)
        .after_help(verbose_help(
            verbose,
            "Examples:\n  lockbox session\n  lockbox session default secrets.lbox\n  lockbox session default --clear\n  lockbox session auto-open lockboxes",
            "Context:\n  Session shows the default Lockbox and Lockboxes with keys cached by the Session Agent. The default Lockbox is the path used by commands that can safely omit a Lockbox argument. An open Lockbox can be read or changed without opening it again.",
        ))
        .arg(output_format_arg())
        .subcommand(
            Command::new("default")
                .about("Set or clear the default lockbox.")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox session default secrets.lbox\n  lockbox session default --clear",
                    "Context:\n  Default sets the Lockbox path used by commands that can safely omit a Lockbox argument. Clearing the default only removes that path; it does not clear cached Lockbox keys or change Auto Open credentials.",
                ))
                .arg(
                    Arg::new("clear")
                        .long("clear")
                        .action(ArgAction::SetTrue)
                        .conflicts_with("lockbox")
                        .help("Clear the default lockbox."),
                )
                .arg(optional("lockbox", "Lockbox path.").required_unless_present("clear")),
        )
        .subcommand(
            Command::new("close-all")
                .about("Close all lockboxes.")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox session close-all",
                    "Context:\n  Close-all clears every cached Lockbox content key from the Session Agent and clears the default Lockbox.",
                )),
        )
        .subcommand(
            Command::new("stop")
                .about("Close all open Lockboxes and stop the Session Agent.")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox session stop",
                    "Context:\n  Stop clears cached Lockbox keys, clears the default Lockbox, and shuts down the Session Agent process. Later commands can start it again when needed.",
                )),
        )
        .subcommand(
            Command::new("auto-open")
                .about("Allow reVault to use your OS login to automatically open the vault and lockboxes as required.")
                .disable_help_subcommand(true)
                .arg_required_else_help(false)
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox session auto-open status\n  lockbox session auto-open disable\n  lockbox session auto-open disable --yes\n  lockbox session auto-open vault\n  lockbox session auto-open lockboxes",
                    "Context:\n  Auto-open controls whether reVault may use your OS login to automatically open only the vault, or both the vault and lockboxes as required.",
                ))
                .subcommand(
                    Command::new("status")
                        .about("Show the current Auto Open scope.")
                        .arg(output_format_arg()),
                )
                .subcommand(
                    Command::new("disable")
                        .about("Disable Auto Open and close all open Lockboxes.")
                        .after_help(verbose_help(
                            verbose,
                            "Examples:\n  lockbox session auto-open disable\n  lockbox session auto-open disable --yes",
                            "Context:\n  Disabling Auto Open removes the stored Vault passphrase from the platform credential store and closes all open Lockboxes.",
                        ))
                        .arg(
                            Arg::new("yes")
                                .long("yes")
                                .action(ArgAction::SetTrue)
                                .help("Disable Auto Open without prompting."),
                        ),
                )
                .subcommand(Command::new("vault").about(
                    "Allow reVault to automatically open the vault only.",
                ))
                .subcommand(Command::new("lockboxes").about(
                    "Allow reVault to automatically open the vault and lockboxes.",
                )),
        )
}

fn access_command(verbose: bool) -> Command {
    sharing_command("access", "Grant or revoke who can open a lockbox.")
        .after_help(verbose_help(
            verbose,
            "Examples:\n  lockbox secrets.lbox access list\n  lockbox secrets.lbox access grant alice\n  lockbox secrets.lbox access revoke alice\n  lockbox secrets.lbox access revoke 2",
            "Context:\n  Access entries are stored on a lockbox and describe which profiles or contacts may open it. Use this command when sharing a lockbox or rotating/revoking access.",
        ))
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("grant")
                .about("Allow a profile or contact to open a lockbox.")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox secrets.lbox access grant alice\n  lockbox secrets.lbox access grant profile:alice\n  lockbox secrets.lbox access grant contact:alice\n  lockbox secrets.lbox access grant alice ./alice.pub",
                    "Context:\n  Access grant allows a profile or contact to open the lockbox. A bare name can refer to one of your saved profiles or saved contacts. If both use the same name, use profile:name or contact:name. For a public key file, provide the contact name first so the lockbox can record who the access entry belongs to.",
                ))
                .arg(
                    Arg::new("args")
                        .value_name("PROFILE PUBLIC_KEY")
                        .num_args(1..=2)
                        .action(ArgAction::Append)
                        .required(true)
                        .help(
                            "Profile name, contact name, profile:name, or contact:name. \
                             Public key path may follow a new contact name.",
                        ),
                ),
        )
        .subcommand(
            Command::new("list")
                .about("List who can open a lockbox.")
                .visible_alias("ls")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox secrets.lbox access list\n  lockbox secrets.lbox access list --format json",
                    "Context:\n  Access list shows the access slots currently attached to a lockbox, plus verified owner-signing status and host created/updated times. Contact names are not stored in lockbox metadata, so this output cannot identify or correlate the same contact across lockboxes. Use slot ids from this output when revoking access.",
                ))
                .arg(output_format_arg()),
        )
        .subcommand(
            Command::new("revoke")
                .about("Revoke access from a lockbox.")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox secrets.lbox access revoke alice\n  lockbox secrets.lbox access revoke 2\n  lockbox secrets.lbox access revoke alice bob 7",
                    "Context:\n  Access revoke removes one or more open slots and rewrites the archive with a fresh content key. Pass local profile/contact names when this vault remembers which slots were granted for those names, or pass the slot id from access list. reVault fails safely if retained access cannot be reconstructed.",
                ))
                .arg(
                    Arg::new("args")
                        .value_name("NAME_OR_SLOT_ID...")
                        .num_args(1..)
                        .action(ArgAction::Append)
                        .required(true)
                        .help("Local access names or slot ids in the selected lockbox."),
                ),
        )
        .subcommand(
            Command::new("refresh")
                .about("Refresh stale lockbox access entries.")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox project.lbox access refresh alice\n  lockbox access refresh --all alice\n  lockbox access refresh --all --dry-run",
                    "Context:\n  Access refresh checks named contact access entries and rewrites matching entries to the current vault Profile key. Use --dry-run first to see the planned changes and missing known lockboxes.",
                ))
                .arg(
                    Arg::new("all")
                        .long("all")
                        .action(ArgAction::SetTrue)
                        .help("Check every lockbox known to the vault."),
                )
                .arg(
                    Arg::new("args")
                        .value_name("PROFILE")
                        .num_args(0..=1)
                        .action(ArgAction::Append)
                        .help("Profile for the selected lockbox, or an optional profile with --all."),
                )
                .arg(
                    Arg::new("dry-run")
                        .long("dry-run")
                        .action(ArgAction::SetTrue)
                        .help("Print the refresh plan without changing lockboxes."),
                )
                .arg(
                    Arg::new("yes")
                        .long("yes")
                        .action(ArgAction::SetTrue)
                        .help("Apply without interactive confirmation."),
                ),
        )
}

fn vault_command(verbose: bool) -> Command {
    base_command("vault", "Manage profiles, contacts, and reusable forms.")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("init")
                .about("Create or open the local vault.")
                .after_help(verbose_help(
                    verbose,
                    "If the vault already exists, init reports the path and makes no changes. Use --verify to validate the pass phrase, or --overwrite only when replacing the vault and losing records stored only there.",
                    "Context:\n  The Vault stores Profiles, Contacts, and access-directory backups. New Vault passphrases must be at least 15 characters. A new Vault also gets a default Profile. Store the Vault passphrase safely; reVault cannot recover the Vault without it.",
                ))
                .arg(
                    Arg::new("verify")
                        .long("verify")
                        .conflicts_with("overwrite")
                        .action(ArgAction::SetTrue)
                        .help("Ask for the Vault passphrase and verify the existing Vault opens."),
                )
                .arg(
                    Arg::new("overwrite")
                        .long("overwrite")
                        .conflicts_with("verify")
                        .action(ArgAction::SetTrue)
                        .help("Replace an existing local vault."),
                ),
        )
        .subcommand(
            Command::new("beget")
                .about("Create an independent vault with a new profile.")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lbx vault beget production\n  lbx vault beget production --output .revault/production-runner.vault.lbx\n  lbx vault beget production --contact-name ci-production\n  lbx vault beget production --no-contact",
                    "Context:\n  Beget creates a new encrypted vault containing one fresh profile. By default, its public key is saved as a contact with the same name in the current vault. It does not copy private material from the current vault or grant access to any lockbox.",
                ))
                .arg(
                    Arg::new("profile")
                        .value_name("PROFILE")
                        .required(true)
                        .help("Profile name created inside the new vault."),
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("VAULT")
                        .value_hint(ValueHint::AnyPath)
                        .help("Output path. Defaults to <PROFILE>.vault.lbx."),
                )
                .arg(
                    Arg::new("contact-name")
                        .long("contact-name")
                        .value_name("NAME")
                        .conflicts_with("no-contact")
                        .help("Contact name in the current vault. Defaults to <PROFILE>."),
                )
                .arg(
                    Arg::new("no-contact")
                        .long("no-contact")
                        .action(ArgAction::SetTrue)
                        .help("Do not add a contact to the current vault."),
                ),
        )
        .subcommand(
            Command::new("backup")
                .about("Create an encrypted backup archive of the local vault.")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox vault backup ./vault-backup.lockbox-backup\n  lockbox vault backup --overwrite ./vault-backup.lockbox-backup",
                    "Context:\n  Backup takes a locked snapshot of the encrypted local-vault.lbox file and stores it with a manifest and checksum. It does not decrypt or export vault records.",
                ))
                .arg(
                    Arg::new("overwrite")
                        .long("overwrite")
                        .action(ArgAction::SetTrue)
                        .help("Replace an existing backup file."),
                )
                .arg(required("output", "Backup archive output path.")),
        )
        .subcommand(
            Command::new("passphrase")
                .about("Change the Vault passphrase.")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox Vault passphrase",
                    "Context:\n  The command verifies the current Vault passphrase, creates an encrypted backup of the Vault file, then replaces the Vault passphrase. Store the new passphrase safely; reVault cannot recover the Vault without it.",
                )),
        )
        .subcommand(
            Command::new("restore")
                .about("Restore the local vault from an encrypted backup archive.")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox vault restore ./vault-backup.lockbox-backup\n  lockbox vault restore --overwrite ./vault-backup.lockbox-backup",
                    "Context:\n  Restore verifies the backup checksum before replacing the local vault. Existing vaults are not overwritten unless --overwrite is passed.",
                ))
                .arg(
                    Arg::new("overwrite")
                        .long("overwrite")
                        .action(ArgAction::SetTrue)
                        .help("Replace the existing local vault."),
                )
                .arg(required("backup", "Backup archive input path.")),
        )
        .subcommand(vault_profile_command(verbose))
        .subcommand(
            Command::new("form")
                .about("Manage reusable form definitions.")
                .disable_help_subcommand(true)
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox vault form define login --field username:text --field password:secret\n  lockbox vault form list\n  lockbox secrets.lbox form use login",
                    "Context:\n  Vault form definitions are reusable templates stored in the local vault. Use form use to copy one into a lockbox before creating records that use it.",
                ))
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("define")
                        .about("Create or revise a reusable form definition.")
                .override_usage(
                    "lockbox vault form define [alias] --field <NAME[:KIND[:required[:LABEL]]]>...",
                )
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox vault form define login --field username:text --field password:secret\n  lockbox vault form define --name Login --description \"Website sign-in\" --field username:text:required:User --field password:secret:required:Password\n  lockbox vault form define login --name Login --description \"Website sign-in\" --field username:text:required:User --field password:secret:required:Password\n\nField form:\n  NAME[:KIND[:required[:LABEL]]]\n\nKinds:\n  text, secret, password, url, email, date, month, notes, number\n\nFormats:\n  date uses YYYY-MM-DD; month uses YYYY-MM",
                    "Context:\n  Define stores the reusable form definition in the vault. The alias is optional; when omitted, --name is required and an alias slug is derived from the display name. If the alias already resolves to one definition, define appends a new revision.",
                ))
                        .arg(optional("alias", "Form alias."))
                        .arg(
                            Arg::new("name")
                                .long("name")
                                .value_name("DISPLAY_NAME")
                                .help("Human display name for this form definition."),
                        )
                        .arg(
                            Arg::new("description")
                                .long("description")
                                .value_name("TEXT")
                                .help("Human description for this form definition."),
                        )
                        .arg(
                            Arg::new("definition-id")
                                .long("definition-id")
                                .alias("type-id")
                                .value_name("DEFINITION_ID")
                                .help("Revise or create this stable form definition id."),
                        )
                        .arg(
                            Arg::new("field")
                                .long("field")
                                .value_name("NAME[:KIND[:required[:LABEL]]]")
                                .action(ArgAction::Append)
                                .required(true)
                                .help("Add one field to the definition."),
                        ),
                )
                .subcommand(
                    Command::new("list")
                        .visible_alias("ls")
                        .about("List reusable form definitions.")
                        .arg(output_format_arg()),
                ),
        )
        .subcommand(
            Command::new("contact")
                .about("Manage contacts that can be given access to a lockbox.")
                .disable_help_subcommand(true)
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox vault contact list\n  lockbox vault contact receive <publish-code> alice\n  lockbox vault contact import alice ./alice.pub --fingerprint <fingerprint-code> --fingerprint-channel phone-call-to-owner\n  lockbox vault contact remove alice",
                    "Context:\n  Contacts are saved public keys for other people or systems. A contact can be added to a lockbox access list, but cannot open a lockbox by itself; opening requires the matching private profile.",
                ))
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("list")
                        .about("List saved contacts.")
                        .visible_alias("ls")
                        .after_help(verbose_help(
                            verbose,
                            "Examples:\n  lockbox vault contact list\n  lockbox vault contact list --format json",
                            "Context:\n  Contact list shows public keys you have saved for other profiles. Saved contacts have already passed fingerprint verification during import or receive; there is no separate trust-state that changes over time. Use these names with access grant when granting lockbox access.",
                        ))
                        .arg(output_format_arg()),
                )
                .subcommand(
                    Command::new("import")
                        .about("Import a contact public key after fingerprint verification.")
                        .after_help(verbose_help(
                            verbose,
                            "Examples:\n  lockbox vault contact import alice ./alice.pub --fingerprint <fingerprint-code> --fingerprint-channel phone-call-to-owner\n  lockbox vault contact import --overwrite alice ./alice-new.pub --fingerprint <fingerprint-code> --fingerprint-channel sms-to-owner",
                            "Context:\n  Contact import saves someone else's public key only after the 96-bit Crockford public-key fingerprint code matches. Ask the key owner for the code over a receiver-initiated second channel before importing the key. Email and owner-initiated messages are rejected.",
                        ))
                        .arg(
                            Arg::new("overwrite")
                                .long("overwrite")
                                .hide(!verbose)
                                .action(ArgAction::SetTrue)
                                .help("Replace an existing contact."),
                        )
                        .arg(
                            Arg::new("fingerprint")
                                .long("fingerprint")
                                .value_name("FINGERPRINT-CODE")
                                .help("96-bit Crockford public-key fingerprint code from the key owner. Prompts when omitted."),
                        )
                        .arg(
                            Arg::new("fingerprint-channel")
                                .long("fingerprint-channel")
                                .value_name("CHANNEL")
                                .help("How the fingerprint was received: phone-call-to-owner, sms-to-owner, or in-person. Prompts when omitted."),
                        )
                        .arg(required("name", "Contact name."))
                        .arg(required("public-key", "Public key path.")),
                )
                .subcommand(
                    Command::new("receive")
                        .about("Receive a published profile and save it as a contact.")
                        .after_help(verbose_help(
                            verbose,
                            "Examples:\n  lockbox vault contact receive <publish-code>\n  lockbox vault contact receive <publish-code> alice",
                            concat!(
                                "Context:\n  Receive saves the published public key and signing key as a local contact. ",
                                "The key server must have verified the publisher email first. Enter the ",
                                "96-bit Crockford fingerprint code by asking the publisher over a communications channel that you ",
                                "already trust. You must initiate the communication. If the publisher sends you ",
                                "the fingerprint before you ask, do not accept it. Short PINs are only ",
                                "accidental-error checks and are too small to authenticate a public key against ",
                                "substitution. Email, newly supplied channels, and owner-initiated messages are rejected.",
                            ),
                        ))
                        .arg(key_server_arg())
                        .arg(publish_topology_arg())
                        .arg(
                            Arg::new("fingerprint")
                                .long("fingerprint")
                                .value_name("FINGERPRINT-CODE")
                                .help("96-bit Crockford contact fingerprint code from a trusted second channel. Prompts when omitted."),
                        )
                        .arg(
                            Arg::new("fingerprint-channel")
                                .long("fingerprint-channel")
                                .value_name("CHANNEL")
                                .help("How the fingerprint was received: phone-call-to-owner, sms-to-owner, or in-person. Prompts when omitted."),
                        )
                        .arg(
                            Arg::new("overwrite")
                                .long("overwrite")
                                .action(ArgAction::SetTrue)
                                .help("Replace an existing contact."),
                        )
                        .arg(required("publish-code", "Publish code."))
                        .arg(optional("contact-name", "Contact name to save.")),
                )
                .subcommand(
                    Command::new("remove")
                        .visible_aliases(["rm", "delete"])
                        .about("Remove a contact.")
                        .after_help(verbose_help(
                            verbose,
                            "Examples:\n  lockbox vault contact remove alice",
                            "Context:\n  Contact remove deletes the saved public key from your vault. It does not remove access already written into any lockbox; use access revoke for that.",
                        ))
                        .arg(
                            required("name", "Contact name.")
                                .add(ArgValueCompleter::new(completion::contact_candidates)),
                        ),
                ),
        )
        .subcommand(
            Command::new("lockbox")
                .about("Manage lockboxes remembered by the vault.")
                .disable_help_subcommand(true)
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox vault lockbox list\n  lockbox vault lockbox remember ./existing.lbox\n  lockbox vault lockbox move ./old.lbox ./archive/new.lbox\n  lockbox vault lockbox forget ./old-project.lbox",
                    "Context:\n  The vault remembers lockboxes it has created, opened, or modified so bulk maintenance commands can find them later. Move coordinates the file, session cache, default path, lock sidecar, and vault record. Forget removes only the vault reference; it does not delete the lockbox file.",
                ))
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("list")
                        .about("List lockboxes remembered by the vault.")
                        .visible_alias("ls")
                        .after_help(verbose_help(
                            verbose,
                            "Examples:\n  lockbox vault lockbox list\n  lockbox vault lockbox list --with-description\n  lockbox vault lockbox list --format json",
                            "Context:\n  The lockbox list command reports remembered lockboxes, including file state, signed-owner status, compact file size, lockbox id, and path. --with-description attempts to open each lockbox and includes its encrypted description when available.",
                        ))
                        .arg(output_format_arg())
                        .arg(
                            Arg::new("with-description")
                                .long("with-description")
                                .action(ArgAction::SetTrue)
                                .help("Open each available lockbox and include its encrypted description."),
                        ),
                )
                .subcommand(
                    Command::new("remember")
                        .about("Remember an existing lockbox by its absolute path.")
                        .after_help(verbose_help(
                            verbose,
                            "Examples:\n  lockbox vault lockbox remember ./secrets.lbox",
                            "Context:\n  Remember validates the lockbox header, stores its canonical absolute path, and replaces a stale remembered path for the same lockbox id. It does not open or modify the lockbox.",
                        ))
                        .arg(required("lockbox", "Existing lockbox path to remember.")),
                )
                .subcommand(
                    Command::new("move")
                        .visible_aliases(["mv", "rename"])
                        .about("Move a lockbox and update its session and vault paths.")
                        .after_help(verbose_help(
                            verbose,
                            "Examples:\n  lockbox vault lockbox move ./secrets.lbox ./archive/\n  lockbox vault lockbox move ./secrets.lbox ./archive/renamed.lbox",
                            "Context:\n  Move closes the old cached path, moves the lockbox and hidden lock sidecar, updates the remembered vault path, and updates the session default when it points at the source. Source and destination must be on the same filesystem.",
                        ))
                        .arg(required("source", "Current lockbox path."))
                        .arg(required(
                            "destination",
                            "New lockbox path, or an existing destination directory.",
                        )),
                )
                .subcommand(
                    Command::new("forget")
                        .about("Forget one remembered lockbox path.")
                        .after_help(verbose_help(
                            verbose,
                            "Examples:\n  lockbox vault lockbox forget ./old-project.lbox",
                            "Context:\n  Forget removes a stale known-lockbox record from the vault. It does not delete the lockbox file.",
                        ))
                        .arg(required("lockbox", "Lockbox path to forget.")),
                ),
        )
}

fn key_server_arg() -> Arg {
    Arg::new("server")
        .long("server")
        .value_name("URL")
        .help("Key server /v1/publish URL or host.")
}

fn publish_topology_arg() -> Arg {
    Arg::new("topology-url")
        .long("topology-url")
        .value_name("URL")
        .help("Key server /v1/topology URL.")
}

fn vault_profile_command(verbose: bool) -> Command {
    Command::new("profile")
        .about("Manage your lockbox open profiles.")
        .disable_help_subcommand(true)
        .after_help(verbose_help(
            verbose,
            "Examples:\n  lockbox vault profile list\n  lockbox vault profile create laptop\n  lockbox vault profile publish laptop\n  lockbox vault profile fingerprint laptop\n  lockbox vault profile backup ./default.profile-backup",
            "Context:\n  A profile has a public key, private open key, and owner signing key. Publish or export the public key so someone else can grant you access to a lockbox. Use profile backup and restore for emergency recovery of one profile.",
        ))
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("list")
                .about("List local profiles.")
                .visible_alias("ls")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox vault profile list\n  lockbox vault profile list --format json",
                    "Context:\n  Profile list shows the private open profiles stored in your vault. These are the profiles reVault can use when opening lockboxes granted to you.",
                ))
                .arg(output_format_arg()),
        )
        .subcommand(
            Command::new("create")
                .about("Create one of your profiles.")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox vault profile create\n  lockbox vault profile create laptop\n  lockbox vault profile export ./laptop.pub --name laptop",
                    "Context:\n  Profile create generates a new profile in your vault. With no name, reVault creates the `default` profile. To publish the profile, create it first and then run `lockbox vault profile publish` or `lockbox vault profile export <path>`.",
                ))
                .arg(
                    Arg::new("overwrite")
                        .long("overwrite")
                        .hide(!verbose)
                        .action(ArgAction::SetTrue)
                        .help("Replace an existing profile."),
                )
                .arg(optional("name", "Profile name."))
        )
        .subcommand(
            Command::new("history")
                .about("Show Profile key generations.")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox vault profile history\n  lockbox vault profile history laptop --format json",
                    "Context:\n  Profile history shows the active and retired key generations for one vault profile. Retired generations are retained so older lockboxes can still be opened until their access entries are refreshed.",
                ))
                .arg(output_format_arg())
                .arg(
                    optional("name", "Profile name.")
                        .add(ArgValueCompleter::new(completion::profile_candidates)),
                ),
        )
        .subcommand(
            Command::new("email")
                .about("Set the email address associated with a profile.")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox vault profile email alice@example.com\n  lockbox vault profile email laptop alice@example.com",
                    "Context:\n  Publish requires a profile email address. The key server sends a verification link to this address before receivers can receive the public key by email.",
                ))
                .arg(
                    Arg::new("args")
                        .value_names(["profile", "email"])
                        .num_args(1..=2)
                        .required(true)
                        .help("Optional profile name followed by the profile email address."),
                ),
        )
        .subcommand(
            Command::new("fingerprint")
                .about("Show the publish fingerprint for one profile.")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox vault profile fingerprint\n  lockbox vault profile fingerprint laptop",
                    "Context:\n  Fingerprint prints the same 96-bit Crockford contact fingerprint code shown by publish. The receiver must ask you for this code through a trusted second channel before saving the contact.",
                ))
                .arg(
                    optional("name", "Profile name. Defaults to default.")
                        .add(ArgValueCompleter::new(completion::profile_candidates)),
                ),
        )
        .subcommand(
            Command::new("publish")
                .about("Publish one profile public key by verified email.")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox vault profile publish\n  lockbox vault profile publish laptop",
                    "Context:\n  Publish sends one profile public key to the key server and prints a 96-bit Crockford fingerprint code. The receiver must ask you for that code through a trusted second channel before saving the contact.",
                ))
                .arg(key_server_arg())
                .arg(publish_topology_arg())
                .arg(
                    Arg::new("ttl")
                        .long("ttl")
                        .value_name("SECONDS")
                        .help("Publish lifetime in seconds."),
                )
                .arg(
                    Arg::new("max-receives")
                        .long("max-receives")
                        .value_name("N")
                        .help("Maximum successful receives."),
                )
                .arg(
                    optional("name", "Profile name. Defaults to default.")
                        .add(ArgValueCompleter::new(completion::profile_candidates)),
                ),
        )
        .subcommand(
            Command::new("backup")
                .about("Back up one profile to a text recovery file.")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox vault profile backup ./default.profile-backup\n  lockbox vault profile backup ./laptop.profile-backup --name laptop",
                    "Context:\n  Profile backup writes the same text recovery block printed by vault init. It contains the profile name, fingerprint, profile private key, and owner signing private key.",
                ))
                .arg(
                    Arg::new("overwrite")
                        .long("overwrite")
                        .action(ArgAction::SetTrue)
                        .help("Replace an existing backup file."),
                )
                .arg(
                    Arg::new("name")
                        .long("name")
                        .value_name("PROFILE")
                        .help("Profile name. Defaults to default.")
                        .add(ArgValueCompleter::new(completion::profile_candidates)),
                )
                .arg(required("output", "Profile backup output path.")),
        )
        .subcommand(
            Command::new("restore")
                .about("Restore one profile from a text recovery file.")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox vault profile restore ./default.profile-backup\n  lockbox vault profile restore ./default.profile-backup --name laptop --overwrite",
                    "Context:\n  Profile restore reads one profile backup text file, derives the public key from the private key, and restores the required owner signing key. If the profile already exists, use --overwrite; reVault backs up the current vault before replacing it.",
                ))
                .arg(
                    Arg::new("overwrite")
                        .long("overwrite")
                        .action(ArgAction::SetTrue)
                        .help("Replace an existing profile after backing up the current vault."),
                )
                .arg(
                    Arg::new("name")
                        .long("name")
                        .value_name("PROFILE")
                        .help("Restore to this profile name instead of the name in the backup."),
                )
                .arg(required("input", "Profile backup input path.")),
        )
        .subcommand(
            Command::new("export")
                .about("Export one profile public key.")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox vault profile export ./default.pub\n  lockbox vault profile export ./laptop.pub --name laptop",
                    "Context:\n  Profile export writes the public key for sharing with someone who needs to grant you access to a lockbox. Use profile backup, not export, for private recovery material.",
                ))
                .arg(format_arg(verbose))
                .arg(
                    Arg::new("name")
                        .long("name")
                        .value_name("PROFILE")
                        .help("Profile name. Defaults to default.")
                        .add(ArgValueCompleter::new(completion::profile_candidates)),
                )
                .arg(required("output", "Public key output path.")),
        )
        .subcommand(
            Command::new("remove")
                .visible_aliases(["rm", "delete"])
                .about("Remove a profile.")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox vault profile remove laptop\n  lockbox vault profile remove --force laptop",
                    "Context:\n  Profile remove deletes a profile from your vault. Lockboxes that only grant access to that profile may become inaccessible from this vault.",
                ))
                .arg(
                    Arg::new("force")
                        .long("force")
                        .action(ArgAction::SetTrue)
                        .help("Remove the key without an interactive confirmation."),
                )
                .arg(
                    optional("name", "Profile name.")
                        .add(ArgValueCompleter::new(completion::profile_candidates)),
                ),
        )
        .subcommand(
            Command::new("rotate")
                .about("Rotate a profile to a new key generation.")
                .after_help(verbose_help(
                    verbose,
                    "Examples:\n  lockbox vault profile rotate\n  lockbox vault profile rotate laptop",
                    "Context:\n  Profile rotate creates a new active private key generation and retires the previous active generation. Refresh remembered lockboxes afterward so they grant access to the new key.",
                ))
                .arg(
                    optional("name", "Profile name.")
                        .add(ArgValueCompleter::new(completion::profile_candidates)),
                ),
        )
}

fn format_arg(verbose: bool) -> Arg {
    Arg::new("format")
        .long("format")
        .hide(!verbose)
        .value_name("lockbox-pem|jwk|jwks|raw-hex")
        .help("Select the key file format.")
}

fn output_format_arg() -> Arg {
    Arg::new("format")
        .long("format")
        .value_name("table|tsv|json")
        .default_value("table")
        .help("Output format.")
}

fn required(name: &'static str, help: &'static str) -> Arg {
    dynamic_completion_arg(Arg::new(name), name)
        .value_name(name)
        .required(true)
        .help(help)
}

fn optional(name: &'static str, help: &'static str) -> Arg {
    dynamic_completion_arg(Arg::new(name), name)
        .value_name(name)
        .required(false)
        .help(help)
}

fn dynamic_completion_arg(arg: Arg, name: &str) -> Arg {
    match name {
        "form" => arg.add(ArgValueCompleter::new(completion::form_candidates)),
        "lockbox" => arg.add(ArgValueCompleter::new(completion::lockbox_path_candidates)),
        "private-key" | "public-key" | "output" | "input" | "backup" | "artifact" | "source"
        | "destination" => arg.value_hint(ValueHint::AnyPath),
        _ => arg,
    }
}

fn completion_command() -> Command {
    Command::new("completion")
        .about("Generate, install, or remove dynamic shell completion.")
        .disable_help_subcommand(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommands([
            Command::new("generate")
                .about("Write a completion registration script to stdout or a file.")
                .arg(completion_shell_arg())
                .arg(
                    Arg::new("output")
                        .long("output")
                        .short('o')
                        .value_name("FILE")
                        .value_hint(ValueHint::AnyPath)
                        .help("Write the script to this file."),
                ),
            Command::new("install")
                .about("Install completion in a standard per-user completion directory.")
                .arg(completion_shell_arg())
                .arg(
                    Arg::new("path")
                        .long("path")
                        .value_name("FILE")
                        .value_hint(ValueHint::AnyPath)
                        .help("Override the standard per-user installation path."),
                ),
            Command::new("uninstall")
                .about("Remove a completion installed by revault.")
                .arg(completion_shell_arg())
                .arg(
                    Arg::new("path")
                        .long("path")
                        .value_name("FILE")
                        .value_hint(ValueHint::AnyPath)
                        .help("Override the standard per-user installation path."),
                ),
        ])
}

fn migration_command(verbose: bool) -> Command {
    Command::new("migrate")
        .about("Migrate vaults and lockboxes between native format versions.")
        .disable_help_subcommand(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommands([
            migration_vault_command(verbose),
            migration_lockbox_command(verbose),
        ])
}

fn migration_vault_command(verbose: bool) -> Command {
    Command::new("vault")
        .about("Migrate the configured vault to the latest format.")
        .args_conflicts_with_subcommands(true)
        .arg(migration_output_arg())
        .arg(migration_replace_arg())
        .arg(migration_exporter_arg())
        .subcommands([
            Command::new("export")
                .about("Export the configured vault to a migration artifact.")
                .hide(!verbose)
                .arg(migration_output_arg().required(true))
                .arg(hidden_secret_stdin_arg("vault-password-stdin"))
                .arg(hidden_secret_stdin_arg("migration-password-stdin")),
            Command::new("upgrade")
                .about("Upgrade a vault migration artifact to the latest schema.")
                .hide(!verbose)
                .arg(required("artifact", "Input migration artifact."))
                .arg(migration_output_arg().required(true)),
            Command::new("import")
                .about("Import a vault migration artifact into a new vault.")
                .hide(!verbose)
                .arg(required("artifact", "Input migration artifact."))
                .arg(migration_output_arg().required(true)),
            Command::new("verify")
                .about("Verify a vault migration artifact.")
                .hide(!verbose)
                .arg(required("artifact", "Migration artifact to verify.")),
        ])
}

fn migration_lockbox_command(verbose: bool) -> Command {
    Command::new("lockbox")
        .about("Migrate a lockbox to the latest format.")
        .args_conflicts_with_subcommands(true)
        .arg(optional("lockbox", "Lockbox to migrate."))
        .arg(migration_output_arg())
        .arg(migration_replace_arg())
        .arg(migration_exporter_arg())
        .subcommands([
            Command::new("export")
                .about("Export a lockbox to a migration artifact.")
                .hide(!verbose)
                .arg(required("lockbox", "Lockbox to export."))
                .arg(migration_output_arg().required(true))
                .arg(hidden_secret_stdin_arg("migration-password-stdin")),
            Command::new("upgrade")
                .about("Upgrade a lockbox migration artifact to the latest schema.")
                .hide(!verbose)
                .arg(required("artifact", "Input migration artifact."))
                .arg(migration_output_arg().required(true)),
            Command::new("import")
                .about("Import a lockbox migration artifact.")
                .hide(!verbose)
                .arg(required("artifact", "Input migration artifact."))
                .arg(migration_output_arg().required(true)),
            Command::new("verify")
                .about("Verify a lockbox migration artifact.")
                .hide(!verbose)
                .arg(required("artifact", "Migration artifact to verify.")),
        ])
}

fn migration_output_arg() -> Arg {
    Arg::new("output")
        .long("output")
        .short('o')
        .value_name("PATH")
        .value_hint(ValueHint::AnyPath)
        .help("Write the migrated artifact to this path.")
}

fn migration_replace_arg() -> Arg {
    Arg::new("replace")
        .long("replace")
        .action(ArgAction::SetTrue)
        .conflicts_with("output")
        .help("Replace the source after retaining a versioned backup.")
}

fn migration_exporter_arg() -> Arg {
    Arg::new("exporter")
        .long("exporter")
        .value_name("PATH")
        .value_hint(ValueHint::FilePath)
        .hide(true)
        .help("Use this historical reVault exporter executable.")
}

fn hidden_secret_stdin_arg(name: &'static str) -> Arg {
    Arg::new(name)
        .long(name)
        .hide(true)
        .action(ArgAction::SetTrue)
}

fn completion_shell_arg() -> Arg {
    Arg::new("shell")
        .long("shell")
        .value_name("bash|zsh|fish|powershell|elvish")
        .help("Shell override; otherwise detect SHELL or PowerShell.")
}

fn verbose_help(verbose: bool, normal: &'static str, context: &'static str) -> String {
    if verbose {
        format!("{context}\n\n{normal}")
    } else {
        normal.to_string()
    }
}

fn apply_verbose_help_template(mut command: Command) -> Command {
    if let Some(after_help) = command.get_after_help().map(|help| help.to_string()) {
        if let Some((context, examples)) = after_help.split_once("\n\nExamples:") {
            if context.starts_with("Context:") {
                command = command
                    .before_help(context.to_string())
                    .after_help(format!("Examples:{examples}"));
            }
        }
    }
    command
        .help_template(VERBOSE_HELP_TEMPLATE)
        .mut_subcommands(apply_verbose_help_template)
}

#[cfg(test)]
mod migration_inventory_tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn migration_command_and_option_inventory_is_explicit() {
        let command = command(true);
        let migration = command_at(&command, "doctor/migrate");
        let mut actual = BTreeMap::new();
        collect(migration, "doctor/migrate", &mut actual);
        let expected = BTreeMap::from([
            (
                "doctor/migrate/lockbox".to_string(),
                strings(&["exporter", "lockbox", "output", "replace"]),
            ),
            (
                "doctor/migrate/lockbox/export".to_string(),
                strings(&["lockbox", "migration-password-stdin", "output"]),
            ),
            (
                "doctor/migrate/lockbox/import".to_string(),
                strings(&["artifact", "output"]),
            ),
            (
                "doctor/migrate/lockbox/upgrade".to_string(),
                strings(&["artifact", "output"]),
            ),
            (
                "doctor/migrate/lockbox/verify".to_string(),
                strings(&["artifact"]),
            ),
            (
                "doctor/migrate/vault".to_string(),
                strings(&["exporter", "output", "replace"]),
            ),
            (
                "doctor/migrate/vault/export".to_string(),
                strings(&["migration-password-stdin", "output", "vault-password-stdin"]),
            ),
            (
                "doctor/migrate/vault/import".to_string(),
                strings(&["artifact", "output"]),
            ),
            (
                "doctor/migrate/vault/upgrade".to_string(),
                strings(&["artifact", "output"]),
            ),
            (
                "doctor/migrate/vault/verify".to_string(),
                strings(&["artifact"]),
            ),
        ]);
        assert_eq!(actual, expected);
    }

    #[test]
    fn conventional_command_aliases_are_consistent() {
        let command = command(false);
        let expected = [
            ("list", &["ls"][..]),
            ("remove", &["delete", "rm"][..]),
            ("move", &["mv", "rename"][..]),
            ("mirror/list", &["ls"][..]),
            ("mirror/remove", &["delete", "rm"][..]),
            ("mirror/move", &["mv", "rename"][..]),
            ("mirror/destroy", &["delete-project"][..]),
            ("mirror/rule/list", &["ls"][..]),
            ("mirror/rule/remove", &["delete", "rm"][..]),
            ("variable/list", &["ls"][..]),
            ("variable/remove", &["delete", "rm"][..]),
            ("variable/move", &["mv", "rename"][..]),
            ("form/list", &["ls"][..]),
            ("form/remove", &["delete", "rm"][..]),
            ("form/move", &["mv", "rename"][..]),
            ("access/list", &["ls"][..]),
            ("vault/form/list", &["ls"][..]),
            ("vault/profile/list", &["ls"][..]),
            ("vault/profile/remove", &["delete", "rm"][..]),
            ("vault/contact/list", &["ls"][..]),
            ("vault/contact/remove", &["delete", "rm"][..]),
            ("vault/lockbox/list", &["ls"][..]),
            ("vault/lockbox/move", &["mv", "rename"][..]),
        ];
        for (path, aliases) in expected {
            let command = command_at(&command, path);
            let mut actual = command.get_visible_aliases().collect::<Vec<_>>();
            actual.sort_unstable();
            assert_eq!(actual, aliases, "aliases for {path}");
        }
    }

    #[test]
    fn required_positionals_are_enforced_by_clap() {
        let cases = [
            vec!["lockbox", "cat"],
            vec!["lockbox", "remove"],
            vec!["lockbox", "move"],
            vec!["lockbox", "variable", "set"],
            vec!["lockbox", "variable", "get"],
            vec!["lockbox", "variable", "remove"],
            vec!["lockbox", "form", "capture"],
            vec!["lockbox", "form", "add", "--type", "login"],
            vec!["lockbox", "form", "edit"],
            vec!["lockbox", "form", "set"],
            vec!["lockbox", "form", "get"],
            vec!["lockbox", "form", "show"],
            vec!["lockbox", "form", "remove"],
            vec!["lockbox", "access", "grant"],
            vec!["lockbox", "access", "revoke"],
        ];
        for args in cases {
            let error = command(false)
                .try_get_matches_from(&args)
                .expect_err(&format!("{args:?} should require a positional argument"));
            assert_eq!(
                error.kind(),
                clap::error::ErrorKind::MissingRequiredArgument,
                "wrong error for {args:?}: {error}"
            );
        }
    }

    #[test]
    fn batch_commands_accept_multiple_values_and_singletons_reject_them() {
        for (args, path, expected) in [
            (
                vec!["lockbox", "secrets.lbox", "cat", "one", "two"],
                "cat",
                vec!["one", "two"],
            ),
            (
                vec![
                    "lockbox",
                    "secrets.lbox",
                    "variable",
                    "remove",
                    "one",
                    "two",
                ],
                "variable/remove",
                vec!["one", "two"],
            ),
            (
                vec!["lockbox", "secrets.lbox", "form", "remove", "/one", "/two"],
                "form/remove",
                vec!["/one", "/two"],
            ),
        ] {
            let matches = command(false).try_get_matches_from(args).unwrap();
            let values = matches_at(&matches, path)
                .get_many::<String>("args")
                .unwrap()
                .map(String::as_str)
                .collect::<Vec<_>>();
            assert_eq!(values, expected, "batch values for {path}");
        }

        for args in [
            vec!["lockbox", "variable", "get", "one", "two"],
            vec!["lockbox", "form", "add", "--type", "login", "/one", "/two"],
            vec!["lockbox", "form", "show", "/one", "/two"],
        ] {
            assert!(
                command(false).try_get_matches_from(&args).is_err(),
                "{args:?} should reject surplus operands"
            );
        }
    }

    #[test]
    fn create_usage_requires_an_explicit_lockbox_name() {
        let command = command(false);
        let mut create = command_at(&command, "create").clone();
        let help = create.render_help().to_string();
        assert!(help.contains("Usage: lockbox <LOCKBOX> create [OPTIONS]"));
    }

    #[test]
    fn secret_options_consistently_support_short_s() {
        let command = command(false);
        for path in ["variable/set", "variable/get", "form/set", "form/get"] {
            let secret = command_at(&command, path)
                .get_arguments()
                .find(|argument| argument.get_id() == "secret")
                .unwrap_or_else(|| panic!("missing --secret on {path}"));
            assert_eq!(
                secret.get_short(),
                Some('s'),
                "short secret option for {path}"
            );
        }
    }

    #[test]
    fn direct_migration_options_conflict_with_nested_commands() {
        for args in [
            vec![
                "lockbox",
                "--verbose",
                "doctor",
                "migrate",
                "vault",
                "--replace",
                "verify",
                "artifact",
            ],
            vec![
                "lockbox",
                "--verbose",
                "doctor",
                "migrate",
                "lockbox",
                "source.lbox",
                "verify",
                "artifact",
            ],
        ] {
            let error = command(true).try_get_matches_from(&args).unwrap_err();
            assert_eq!(error.kind(), clap::error::ErrorKind::ArgumentConflict);
        }
    }

    fn command_at<'a>(root: &'a Command, path: &str) -> &'a Command {
        path.split('/').fold(root, |command, name| {
            command
                .get_subcommands()
                .find(|child| child.get_name() == name)
                .unwrap_or_else(|| panic!("missing command {path}"))
        })
    }

    fn matches_at<'a>(root: &'a clap::ArgMatches, path: &str) -> &'a clap::ArgMatches {
        path.split('/').fold(root, |matches, name| {
            matches
                .subcommand_matches(name)
                .unwrap_or_else(|| panic!("missing command matches for {path}"))
        })
    }

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    fn collect(command: &Command, path: &str, output: &mut BTreeMap<String, Vec<String>>) {
        if path != "doctor/migrate" {
            let mut arguments = command
                .get_arguments()
                .map(|argument| argument.get_id().as_str().to_string())
                .filter(|id| id != "help")
                .collect::<Vec<_>>();
            arguments.sort_unstable();
            output.insert(path.to_string(), arguments);
        }
        for child in command.get_subcommands() {
            collect(child, &format!("{path}/{}", child.get_name()), output);
        }
    }

    #[test]
    #[ignore = "developer inventory aid; coverage is enforced by the E2E contract suite"]
    fn print_complete_command_inventory() {
        fn print_leaves(command: &Command, path: &str) {
            if command.get_subcommands().next().is_none() {
                let mut arguments = command
                    .get_arguments()
                    .filter(|argument| {
                        argument.get_id() != "help"
                            && (argument.get_long().is_some() || argument.get_short().is_some())
                    })
                    .map(|argument| argument.get_id().as_str().to_string())
                    .collect::<Vec<_>>();
                arguments.sort_unstable();
                println!("{path}\t{}", arguments.join(","));
            }
            for child in command.get_subcommands() {
                let child_path = if path.is_empty() {
                    child.get_name().to_string()
                } else {
                    format!("{path}/{}", child.get_name())
                };
                print_leaves(child, &child_path);
            }
        }

        print_leaves(&command(true), "");
    }
}
