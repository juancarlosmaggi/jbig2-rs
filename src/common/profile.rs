use std::collections::BTreeMap;
use std::time::Duration;

#[derive(Clone, Debug, Default)]
pub struct DecodeProfile {
    stats: BTreeMap<&'static str, ProfileStat>,
    total: Duration,
}

#[derive(Clone, Debug, Default)]
struct ProfileStat {
    total: Duration,
    count: u64,
}

impl DecodeProfile {
    pub fn record(&mut self, label: &'static str, duration: Duration) {
        let entry = self.stats.entry(label).or_default();
        entry.total += duration;
        entry.count = entry.count.saturating_add(1);
        self.total += duration;
    }

    pub fn report(&self) -> String {
        let mut rows: Vec<(&'static str, Duration, u64)> = self
            .stats
            .iter()
            .map(|(label, stat)| (*label, stat.total, stat.count))
            .collect();
        rows.sort_by(|a, b| b.1.cmp(&a.1));

        let mut out = String::new();
        out.push_str(&format!(
            "Decode profile (total {:.3} ms)\n",
            self.total.as_secs_f64() * 1000.0
        ));
        for (label, total, count) in rows {
            let total_ms = total.as_secs_f64() * 1000.0;
            let avg_ms = if count > 0 {
                total_ms / count as f64
            } else {
                0.0
            };
            out.push_str(&format!(
                "{:<32} {:>10.3} ms  {:>6}x  avg {:>8.3} ms\n",
                label, total_ms, count, avg_ms
            ));
        }
        out
    }
}
