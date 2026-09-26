#[cfg(feature = "std")]
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct MatcherTuningOverrides {
    pub(super) lockfile_probe_step: Option<usize>,
    pub(super) composer_probe_step: Option<usize>,
    pub(super) structured_json_probe_step: Option<usize>,
    pub(super) tsconfig_probe_step: Option<usize>,
    pub(super) dictionary_same_start_bits_gain_min: Option<usize>,
    pub(super) dictionary_same_start_match_loss_max: Option<usize>,
    pub(super) lockfile_same_end_bits_gain_min: Option<usize>,
    pub(super) lockfile_same_end_match_loss_max: Option<usize>,
    pub(super) lockfile_repeat_kind_match_loss_max: Option<usize>,
    pub(super) lockfile_second_newest_zero_literals: Option<bool>,
    pub(super) lockfile_zero_literal_window_disable: Option<bool>,
    pub(super) lockfile_zero_literal_window_max_match_len: Option<usize>,
    pub(super) lockfile_zero_literal_window_min_offset_bits: Option<usize>,
    pub(super) lockfile_next_position_max_skip_literals: Option<usize>,
    pub(super) lockfile_next_position_max_current_match_len: Option<usize>,
    pub(super) lockfile_next_position_match_loss_max: Option<usize>,
    pub(super) lockfile_next_position_literal_weight: Option<usize>,
    pub(super) lockfile_next_position_match_reward: Option<usize>,
    pub(super) lockfile_next_position_offset_weight: Option<usize>,
    pub(super) lockfile_next_position_margin: Option<usize>,
    pub(super) composer_window_disable: Option<bool>,
    pub(super) composer_repeat_kind_match_loss_max: Option<usize>,
    pub(super) composer_repeat_kind_zero_literals_only: Option<bool>,
    pub(super) composer_zero_literal_repeat_candidate_limit: Option<usize>,
}

impl MatcherTuningOverrides {
    #[cfg(not(feature = "std"))]
    const fn none() -> Self {
        Self {
            lockfile_probe_step: None,
            composer_probe_step: None,
            structured_json_probe_step: None,
            tsconfig_probe_step: None,
            dictionary_same_start_bits_gain_min: None,
            dictionary_same_start_match_loss_max: None,
            lockfile_same_end_bits_gain_min: None,
            lockfile_same_end_match_loss_max: None,
            lockfile_repeat_kind_match_loss_max: None,
            lockfile_second_newest_zero_literals: None,
            lockfile_zero_literal_window_disable: None,
            lockfile_zero_literal_window_max_match_len: None,
            lockfile_zero_literal_window_min_offset_bits: None,
            lockfile_next_position_max_skip_literals: None,
            lockfile_next_position_max_current_match_len: None,
            lockfile_next_position_match_loss_max: None,
            lockfile_next_position_literal_weight: None,
            lockfile_next_position_match_reward: None,
            lockfile_next_position_offset_weight: None,
            lockfile_next_position_margin: None,
            composer_window_disable: None,
            composer_repeat_kind_match_loss_max: None,
            composer_repeat_kind_zero_literals_only: None,
            composer_zero_literal_repeat_candidate_limit: None,
        }
    }
}

#[cfg(feature = "std")]
static MATCHER_TUNING_OVERRIDES: OnceLock<MatcherTuningOverrides> = OnceLock::new();

#[cfg(not(feature = "std"))]
static MATCHER_TUNING_OVERRIDES: MatcherTuningOverrides = MatcherTuningOverrides::none();

#[cfg(feature = "std")]
pub(super) fn matcher_tuning_overrides() -> &'static MatcherTuningOverrides {
    MATCHER_TUNING_OVERRIDES.get_or_init(MatcherTuningOverrides::from_env)
}

#[cfg(not(feature = "std"))]
pub(super) fn matcher_tuning_overrides() -> &'static MatcherTuningOverrides {
    &MATCHER_TUNING_OVERRIDES
}

#[cfg(feature = "std")]
impl MatcherTuningOverrides {
    fn from_env() -> Self {
        Self {
            lockfile_probe_step: Self::parse_usize("RUZSTD_TUNE_LOCKFILE_PROBE_STEP"),
            composer_probe_step: Self::parse_usize("RUZSTD_TUNE_COMPOSER_PROBE_STEP"),
            structured_json_probe_step: Self::parse_usize("RUZSTD_TUNE_STRUCTURED_JSON_PROBE_STEP"),
            tsconfig_probe_step: Self::parse_usize("RUZSTD_TUNE_TSCONFIG_PROBE_STEP"),
            dictionary_same_start_bits_gain_min: Self::parse_usize(
                "RUZSTD_TUNE_DICTIONARY_SAME_START_BITS_GAIN_MIN",
            ),
            dictionary_same_start_match_loss_max: Self::parse_usize(
                "RUZSTD_TUNE_DICTIONARY_SAME_START_MATCH_LOSS_MAX",
            ),
            lockfile_same_end_bits_gain_min: Self::parse_usize(
                "RUZSTD_TUNE_LOCKFILE_SAME_END_BITS_GAIN_MIN",
            ),
            lockfile_same_end_match_loss_max: Self::parse_usize(
                "RUZSTD_TUNE_LOCKFILE_SAME_END_MATCH_LOSS_MAX",
            ),
            lockfile_repeat_kind_match_loss_max: Self::parse_usize(
                "RUZSTD_TUNE_LOCKFILE_REPEAT_KIND_MATCH_LOSS_MAX",
            ),
            lockfile_second_newest_zero_literals: Self::parse_bool(
                "RUZSTD_TUNE_LOCKFILE_SECOND_NEWEST_ZERO_LITERALS",
            ),
            lockfile_zero_literal_window_disable: Self::parse_bool(
                "RUZSTD_TUNE_LOCKFILE_ZERO_LITERAL_WINDOW_DISABLE",
            ),
            lockfile_zero_literal_window_max_match_len: Self::parse_usize(
                "RUZSTD_TUNE_LOCKFILE_ZERO_LITERAL_WINDOW_MAX_MATCH_LEN",
            ),
            lockfile_zero_literal_window_min_offset_bits: Self::parse_usize(
                "RUZSTD_TUNE_LOCKFILE_ZERO_LITERAL_WINDOW_MIN_OFFSET_BITS",
            ),
            lockfile_next_position_max_skip_literals: Self::parse_usize(
                "RUZSTD_TUNE_LOCKFILE_NEXT_POSITION_MAX_SKIP_LITERALS",
            ),
            lockfile_next_position_max_current_match_len: Self::parse_usize(
                "RUZSTD_TUNE_LOCKFILE_NEXT_POSITION_MAX_CURRENT_MATCH_LEN",
            ),
            lockfile_next_position_match_loss_max: Self::parse_usize(
                "RUZSTD_TUNE_LOCKFILE_NEXT_POSITION_MATCH_LOSS_MAX",
            ),
            lockfile_next_position_literal_weight: Self::parse_usize(
                "RUZSTD_TUNE_LOCKFILE_NEXT_POSITION_LITERAL_WEIGHT",
            ),
            lockfile_next_position_match_reward: Self::parse_usize(
                "RUZSTD_TUNE_LOCKFILE_NEXT_POSITION_MATCH_REWARD",
            ),
            lockfile_next_position_offset_weight: Self::parse_usize(
                "RUZSTD_TUNE_LOCKFILE_NEXT_POSITION_OFFSET_WEIGHT",
            ),
            lockfile_next_position_margin: Self::parse_usize(
                "RUZSTD_TUNE_LOCKFILE_NEXT_POSITION_MARGIN",
            ),
            composer_window_disable: Self::parse_bool("RUZSTD_TUNE_COMPOSER_WINDOW_DISABLE"),
            composer_repeat_kind_match_loss_max: Self::parse_usize(
                "RUZSTD_TUNE_COMPOSER_REPEAT_KIND_MATCH_LOSS_MAX",
            ),
            composer_repeat_kind_zero_literals_only: Self::parse_bool(
                "RUZSTD_TUNE_COMPOSER_REPEAT_KIND_ZERO_LITERALS_ONLY",
            ),
            composer_zero_literal_repeat_candidate_limit: Self::parse_usize(
                "RUZSTD_TUNE_COMPOSER_ZERO_LITERAL_REPEAT_CANDIDATE_LIMIT",
            ),
        }
    }

    fn parse_usize(name: &str) -> Option<usize> {
        std::env::var(name).ok()?.parse().ok()
    }

    fn parse_bool(name: &str) -> Option<bool> {
        match std::env::var(name).ok()?.as_str() {
            "1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON" => Some(true),
            "0" | "false" | "FALSE" | "no" | "NO" | "off" | "OFF" => Some(false),
            _ => None,
        }
    }
}
