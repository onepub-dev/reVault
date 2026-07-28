use super::Lockbox;
use crate::{Error, LockboxPath, Result, VariableName, WritableLockboxState};
use serde_json::{json, Value};

const MIRROR_PREFIX: &str = "/.revault/mirrors/";

/// Controls what an update does with files that exist only in a mirror's
/// managed lockbox directory.
///
/// This policy is stored with the project. It applies both to files removed
/// from the host and files no longer selected by the project's rules.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum MirrorMissingFilePolicy {
    /// Remove entries that are missing or excluded from the host source.
    #[default]
    Remove,
    /// Retain entries that are missing or excluded from the host source.
    Retain,
}

/// Persistent configuration for one host-to-lockbox mirror project.
///
/// A project gives one host directory exclusive ownership of one lockbox
/// subtree. Updates may add, replace, or remove entries only below
/// [`Self::destination`]. Two project destinations cannot overlap, and a
/// destination of `/` therefore prevents any other mirror project.
///
/// Creating this value does not inspect or copy the host directory. Call
/// [`Lockbox::create_mirror_project`] to store it, then use
/// [`Lockbox::with_mirror_project_mutation`] when applying a separately
/// calculated host update. The core deliberately stores host paths as opaque
/// canonical strings so callers can present and validate them using their
/// platform's filesystem APIs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MirrorProject {
    /// Stable user-facing project name.
    pub name: String,
    /// Canonical absolute host source directory.
    pub source: String,
    /// Exclusively managed lockbox directory.
    pub destination: LockboxPath,
    /// Source-relative include patterns. An empty list selects all paths.
    pub includes: Vec<String>,
    /// Source-relative exclude patterns.
    pub excludes: Vec<String>,
    /// Behaviour for destination entries absent from the selected source set.
    pub missing_file_policy: MirrorMissingFilePolicy,
    /// Opaque platform-specific directory identity, when available.
    ///
    /// Higher-level code compares this value before updating so replacing a
    /// directory at the same path is not silently accepted. The CLI currently
    /// records a Unix device/inode pair and leaves it unset on platforms where
    /// it cannot obtain a stable identity.
    pub host_identity: Option<String>,
}

impl<State> Lockbox<State> {
    /// Lists every configured mirror project in stable name order.
    ///
    /// Project definitions are encrypted as hidden variables inside the
    /// lockbox, so they travel with the archive rather than relying on a
    /// separate host-side manifest.
    pub fn list_mirror_projects(&self) -> Result<Vec<MirrorProject>> {
        let mut projects = self
            .list_variables()?
            .into_iter()
            .filter(|(name, _)| name.as_str().starts_with(MIRROR_PREFIX))
            .filter_map(|(name, _)| self.get_variable(&name).transpose())
            .map(|value| value.and_then(|value| decode_project(&value)))
            .collect::<Result<Vec<_>>>()?;
        projects.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(projects)
    }

    /// Returns the named mirror project, or `None` when it is not configured.
    pub fn mirror_project(&self, name: &str) -> Result<Option<MirrorProject>> {
        validate_project_name(name)?;
        let variable = project_variable_name(name)?;
        self.get_variable(&variable)?
            .map(|value| decode_project(&value))
            .transpose()
    }

    pub(crate) fn ensure_mirror_path_mutable(&self, path: &LockboxPath) -> Result<()> {
        let projects = self.list_mirror_projects()?;
        if let Some(root) = &self.mirror_mutation_root {
            if path == root || path.is_descendant_of(root) {
                return Ok(());
            }
            return Err(Error::InvalidOperation(format!(
                "{} is outside the selected mirror directory {}",
                path, root
            )));
        }
        if let Some(project) = projects.iter().find(|project| {
            path == &project.destination || path.is_descendant_of(&project.destination)
        }) {
            return Err(Error::InvalidOperation(format!(
                "{} is managed by mirror '{}'; use a mirror-scoped operation",
                path, project.name
            )));
        }
        Ok(())
    }
}

impl<State: WritableLockboxState> Lockbox<State> {
    /// Stores a new mirror project without copying host files.
    ///
    /// The destination must not overlap another project. A non-empty
    /// destination is rejected unless `adopt` is `true`; adoption makes the
    /// existing subtree managed without changing its current entries.
    pub fn create_mirror_project(&mut self, project: MirrorProject, adopt: bool) -> Result<()> {
        validate_project(&project)?;
        if self.mirror_project(&project.name)?.is_some() {
            return Err(Error::AlreadyExists(project.name));
        }
        for existing in self.list_mirror_projects()? {
            if overlaps(&project.destination, &existing.destination) {
                return Err(Error::InvalidOperation(format!(
                    "mirror destination {} overlaps '{}' at {}",
                    project.destination, existing.name, existing.destination
                )));
            }
        }
        if self.stat(&project.destination).is_some() && !self.is_dir(&project.destination) {
            return Err(Error::InvalidOperation(format!(
                "{} is not a lockbox directory",
                project.destination
            )));
        }
        let occupied = self.toc_entries.values().any(|entry| {
            !entry.deleted
                && (entry.path == project.destination
                    || entry.path.is_descendant_of(&project.destination))
        });
        if occupied && !adopt {
            return Err(Error::InvalidOperation(format!(
                "{} is not empty; explicitly adopt its existing entries",
                project.destination
            )));
        }
        self.store_mirror_project(&project)
    }

    /// Replaces a mirror project's stored configuration.
    ///
    /// This is intended for explicit rule, policy, or host rebind operations.
    /// It performs the same name, path, rule, and overlap validation as
    /// creation but does not modify managed files. The destination is
    /// immutable; changing ownership requires forgetting or deleting the old
    /// project and explicitly creating a new one.
    pub fn update_mirror_project(&mut self, project: &MirrorProject) -> Result<()> {
        validate_project(project)?;
        let Some(current) = self.mirror_project(&project.name)? else {
            return Err(Error::NotFound(project.name.clone()));
        };
        if current.destination != project.destination {
            return Err(Error::InvalidOperation(
                "a mirror destination cannot be changed; create a new project instead".to_string(),
            ));
        }
        for existing in self.list_mirror_projects()? {
            if existing.name != project.name
                && overlaps(&project.destination, &existing.destination)
            {
                return Err(Error::InvalidOperation(format!(
                    "mirror destination {} overlaps '{}' at {}",
                    project.destination, existing.name, existing.destination
                )));
            }
        }
        self.store_mirror_project(project)
    }

    /// Forgets a mirror project while preserving its managed files.
    ///
    /// After this call the former destination is an ordinary lockbox subtree
    /// and can be changed through the normal file APIs.
    pub fn forget_mirror_project(&mut self, name: &str) -> Result<()> {
        if self.mirror_project(name)?.is_none() {
            return Err(Error::NotFound(name.to_string()));
        }
        self.delete_variable(&project_variable_name(name)?)
    }

    /// Runs a mutation scoped to one mirror's managed directory.
    ///
    /// Inside the callback the ordinary file mutation APIs accept paths at or
    /// below the selected destination and reject every path outside it. Outside
    /// the callback those APIs reject changes to all managed destinations.
    /// This lets higher-level clients reuse the normal file API while keeping
    /// project ownership enforcement in the core.
    pub fn with_mirror_project_mutation<T>(
        &mut self,
        name: &str,
        operation: impl FnOnce(&mut Self, &MirrorProject) -> Result<T>,
    ) -> Result<T> {
        let project = self
            .mirror_project(name)?
            .ok_or_else(|| Error::NotFound(name.to_string()))?;
        if self.mirror_mutation_root.is_some() {
            return Err(Error::InvalidOperation(
                "nested mirror mutations are not supported".to_string(),
            ));
        }
        self.mirror_mutation_root = Some(project.destination.clone());
        let result = operation(self, &project);
        self.mirror_mutation_root = None;
        result
    }

    fn store_mirror_project(&mut self, project: &MirrorProject) -> Result<()> {
        let encoded = json!({
            "version": 1,
            "name": project.name,
            "source": project.source,
            "destination": project.destination.to_string(),
            "includes": project.includes,
            "excludes": project.excludes,
            "missing_file_policy": match project.missing_file_policy {
                MirrorMissingFilePolicy::Remove => "remove",
                MirrorMissingFilePolicy::Retain => "retain",
            },
            "host_identity": project.host_identity,
        })
        .to_string();
        self.set_variable(&project_variable_name(&project.name)?, &encoded)
    }
}

fn validate_project(project: &MirrorProject) -> Result<()> {
    validate_project_name(&project.name)?;
    if !is_absolute_host_path(&project.source) {
        return Err(Error::InvalidInput(
            "mirror source must be a canonical absolute path".to_string(),
        ));
    }
    validate_rules(&project.includes)?;
    validate_rules(&project.excludes)
}

fn validate_project_name(name: &str) -> Result<()> {
    let valid = !name.is_empty()
        && name.len() <= 128
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'));
    if valid {
        Ok(())
    } else {
        Err(Error::InvalidInput(format!(
            "invalid mirror project name: {name}"
        )))
    }
}

fn validate_rules(rules: &[String]) -> Result<()> {
    if let Some(rule) = rules.iter().find(|rule| {
        rule.is_empty()
            || rule.starts_with('/')
            || rule.contains('\\')
            || is_windows_drive_path(rule)
            || rule.split('/').any(|component| component == "..")
    }) {
        return Err(Error::InvalidInput(format!(
            "invalid source-relative mirror rule: {rule}"
        )));
    }
    Ok(())
}

fn is_absolute_host_path(path: &str) -> bool {
    path.starts_with('/') || path.starts_with("\\\\") || is_windows_drive_path(path)
}

fn is_windows_drive_path(path: &str) -> bool {
    let bytes = path.as_bytes();
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && matches!(bytes[2], b'/' | b'\\')
}

fn overlaps(left: &LockboxPath, right: &LockboxPath) -> bool {
    left == right || left.is_descendant_of(right) || right.is_descendant_of(left)
}

fn project_variable_name(name: &str) -> Result<VariableName> {
    VariableName::new(format!("{MIRROR_PREFIX}{name}"))
}

fn decode_project(encoded: &str) -> Result<MirrorProject> {
    let value: Value = serde_json::from_str(encoded).map_err(|_| Error::CorruptRecord)?;
    if value["version"].as_u64() != Some(1) {
        return Err(Error::CorruptRecord);
    }
    let string = |field: &str| {
        value[field]
            .as_str()
            .map(str::to_string)
            .ok_or(Error::CorruptRecord)
    };
    let strings = |field: &str| {
        value[field]
            .as_array()
            .ok_or(Error::CorruptRecord)?
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .map(str::to_string)
                    .ok_or(Error::CorruptRecord)
            })
            .collect::<Result<Vec<_>>>()
    };
    let project = MirrorProject {
        name: string("name")?,
        source: string("source")?,
        destination: LockboxPath::new(string("destination")?)?,
        includes: strings("includes")?,
        excludes: strings("excludes")?,
        missing_file_policy: match value["missing_file_policy"].as_str() {
            Some("remove") => MirrorMissingFilePolicy::Remove,
            Some("retain") => MirrorMissingFilePolicy::Retain,
            _ => return Err(Error::CorruptRecord),
        },
        host_identity: value["host_identity"].as_str().map(str::to_string),
    };
    validate_project(&project)?;
    Ok(project)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_path_validation_is_portable_across_archive_moves() {
        assert!(is_absolute_host_path("/srv/project"));
        assert!(is_absolute_host_path(r"C:\Users\alice\project"));
        assert!(is_absolute_host_path(r"\\server\share\project"));
        assert!(!is_absolute_host_path("relative/project"));
    }

    #[test]
    fn rules_use_portable_source_relative_slash_paths() {
        assert!(validate_rules(&["src/**/*.rs".to_string()]).is_ok());
        assert!(validate_rules(&[r"src\**\*.rs".to_string()]).is_err());
        assert!(validate_rules(&["C:/source/**".to_string()]).is_err());
    }
}
