//! Standalone summary for the 64-case, three-repeat issue #310 read matrix.
//! rustc summarize_read_matrix.rs -o /tmp/summarize-read-matrix
//! /tmp/summarize-read-matrix RAW_CSV
use std::{collections::BTreeMap, env, fs};
fn median(values: &[f64]) -> f64 {
    let mut values = values.to_vec();
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}
fn main() {
    let csv = fs::read_to_string(env::args().nth(1).expect("RAW_CSV argument")).unwrap();
    let mut groups: BTreeMap<(String, String), [BTreeMap<usize, f64>; 2]> = BTreeMap::new();
    for line in csv.lines().skip(1) {
        let fields: Vec<_> = line.split(',').collect();
        assert_eq!(fields.len(), 19);
        let variant = match fields[1] {
            "before" => 0,
            "after" => 1,
            _ => panic!("variant"),
        };
        let run: usize = fields[2].parse().unwrap();
        assert!((1..=3).contains(&run));
        let case = format!(
            "{}/{}/{}/{}/{}",
            fields[18], fields[6], fields[5], fields[7], fields[8]
        );
        for (metric, index) in [("total", 10), ("open", 14), ("read", 15)] {
            let value: f64 = fields[index].parse().unwrap();
            assert!(value.is_finite() && value > 0.0);
            assert!(
                groups.entry((case.clone(), metric.to_owned())).or_default()[variant]
                    .insert(run, value)
                    .is_none(),
                "duplicate run"
            );
        }
    }
    assert_eq!(groups.len(), 64 * 3, "incomplete workload matrix");
    println!("case,metric,before_ms,after_ms,median_paired_change_pct,min_paired_change_pct,max_paired_change_pct,runs");
    for ((case, metric), values) in groups {
        for variant in &values {
            assert_eq!(variant.keys().copied().collect::<Vec<_>>(), [1, 2, 3]);
        }
        let before: Vec<_> = values[0].values().copied().collect();
        let after: Vec<_> = values[1].values().copied().collect();
        let changes: Vec<_> = before
            .iter()
            .zip(&after)
            .map(|(a, b)| (b / a - 1.0) * 100.0)
            .collect();
        println!(
            "{case},{metric},{:.3},{:.3},{:.3},{:.3},{:.3},3",
            median(&before),
            median(&after),
            median(&changes),
            changes.iter().copied().fold(f64::INFINITY, f64::min),
            changes.iter().copied().fold(f64::NEG_INFINITY, f64::max)
        );
    }
}
