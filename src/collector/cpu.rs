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
        brand: cpus
            .first()
            .map(|c| c.brand().to_string())
            .unwrap_or_default(),
        global,
        cores,
        freqs,
        history: Vec::new(),
        load_avg,
    }
}

fn read_load_avg() -> [f64; 3] {
    // getloadavg is POSIX — works on Linux and macOS.
    let mut avg = [0.0f64; 3];
    let ret = unsafe { libc::getloadavg(avg.as_mut_ptr(), 3) };
    if ret >= 0 {
        avg
    } else {
        [0.0; 3]
    }
}
