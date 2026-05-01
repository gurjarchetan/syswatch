use sysinfo::{System, ProcessStatus};

#[derive(Default, Clone)]
pub struct ProcessInfo {
    pub pid:      u32,
    pub name:     String,
    pub user:     String,
    pub cpu_pct:  f32,
    pub mem_kb:   u64,
    pub mem_pct:  f32,
    /// Single-char status: R/S/D/T/Z/I/?
    pub status:   String,
    /// Full status label for display
    pub status_full: String,
    pub priority: i32,
    pub threads:  u32,
}

/// Aggregate process state counts shown in the summary bar.
#[derive(Default, Clone)]
pub struct ProcSummary {
    pub total:   usize,
    pub running: usize,
    pub sleeping:usize,
    pub stopped: usize,
    pub zombie:  usize,
    pub other:   usize,
}

pub fn collect(sys: &System) -> (Vec<ProcessInfo>, ProcSummary) {
    let total_mem = sys.total_memory().max(1);
    let mut summary = ProcSummary::default();

    let mut procs: Vec<ProcessInfo> = sys.processes().iter().map(|(pid, p)| {
        let (sc, sf) = match p.status() {
            ProcessStatus::Run    => { summary.running  += 1; ("R", "Running") }
            ProcessStatus::Sleep  => { summary.sleeping += 1; ("S", "Sleeping") }
            ProcessStatus::Idle   => { summary.sleeping += 1; ("I", "Idle") }
            ProcessStatus::Stop   => { summary.stopped  += 1; ("T", "Stopped") }
            ProcessStatus::Zombie => { summary.zombie   += 1; ("Z", "Zombie") }
            ProcessStatus::Dead   => { summary.other    += 1; ("X", "Dead") }
            _                     => { summary.other    += 1; ("?", "Unknown") }
        };
        summary.total += 1;

        ProcessInfo {
            pid:         pid.as_u32(),
            name:        p.name().to_string_lossy().to_string(),
            user:        p.user_id()
                          .map(|u| u.to_string())
                          .unwrap_or_else(|| "?".to_string()),
            cpu_pct:     p.cpu_usage(),
            mem_kb:      p.memory() / 1024,
            mem_pct:     p.memory() as f32 / total_mem as f32 * 100.0,
            status:      sc.to_string(),
            status_full: sf.to_string(),
            threads:     p.tasks().map(|t| t.len() as u32).unwrap_or(1),
            priority:    0,
        }
    }).collect();

    // Default sort: by CPU% descending
    procs.sort_by(|a, b| b.cpu_pct.partial_cmp(&a.cpu_pct).unwrap_or(std::cmp::Ordering::Equal));
    (procs, summary)
}

