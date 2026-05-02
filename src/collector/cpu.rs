use sysinfo::System;

#[derive(Default, Clone)]
pub struct CpuStats {
    /// Per-core utilization [0.0 .. 100.0]
    pub cores: Vec<f32>,
    /// Per-core frequency in MHz
    pub freqs: Vec<u64>,
    /// Global CPU usage
    pub global: f32,
    /// Rolling 60-sample history of global usage
    pub history: Vec<f32>,
    /// CPU brand string
    pub brand: String,
    /// Logical core count
    pub count: usize,
    /// Load average: 1min, 5min, 15min
    pub load_avg: [f64; 3],
}

pub fn collect(sys: &System) -> CpuStats {
    let cpus = sys.cpus();
    let cores: Vec<f32> = cpus.iter().map(|c| c.cpu_usage()).collect();
    let freqs: Vec<u64> = cpus.iter().map(|c| c.frequency()).collect();
    let global = sys.global_cpu_usage();

    // Read /proc/loadavg directly — most portable
    let load_avg = read_load_avg();

    CpuStats {
        count: cores.len(),
        brand: cpus.first().map(|c| c.brand().to_string()).unwrap_or_default(),
        global,
        cores,
        freqs,
        history: Vec::new(),
        load_avg,
    }
}

fn read_load_avg() -> [f64; 3] {
    if let Ok(s) = std::fs::read_to_string("/proc/loadavg") {
        let mut parts = s.split_whitespace();
        let a = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0.0);
        let b = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0.0);
        let c = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0.0);
        return [a, b, c];
    }
    [0.0; 3]
}
