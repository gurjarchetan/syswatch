use sysinfo::System;

#[derive(Default, Clone)]
pub struct MemStats {
    pub total_kb: u64,
    pub used_kb: u64,
    pub free_kb: u64,
    pub swap_total_kb: u64,
    pub swap_used_kb: u64,
}

impl MemStats {
    pub fn used_pct(&self) -> f64 {
        if self.total_kb == 0 {
            return 0.0;
        }
        self.used_kb as f64 / self.total_kb as f64 * 100.0
    }
    pub fn swap_pct(&self) -> f64 {
        if self.swap_total_kb == 0 {
            return 0.0;
        }
        self.swap_used_kb as f64 / self.swap_total_kb as f64 * 100.0
    }
    pub fn fmt_kb(kb: u64) -> String {
        if kb >= 1_048_576 {
            format!("{:.1} GiB", kb as f64 / 1_048_576.0)
        } else if kb >= 1024 {
            format!("{:.1} MiB", kb as f64 / 1024.0)
        } else {
            format!("{} KiB", kb)
        }
    }
}

pub fn collect(sys: &System) -> MemStats {
    MemStats {
        total_kb: sys.total_memory() / 1024,
        used_kb: sys.used_memory() / 1024,
        free_kb: sys.free_memory() / 1024,
        swap_total_kb: sys.total_swap() / 1024,
        swap_used_kb: sys.used_swap() / 1024,
    }
}
