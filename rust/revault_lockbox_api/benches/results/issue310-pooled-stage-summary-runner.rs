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
    let s = fs::read_to_string("target/issue310-pooled-stage-run.txt").unwrap();
    let mut frames = Vec::new();
    let mut all = BTreeMap::<String, Vec<f64>>::new();
    let mut samples = 0;
    for l in s.lines() {
        if l.starts_with("native-stage,") {
            let mut row = BTreeMap::new();
            for field in l.split(',').skip(1) {
                let (k, v) = field.split_once('=').unwrap();
                row.insert(k.to_owned(), v.parse::<f64>().unwrap());
            }
            frames.push(row);
        } else if l.starts_with("write-profile,") {
            let c: Vec<_> = l.split(',').collect();
            assert_eq!(frames.len(), 4);
            assert_eq!(c[5].parse::<usize>().unwrap(), samples);
            if samples > 0 {
                for key in ["encode_us", "stage_us", "flush_us", "publish_us"] {
                    all.entry(format!("per_write_{key}"))
                        .or_default()
                        .push(frames.iter().map(|r| r[key]).sum());
                }
                all.entry("first_frame_flush_us".into())
                    .or_default()
                    .push(frames[0]["flush_us"]);
                for f in &frames[1..] {
                    all.entry("later_frame_flush_us".into())
                        .or_default()
                        .push(f["flush_us"]);
                }
            }
            samples += 1;
            frames.clear();
        } else {
            panic!("unexpected diagnostic line");
        }
    }
    assert_eq!(samples, 100);
    assert!(frames.is_empty());
    let mut out = String::from("metric,median_us,observations\n");
    for (k, mut v) in all {
        out.push_str(&format!("{k},{:.3},{}\n", median(&mut v), v.len()));
    }
    fs::write("target/issue310-pooled-stage-summary.csv", &out).unwrap();
    print!("{out}");
}
