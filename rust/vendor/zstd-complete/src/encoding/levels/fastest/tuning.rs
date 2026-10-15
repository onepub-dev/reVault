use std::sync::OnceLock;

#[cfg(feature = "std")]
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct FastestTuningOverrides {
    pub(super) composer_max_partitions: Option<usize>,
    pub(super) lockfile_fastest_splits: Option<bool>,
    pub(super) lockfile_compare_whole_text: Option<bool>,
}

#[cfg(feature = "std")]
static FASTEST_TUNING_OVERRIDES: OnceLock<FastestTuningOverrides> = OnceLock::new();

#[cfg(feature = "std")]
pub(super) fn fastest_tuning_overrides() -> &'static FastestTuningOverrides {
    FASTEST_TUNING_OVERRIDES.get_or_init(FastestTuningOverrides::from_env)
}

#[cfg(feature = "std")]
impl FastestTuningOverrides {
    fn from_env() -> Self {
        Self {
            composer_max_partitions: std::env::var("RUZSTD_TUNE_COMPOSER_MAX_PARTITIONS")
                .ok()
                .and_then(|value| value.parse().ok()),
            lockfile_fastest_splits: parse_bool_env("RUZSTD_TUNE_LOCKFILE_FASTEST_SPLITS"),
            lockfile_compare_whole_text: parse_bool_env("RUZSTD_TUNE_LOCKFILE_COMPARE_WHOLE_TEXT"),
        }
    }
}

#[cfg(feature = "std")]
fn parse_bool_env(name: &str) -> Option<bool> {
    match std::env::var(name).ok()?.as_str() {
        "1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON" => Some(true),
        "0" | "false" | "FALSE" | "no" | "NO" | "off" | "OFF" => Some(false),
        _ => None,
    }
}
