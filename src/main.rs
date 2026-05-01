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
use tokio::time::{interval, Duration};

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

#[tokio::main]
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
    let mut render_ticker = interval(Duration::from_millis(33));

    loop {
        if input::handle_events(&mut app).await? {
            break;
        }
        render_ticker.tick().await;
        terminal.draw(|f| ui::draw(f, &app))?;
    }

    // ── restore terminal ───────────────────────────────────────────────────────
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}

