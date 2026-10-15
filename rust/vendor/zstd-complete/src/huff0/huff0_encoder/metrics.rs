use super::HuffmanTable;

impl HuffmanTable {
    pub(crate) fn can_encode_counts(&self, counts: &[usize]) -> bool {
        if counts.len() > self.codes.len() {
            return false;
        }

        counts
            .iter()
            .copied()
            .zip(self.codes.iter().copied())
            .all(|(count, (_, num_bits))| count == 0 || num_bits != 0)
    }

    pub(crate) fn encoded_len(&self, data: &[u8], with_table: bool, four_streams: bool) -> usize {
        let table_len = if with_table {
            self.table_description_len()
        } else {
            0
        };
        let data_len = if four_streams {
            let split_size = data.len().div_ceil(4);
            6 + self.stream_encoded_len(&data[..split_size])
                + self.stream_encoded_len(&data[split_size..split_size * 2])
                + self.stream_encoded_len(&data[split_size * 2..split_size * 3])
                + self.stream_encoded_len(&data[split_size * 3..])
        } else {
            self.stream_encoded_len(data)
        };
        table_len + data_len
    }

    pub(crate) fn encoded_len_from_counts(&self, counts: &[usize], with_table: bool) -> usize {
        let table_len = if with_table {
            self.table_description_len()
        } else {
            0
        };
        table_len + self.stream_encoded_len_from_counts(counts)
    }

    pub(crate) fn encoded_len_from_stream_counts(
        &self,
        stream_counts: &[[usize; 256]; 4],
        with_table: bool,
    ) -> usize {
        let table_len = if with_table {
            self.table_description_len()
        } else {
            0
        };
        table_len
            + 6
            + stream_counts
                .iter()
                .map(|counts| self.stream_encoded_len_from_counts(counts))
                .sum::<usize>()
    }

    pub(crate) fn estimated_compressed_size_from_counts(&self, counts: &[usize]) -> usize {
        let bit_len = counts
            .iter()
            .copied()
            .zip(self.codes.iter().copied())
            .map(|(count, (_, num_bits))| count * usize::from(num_bits))
            .sum::<usize>();
        bit_len >> 3
    }

    pub(crate) fn table_description_len(&self) -> usize {
        self.table_description.len()
    }

    fn stream_encoded_len(&self, data: &[u8]) -> usize {
        let mut bit_len = 0usize;
        for symbol in data {
            bit_len += self.codes[*symbol as usize].1 as usize;
        }
        bit_len / 8 + 1
    }

    fn stream_encoded_len_from_counts(&self, counts: &[usize]) -> usize {
        let bit_len = counts
            .iter()
            .copied()
            .zip(self.codes.iter().copied())
            .map(|(count, (_, num_bits))| count * usize::from(num_bits))
            .sum::<usize>();
        bit_len / 8 + 1
    }
}
