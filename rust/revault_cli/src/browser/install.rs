use revault_browser_protocol::Error;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::PathBuf,
};

pub const HOST: &str = "dev.revault.browser";
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hosts {
    pub chrome: String,
    pub firefox: String,
}
fn home() -> Result<PathBuf, Error> {
    std::env::var_os("REVAULT_BROWSER_INSTALL_ROOT")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .ok_or(Error::Internal)
}
fn locations() -> Result<Vec<(PathBuf, bool)>, Error> {
    let home = home()?;
    Ok(vec![
        (
            home.join(".config/google-chrome/NativeMessagingHosts"),
            true,
        ),
        (home.join(".config/chromium/NativeMessagingHosts"), true),
        (home.join(".mozilla/native-messaging-hosts"), false),
    ])
}
pub fn install(chrome: &str, firefox: &str) -> Result<(), Error> {
    if chrome.len() != 32
        || !chrome.bytes().all(|b| (b'a'..=b'p').contains(&b))
        || !revault_browser_protocol::validation::identifier(firefox)
    {
        return Err(Error::InvalidRequest);
    }
    let binary = std::env::current_exe()
        .and_then(fs::canonicalize)
        .map_err(|_| Error::Internal)?;
    for (dir, chromium) in locations()? {
        fs::create_dir_all(&dir).map_err(|_| Error::Internal)?;
        let mut manifest = serde_json::json!({"name": HOST, "description": "reVault browser authorisation", "path": binary, "type": "stdio"});
        if chromium {
            manifest["allowed_origins"] =
                serde_json::json!([format!("chrome-extension://{chrome}/")]);
        } else {
            manifest["allowed_extensions"] = serde_json::json!([firefox]);
        }
        write(
            &dir.join(format!("{HOST}.json")),
            &serde_json::to_vec_pretty(&manifest).map_err(|_| Error::Internal)?,
        )?;
    }
    let root = revault_vault_api::browser::state_directory()?;
    fs::create_dir_all(&root).map_err(|_| Error::Internal)?;
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).map_err(|_| Error::Internal)?;
    write(
        &root.join("hosts.state"),
        &serde_json::to_vec(&Hosts {
            chrome: chrome.into(),
            firefox: firefox.into(),
        })
        .map_err(|_| Error::Internal)?,
    )
}
fn write(path: &std::path::Path, bytes: &[u8]) -> Result<(), Error> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|_| Error::Internal)?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| Error::Internal)
}
pub fn uninstall() -> Result<(), Error> {
    for (dir, _) in locations()? {
        match fs::remove_file(dir.join(format!("{HOST}.json"))) {
            Ok(()) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(_) => return Err(Error::Internal),
        }
    }
    for (id, _) in revault_vault_api::browser::list_pairings()? {
        revault_vault_api::browser::revoke(&id)?;
    }
    match fs::remove_file(revault_vault_api::browser::state_directory()?.join("hosts.state")) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(Error::Internal),
    }
}
pub fn caller(args: &[String]) -> Result<String, Error> {
    let bytes = fs::read(revault_vault_api::browser::state_directory()?.join("hosts.state"))
        .map_err(|_| Error::UnpairedRecipient)?;
    if bytes.len() > 1024 {
        return Err(Error::UnpairedRecipient);
    }
    let hosts: Hosts = serde_json::from_slice(&bytes).map_err(|_| Error::UnpairedRecipient)?;
    if args.len() == 1 && args[0] == format!("chrome-extension://{}/", hosts.chrome) {
        return Ok(hosts.chrome);
    }
    if args.len() == 2 && args[1] == hosts.firefox {
        let expected = home()?.join(format!(".mozilla/native-messaging-hosts/{HOST}.json"));
        if std::path::Path::new(&args[0]) == expected {
            return Ok(hosts.firefox);
        }
    }
    Err(Error::UnpairedRecipient)
}

pub fn inspect() -> Result<(), Error> {
    let mut hosts = Vec::new();
    for (dir, _) in locations()? {
        let path = dir.join(format!("{HOST}.json"));
        match fs::read(&path) {
            Ok(bytes) if bytes.len() <= 4096 => hosts.push(serde_json::json!({"path": path, "manifest": serde_json::from_slice::<serde_json::Value>(&bytes).map_err(|_| Error::Internal)?})),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            _ => return Err(Error::Internal),
        }
    }
    println!(
        "{}",
        serde_json::to_string(&hosts).map_err(|_| Error::Internal)?
    );
    Ok(())
}
