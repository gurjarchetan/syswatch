use sysinfo::{System, ProcessStatus};

#[derive(Default, Clone)]
pub struct ProcessInfo {
    pub pid:      u32,
    pub name:     String,
    pub user:     String,
    pub cpu_pct:  f32,
    pub mem_kb:   u64,
    pub mem_pct:  f32,
    pub status:   String,
    pub priority: i32,
}

pub fn collect(sys: &System) -> Vec<ProcessInfo> {
    let total_mem = sys.total_memory().max(1);
    let mut procs: Vec<ProcessInfo> = sys.processes().iter().map(|(pid, p)| {
        ProcessInfo {
            pid:     pid.as_u32(),
            name:    p.name().to_string_lossy().to_string(),
            user:    p.user_id()
                      .map(|u| u.to_string())
                      .unwrap_or_else(|| "?".to_string()),
            cpu_pct: p.cpu_usage(),
            mem_kb:  p.memory() / 1024,
            mem_pct: p.memory() as f32 / total_mem as f32 * 100.0,
            status:  match p.status() {
                ProcessStatus::Run   => "R".to_string(),
                ProcessStatus::Sleep => "S".to_string(),
                ProcessStatus::Stop  => "T".to_string(),
                ProcessStatus::Zombie=> "Z".to_string(),
                _                    => "?".to_string(),
            },
            priority: 0,
        }
    }).collect();

    // Default sort: by CPU% descending
    procs.sort_by(|a, b| b.cpu_pct.partial_cmp(&a.cpu_pct).unwrap_or(std::cmp::Ordering::Equal));
    procs
}
