//! Linux Secret Service operations with one deadline covering connection,
//! negotiation, methods and any service-requested prompt. No detached worker
//! thread retains credentials after the caller times out.
use super::{Error, Result, SecretString, SERVICE};
use secret_service::{EncryptionType, Item, SecretService};
use std::{collections::HashMap, future::Future, time::Duration};

const DEADLINE: Duration = Duration::from_secs(5);
fn unavailable(message: impl std::fmt::Display) -> Error {
    Error::VaultUnavailable(format!(
        "platform credential store is unavailable in this user session: {message}; \
         unlock the desktop credential store and retry, or use explicit Vault credentials"
    ))
}
fn bounded<T>(operation: impl Future<Output = Result<T>>) -> Result<T> {
    async_io::block_on(futures_lite::future::or(operation, async {
        async_io::Timer::after(DEADLINE).await;
        Err(unavailable(
            "Secret Service operation timed out after 5 seconds",
        ))
    }))
}
async fn connect(address: Option<&str>) -> Result<SecretService<'static>> {
    let builder = match address {
        Some(address) => zbus::connection::Builder::address(address),
        None => zbus::connection::Builder::session(),
    }
    .map_err(unavailable)?;
    let connection = builder
        .method_timeout(DEADLINE)
        .build()
        .await
        .map_err(unavailable)?;
    SecretService::connect_with_existing(EncryptionType::Dh, connection)
        .await
        .map_err(unavailable)
}
fn attributes(item: &str) -> HashMap<&str, &str> {
    // Preserve the attributes used by existing keyring credentials.
    HashMap::from([("service", SERVICE), ("username", item)])
}
async fn selected<'a>(service: &'a SecretService<'_>, item: &str) -> Result<Option<Item<'a>>> {
    let mut found = service
        .search_items(attributes(item))
        .await
        .map_err(unavailable)?;
    if found.unlocked.len() + found.locked.len() > 1 {
        return Err(unavailable("multiple matching Vault credentials"));
    }
    if !found.locked.is_empty() {
        // Auto-open must not initiate an unattended desktop unlock prompt.
        return Err(unavailable("the matching Vault credential is locked"));
    }
    let selected = found.unlocked.pop();
    if let Some(selected) = &selected {
        selected.ensure_unlocked().await.map_err(unavailable)?;
    }
    Ok(selected)
}
pub(super) fn get(item: &str, address: Option<&str>) -> Result<Option<SecretString>> {
    bounded(async {
        let service = connect(address).await?;
        match selected(&service, item).await? {
            None => Ok(None),
            Some(selected) => {
                let secret = selected.get_secret().await.map_err(unavailable)?;
                SecretString::try_from_utf8(secret)
                    .map(Some)
                    .map_err(|error| Error::InvalidKeyMaterial(error.to_string()))
            }
        }
    })
}
pub(super) fn put(item: &str, secret: &[u8]) -> Result<()> {
    bounded(async {
        let service = connect(None).await?;
        if let Some(selected) = selected(&service, item).await? {
            selected
                .set_secret(secret, "application/octet-stream")
                .await
                .map_err(unavailable)?;
        } else {
            let collection = service
                .get_default_collection()
                .await
                .map_err(unavailable)?;
            collection.ensure_unlocked().await.map_err(unavailable)?;
            collection
                .create_item(
                    &format!("keyring:{item}@{SERVICE}"),
                    attributes(item),
                    secret,
                    true,
                    "application/octet-stream",
                )
                .await
                .map_err(unavailable)?;
        }
        Ok(())
    })
}
pub(super) fn forget(item: &str) -> Result<()> {
    bounded(async {
        let service = connect(None).await?;
        if let Some(selected) = selected(&service, item).await? {
            selected.delete().await.map_err(unavailable)?;
        }
        Ok(())
    })
}
