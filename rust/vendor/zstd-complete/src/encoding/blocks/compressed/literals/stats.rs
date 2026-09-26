const PARALLEL_HISTOGRAM_MIN_LITERALS: usize = 1500;

/// Aggregate literal frequencies and the metadata needed for Huffman choices.
pub(in crate::encoding::blocks::compressed) struct LiteralStats {
    counts: [usize; 256],
    max_symbol: usize,
    largest: usize,
}

impl LiteralStats {
    #[cfg(test)]
    pub(in crate::encoding::blocks::compressed) fn from_literals(literals: &[u8]) -> Self {
        Self::from_literals_with_stream_counts(literals, false).0
    }

    #[cfg_attr(target_vendor = "apple", link_section = "__TEXT,__rz_lit")]
    #[cfg_attr(target_family = "windows", link_section = ".text$042.rz.lit")]
    #[cfg_attr(
        all(
            not(target_vendor = "apple"),
            not(target_family = "windows"),
            not(target_family = "wasm")
        ),
        link_section = ".text.sorted.042.ruzstd.literal.stats"
    )]
    pub(in crate::encoding::blocks::compressed) fn from_literals_with_stream_counts(
        literals: &[u8],
        collect_stream_counts: bool,
    ) -> (Self, Option<[[usize; 256]; 4]>) {
        let mut counts = [0; 256];
        let mut stream_counts = collect_stream_counts.then_some([[0usize; 256]; 4]);

        if let Some(stream_counts) = &mut stream_counts {
            if !literals.is_empty() {
                let split_size = literals.len().div_ceil(4);
                for (stream_idx, stream) in literals.chunks(split_size).enumerate() {
                    for &literal in stream {
                        let symbol = usize::from(literal);
                        counts[symbol] += 1;
                        stream_counts[stream_idx][symbol] += 1;
                    }
                }
            }
        } else if literals.len() >= PARALLEL_HISTOGRAM_MIN_LITERALS {
            count_literals_parallel(literals, &mut counts);
        } else {
            for &literal in literals {
                let symbol = usize::from(literal);
                counts[symbol] += 1;
            }
        }

        let mut max_symbol = 0usize;
        let mut largest = 0usize;
        for (symbol, count) in counts.iter().copied().enumerate() {
            if count != 0 {
                max_symbol = symbol;
                largest = largest.max(count);
            }
        }

        (
            Self {
                counts,
                max_symbol,
                largest,
            },
            stream_counts,
        )
    }

    pub(in crate::encoding::blocks::compressed) fn counts(&self) -> &[usize] {
        &self.counts[..=self.max_symbol]
    }

    pub(in crate::encoding::blocks::compressed) fn likely_incompressible(
        &self,
        len: usize,
    ) -> bool {
        self.largest <= (len >> 7) + 4
    }

    pub(in crate::encoding::blocks::compressed) fn largest(&self) -> usize {
        self.largest
    }
}

/// Mirrors C's four-lane `HIST_countFast_wksp()` dependency-breaking layout.
fn count_literals_parallel(literals: &[u8], counts: &mut [usize; 256]) {
    let mut lanes = [[0u32; 256]; 4];
    let mut stripes = literals.chunks_exact(16);

    for stripe in stripes.by_ref() {
        lanes[0][usize::from(stripe[0])] += 1;
        lanes[1][usize::from(stripe[1])] += 1;
        lanes[2][usize::from(stripe[2])] += 1;
        lanes[3][usize::from(stripe[3])] += 1;
        lanes[0][usize::from(stripe[4])] += 1;
        lanes[1][usize::from(stripe[5])] += 1;
        lanes[2][usize::from(stripe[6])] += 1;
        lanes[3][usize::from(stripe[7])] += 1;
        lanes[0][usize::from(stripe[8])] += 1;
        lanes[1][usize::from(stripe[9])] += 1;
        lanes[2][usize::from(stripe[10])] += 1;
        lanes[3][usize::from(stripe[11])] += 1;
        lanes[0][usize::from(stripe[12])] += 1;
        lanes[1][usize::from(stripe[13])] += 1;
        lanes[2][usize::from(stripe[14])] += 1;
        lanes[3][usize::from(stripe[15])] += 1;
    }

    for &literal in stripes.remainder() {
        lanes[0][usize::from(literal)] += 1;
    }
    for (symbol, count) in counts.iter_mut().enumerate() {
        *count =
            (lanes[0][symbol] + lanes[1][symbol] + lanes[2][symbol] + lanes[3][symbol]) as usize;
    }
}
