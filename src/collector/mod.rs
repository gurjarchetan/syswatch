pub mod cpu;
pub mod memory;
pub mod disk;
pub mod network;
pub mod process;

use std::sync::Arc;
use parking_lot::RwLock;
use tokio::time::{interval_at, Duration, Instant};
use anyhow::Result;
use sysinfo::{CpuRefreshKind, ProcessRefreshKind, ProcessesToUpdate};

use std::collections::VecDeque;
use cpu::CpuStats;
use memory::MemStats;
use disk::{DiskStats, DiskIoState};
use network::NetStats;
use process::{ProcessInfo, ProcSummary};

pub const HISTORY_LEN: usize = 512;

/// The central shared state updated by the collector thread.
#[derive(Default)]
pub struct SystemState {
    pub cpu: CpuStats,
    pub memory: MemStats,
    /// RAM used% history (0–100 scaled ×100 as u64), capped at HISTORY_LEN
    pub mem_history: Vec<u64>,
    pub disks: Vec<DiskStats>,
    pub network: NetStats,
    pub processes: Vec<ProcessInfo>,
    pub proc_summary: ProcSummary,
    /// Increments every time the process list is refreshed (~1 s).
    /// Render thread compares this to skip re-sort when data hasn't changed.
    pub proc_gen: u64,
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
    // System::new_all(), Disks::new_with_refreshed_list(), and
    // Networks::new_with_refreshed_list() all do blocking /proc reads.
    // With worker_threads=2, calling them directly inside an async task
    // starves the render loop thread, freezing the UI during startup.
    let (mut sys, mut disks_tracker, mut nets_tracker) =
        tokio::task::spawn_blocking(|| (
            sysinfo::System::new_all(),
            sysinfo::Disks::new_with_refreshed_list(),
            sysinfo::Networks::new_with_refreshed_list(),
        ))
        .await
        .map_err(|e| anyhow::anyhow!("collector init: {}", e))?;
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

    // First tick fires immediately — new_all() already refreshed all data,
    // so we can collect it right away instead of waiting a full interval.
    // Subsequent ticks fire every interval_ms.
    let interval_dur = Duration::from_millis(interval_ms.max(100));
    let mut ticker = interval_at(Instant::now(), interval_dur);
    let mut tick: u64 = 0;

    // Minimal process refresh: CPU% + memory only.
    let proc_refresh = ProcessRefreshKind::new().with_cpu().with_memory();

    // ── Collector-local history ring buffers (VecDeque → O(1) push/pop) ──────
    // These live here so the write lock never needs to read-then-clone the old
    // history from SharedState.  We only WRITE to SharedState, never read back.
    let mut cpu_history:    VecDeque<f32> = VecDeque::with_capacity(HISTORY_LEN + 1);
    let mut net_rx_history: VecDeque<u64> = VecDeque::with_capacity(HISTORY_LEN + 1);
    let mut net_tx_history: VecDeque<u64> = VecDeque::with_capacity(HISTORY_LEN + 1);
    let mut mem_hist:       VecDeque<u64> = VecDeque::with_capacity(HISTORY_LEN + 1);
    let mut net_total_rx:   u64 = 0;
    let mut net_total_tx:   u64 = 0;

    /// Push a value into a fixed-capacity ring buffer — O(1).
    #[inline]
    fn push_ring<T>(buf: &mut VecDeque<T>, val: T) {
        if buf.len() >= HISTORY_LEN { buf.pop_front(); }
        buf.push_back(val);
    }

    loop {
        ticker.tick().await;

        // ── Targeted system refresh ───────────────────────────────────────
        // tick=0 on the very first iteration (interval_at fires immediately),
        // which means tick%3==0 and tick%6==0 both fire — processes and disks
        // are collected on the first tick so they appear on startup with no delay.
        //
        // CPU freq changes slowly; refresh it on tick 0 then every 10 ticks
        // (saves 16+ sysfs reads/tick on a multi-core machine).
        let cpu_kind = if tick % 10 == 0 {
            CpuRefreshKind::everything()          // usage + freq (~5 s cadence)
        } else {
            CpuRefreshKind::new().with_cpu_usage() // usage only
        };
        sys.refresh_cpu_specifics(cpu_kind);
        sys.refresh_memory();
        nets_tracker.refresh();

        // Processes: every 3rd tick (~1.5 s); tick=0 fires on startup.
        if tick % 3 == 0 {
            sys.refresh_processes_specifics(ProcessesToUpdate::All, proc_refresh);
        }

        // Disks: every 6th tick (~3 s); tick=0 fires on startup.
        if tick % 6 == 0 {
            disks_tracker.refresh();
        }

        // ── Compute outside write lock ────────────────────────────────────
        let new_cpu = cpu::collect(&sys);
        push_ring(&mut cpu_history, new_cpu.global);

        let new_mem    = memory::collect(&sys);
        let mem_sample = (new_mem.used_pct() * 100.0) as u64;
        push_ring(&mut mem_hist, mem_sample);

        // Network: collect instant bps (no history management inside the fn).
        let (rx_bps, tx_bps, interfaces) = network::collect_instant(&nets_tracker, interval_ms);
        // Accumulate actual bytes per tick: bps × interval_ms ÷ 1000.
        net_total_rx = net_total_rx.saturating_add(rx_bps * interval_ms / 1000);
        net_total_tx = net_total_tx.saturating_add(tx_bps * interval_ms / 1000);
        push_ring(&mut net_rx_history, rx_bps);
        push_ring(&mut net_tx_history, tx_bps);

        let new_disks = if tick % 6 == 0 {
            Some(disk::collect(&disks_tracker, &mut disk_io_state, interval_ms))
        } else {
            None
        };

        let proc_data: Option<(Vec<ProcessInfo>, ProcSummary)> = if tick % 3 == 0 {
            Some(process::collect(&sys))
        } else {
            None
        };

        // ── Write lock: assignment only, no computation, no reads ─────────
        // We use mem::take to reclaim the Vec allocation from SharedState before
        // overwriting it.  This way, after the first tick, no Vec ever
        // reallocates — each "history" write is just a memcpy of ~2 KB.
        let mut s = state.write();
        s.uptime_secs = sysinfo::System::uptime();

        // -- CPU -------------------------------------------------------
        // Take the old history Vec (has capacity), refill from local deque.
        let mut cpu_hist_buf = std::mem::take(&mut s.cpu.history);
        s.cpu = new_cpu; // overwrites s.cpu.history with empty Vec (fine)
        cpu_hist_buf.clear();
        let (a, b) = cpu_history.as_slices();
        cpu_hist_buf.extend_from_slice(a);
        cpu_hist_buf.extend_from_slice(b);
        s.cpu.history = cpu_hist_buf;

        // -- Memory ----------------------------------------------------
        s.memory = new_mem;
        let mut mem_hist_buf = std::mem::take(&mut s.mem_history);
        mem_hist_buf.clear();
        let (a, b) = mem_hist.as_slices();
        mem_hist_buf.extend_from_slice(a);
        mem_hist_buf.extend_from_slice(b);
        s.mem_history = mem_hist_buf;

        // -- Network ---------------------------------------------------
        let mut rx_buf = std::mem::take(&mut s.network.rx_history);
        let mut tx_buf = std::mem::take(&mut s.network.tx_history);
        s.network.rx_bps     = rx_bps;
        s.network.tx_bps     = tx_bps;
        s.network.total_rx   = net_total_rx;
        s.network.total_tx   = net_total_tx;
        s.network.interfaces = interfaces;
        rx_buf.clear();
        let (a, b) = net_rx_history.as_slices();
        rx_buf.extend_from_slice(a);
        rx_buf.extend_from_slice(b);
        s.network.rx_history = rx_buf;
        tx_buf.clear();
        let (a, b) = net_tx_history.as_slices();
        tx_buf.extend_from_slice(a);
        tx_buf.extend_from_slice(b);
        s.network.tx_history = tx_buf;

        if let Some(disks) = new_disks {
            s.disks = disks;
        }
        if let Some((procs, summary)) = proc_data {
            s.processes    = procs;
            s.proc_summary = summary;
            s.proc_gen     = s.proc_gen.wrapping_add(1);
        }
        // lock releases here

        tick += 1;
    }
}
