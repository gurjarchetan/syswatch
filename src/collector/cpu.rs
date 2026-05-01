use super::HISTORY_LEN;
use sysinfo::System;

#[derive(Default, Clone)]
pub struct CpuStats {
    /// Per-core utilization [0.0 .. 100.0]
    pub cores: Vec<f32>,
    /// Global CPU usage
    pub global: f32,
    /// Rolling 60-sample history of global usage
    pub history: Vec<f32>,
    /// CPU brand string
    pub brand: String,
    /// Logical core count
    pub count: usize,
}

pub fn collect(sys: &System) -> CpuStats {
    let cpus = sys.cpus();
    let cores: Vec<f32> = cpus.iter().map(|c| c.cpu_usage()).collect();
    let global = sys.global_cpu_usage();

    // We pass in the old stats through the caller to preserve history.
    // However, because collect() doesn't have the old CpuStats here,
    // the caller (mod.rs) will merge history in.
    CpuStats {
        count: cores.len(),
        brand: cpus.first().map(|c| c.brand().to_string()).unwrap_or_default(),
        global,
        cores,
        history: Vec::new(), // filled by merge_history in mod.rs
    }
}

pub fn merge_history(old: &[f32], new_val: f32) -> Vec<f32> {
    let mut h = old.to_vec();
    h.push(new_val);
    if h.len() > HISTORY_LEN {
        h.drain(0..h.len() - HISTORY_LEN);
    }
    h
}
