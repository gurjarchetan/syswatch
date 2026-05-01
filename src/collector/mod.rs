pub mod cpu;
pub mod memory;
pub mod disk;
pub mod network;
pub mod process;

use std::sync::Arc;
use parking_lot::RwLock;
use tokio::time::{interval, Duration};
use anyhow::Result;

use cpu::CpuStats;
use memory::MemStats;
use disk::{DiskStats, DiskIoState};
use network::NetStats;
use process::{ProcessInfo, ProcSummary};

pub const HISTORY_LEN: usize = 60;

/// The central shared state updated by the collector thread.
#[derive(Default)]
pub struct SystemState {
    pub cpu: CpuStats,
    pub memory: MemStats,
    pub disks: Vec<DiskStats>,
    pub network: NetStats,
    pub processes: Vec<ProcessInfo>,
    pub proc_summary: ProcSummary,
    pub uptime_secs: u64,
    pub hostname: String,
    pub os_name: String,
}

pub type SharedState = Arc<RwLock<SystemState>>;

/// Spawn the background data-collection task.
/// Updates every 500 ms; each module reads /proc or uses sysinfo.
pub async fn spawn_collector(state: SharedState) -> Result<()> {
    let mut sys = sysinfo::System::new_all();
    let mut disks_tracker = sysinfo::Disks::new_with_refreshed_list();
    let mut nets_tracker = sysinfo::Networks::new_with_refreshed_list();
    let mut disk_io_state = DiskIoState::default();

    // Gather one-time info
    {
        let mut s = state.write();
        s.hostname = sysinfo::System::host_name().unwrap_or_default();
        s.os_name = format!(
            "{} {}",
            sysinfo::System::name().unwrap_or_default(),
            sysinfo::System::os_version().unwrap_or_default()
        );
    }

    let mut ticker = interval(Duration::from_millis(500));

    loop {
        ticker.tick().await;

        sys.refresh_all();
        disks_tracker.refresh();
        nets_tracker.refresh();

        let mut s = state.write();

        s.uptime_secs = sysinfo::System::uptime();
        let old_history = s.cpu.history.clone();
        let mut new_cpu = cpu::collect(&sys);
        new_cpu.history = cpu::merge_history(&old_history, new_cpu.global);
        s.cpu    = new_cpu;
        s.memory = memory::collect(&sys);
        s.disks  = disk::collect(&disks_tracker, &mut disk_io_state);
        let old_net = s.network.clone();
        s.network = network::collect(&nets_tracker, &mut { old_net });
        let (procs, summary) = process::collect(&sys);
        s.processes    = procs;
        s.proc_summary = summary;
    }
}
