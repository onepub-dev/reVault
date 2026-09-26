use super::{MatchCandidate, RepeatCandidateKind, BEST_WINDOW_BLOCKS};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CandidateSource {
    RepeatCurrent(RepeatCandidateKind),
    RepeatNextPosition(RepeatCandidateKind),
    WindowCurrentLongHash,
    WindowCurrentNewest { entry_distance: usize },
    WindowCurrentSecondNewest { entry_distance: usize },
    WindowCurrentOldest { entry_distance: usize },
    WindowNextPositionNewest { entry_distance: usize },
    WindowNextPositionSecondNewest { entry_distance: usize },
    WindowNextPositionOldest { entry_distance: usize },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RepeatNextPositionSelectionReason {
    NoCurrentCandidate,
    BeatsCurrentMinNonRepeat,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct MatcherDiagnostics {
    pub(crate) total_sequences: usize,
    pub(crate) repeat_current: [usize; 3],
    pub(crate) repeat_current_zero_literals: [usize; 3],
    pub(crate) repeat_current_with_literals: [usize; 3],
    pub(crate) repeat_best_before_window: [usize; 3],
    pub(crate) repeat_best_before_window_zero_literals: [usize; 3],
    pub(crate) repeat_best_before_window_with_literals: [usize; 3],
    pub(crate) repeat_best_before_window_overridden_by_window: [usize; 3],
    pub(crate) repeat_next_position: [usize; 3],
    pub(crate) repeat_next_position_zero_literals: [usize; 3],
    pub(crate) repeat_next_position_with_literals: [usize; 3],
    pub(crate) repeat_next_position_selected_without_current_candidate: [usize; 3],
    pub(crate) repeat_next_position_selected_over_current_min_non_repeat: [usize; 3],
    pub(crate) current_long_hash_found: usize,
    pub(crate) current_long_hash_overridden: usize,
    pub(crate) current_long_hash_improved_by_newest: [usize; BEST_WINDOW_BLOCKS],
    pub(crate) current_long_hash_improved_by_second_newest: [usize; BEST_WINDOW_BLOCKS],
    pub(crate) current_long_hash_improved_by_oldest: [usize; BEST_WINDOW_BLOCKS],
    pub(crate) current_long_hash_end_break_by_newest: [usize; BEST_WINDOW_BLOCKS],
    pub(crate) current_long_hash_end_break_by_second_newest: [usize; BEST_WINDOW_BLOCKS],
    pub(crate) current_long_hash_end_break_by_oldest: [usize; BEST_WINDOW_BLOCKS],
    pub(crate) current_long_hash_end_break_without_improvement_by_newest:
        [usize; BEST_WINDOW_BLOCKS],
    pub(crate) current_long_hash_end_break_without_improvement_by_second_newest:
        [usize; BEST_WINDOW_BLOCKS],
    pub(crate) current_long_hash_end_break_without_improvement_by_oldest:
        [usize; BEST_WINDOW_BLOCKS],
    pub(crate) current_long_hash_overridden_by_newest: [usize; BEST_WINDOW_BLOCKS],
    pub(crate) current_long_hash_overridden_by_second_newest: [usize; BEST_WINDOW_BLOCKS],
    pub(crate) current_long_hash_overridden_by_oldest: [usize; BEST_WINDOW_BLOCKS],
    pub(crate) window_current_long_hash: usize,
    pub(crate) window_current_long_hash_zero_literals: usize,
    pub(crate) window_current_long_hash_with_literals: usize,
    pub(crate) window_current_newest: [usize; BEST_WINDOW_BLOCKS],
    pub(crate) window_current_second_newest: [usize; BEST_WINDOW_BLOCKS],
    pub(crate) window_current_second_newest_zero_literals: [usize; BEST_WINDOW_BLOCKS],
    pub(crate) window_current_second_newest_with_literals: [usize; BEST_WINDOW_BLOCKS],
    pub(crate) window_current_oldest: [usize; BEST_WINDOW_BLOCKS],
    pub(crate) window_next_position_newest: [usize; BEST_WINDOW_BLOCKS],
    pub(crate) window_next_position_second_newest: [usize; BEST_WINDOW_BLOCKS],
    pub(crate) window_next_position_oldest: [usize; BEST_WINDOW_BLOCKS],
}

impl MatcherDiagnostics {
    fn repeat_kind_index(kind: RepeatCandidateKind) -> usize {
        match kind {
            RepeatCandidateKind::First => 0,
            RepeatCandidateKind::Second => 1,
            RepeatCandidateKind::Third => 2,
        }
    }

    fn increment_distance_bucket(buckets: &mut [usize; BEST_WINDOW_BLOCKS], entry_distance: usize) {
        if let Some(bucket) = buckets.get_mut(entry_distance) {
            *bucket += 1;
        }
    }

    pub(super) fn record_current_long_hash_outcome(
        &mut self,
        found: MatchCandidate,
        selected: Option<MatchCandidate>,
    ) {
        debug_assert!(matches!(
            found.source,
            CandidateSource::WindowCurrentLongHash
        ));
        self.current_long_hash_found += 1;
        let Some(selected) = selected else {
            return;
        };
        if matches!(selected.source, CandidateSource::WindowCurrentLongHash) {
            return;
        }
        self.current_long_hash_overridden += 1;
        match selected.source {
            CandidateSource::WindowCurrentNewest { entry_distance } => {
                Self::increment_distance_bucket(
                    &mut self.current_long_hash_overridden_by_newest,
                    entry_distance,
                );
            }
            CandidateSource::WindowCurrentSecondNewest { entry_distance } => {
                Self::increment_distance_bucket(
                    &mut self.current_long_hash_overridden_by_second_newest,
                    entry_distance,
                );
            }
            CandidateSource::WindowCurrentOldest { entry_distance } => {
                Self::increment_distance_bucket(
                    &mut self.current_long_hash_overridden_by_oldest,
                    entry_distance,
                );
            }
            _ => {}
        }
    }

    pub(super) fn record_repeat_next_position_selection(
        &mut self,
        kind: RepeatCandidateKind,
        reason: RepeatNextPositionSelectionReason,
    ) {
        let index = Self::repeat_kind_index(kind);
        match reason {
            RepeatNextPositionSelectionReason::NoCurrentCandidate => {
                self.repeat_next_position_selected_without_current_candidate[index] += 1;
            }
            RepeatNextPositionSelectionReason::BeatsCurrentMinNonRepeat => {
                self.repeat_next_position_selected_over_current_min_non_repeat[index] += 1;
            }
        }
    }

    pub(super) fn record_repeat_best_before_window(
        &mut self,
        kind: RepeatCandidateKind,
        literals_empty: bool,
    ) {
        let index = Self::repeat_kind_index(kind);
        self.repeat_best_before_window[index] += 1;
        if literals_empty {
            self.repeat_best_before_window_zero_literals[index] += 1;
        } else {
            self.repeat_best_before_window_with_literals[index] += 1;
        }
    }

    pub(super) fn record_repeat_best_before_window_overridden_by_window(
        &mut self,
        kind: RepeatCandidateKind,
    ) {
        let index = Self::repeat_kind_index(kind);
        self.repeat_best_before_window_overridden_by_window[index] += 1;
    }

    pub(super) fn record_current_long_hash_improvement(&mut self, selected: MatchCandidate) {
        match selected.source {
            CandidateSource::WindowCurrentNewest { entry_distance } => {
                Self::increment_distance_bucket(
                    &mut self.current_long_hash_improved_by_newest,
                    entry_distance,
                );
            }
            CandidateSource::WindowCurrentSecondNewest { entry_distance } => {
                Self::increment_distance_bucket(
                    &mut self.current_long_hash_improved_by_second_newest,
                    entry_distance,
                );
            }
            CandidateSource::WindowCurrentOldest { entry_distance } => {
                Self::increment_distance_bucket(
                    &mut self.current_long_hash_improved_by_oldest,
                    entry_distance,
                );
            }
            _ => {}
        }
    }

    pub(super) fn record_current_long_hash_end_break(
        &mut self,
        selected: MatchCandidate,
        improved: bool,
    ) {
        let target = match (selected.source, improved) {
            (CandidateSource::WindowCurrentNewest { .. }, true) => {
                &mut self.current_long_hash_end_break_by_newest
            }
            (CandidateSource::WindowCurrentSecondNewest { .. }, true) => {
                &mut self.current_long_hash_end_break_by_second_newest
            }
            (CandidateSource::WindowCurrentOldest { .. }, true) => {
                &mut self.current_long_hash_end_break_by_oldest
            }
            (CandidateSource::WindowCurrentNewest { .. }, false) => {
                &mut self.current_long_hash_end_break_without_improvement_by_newest
            }
            (CandidateSource::WindowCurrentSecondNewest { .. }, false) => {
                &mut self.current_long_hash_end_break_without_improvement_by_second_newest
            }
            (CandidateSource::WindowCurrentOldest { .. }, false) => {
                &mut self.current_long_hash_end_break_without_improvement_by_oldest
            }
            _ => return,
        };

        let entry_distance = match selected.source {
            CandidateSource::WindowCurrentNewest { entry_distance }
            | CandidateSource::WindowCurrentSecondNewest { entry_distance }
            | CandidateSource::WindowCurrentOldest { entry_distance } => entry_distance,
            _ => return,
        };
        Self::increment_distance_bucket(target, entry_distance);
    }

    pub(super) fn record(&mut self, candidate: MatchCandidate, literals_empty: bool) {
        self.total_sequences += 1;
        match candidate.source {
            CandidateSource::RepeatCurrent(kind) => {
                let index = Self::repeat_kind_index(kind);
                self.repeat_current[index] += 1;
                if literals_empty {
                    self.repeat_current_zero_literals[index] += 1;
                } else {
                    self.repeat_current_with_literals[index] += 1;
                }
            }
            CandidateSource::RepeatNextPosition(kind) => {
                let index = Self::repeat_kind_index(kind);
                self.repeat_next_position[index] += 1;
                if literals_empty {
                    self.repeat_next_position_zero_literals[index] += 1;
                } else {
                    self.repeat_next_position_with_literals[index] += 1;
                }
            }
            CandidateSource::WindowCurrentLongHash => {
                self.window_current_long_hash += 1;
                if literals_empty {
                    self.window_current_long_hash_zero_literals += 1;
                } else {
                    self.window_current_long_hash_with_literals += 1;
                }
            }
            CandidateSource::WindowCurrentNewest { entry_distance } => {
                Self::increment_distance_bucket(&mut self.window_current_newest, entry_distance);
            }
            CandidateSource::WindowCurrentSecondNewest { entry_distance } => {
                Self::increment_distance_bucket(
                    &mut self.window_current_second_newest,
                    entry_distance,
                );
                if literals_empty {
                    Self::increment_distance_bucket(
                        &mut self.window_current_second_newest_zero_literals,
                        entry_distance,
                    );
                } else {
                    Self::increment_distance_bucket(
                        &mut self.window_current_second_newest_with_literals,
                        entry_distance,
                    );
                }
            }
            CandidateSource::WindowCurrentOldest { entry_distance } => {
                Self::increment_distance_bucket(&mut self.window_current_oldest, entry_distance);
            }
            CandidateSource::WindowNextPositionNewest { entry_distance } => {
                Self::increment_distance_bucket(
                    &mut self.window_next_position_newest,
                    entry_distance,
                );
            }
            CandidateSource::WindowNextPositionSecondNewest { entry_distance } => {
                Self::increment_distance_bucket(
                    &mut self.window_next_position_second_newest,
                    entry_distance,
                );
            }
            CandidateSource::WindowNextPositionOldest { entry_distance } => {
                Self::increment_distance_bucket(
                    &mut self.window_next_position_oldest,
                    entry_distance,
                );
            }
        }
    }
}
