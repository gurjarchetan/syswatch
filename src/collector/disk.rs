use sysinfo::Disks;
use std::collections::HashMap;

#[derive(Default, Clone)]
pub struct DiskStats {
    pub name:        String,
    pub mount:       String,
    pub fs_type:     String,
    pub total_kb:    u64,
    pub used_kb:     u64,
    pub avail_kb:    u64,
    /// Read throughput bytes/sec
    pub read_bps:    u64,
    /// Write throughput bytes/sec
    pub write_bps:   u64,
    /// Read IOPS (ops/sec)
    pub read_iops:   u64,
    /// Write IOPS (ops/sec)
    pub write_iops:  u64,
}

impl DiskStats {
    pub fn used_pct(&self) -> f64 {
        if self.total_kb == 0 { return 0.0; }
        self.used_kb as f64 / self.total_kb as f64 * 100.0
    }
    pub fn fmt_speed(bps: u64) -> String {
        if bps >= 1_073_741_824 { format!("{:.1}G/s", bps as f64 / 1_073_741_824.0) }
        else if bps >= 1_048_576 { format!("{:.1}M/s", bps as f64 / 1_048_576.0) }
        else if bps >= 1024      { format!("{:.0}K/s", bps as f64 / 1024.0) }
        else { format!("{}B/s", bps) }
    }
    pub fn fmt_size(kb: u64) -> String {
        if kb >= 1_073_741_824 { format!("{:.1}T", kb as f64 / 1_073_741_824.0) }
        else if kb >= 1_048_576 { format!("{:.1}G", kb as f64 / 1_048_576.0) }
        else if kb >= 1024      { format!("{:.1}M", kb as f64 / 1024.0) }
        else { format!("{}K", kb) }
    }
}

/// Raw counters read from /proc/diskstats per device.
#[derive(Default, Clone)]
struct RawIo {
    reads:        u64,
    writes:       u64,
    read_sectors: u64,
    write_sectors:u64,
}

/// Parse /proc/diskstats into a map keyed by device name.
fn read_diskstats() -> HashMap<String, RawIo> {
    let mut map = HashMap::new();
    if let Ok(data) = std::fs::read_to_string("/proc/diskstats") {
        for line in data.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 14 { continue; }
            let dev = parts[2].to_string();
            let raw = RawIo {
                reads:         parts[3].parse().unwrap_or(0),
                read_sectors:  parts[5].parse().unwrap_or(0),
                writes:        parts[7].parse().unwrap_or(0),
                write_sectors: parts[9].parse().unwrap_or(0),
            };
            map.insert(dev, raw);
        }
    }
    map
}

/// Derive the kernel device name from a mount-point by matching sysinfo Disk
/// against /proc/mounts — or just strip the last path component.
fn dev_from_name(raw_name: &str) -> String {
    // sysinfo gives the full path like /dev/sda1 on Linux
    std::path::Path::new(raw_name)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| raw_name.to_string())
}

/// State carried between ticks for delta calculation.
#[derive(Default, Clone)]
pub struct DiskIoState {
    prev: HashMap<String, RawIo>,
}

pub fn collect(disks: &Disks, io_state: &mut DiskIoState) -> Vec<DiskStats> {
    let current = read_diskstats();
    // 500 ms interval → multiply delta by 2 to get per-second
    const INTERVAL_MS: u64 = 500;
    const SECTOR_SIZE: u64 = 512;

    let result = disks.iter().map(|d| {
        let total = d.total_space() / 1024;
        let avail = d.available_space() / 1024;
        let used  = total.saturating_sub(avail);

        let dev_name = dev_from_name(&d.name().to_string_lossy());
        let (read_bps, write_bps, read_iops, write_iops) =
            if let (Some(prev), Some(cur)) = (io_state.prev.get(&dev_name), current.get(&dev_name)) {
                let d_reads  = cur.reads.saturating_sub(prev.reads);
                let d_writes = cur.writes.saturating_sub(prev.writes);
                let d_rsect  = cur.read_sectors.saturating_sub(prev.read_sectors);
                let d_wsect  = cur.write_sectors.saturating_sub(prev.write_sectors);
                let bps_r = d_rsect * SECTOR_SIZE * 1000 / INTERVAL_MS;
                let bps_w = d_wsect * SECTOR_SIZE * 1000 / INTERVAL_MS;
                let iops_r = d_reads  * 1000 / INTERVAL_MS;
                let iops_w = d_writes * 1000 / INTERVAL_MS;
                (bps_r, bps_w, iops_r, iops_w)
            } else {
                (0, 0, 0, 0)
            };

        DiskStats {
            name:       d.name().to_string_lossy().to_string(),
            mount:      d.mount_point().to_string_lossy().to_string(),
            fs_type:    d.file_system().to_string_lossy().to_string(),
            total_kb:   total,
            used_kb:    used,
            avail_kb:   avail,
            read_bps,
            write_bps,
            read_iops,
            write_iops,
        }
    }).collect();

    io_state.prev = current;
    result
}

