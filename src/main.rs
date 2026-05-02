mod collector;
mod ui;
mod input;
mod app;

use std::sync::Arc;
use parking_lot::RwLock;
use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    event::{EnableMouseCapture, DisableMouseCapture},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::time::{Duration, Instant};

use collector::{SharedState, SystemState, spawn_collector};
use app::AppState;
use input::ActiveTab;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// Parsed CLI configuration.
struct Config {
    /// Starting tab (0=Overview, 1=Processes, 2=Network, 3=Disk)
    start_tab: ActiveTab,
    /// Collector interval in milliseconds (default 500)
    interval_ms: u64,
}

fn print_help() {
    println!(
        r#"syswatch {VERSION} — {DESCRIPTION}

USAGE:
    syswatch [OPTIONS]

OPTIONS:
    -h, --help              Show this help message and exit
    -V, --version           Show version and exit
    -t, --tab <TAB>         Start on a specific tab
                              overview   (default)
                              processes
                              network
                              disk
    -i, --interval <MS>     Data-collection interval in milliseconds (default: 500, min: 100)

KEYBOARD SHORTCUTS:
    F1 / F2 / F3 / F4       Switch tab: Overview / Processes / Network / Disk
    Tab                     Cycle through tabs
    ↑ ↓ / j k               Scroll / select row
    /                       Enter process filter (type to search by name)
    Esc / Enter             Exit filter or cancel kill
    f                       Cycle sort column: CPU% → MEM% → PID → Name
    k                       Arm kill — row turns red; press k again → SIGTERM
    K                       SIGKILL immediately
    q / Ctrl-C              Quit

EXAMPLES:
    syswatch                           # Start on Overview tab
    syswatch --tab processes           # Start directly on Processes tab
    syswatch --tab disk                # Start on Disk tab
    syswatch --interval 250            # Refresh twice as fast
    syswatch -t network -i 1000        # Network tab, 1 second interval

REPOSITORY:
    https://github.com/gurjarchetan/syswatch
"#
    );
}

fn parse_args() -> Result<Option<Config>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut start_tab = ActiveTab::Overview;
    let mut interval_ms: u64 = 500;
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help();
                return Ok(None);
            }
            "-V" | "--version" => {
                println!("syswatch {VERSION}");
                return Ok(None);
            }
            "-t" | "--tab" => {
                i += 1;
                let val = args.get(i).map(String::as_str).unwrap_or("");
                start_tab = match val {
                    "overview"   | "1" => ActiveTab::Overview,
                    "processes"  | "2" => ActiveTab::Processes,
                    "network"    | "3" => ActiveTab::Network,
                    "disk"       | "4" => ActiveTab::Disk,
                    _ => {
                        eprintln!("syswatch: unknown tab '{}'. Use: overview, processes, network, disk", val);
                        std::process::exit(1);
                    }
                };
            }
            "-i" | "--interval" => {
                i += 1;
                let val = args.get(i).map(String::as_str).unwrap_or("500");
                interval_ms = val.parse::<u64>().unwrap_or_else(|_| {
                    eprintln!("syswatch: --interval must be a positive integer (milliseconds)");
                    std::process::exit(1);
                }).max(100);
            }
            unknown => {
                eprintln!("syswatch: unknown argument '{}'. Run 'syswatch --help' for usage.", unknown);
                std::process::exit(1);
            }
        }
        i += 1;
    }

    Ok(Some(Config { start_tab, interval_ms }))
}

// Two worker threads are sufficient: one runs the render+event loop,
// one runs the background collector. The default runtime spawns one
// thread per CPU core (16 on this machine = 33 threads total) — every
// idle thread burns ~0.3% CPU through tokio's work-stealing scheduler.
#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> Result<()> {
    let config = match parse_args()? {
        Some(c) => c,
        None => return Ok(()), // --help or --version already printed
    };

    // ── shared state ──────────────────────────────────────────────────────────
    let shared: SharedState = Arc::new(RwLock::new(SystemState::default()));

    // ── spawn data collector ──────────────────────────────────────────────────
    let state_clone = Arc::clone(&shared);
    let collect_interval = config.interval_ms;
    tokio::spawn(async move {
        if let Err(e) = spawn_collector(state_clone, collect_interval).await {
            eprintln!("Collector error: {}", e);
        }
    });

    // ── setup terminal ─────────────────────────────────────────────────────────
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // ── app state ──────────────────────────────────────────────────────────────
    let mut app = AppState::new(Arc::clone(&shared));
    app.active_tab = config.start_tab;

    // ── render loop ────────────────────────────────────────────────────────────
    // Render only when: (a) user input (needs_redraw), (b) new data arrived
    // (proc_changed), or (c) IDLE_REDRAW elapsed for graph updates (2fps).
    //
    // Critically: event::poll blocks for exactly (time_until_next_redraw),
    // so the loop only wakes ~2×/second when idle instead of 20×/second
    // with a fixed 50ms timeout. This is the main CPU saving.
    let mut last_draw = Instant::now() - Duration::from_secs(1); // force first frame
    const IDLE_REDRAW: Duration = Duration::from_millis(500);

    loop {
        // Check for new data and rebuild proc cache if needed.
        let proc_changed = app.update_proc_cache();

        // Draw if anything changed or interval elapsed.
        let elapsed = last_draw.elapsed();
        if app.needs_redraw || proc_changed || elapsed >= IDLE_REDRAW {
            terminal.draw(|f| ui::draw(f, &app))?;
            last_draw = Instant::now();
            app.needs_redraw = false;
        }

        // Block exactly until the next scheduled redraw (or until user input).
        // This replaces the fixed 50ms poll that ran at 20Hz doing nothing.
        let time_to_draw = IDLE_REDRAW
            .saturating_sub(last_draw.elapsed())
            .max(Duration::from_millis(1));
        if input::handle_events(&mut app, time_to_draw).await? {
            break;
        }
    }

    // ── restore terminal ───────────────────────────────────────────────────────
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}

