use std::{collections::BTreeMap, fs};
fn median(v: &mut [f64]) -> f64 {
    v.sort_by(f64::total_cmp);
    let n = v.len();
    if n % 2 == 0 {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    } else {
        v[n / 2]
    }
}
fn main() {
    let mut out = String::from("layout,metric,median_us,samples\n");
    for layout in ["native", "default"] {
        let s = fs::read_to_string(format!(
            "target/issue310-pooled-profile-{layout}-stages.txt"
        ))
        .unwrap();
        let mut metrics = BTreeMap::<String, Vec<f64>>::new();
        let mut count = 0;
        for line in s.lines().filter(|l| l.starts_with("write-profile,")) {
            let c: Vec<_> = line.split(',').collect();
            if c[5] == "0" {
                continue;
            }
            count += 1;
            let mut add = 0.0;
            let mut prep = 0.0;
            for field in &c[6..] {
                let (key, value) = field.split_once('=').unwrap();
                let value = value.parse::<f64>().unwrap();
                metrics.entry(key.to_owned()).or_default().push(value);
                if key == "add_us" {
                    add = value
                }
                if key == "prepare_us" {
                    prep = value
                }
            }
            metrics
                .entry("add_minus_prepare_us".into())
                .or_default()
                .push(add - prep);
        }
        assert_eq!(count, 499);
        for (key, mut values) in metrics {
            out.push_str(&format!(
                "{layout},{key},{:.3},{}\n",
                median(&mut values),
                values.len()
            ));
        }
    }
    fs::write(
        "revault_lockbox_api/benches/results/issue310-pooled-profile-stage-summary.csv",
        &out,
    )
    .unwrap();
    print!("{out}");
}
