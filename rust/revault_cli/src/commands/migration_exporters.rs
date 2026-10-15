//! Exact runtime dependencies, shared with the release publication gate.
//! Versions intentionally remain pinned to validated historical protocols.

#[derive(Clone, Copy)]
pub struct ExporterRelease {
    pub package: &'static str,
    pub version: &'static str,
    pub binary: &'static str,
    pub protocol: u64,
    pub artifact: &'static str,
    pub native_version: u32,
    pub migration_schema: u32,
}

pub const EXPORTERS: &[ExporterRelease] = &[
    ExporterRelease {
        package: "revault_migrate_vault_v2",
        version: "0.0.1",
        binary: "revault-migrate-vault-v2",
        protocol: 2,
        artifact: "vault",
        native_version: 2,
        migration_schema: 2,
    },
    ExporterRelease {
        package: "revault_migrate_vault_v1",
        version: "0.0.4",
        binary: "revault-migrate-vault-v1",
        protocol: 2,
        artifact: "vault",
        native_version: 1,
        migration_schema: 2,
    },
    ExporterRelease {
        package: "revault_migrate_archive_v1",
        version: "0.0.5",
        binary: "revault-migrate-archive-v1",
        protocol: 3,
        artifact: "archive",
        native_version: 1,
        migration_schema: 2,
    },
];

pub fn capabilities_match(bytes: &[u8], release: ExporterRelease) -> bool {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(bytes) else {
        return false;
    };
    let valid = value.get("protocol").and_then(|value| value.as_u64()) == Some(release.protocol)
        && value.get("artifact").and_then(|value| value.as_str()) == Some(release.artifact)
        && value.get("native_version").and_then(|value| value.as_u64())
            == Some(u64::from(release.native_version))
        && value
            .get("migration_schema")
            .and_then(|value| value.as_u64())
            == Some(u64::from(release.migration_schema));
    if !valid || release.artifact != "vault" || release.protocol < 2 {
        return valid;
    }
    if release.native_version == 2 {
        return value
            .get("container_version")
            .and_then(|value| value.as_u64())
            == Some(2)
            && json_u64_array(&value, "structure_versions") == [2]
            && json_u64_array(&value, "migration_schemas") == [2];
    }
    release.native_version == 1
        && value
            .get("container_version")
            .and_then(|value| value.as_u64())
            == Some(1)
        && json_u64_array(&value, "structure_versions") == [1, 2]
        && json_u64_array(&value, "migration_schemas") == [1, 2]
}

fn json_u64_array(value: &serde_json::Value, field: &str) -> Vec<u64> {
    value
        .get(field)
        .and_then(|value| value.as_array())
        .map(|values| {
            values
                .iter()
                .filter_map(serde_json::Value::as_u64)
                .collect()
        })
        .unwrap_or_default()
}
