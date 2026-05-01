use sysinfo::Disks;

#[derive(Default, Clone)]
pub struct DiskStats {
    pub name:        String,
    pub mount:       String,
    pub fs_type:     String,
    pub total_kb:    u64,
    pub used_kb:     u64,
    pub read_bps:    u64,
    pub write_bps:   u64,
}

impl DiskStats {
    pub fn used_pct(&self) -> f64 {
        if self.total_kb == 0 { return 0.0; }
        self.used_kb as f64 / self.total_kb as f64 * 100.0
    }
    pub fn fmt_speed(bps: u64) -> String {
        if bps >= 1_073_741_824 { format!("{:.1} GB/s", bps as f64 / 1_073_741_824.0) }
        else if bps >= 1_048_576 { format!("{:.1} MB/s", bps as f64 / 1_048_576.0) }
        else if bps >= 1024      { format!("{:.1} KB/s", bps as f64 / 1024.0) }
        else { format!("{} B/s", bps) }
    }
}

pub fn collect(disks: &Disks) -> Vec<DiskStats> {
    disks.iter().map(|d| {
        let total = d.total_space() / 1024;
        let avail = d.available_space() / 1024;
        DiskStats {
            name:      d.name().to_string_lossy().to_string(),
            mount:     d.mount_point().to_string_lossy().to_string(),
            fs_type:   d.file_system().to_string_lossy().to_string(),
            total_kb:  total,
            used_kb:   total.saturating_sub(avail),
            read_bps:  0,
            write_bps: 0,
        }
    }).collect()
}
