use std::collections::HashMap;
use std::ffi::CString;

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
        (self.used_kb as f64 / self.total_kb as f64 * 100.0).min(100.0)
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

/// Strip the last path component to get the kernel device name.
fn dev_from_name(raw_name: &str) -> String {
    std::path::Path::new(raw_name)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| raw_name.to_string())
}

/// Filesystem types to skip completely.
const SKIP_FS: &[&str] = &[
    "squashfs", "proc", "sysfs", "devpts", "cgroup", "cgroup2",
    "pstore", "bpf", "tracefs", "debugfs", "fusectl", "securityfs",
    "hugetlbfs", "mqueue", "configfs", "nsfs", "overlay", "autofs",
    "devtmpfs", "efivarfs",
];

/// Mount path prefixes to skip.
const SKIP_MOUNT_PREFIX: &[&str] = &[
    "/run/credentials/",
    "/proc/",
    "/sys/",
    "/snap/",
];

/// For tmpfs, only show mounts that are meaningful to a user.
fn is_useful_tmpfs(mount: &str) -> bool {
    matches!(mount, "/dev/shm" | "/tmp" | "/run")
        || mount.starts_with("/run/user/")
}

/// State carried between ticks for I/O delta calculation.
#[derive(Default, Clone)]
pub struct DiskIoState {
    prev: HashMap<String, RawIo>,
}

/// Read /proc/mounts and call statvfs for each relevant mount point.
/// This surfaces real block devices + useful tmpfs entries.
pub fn collect(_unused: &sysinfo::Disks, io_state: &mut DiskIoState, interval_ms: u64) -> Vec<DiskStats> {
    let current = read_diskstats();
    let interval_ms = interval_ms.max(100);
    const SECTOR_SIZE: u64 = 512;

    let mounts_data = std::fs::read_to_string("/proc/mounts").unwrap_or_default();
    let mut result = Vec::new();

    for line in mounts_data.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 { continue; }
        let dev   = parts[0];
        let mount = parts[1];
        let fs    = parts[2];

        // Skip unwanted fs types
        if SKIP_FS.iter().any(|&f| f == fs) { continue; }

        // Skip unwanted mount prefixes
        if SKIP_MOUNT_PREFIX.iter().any(|&p| mount.starts_with(p)) { continue; }

        // For tmpfs, only include useful mounts
        if fs == "tmpfs" && !is_useful_tmpfs(mount) { continue; }

        // Call statvfs to get disk space
        let c_mount = match CString::new(mount) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
        let ret = unsafe { libc::statvfs(c_mount.as_ptr(), &mut stat) };
        if ret != 0 { continue; }

        let bsz = stat.f_frsize as u64;
        if bsz == 0 { continue; }

        let total_kb = stat.f_blocks * bsz / 1024;
        let free_kb  = stat.f_bfree  * bsz / 1024;
        let avail_kb = stat.f_bavail * bsz / 1024;
        let used_kb  = total_kb.saturating_sub(free_kb);

        if total_kb == 0 { continue; }

        let dev_name = dev_from_name(dev);
        let (read_bps, write_bps, read_iops, write_iops) =
            if let (Some(prev), Some(cur)) = (io_state.prev.get(&dev_name), current.get(&dev_name)) {
                let d_reads  = cur.reads.saturating_sub(prev.reads);
                let d_writes = cur.writes.saturating_sub(prev.writes);
                let d_rsect  = cur.read_sectors.saturating_sub(prev.read_sectors);
                let d_wsect  = cur.write_sectors.saturating_sub(prev.write_sectors);
                (
                    d_rsect * SECTOR_SIZE * 1000 / interval_ms,
                    d_wsect * SECTOR_SIZE * 1000 / interval_ms,
                    d_reads  * 1000 / interval_ms,
                    d_writes * 1000 / interval_ms,
                )
            } else {
                (0, 0, 0, 0)
            };

        result.push(DiskStats {
            name: dev.to_string(),
            mount: mount.to_string(),
            fs_type: fs.to_string(),
            total_kb,
            used_kb,
            avail_kb,
            read_bps,
            write_bps,
            read_iops,
            write_iops,
        });
    }

    io_state.prev = current;
    result
}

