use sysinfo::Networks;

#[derive(Default, Clone)]
pub struct NetStats {
    /// Current RX bytes/sec
    pub rx_bps: u64,
    /// Current TX bytes/sec
    pub tx_bps: u64,
    /// Total RX bytes since start
    pub total_rx: u64,
    /// Total TX bytes since start
    pub total_tx: u64,
    /// History for sparklines
    pub rx_history: Vec<u64>,
    pub tx_history: Vec<u64>,
    /// Interface breakdown
    pub interfaces: Vec<IfaceStats>,
}

#[derive(Default, Clone)]
pub struct IfaceStats {
    pub name: String,
    pub rx_bps: u64,
    pub tx_bps: u64,
}

impl NetStats {
    pub fn fmt_speed(bps: u64) -> String {
        if bps >= 1_073_741_824 {
            format!("{:.1} GB/s", bps as f64 / 1_073_741_824.0)
        } else if bps >= 1_048_576 {
            format!("{:.1} MB/s", bps as f64 / 1_048_576.0)
        } else if bps >= 1024 {
            format!("{:.1} KB/s", bps as f64 / 1024.0)
        } else {
            format!("{} B/s", bps)
        }
    }
    pub fn fmt_bytes(b: u64) -> String {
        if b >= 1_073_741_824 {
            format!("{:.2} GiB", b as f64 / 1_073_741_824.0)
        } else if b >= 1_048_576 {
            format!("{:.1} MiB", b as f64 / 1_048_576.0)
        } else if b >= 1024 {
            format!("{:.1} KiB", b as f64 / 1024.0)
        } else {
            format!("{} B", b)
        }
    }
}

/// Returns `(rx_bps, tx_bps, interfaces)` for the current refresh interval.
/// History management is done by the caller (spawn_collector) using local
/// VecDeque buffers — no cloning of history Vecs inside this function.
pub fn collect_instant(nets: &Networks, interval_ms: u64) -> (u64, u64, Vec<IfaceStats>) {
    // interval_ms is the actual measurement window; convert to per-second rate.
    let ticks_per_sec = 1000u64 / interval_ms.max(1);
    let mut rx_bps = 0u64;
    let mut tx_bps = 0u64;
    let mut interfaces = Vec::new();

    for (name, data) in nets.iter() {
        let rx = data.received().saturating_mul(ticks_per_sec);
        let tx = data.transmitted().saturating_mul(ticks_per_sec);
        rx_bps = rx_bps.saturating_add(rx);
        tx_bps = tx_bps.saturating_add(tx);
        interfaces.push(IfaceStats {
            name: name.clone(),
            rx_bps: rx,
            tx_bps: tx,
        });
    }

    (rx_bps, tx_bps, interfaces)
}
