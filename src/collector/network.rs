use sysinfo::Networks;
use super::HISTORY_LEN;

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
    pub name:   String,
    pub rx_bps: u64,
    pub tx_bps: u64,
}

impl NetStats {
    pub fn fmt_speed(bps: u64) -> String {
        if bps >= 1_073_741_824 { format!("{:.1} GB/s", bps as f64 / 1_073_741_824.0) }
        else if bps >= 1_048_576 { format!("{:.1} MB/s", bps as f64 / 1_048_576.0) }
        else if bps >= 1024      { format!("{:.1} KB/s", bps as f64 / 1024.0) }
        else { format!("{} B/s", bps) }
    }
    pub fn fmt_bytes(b: u64) -> String {
        if b >= 1_073_741_824 { format!("{:.2} GiB", b as f64 / 1_073_741_824.0) }
        else if b >= 1_048_576 { format!("{:.1} MiB", b as f64 / 1_048_576.0) }
        else if b >= 1024      { format!("{:.1} KiB", b as f64 / 1024.0) }
        else { format!("{} B", b) }
    }
}

/// `old` carries accumulated totals; we diff against fresh values.
pub fn collect(nets: &Networks, old: &mut NetStats) -> NetStats {
    let mut rx_bps = 0u64;
    let mut tx_bps = 0u64;
    let mut interfaces = Vec::new();

    for (name, data) in nets.iter() {
        // sysinfo gives bytes received/transmitted per refresh interval
        let rx = data.received();
        let tx = data.transmitted();
        // Multiply by 2 since we refresh every 500ms
        let rx_s = rx * 2;
        let tx_s = tx * 2;
        rx_bps += rx_s;
        tx_bps += tx_s;
        interfaces.push(IfaceStats { name: name.clone(), rx_bps: rx_s, tx_bps: tx_s });
    }

    let total_rx = old.total_rx + rx_bps / 2;
    let total_tx = old.total_tx + tx_bps / 2;

    let mut rx_history = old.rx_history.clone();
    let mut tx_history = old.tx_history.clone();
    rx_history.push(rx_bps);
    tx_history.push(tx_bps);
    if rx_history.len() > HISTORY_LEN { rx_history.drain(0..rx_history.len() - HISTORY_LEN); }
    if tx_history.len() > HISTORY_LEN { tx_history.drain(0..tx_history.len() - HISTORY_LEN); }

    NetStats { rx_bps, tx_bps, total_rx, total_tx, rx_history, tx_history, interfaces }
}
