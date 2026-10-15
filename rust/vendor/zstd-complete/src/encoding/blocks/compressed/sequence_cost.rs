use crate::fse::fse_encoder::FSETable;

pub(super) struct CodeCounts {
    counts: [usize; 256],
    most_frequent: usize,
    total: usize,
}

impl CodeCounts {
    pub(super) fn new() -> Self {
        Self {
            counts: [0; 256],
            most_frequent: 0,
            total: 0,
        }
    }

    pub(super) fn from_codes(codes: impl Iterator<Item = u8>) -> Self {
        let mut counts = Self::new();
        for code in codes {
            counts.add_code(code);
        }
        counts
    }

    pub(super) fn add_code(&mut self, code: u8) {
        let idx = usize::from(code);
        self.counts[idx] += 1;
        self.most_frequent = self.most_frequent.max(self.counts[idx]);
        self.total += 1;
    }

    pub(super) fn add_code_untracked(&mut self, code: u8) {
        self.counts[usize::from(code)] += 1;
    }

    pub(super) fn finish_untracked(&mut self, total: usize, max_code: u8) {
        self.total = total;
        self.most_frequent = self.counts[..=usize::from(max_code)]
            .iter()
            .copied()
            .max()
            .unwrap_or(0);
    }

    pub(super) fn most_frequent(&self) -> usize {
        self.most_frequent
    }

    pub(super) fn total(&self) -> usize {
        self.total
    }

    pub(super) fn counts(&self) -> &[usize; 256] {
        &self.counts
    }

    pub(super) fn max_symbol(&self) -> usize {
        self.counts
            .iter()
            .rposition(|count| *count != 0)
            .unwrap_or(0)
    }

    pub(super) fn default_allowed(&self, table: &FSETable) -> bool {
        self.iter_present()
            .all(|(symbol, _)| table.can_encode_symbol(symbol))
    }

    pub(super) fn iter_present(&self) -> impl Iterator<Item = (u8, usize)> + '_ {
        self.counts
            .iter()
            .copied()
            .enumerate()
            .filter(|(_, count)| *count != 0)
            .map(|(symbol, count)| (symbol as u8, count))
    }
}

pub(super) fn cross_entropy_cost(table: &FSETable, counts: &CodeCounts) -> Option<usize> {
    cross_entropy_cost_up_to(table, counts, counts.max_symbol())
}

pub(super) fn cross_entropy_cost_up_to(
    table: &FSETable,
    counts: &CodeCounts,
    max_symbol: usize,
) -> Option<usize> {
    let shift = 8usize.checked_sub(usize::from(table.acc_log()))?;
    let mut cost = 0usize;

    // Keep the measured hot loop's direct indexed shape stable across Clippy releases.
    #[allow(clippy::needless_range_loop)]
    for symbol in 0..=max_symbol {
        let count = counts.counts[symbol];
        if count == 0 {
            continue;
        }
        let probability = table.normalized_probability(symbol as u8);
        if probability == 0 {
            return None;
        }
        let norm = if probability == -1 {
            1usize
        } else {
            probability as usize
        };
        let norm256 = norm << shift;
        if norm256 == 0 || norm256 >= INVERSE_PROBABILITY_LOG256.len() {
            return None;
        }
        cost += count * INVERSE_PROBABILITY_LOG256[norm256];
    }

    Some(cost >> 8)
}

pub(super) fn repeat_table_cost(table: &FSETable, counts: &CodeCounts) -> Option<usize> {
    repeat_table_cost_up_to(table, counts, counts.max_symbol())
}

pub(super) fn repeat_table_cost_up_to(
    table: &FSETable,
    counts: &CodeCounts,
    max_symbol: usize,
) -> Option<usize> {
    let accuracy_log = 8u8;
    let bad_cost = (usize::from(table.acc_log()) + 1) << accuracy_log;
    let mut cost = 0usize;

    // Keep the measured hot loop's direct indexed shape stable across Clippy releases.
    #[allow(clippy::needless_range_loop)]
    for symbol in 0..=max_symbol {
        let count = counts.counts[symbol];
        if count == 0 {
            continue;
        }
        let bit_cost = table.bit_cost(symbol as u8, accuracy_log)?;
        if bit_cost >= bad_cost {
            return None;
        }
        cost += count * bit_cost;
    }

    Some(cost >> accuracy_log)
}

pub(super) fn entropy_cost_up_to(counts: &CodeCounts, max_symbol: usize) -> usize {
    let mut cost = 0usize;
    let total = counts.total();

    for symbol in 0..=max_symbol {
        let count = counts.counts[symbol];
        if count == 0 {
            continue;
        }
        let mut norm = (256 * count) / total;
        if norm == 0 {
            norm = 1;
        }
        debug_assert!(count < total || counts.most_frequent() == total);
        cost += count * INVERSE_PROBABILITY_LOG256[norm];
    }

    cost >> 8
}

const INVERSE_PROBABILITY_LOG256: [usize; 256] = [
    0, 2048, 1792, 1642, 1536, 1453, 1386, 1329, 1280, 1236, 1197, 1162, 1130, 1100, 1073, 1047,
    1024, 1001, 980, 960, 941, 923, 906, 889, 874, 859, 844, 830, 817, 804, 791, 779, 768, 756,
    745, 734, 724, 714, 704, 694, 685, 676, 667, 658, 650, 642, 633, 626, 618, 610, 603, 595, 588,
    581, 574, 567, 561, 554, 548, 542, 535, 529, 523, 517, 512, 506, 500, 495, 489, 484, 478, 473,
    468, 463, 458, 453, 448, 443, 438, 434, 429, 424, 420, 415, 411, 407, 402, 398, 394, 390, 386,
    382, 377, 373, 370, 366, 362, 358, 354, 350, 347, 343, 339, 336, 332, 329, 325, 322, 318, 315,
    311, 308, 305, 302, 298, 295, 292, 289, 286, 282, 279, 276, 273, 270, 267, 264, 261, 258, 256,
    253, 250, 247, 244, 241, 239, 236, 233, 230, 228, 225, 222, 220, 217, 215, 212, 209, 207, 204,
    202, 199, 197, 194, 192, 190, 187, 185, 182, 180, 178, 175, 173, 171, 168, 166, 164, 162, 159,
    157, 155, 153, 151, 149, 146, 144, 142, 140, 138, 136, 134, 132, 130, 128, 126, 123, 121, 119,
    117, 115, 114, 112, 110, 108, 106, 104, 102, 100, 98, 96, 94, 93, 91, 89, 87, 85, 83, 82, 80,
    78, 76, 74, 73, 71, 69, 67, 66, 64, 62, 61, 59, 57, 55, 54, 52, 50, 49, 47, 46, 44, 42, 41, 39,
    37, 36, 34, 33, 31, 30, 28, 26, 25, 23, 22, 20, 19, 17, 16, 14, 13, 11, 10, 8, 7, 5, 4, 2, 1,
];

#[cfg(test)]
mod tests {
    use super::CodeCounts;

    #[test]
    fn post_count_statistics_match_incremental_statistics() {
        let codes = [0, 7, 7, 12, 35, 7, 35, 1, 12, 35, 35];
        let mut incremental = CodeCounts::new();
        let mut deferred = CodeCounts::new();
        for code in codes {
            incremental.add_code(code);
            deferred.add_code_untracked(code);
        }
        deferred.finish_untracked(codes.len(), 35);

        assert_eq!(deferred.counts(), incremental.counts());
        assert_eq!(deferred.total(), incremental.total());
        assert_eq!(deferred.most_frequent(), incremental.most_frequent());
        assert_eq!(deferred.max_symbol(), incremental.max_symbol());
    }
}
