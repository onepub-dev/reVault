#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::undocumented_unsafe_blocks)]

#[path = "../commands/mod.rs"]
mod commands;
#[path = "../secret_prompt.rs"]
mod secret_prompt;

fn main() {
    if let Err(err) = commands::run() {
        let exit_code = commands::exit_code(err.as_ref());
        if let Some(err) = err.downcast_ref::<clap::Error>() {
            if let Err(print_err) = err.print() {
                eprintln!("error: {print_err}");
            }
        } else if let Err(print_err) = commands::print_error(err.as_ref()) {
            eprintln!("Error: {err}\nCould not format the error: {print_err}");
        }
        std::process::exit(exit_code);
    }
}
