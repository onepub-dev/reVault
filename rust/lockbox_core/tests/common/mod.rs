#![allow(dead_code)]

use lockbox_core::{
    Lockbox, LockboxPath, OwnerSigningKeyPair, SecretString, VariableName, WritableLockboxState,
};
use std::io::Read;
use std::path::{Path, PathBuf};

pub fn p(path: impl AsRef<str>) -> LockboxPath {
    LockboxPath::new(path).unwrap()
}

pub fn variable(name: impl AsRef<str>) -> VariableName {
    VariableName::new(name).unwrap()
}

pub fn signing_key() -> OwnerSigningKeyPair {
    OwnerSigningKeyPair::generate().unwrap()
}

pub fn password(value: &str) -> SecretString {
    SecretString::try_from_bytes(value.as_bytes().to_vec()).unwrap()
}

pub fn add_file<State>(
    lb: &mut Lockbox<State>,
    path: &LockboxPath,
    data: &[u8],
    replace: bool,
) -> lockbox_core::Result<()>
where
    State: WritableLockboxState,
{
    lb.create_parent_dirs_for(path)?;
    Lockbox::add_file(lb, path, data, replace)
}

pub fn add_file_with_permissions<State>(
    lb: &mut Lockbox<State>,
    path: &LockboxPath,
    data: &[u8],
    permissions: u32,
    replace: bool,
) -> lockbox_core::Result<()>
where
    State: WritableLockboxState,
{
    lb.create_parent_dirs_for(path)?;
    Lockbox::add_file_with_permissions(lb, path, data, permissions, replace)
}

pub fn add_file_from_reader<State>(
    lb: &mut Lockbox<State>,
    path: &LockboxPath,
    reader: impl Read,
    replace: bool,
) -> lockbox_core::Result<()>
where
    State: WritableLockboxState,
{
    lb.create_parent_dirs_for(path)?;
    Lockbox::add_file_from_reader(lb, path, reader, replace)
}

pub fn add_file_from_path<State>(
    lb: &mut Lockbox<State>,
    source: &Path,
    destination: &LockboxPath,
    replace: bool,
) -> lockbox_core::Result<()>
where
    State: WritableLockboxState,
{
    lb.create_parent_dirs_for(destination)?;
    Lockbox::add_file_from_path(lb, source, destination, replace)
}

pub fn add_symlink<State>(
    lb: &mut Lockbox<State>,
    path: &LockboxPath,
    target: &LockboxPath,
    replace: bool,
) -> lockbox_core::Result<()>
where
    State: WritableLockboxState,
{
    lb.create_parent_dirs_for(path)?;
    Lockbox::add_symlink(lb, path, target, replace)
}

pub fn public_api_unique_dir(label: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../target/test-tmp")
        .join(format!(
            "lockbox-core-public-api-{label}-{}-{}",
            std::process::id(),
            monotonic_suffix()
        ))
}

fn monotonic_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}
