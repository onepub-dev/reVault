#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::undocumented_unsafe_blocks)]
#[cfg(target_os = "linux")]
mod install;
#[cfg(target_os = "linux")]
mod native;

#[cfg(target_os = "linux")]
fn main() {
    use revault_browser_protocol::Error;
    let args: Vec<String> = std::env::args().skip(1).collect();
    // Browsers launch the host with their identity, not a webpage command.
    if args
        .first()
        .is_some_and(|s| s.starts_with("chrome-extension://") || s.ends_with(".json"))
    {
        if native::run(&args).is_err() {
            std::process::exit(1);
        }
        return;
    }
    let result = match args.as_slice() {
        [op, chrome, firefox] if op == "install" => install::install(chrome, firefox),
        [op] if op == "uninstall" => install::uninstall(),
        [op] if op == "hosts" => install::inspect(),
        [op, file] if op == "pair" => (|| {
            let bytes = std::fs::read(file).map_err(|_| Error::InvalidRequest)?;
            if bytes.len() > 8192 {
                return Err(Error::InvalidRequest);
            }
            let pair = serde_json::from_slice(&bytes).map_err(|_| Error::InvalidRequest)?;
            println!("{}", revault_vault_api::browser::pair(pair)?);
            Ok(())
        })(),
        [op, id] if op == "revoke" => revault_vault_api::browser::revoke(id),
        [op] if op == "list" => revault_vault_api::browser::list_pairings().map(|pairs| {
            for (id, pair) in pairs {
                println!(
                    "{id} {} {} {} {}",
                    pair.origin, pair.application_id, pair.lockbox_id, pair.extension_id
                );
            }
        }),
        _ => {
            eprintln!("Experimental: synthetic lockboxes only pending security review.\nUsage: revault-browser install <chromium-extension-id> <firefox-extension-id>\n       revault-browser pair <local-pairing.json>\n       revault-browser list | revoke <pairing-id> | uninstall");
            Err(Error::InvalidRequest)
        }
    };
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("upgrade_required: browser integration currently supports Linux");
    std::process::exit(1);
}
