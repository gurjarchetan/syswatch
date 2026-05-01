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

#[tokio::main]
async fn main() -> Result<()> {
    // ── shared state ──────────────────────────────────────────────────────────
    let shared: SharedState = Arc::new(RwLock::new(SystemState::default()));

    // ── spawn data collector ──────────────────────────────────────────────────
    let state_clone = Arc::clone(&shared);
    tokio::spawn(async move {
        if let Err(e) = spawn_collector(state_clone).await {
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

    // ── render loop ────────────────────────────────────────────────────────────
    // Render at ~30 fps; input is polled inside handle_events with a 50ms timeout.
    let mut render_ticker = interval(Duration::from_millis(33));

    loop {
        // Handle input (non-blocking, 50ms poll)
        if input::handle_events(&mut app).await? {
            break;
        }

        // Render frame
        render_ticker.tick().await;
        terminal.draw(|f| ui::draw(f, &app))?;
    }

    // ── restore terminal ───────────────────────────────────────────────────────
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}

