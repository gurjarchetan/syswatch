use crossterm::event::{self, Event, KeyCode, KeyModifiers, MouseEventKind};
use tokio::time::Duration;
use anyhow::Result;
use crate::app::AppState;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Overview,
    Processes,
    Network,
    Disk,
}

/// Non-blocking: poll for an event with 50 ms timeout.
/// Returns true if the app should exit.
pub async fn handle_events(app: &mut AppState) -> Result<bool> {
    if !event::poll(Duration::from_millis(50))? {
        return Ok(false);
    }

    match event::read()? {
        Event::Key(key) => {
            if app.filter_mode {
                match key.code {
                    KeyCode::Esc | KeyCode::Enter => {
                        app.filter_mode = false;
                    }
                    KeyCode::Backspace => {
                        app.filter_text.pop();
                    }
                    KeyCode::Char(c) => {
                        app.filter_text.push(c);
                    }
                    _ => {}
                }
                return Ok(false);
            }

            match key.code {
                // Quit
                KeyCode::Char('q') => return Ok(true),
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(true);
                }

                // Tab switching
                KeyCode::F(1)  => app.active_tab = ActiveTab::Overview,
                KeyCode::F(2)  => app.active_tab = ActiveTab::Processes,
                KeyCode::F(3)  => app.active_tab = ActiveTab::Network,
                KeyCode::F(4)  => app.active_tab = ActiveTab::Disk,
                KeyCode::Tab   => {
                    app.active_tab = match app.active_tab {
                        ActiveTab::Overview  => ActiveTab::Processes,
                        ActiveTab::Processes => ActiveTab::Network,
                        ActiveTab::Network   => ActiveTab::Disk,
                        ActiveTab::Disk      => ActiveTab::Overview,
                    };
                }

                // Scroll
                KeyCode::Up => {
                    app.scroll_offset = app.scroll_offset.saturating_sub(1);
                    if app.selected_proc > 0 { app.selected_proc -= 1; }
                }
                KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => {
                    app.scroll_offset += 1;
                    app.selected_proc  += 1;
                }

                // Filter
                KeyCode::Char('/') => {
                    app.filter_mode = true;
                    app.filter_text.clear();
                }

                // Sort cycle
                KeyCode::Char('f') => {
                    app.sort_col = app.sort_col.next();
                }

                // Kill selected process
                KeyCode::Char('k') => {
                    kill_selected(app);
                }

                _ => {}
            }
        }

        Event::Mouse(m) => match m.kind {
            MouseEventKind::ScrollUp   => {
                app.scroll_offset = app.scroll_offset.saturating_sub(1);
                if app.selected_proc > 0 { app.selected_proc -= 1; }
            }
            MouseEventKind::ScrollDown => {
                app.scroll_offset += 1;
                app.selected_proc  += 1;
            }
            _ => {}
        },

        Event::Resize(_, _) => {} // ratatui handles resize automatically

        _ => {}
    }

    Ok(false)
}

fn kill_selected(app: &AppState) {
    let state = app.state.read();
    let procs = &state.processes;
    if let Some(proc) = procs.get(app.selected_proc) {
        let pid = proc.pid as i32;
        // SAFETY: sending SIGTERM to the process; only our own user's processes
        // are reachable without root, which is expected behaviour.
        #[cfg(unix)]
        unsafe {
            libc::kill(pid, libc::SIGTERM);
        }
    }
}
