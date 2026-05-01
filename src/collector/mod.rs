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
    pub private_ip: String,
    pub public_ip: String,
}

pub type SharedState = Arc<RwLock<SystemState>>;

/// Derive the primary private IP by routing-table trick (no packets sent).
fn get_private_ip() -> String {
    use std::net::UdpSocket;
    UdpSocket::bind("0.0.0.0:0")
        .ok()
        .and_then(|s| {
            s.connect("8.8.8.8:80").ok()?;
            s.local_addr().ok()
        })
        .map(|a| a.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Fetch the public IP from api.ipify.org via a raw HTTP/1.0 request.
async fn fetch_public_ip() -> String {
    use tokio::net::TcpStream;
    use tokio::io::{AsyncWriteExt, AsyncReadExt};

    let result: anyhow::Result<String> = async {
        let mut stream = tokio::time::timeout(
            Duration::from_secs(8),
            TcpStream::connect("api.ipify.org:80"),
        ).await??;
        stream.write_all(b"GET / HTTP/1.0\r\nHost: api.ipify.org\r\nConnection: close\r\n\r\n").await?;
        let mut buf = Vec::new();
        tokio::time::timeout(
            Duration::from_secs(8),
            stream.read_to_end(&mut buf),
        ).await??;
        let resp = String::from_utf8_lossy(&buf);
        let body = resp.split("\r\n\r\n").nth(1).unwrap_or("").trim().to_string();
        anyhow::ensure!(!body.is_empty(), "empty response");
        Ok(body)
    }.await;

    result.unwrap_or_else(|_| "unavailable".to_string())
}

/// Spawn the background data-collection task.
/// `interval_ms` controls how often data is refreshed (default 500, min 100).
pub async fn spawn_collector(state: SharedState, interval_ms: u64) -> Result<()> {
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
        s.private_ip = get_private_ip();
        s.public_ip  = "fetching…".to_string();
    }

    // Fetch public IP in background — updates state when ready.
    {
        let state_clone = Arc::clone(&state);
        tokio::spawn(async move {
            let ip = fetch_public_ip().await;
            state_clone.write().public_ip = ip;
        });
    }

    let mut ticker = interval(Duration::from_millis(interval_ms.max(100)));

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
