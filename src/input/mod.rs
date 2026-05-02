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

/// Poll for a terminal event, blocking for at most `timeout`.
/// Returns true if the app should exit.
pub async fn handle_events(app: &mut AppState, timeout: Duration) -> Result<bool> {
    if !event::poll(timeout)? {
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
                app.needs_redraw = true;
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

                // Kill selected process — two-step: first press enters confirm, second sends signal
                KeyCode::Char('k') if app.active_tab == ActiveTab::Processes => {
                    if app.kill_confirm {
                        kill_selected(app, false);
                        app.kill_confirm = false;
                    } else {
                        app.kill_confirm = true;
                    }
                }
                // SIGKILL immediately on K (uppercase)
                KeyCode::Char('K') if app.active_tab == ActiveTab::Processes => {
                    kill_selected(app, true);
                    app.kill_confirm = false;
                }
                // Cancel confirm or close detail with Esc
                KeyCode::Esc => {
                    app.kill_confirm = false;
                    app.show_proc_detail = false;
                }

                // Show/hide process detail popup (Enter in Processes tab)
                KeyCode::Enter if app.active_tab == ActiveTab::Processes => {
                    if !app.show_proc_detail {
                        // proc_cache is already sorted+filtered; capture the PID
                        // of the selected row so the popup stays stable across re-sorts.
                        app.detail_pid = app.proc_cache.get(app.selected_proc).map(|p| p.pid);
                    }
                    app.show_proc_detail = !app.show_proc_detail;
                    app.kill_confirm = false;
                }

                _ => {}
            }
            app.needs_redraw = true;
        }

        Event::Mouse(m) => match m.kind {
            MouseEventKind::ScrollUp   => {
                app.scroll_offset = app.scroll_offset.saturating_sub(1);
                if app.selected_proc > 0 { app.selected_proc -= 1; }
                app.needs_redraw = true;
            }
            MouseEventKind::ScrollDown => {
                app.scroll_offset += 1;
                app.selected_proc  += 1;
                app.needs_redraw = true;
            }
            _ => {} // mouse move — do NOT set needs_redraw; avoids constant redraws
        },

        Event::Resize(_, _) => {
            app.needs_redraw = true;
        }

        _ => {}
    }

    Ok(false)
}

fn kill_selected(app: &AppState, force: bool) {
    // proc_cache is sorted+filtered — selected_proc indexes into it correctly.
    if let Some(proc) = app.proc_cache.get(app.selected_proc) {
        let pid = proc.pid as i32;
        let sig = if force { libc::SIGKILL } else { libc::SIGTERM };
        // SAFETY: sending signal to process; without root, can only reach own processes.
        #[cfg(unix)]
        unsafe {
            libc::kill(pid, sig);
        }
    }
}
